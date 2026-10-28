//! The host list: `hosts.json` in the app config folder, each token in the
//! vault (see `tokens`), and the commands the shell and a UI call.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, State, WebviewWindow};

use crate::app_window::AppWindows;
use crate::ssh::tunnel::Tunnels;
use crate::ssh::{password, Secrets, SignIn};
use crate::statuses::{HostRunning, HostStatuses};
use crate::tokens::{self, Vault};

/// The event every save and remove sends, with the whole list.
pub const HOSTS_CHANGED: &str = "hosts-changed";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HostKind {
    Url,
    Local,
    /// A server reached through a tunnel the desk holds over SSH.
    Ssh,
}

/// How the desk reaches the ahpd of a host over SSH.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Reach {
    /// Through the SSH connection, to ahpd on the server's 127.0.0.1.
    #[default]
    Tunnel,
    /// On ahpd's open port, with its token.
    Port,
}

/// A host as `hosts.json` keeps it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Host {
    pub id: String,
    pub name: String,
    pub kind: HostKind,
    pub url: String,
    /// The ahpd a `local` host uses, when one is set instead of the one found.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ahpd_path: Option<String>,
    /// The token, only when the keyring could not keep it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
    /// A `local` host's ahpd that is on because the desk started it, so the
    /// desk may stop it; one it only found it never stops.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub started_by_desk: bool,
    /// How the desk reaches the host's server over SSH, on a host with one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ssh: Option<SshHost>,
    #[serde(flatten)]
    pub on_desk: OnDesk,
}

/// A host's server over SSH, as `hosts.json` keeps it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SshHost {
    pub address: String,
    #[serde(default = "ssh_port")]
    pub port: u16,
    pub user: String,
    /// The way the last sign-in went through.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sign_in: Option<SignIn>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub key_file: Option<String>,
    #[serde(default)]
    pub reach: Reach,
    /// The port ahpd listens on, on the server.
    #[serde(default = "remote_port")]
    pub remote_port: u16,
    /// The folder ahpd serves.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub folder: Option<String>,
    /// The desk installed this ahpd, so Update and Remove may touch it.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub by_desk: bool,
    /// A tunnel's local port, chosen once, when the host is added.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub local_port: Option<u16>,
    /// The kept password, only when the keyring could not keep it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
}

fn ssh_port() -> u16 {
    22
}

fn remote_port() -> u16 {
    crate::ssh_install::AHPD_PORT
}

/// `SshHost` as a window reads and the form sends it: never a password.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SshHostView {
    pub address: String,
    #[serde(default = "ssh_port")]
    pub port: u16,
    pub user: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sign_in: Option<SignIn>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub key_file: Option<String>,
    #[serde(default)]
    pub reach: Reach,
    #[serde(default = "remote_port")]
    pub remote_port: u16,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub folder: Option<String>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub by_desk: bool,
    /// Absent from the form on an edit, which keeps the one the host has.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub local_port: Option<u16>,
}

impl From<&SshHost> for SshHostView {
    fn from(ssh: &SshHost) -> Self {
        SshHostView {
            address: ssh.address.clone(),
            port: ssh.port,
            user: ssh.user.clone(),
            sign_in: ssh.sign_in,
            key_file: ssh.key_file.clone(),
            reach: ssh.reach,
            remote_port: ssh.remote_port,
            folder: ssh.folder.clone(),
            by_desk: ssh.by_desk,
            local_port: ssh.local_port,
        }
    }
}

/// What a `local` host's ahpd does when the desk starts and quits.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OnDesk {
    /// "Start when the desktop app starts", on by default.
    #[serde(default = "on", skip_serializing_if = "is_on")]
    pub start_on_launch: bool,
    /// "Stop when the desktop app quits", off by default.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub stop_on_quit: bool,
    /// "Update ahpd automatically", on by default: an ahpd the desk installed
    /// updates through its own update command while no turn runs (plan 03 decision 6).
    #[serde(default = "on", skip_serializing_if = "is_on")]
    pub auto_update: bool,
}

impl Default for OnDesk {
    fn default() -> Self {
        OnDesk {
            start_on_launch: true,
            stop_on_quit: false,
            auto_update: true,
        }
    }
}

fn on() -> bool {
    true
}

fn is_on(value: &bool) -> bool {
    *value
}

/// A host as the shell sends it to `save_host`; its token comes apart.
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HostInput {
    pub id: String,
    pub name: String,
    pub kind: HostKind,
    pub url: String,
    /// The ahpd a `local` host uses, when one is set instead of the one found.
    #[serde(default)]
    pub ahpd_path: Option<String>,
    /// A `local` host's settings; absent keeps what the host has.
    #[serde(default)]
    pub on_desk: Option<OnDeskView>,
    /// The host's server over SSH; its kept password comes apart.
    #[serde(default)]
    pub ssh: Option<SshHostView>,
}

/// `OnDesk` as the shell reads and sends it: every setting, always.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OnDeskView {
    pub start_on_launch: bool,
    pub stop_on_quit: bool,
    pub auto_update: bool,
}

impl From<OnDeskView> for OnDesk {
    fn from(view: OnDeskView) -> Self {
        OnDesk {
            start_on_launch: view.start_on_launch,
            stop_on_quit: view.stop_on_quit,
            auto_update: view.auto_update,
        }
    }
}

/// A host as a window sees it: never the token, only whether it sits in the plain file.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HostView {
    pub id: String,
    pub name: String,
    pub kind: HostKind,
    pub url: String,
    /// The ahpd a `local` host uses, when one is set instead of the one found.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ahpd_path: Option<String>,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub token_in_file: bool,
    /// A `local` host's settings; none on a `url` host.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub on_desk: Option<OnDeskView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ssh: Option<SshHostView>,
    /// The SSH password sits in the plain file, as a token can.
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub ssh_password_in_file: bool,
}

impl From<&Host> for HostView {
    fn from(host: &Host) -> Self {
        HostView {
            id: host.id.clone(),
            name: host.name.clone(),
            kind: host.kind,
            url: host.url.clone(),
            ahpd_path: host.ahpd_path.clone(),
            token_in_file: host.token.is_some(),
            on_desk: (host.kind == HostKind::Local).then_some(OnDeskView {
                start_on_launch: host.on_desk.start_on_launch,
                stop_on_quit: host.on_desk.stop_on_quit,
                auto_update: host.on_desk.auto_update,
            }),
            ssh: host.ssh.as_ref().map(SshHostView::from),
            ssh_password_in_file: host.ssh.as_ref().is_some_and(|ssh| ssh.password.is_some()),
        }
    }
}

pub struct HostStore {
    path: PathBuf,
    hosts: Mutex<Vec<Host>>,
    vault: Box<dyn Vault>,
    /// What a person typed for each host's SSH sign-in, until the desk quits;
    /// here so that removing a host drops it with the host.
    secrets: Arc<Secrets>,
}

impl HostStore {
    /// Reads `path`, or starts empty when it does not exist, and moves any
    /// token or SSH password left in the file into the vault when the vault works.
    pub fn load(path: PathBuf, vault: Box<dyn Vault>) -> Result<Self, String> {
        let mut hosts: Vec<Host> = match fs::read_to_string(&path) {
            Ok(text) => serde_json::from_str(&text)
                .map_err(|err| format!("{} is not a host list: {err}", path.display()))?,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Vec::new(),
            Err(err) => return Err(format!("cannot read {}: {err}", path.display())),
        };
        let mut moved = false;
        for host in &mut hosts {
            moved |= tokens::move_to_vault(vault.as_ref(), host);
            moved |= password::move_to_vault(vault.as_ref(), host);
        }
        if moved {
            write_json(&path, &hosts[..])?;
        }
        Ok(HostStore {
            path,
            hosts: Mutex::new(hosts),
            vault,
            secrets: Arc::default(),
        })
    }

    fn lock(&self) -> MutexGuard<'_, Vec<Host>> {
        // A panic while the list was held leaves it whole: each change is one push or remove.
        self.hosts
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    pub fn list(&self) -> Vec<HostView> {
        self.lock().iter().map(HostView::from).collect()
    }

    pub fn find(&self, id: &str) -> Option<HostView> {
        self.lock()
            .iter()
            .find(|host| host.id == id)
            .map(HostView::from)
    }

    /// Adds the host, or replaces the one with its id. `token`: `None` keeps the
    /// token there is, `Some("")` deletes it, anything else replaces it.
    pub fn save(&self, input: HostInput, token: Option<String>) -> Result<Vec<HostView>, String> {
        for (field, value) in [
            ("id", &input.id),
            ("name", &input.name),
            ("url", &input.url),
        ] {
            if value.trim().is_empty() {
                return Err(format!("a host needs a {field}"));
            }
        }
        let mut hosts = self.lock();
        let index = hosts.iter().position(|host| host.id == input.id);
        let before = index.and_then(|i| hosts[i].ssh.clone());
        let mut host = Host {
            token: index.and_then(|i| hosts[i].token.clone()),
            // The form never says whether the desk started an ahpd; the desk does.
            started_by_desk: index.is_some_and(|i| hosts[i].started_by_desk),
            id: input.id,
            name: input.name,
            kind: input.kind,
            url: input.url,
            ahpd_path: input.ahpd_path,
            on_desk: match input.on_desk {
                Some(view) => view.into(),
                None => index.map(|i| hosts[i].on_desk).unwrap_or_default(),
            },
            // The form never sends a password, nor needs to say how the last sign-in went.
            ssh: input.ssh.map(|view| SshHost {
                sign_in: view.sign_in.or(before.as_ref().and_then(|ssh| ssh.sign_in)),
                password: before.as_ref().and_then(|ssh| ssh.password.clone()),
                // Chosen once: an edit keeps the port the host's address names.
                local_port: before
                    .as_ref()
                    .and_then(|ssh| ssh.local_port)
                    .or(view.local_port),
                address: view.address,
                port: view.port,
                user: view.user,
                key_file: view.key_file,
                reach: view.reach,
                remote_port: view.remote_port,
                folder: view.folder,
                by_desk: view.by_desk,
            }),
        };
        if host.kind == HostKind::Ssh && host.ssh.as_ref().and_then(|ssh| ssh.local_port).is_none()
        {
            return Err("an ssh host needs its SSH details and a local port".into());
        }
        if let Some(token) = token {
            tokens::set(self.vault.as_ref(), &mut host, &token);
        }
        match index {
            Some(i) => hosts[i] = host,
            None => hosts.push(host),
        }
        write_json(&self.path, &hosts[..])?;
        Ok(hosts.iter().map(HostView::from).collect())
    }

    pub fn remove(&self, id: &str) -> Result<Vec<HostView>, String> {
        let mut hosts = self.lock();
        let before = hosts.len();
        hosts.retain(|host| host.id != id);
        if hosts.len() == before {
            return Err(format!("no host {id}"));
        }
        // A keyring that cannot be reached holds no copy to delete.
        let _ = self.vault.delete(id);
        password::forget(self.vault.as_ref(), id);
        self.secrets.forget(id);
        write_json(&self.path, &hosts[..])?;
        Ok(hosts.iter().map(HostView::from).collect())
    }

    pub fn token(&self, id: &str) -> Result<Option<String>, String> {
        let hosts = self.lock();
        let host = hosts
            .iter()
            .find(|host| host.id == id)
            .ok_or_else(|| format!("no host {id}"))?;
        Ok(tokens::get(self.vault.as_ref(), host))
    }

    /// What a person typed for each host's SSH sign-in.
    pub fn secrets(&self) -> &Arc<Secrets> {
        &self.secrets
    }

    pub fn keyring_works(&self) -> bool {
        password::keyring_works(self.vault.as_ref())
    }

    /// A host's SSH details, `None` on a host with none.
    pub fn ssh(&self, id: &str) -> Result<Option<SshHost>, String> {
        let hosts = self.lock();
        let host = hosts
            .iter()
            .find(|host| host.id == id)
            .ok_or_else(|| format!("no host {id}"))?;
        Ok(host.ssh.clone())
    }

    /// Keeps the SSH password the person asked to keep, as `password::keep` does.
    pub fn keep_ssh_password(&self, id: &str, password: &str) -> Result<Vec<HostView>, String> {
        let mut hosts = self.lock();
        let host = hosts
            .iter_mut()
            .find(|host| host.id == id)
            .ok_or_else(|| format!("no host {id}"))?;
        if host.ssh.is_none() {
            return Err(format!("{id} has no SSH details"));
        }
        password::keep(self.vault.as_ref(), host, password);
        write_json(&self.path, &hosts[..])?;
        Ok(hosts.iter().map(HostView::from).collect())
    }

    /// Forgets the SSH password kept for a host, in the keyring and in the file.
    pub fn forget_ssh_password(&self, id: &str) -> Result<Vec<HostView>, String> {
        let mut hosts = self.lock();
        let host = hosts
            .iter_mut()
            .find(|host| host.id == id)
            .ok_or_else(|| format!("no host {id}"))?;
        password::forget(self.vault.as_ref(), id);
        if let Some(ssh) = host.ssh.as_mut() {
            ssh.password = None;
        }
        write_json(&self.path, &hosts[..])?;
        Ok(hosts.iter().map(HostView::from).collect())
    }

    /// The SSH password kept for a host.
    pub fn ssh_password(&self, id: &str) -> Result<Option<String>, String> {
        let hosts = self.lock();
        let host = hosts
            .iter()
            .find(|host| host.id == id)
            .ok_or_else(|| format!("no host {id}"))?;
        Ok(password::kept(self.vault.as_ref(), host))
    }

    /// The `local` hosts whose ahpd the desk starts when it starts.
    pub fn to_start_on_launch(&self) -> Vec<String> {
        self.lock()
            .iter()
            .filter(|host| host.kind == HostKind::Local && host.on_desk.start_on_launch)
            .map(|host| host.id.clone())
            .collect()
    }

    /// The `local` hosts whose ahpd the desk stops when it quits: set to, and
    /// started by the desk.
    pub fn to_stop_on_quit(&self) -> Vec<String> {
        self.lock()
            .iter()
            .filter(|host| {
                host.kind == HostKind::Local && host.on_desk.stop_on_quit && host.started_by_desk
            })
            .map(|host| host.id.clone())
            .collect()
    }

    /// The `local` hosts whose ahpd a desk update stops before its installer
    /// runs: every one the desk started, whatever its quit setting, since it
    /// runs on the desk's own Node, which the installer replaces.
    pub fn to_stop_for_desk_install(&self) -> Vec<String> {
        self.lock()
            .iter()
            .filter(|host| host.kind == HostKind::Local && host.started_by_desk)
            .map(|host| host.id.clone())
            .collect()
    }

    /// A `local` host as the desk drives it: its set ahpd path, and whether
    /// the desk started the ahpd that is on.
    pub fn local(&self, id: &str) -> Result<(Option<String>, bool), String> {
        let hosts = self.lock();
        let host = hosts
            .iter()
            .find(|host| host.id == id)
            .ok_or_else(|| format!("no host {id}"))?;
        if host.kind != HostKind::Local {
            return Err(format!("{id} is not this computer"));
        }
        Ok((host.ahpd_path.clone(), host.started_by_desk))
    }

    /// Saves what the local ahpd says of itself: its address,
    /// its token when it has one, and whether the desk started it.
    pub fn set_local(
        &self,
        id: &str,
        on: Option<(&str, Option<&str>)>,
        started_by_desk: bool,
    ) -> Result<Vec<HostView>, String> {
        let mut hosts = self.lock();
        let host = hosts
            .iter_mut()
            .find(|host| host.id == id)
            .ok_or_else(|| format!("no host {id}"))?;
        if let Some((url, token)) = on {
            host.url = url.to_string();
            tokens::set(self.vault.as_ref(), host, token.unwrap_or_default());
        }
        host.started_by_desk = started_by_desk;
        write_json(&self.path, &hosts[..])?;
        Ok(hosts.iter().map(HostView::from).collect())
    }
}

/// Writes `value` as JSON to a temporary file and renames it over `path`, so a
/// crash mid-write never leaves half a file. `settings.json` is written the same way.
pub fn write_json<T: Serialize + ?Sized>(path: &Path, value: &T) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }
    let text = serde_json::to_string_pretty(value).map_err(|err| err.to_string())?;
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, text).map_err(|err| err.to_string())?;
    fs::rename(&tmp, path).map_err(|err| err.to_string())
}

#[tauri::command]
pub fn hosts(store: State<'_, HostStore>) -> Vec<HostView> {
    store.list()
}

#[tauri::command]
pub fn save_host(
    app: AppHandle,
    store: State<'_, HostStore>,
    host: HostInput,
    token: Option<String>,
    ssh_password: Option<String>,
) -> Result<(), String> {
    let id = host.id.clone();
    let mut list = store.save(host, token)?;
    // `None` keeps what is kept, `""` forgets it, a value keeps it as Remember asked.
    match ssh_password.as_deref() {
        None => {}
        Some("") => list = store.forget_ssh_password(&id)?,
        Some(password) => list = store.keep_ssh_password(&id, password)?,
    }
    app.emit(HOSTS_CHANGED, list).map_err(|err| err.to_string())
}

/// Removes the host and closes the windows that show it.
#[tauri::command]
pub fn remove_host(
    app: AppHandle,
    store: State<'_, HostStore>,
    windows: State<'_, AppWindows>,
    statuses: State<'_, HostStatuses>,
    running: State<'_, HostRunning>,
    tunnels: State<'_, Tunnels>,
    id: String,
) -> Result<(), String> {
    let list = store.remove(&id)?;
    tunnels.close(&id);
    windows.close_host(&app, &id, list.len());
    statuses.forget(&id);
    running.forget(&id);
    app.emit(HOSTS_CHANGED, list).map_err(|err| err.to_string())
}

/// A host's token, for the host list or for the window that shows that host.
#[tauri::command]
pub fn host_token(
    window: WebviewWindow,
    windows: State<'_, AppWindows>,
    store: State<'_, HostStore>,
    id: String,
) -> Result<Option<String>, String> {
    if !windows.may_read(window.label(), &id) {
        return Err(format!(
            "window {} may not read the token of {id}",
            window.label()
        ));
    }
    store.token(&id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokens::test_vaults::{Broken, Memory};
    use std::sync::atomic::{AtomicUsize, Ordering};

    /// A `hosts.json` path in a folder of its own, gone at the end of the test.
    struct Scratch(PathBuf);
    impl Scratch {
        fn new() -> Self {
            static NEXT: AtomicUsize = AtomicUsize::new(0);
            let dir = std::env::temp_dir().join(format!(
                "ahp-desk-hosts-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir_all(&dir).unwrap();
            Scratch(dir)
        }
        fn file(&self) -> PathBuf {
            self.0.join("hosts.json")
        }
        fn text(&self) -> String {
            fs::read_to_string(self.file()).unwrap()
        }
    }
    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn input(id: &str, name: &str) -> HostInput {
        HostInput {
            id: id.into(),
            name: name.into(),
            kind: HostKind::Url,
            url: "ws://127.0.0.1:9187".into(),
            ahpd_path: None,
            on_desk: None,
            ssh: None,
        }
    }

    #[test]
    fn hosts_a_missing_file_is_an_empty_list() {
        let scratch = Scratch::new();
        let store = HostStore::load(scratch.file(), Box::new(Memory::default())).unwrap();
        assert!(store.list().is_empty());
    }

    #[test]
    fn hosts_save_keeps_the_token_in_the_vault_and_out_of_the_file() {
        let scratch = Scratch::new();
        let store = HostStore::load(scratch.file(), Box::new(Memory::default())).unwrap();
        let list = store
            .save(input("home", "Home"), Some("secret".into()))
            .unwrap();
        assert_eq!(list.len(), 1);
        assert!(!list[0].token_in_file);
        assert!(!scratch.text().contains("secret"));
        assert_eq!(store.token("home").unwrap().as_deref(), Some("secret"));
    }

    #[test]
    fn hosts_save_replaces_by_id_and_keeps_the_token_when_none_is_given() {
        let scratch = Scratch::new();
        let store = HostStore::load(scratch.file(), Box::new(Memory::default())).unwrap();
        store
            .save(input("home", "Home"), Some("secret".into()))
            .unwrap();
        let list = store.save(input("home", "Home desk"), None).unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].name, "Home desk");
        assert_eq!(store.token("home").unwrap().as_deref(), Some("secret"));
    }

    #[test]
    fn hosts_an_empty_token_deletes_it() {
        let scratch = Scratch::new();
        let store = HostStore::load(scratch.file(), Box::new(Memory::default())).unwrap();
        store
            .save(input("home", "Home"), Some("secret".into()))
            .unwrap();
        store
            .save(input("home", "Home"), Some(String::new()))
            .unwrap();
        assert_eq!(store.token("home").unwrap(), None);
    }

    #[test]
    fn hosts_remove_deletes_the_host_and_its_token() {
        let scratch = Scratch::new();
        let store = HostStore::load(scratch.file(), Box::new(Memory::default())).unwrap();
        store
            .save(input("home", "Home"), Some("secret".into()))
            .unwrap();
        assert!(store.remove("home").unwrap().is_empty());
        assert!(store.token("home").is_err());
        assert!(store.remove("home").is_err());
        let reloaded = HostStore::load(scratch.file(), Box::new(Memory::default())).unwrap();
        assert!(reloaded.list().is_empty());
    }

    #[test]
    fn hosts_with_no_keyring_the_token_is_in_the_file_and_flagged() {
        let scratch = Scratch::new();
        let store = HostStore::load(scratch.file(), Box::new(Broken)).unwrap();
        let list = store
            .save(input("home", "Home"), Some("secret".into()))
            .unwrap();
        assert!(list[0].token_in_file);
        assert!(scratch.text().contains("\"token\": \"secret\""));
        assert_eq!(store.token("home").unwrap().as_deref(), Some("secret"));
    }

    #[test]
    fn hosts_a_token_in_the_file_moves_to_the_vault_on_load() {
        let scratch = Scratch::new();
        fs::write(
            scratch.file(),
            r#"[{"id":"home","name":"Home","kind":"url","url":"ws://h","token":"secret"}]"#,
        )
        .unwrap();
        let store = HostStore::load(scratch.file(), Box::new(Memory::default())).unwrap();
        assert!(!store.list()[0].token_in_file);
        assert!(!scratch.text().contains("secret"));
        assert_eq!(store.token("home").unwrap().as_deref(), Some("secret"));
    }

    #[test]
    fn hosts_the_vault_is_read_before_the_file() {
        let vault = Memory::default();
        vault.set("home", "from-vault").unwrap();
        let host = Host {
            id: "home".into(),
            name: "Home".into(),
            kind: HostKind::Url,
            url: "ws://h".into(),
            ahpd_path: None,
            token: Some("from-file".into()),
            started_by_desk: false,
            ssh: None,
            on_desk: OnDesk::default(),
        };
        assert_eq!(tokens::get(&vault, &host).as_deref(), Some("from-vault"));
        assert_eq!(tokens::get(&Broken, &host).as_deref(), Some("from-file"));
    }

    #[test]
    fn hosts_a_local_host_takes_what_its_ahpd_says() {
        let scratch = Scratch::new();
        let store = HostStore::load(scratch.file(), Box::new(Memory::default())).unwrap();
        store.save(input("home", "Home"), None).unwrap();
        assert!(
            store.local("home").is_err(),
            "a url host is not this computer"
        );
        let mut local = input("pc", "This computer");
        local.kind = HostKind::Local;
        local.ahpd_path = Some("C:/tools/ahpd.cmd".into());
        store.save(local.clone(), None).unwrap();
        assert_eq!(
            store.local("pc").unwrap(),
            (Some("C:/tools/ahpd.cmd".into()), false)
        );

        let list = store
            .set_local("pc", Some(("ws://127.0.0.1:9200", Some("t"))), true)
            .unwrap();
        assert_eq!(list[1].url, "ws://127.0.0.1:9200");
        assert_eq!(store.token("pc").unwrap().as_deref(), Some("t"));
        assert!(store.local("pc").unwrap().1);
        assert!(scratch.text().contains("\"startedByDesk\": true"));

        // A save from the form keeps what the desk knows.
        store.save(local, None).unwrap();
        assert!(store.local("pc").unwrap().1);

        store
            .set_local("pc", Some(("ws://127.0.0.1:9200", None)), false)
            .unwrap();
        assert_eq!(store.token("pc").unwrap(), None, "an ahpd with no token");
        assert!(!scratch.text().contains("startedByDesk"));
    }

    #[test]
    fn hosts_the_start_and_quit_settings_pick_the_hosts() {
        let scratch = Scratch::new();
        let store = HostStore::load(scratch.file(), Box::new(Memory::default())).unwrap();
        let local = |id: &str, start_on_launch: bool, stop_on_quit: bool| {
            let mut host = input(id, id);
            host.kind = HostKind::Local;
            host.on_desk = Some(OnDeskView {
                start_on_launch,
                stop_on_quit,
                auto_update: id != "off",
            });
            host
        };
        store.save(input("url", "url"), None).unwrap();
        store.save(local("plain", true, false), None).unwrap();
        store.save(local("off", false, true), None).unwrap();
        store.save(local("ours", true, true), None).unwrap();
        assert_eq!(store.to_start_on_launch(), ["plain", "ours"]);
        assert!(store.to_stop_on_quit().is_empty(), "the desk started none");
        assert!(store.to_stop_for_desk_install().is_empty());
        store
            .set_local("off", Some(("ws://h", None)), true)
            .unwrap();
        store
            .set_local("ours", Some(("ws://h", None)), true)
            .unwrap();
        store
            .set_local("plain", Some(("ws://h", None)), true)
            .unwrap();
        assert_eq!(store.to_stop_on_quit(), ["off", "ours"]);
        assert_eq!(store.to_stop_for_desk_install(), ["plain", "off", "ours"]);

        // The defaults are not written, and a host read with none gets them.
        assert!(!scratch.text().contains("startOnLaunch\": true"));
        assert!(scratch.text().contains("\"startOnLaunch\": false"));
        assert!(!scratch.text().contains("autoUpdate\": true"));
        assert!(scratch.text().contains("\"autoUpdate\": false"));
        let view = store
            .list()
            .into_iter()
            .find(|host| host.id == "plain")
            .unwrap();
        assert_eq!(
            view.on_desk,
            Some(OnDeskView {
                start_on_launch: true,
                stop_on_quit: false,
                auto_update: true
            })
        );
        assert_eq!(store.list()[0].on_desk, None, "a url host has none");
        // A save with no settings keeps them.
        let mut kept = local("off", true, true);
        kept.on_desk = None;
        store.save(kept, None).unwrap();
        assert_eq!(store.to_start_on_launch(), ["plain", "ours"]);
    }

    fn ssh_input(id: &str) -> HostInput {
        let mut host = input(id, id);
        host.ssh = Some(SshHostView {
            address: "server.example".into(),
            port: 2222,
            user: "me".into(),
            sign_in: None,
            key_file: Some("C:/keys/id_ed25519".into()),
            reach: Reach::Port,
            remote_port: 9187,
            folder: None,
            by_desk: false,
            local_port: None,
        });
        host
    }

    #[test]
    fn hosts_an_ssh_host_round_trips_and_keeps_its_local_port() {
        let scratch = Scratch::new();
        let store = HostStore::load(scratch.file(), Box::new(Memory::default())).unwrap();
        let mut tunnelled = ssh_input("dev");
        tunnelled.kind = HostKind::Ssh;
        tunnelled.url = "ws://127.0.0.1:40123".into();
        if let Some(ssh) = tunnelled.ssh.as_mut() {
            ssh.reach = Reach::Tunnel;
            ssh.remote_port = 9300;
            ssh.folder = Some("/srv/work".into());
            ssh.by_desk = true;
            ssh.local_port = Some(40123);
        }
        store.save(tunnelled.clone(), None).unwrap();
        let reloaded = HostStore::load(scratch.file(), Box::new(Memory::default())).unwrap();
        let ssh = reloaded.ssh("dev").unwrap().unwrap();
        assert_eq!(
            (
                ssh.reach,
                ssh.remote_port,
                ssh.local_port,
                ssh.folder.as_deref(),
                ssh.by_desk
            ),
            (Reach::Tunnel, 9300, Some(40123), Some("/srv/work"), true)
        );
        assert!(scratch.text().contains("\"kind\": \"ssh\""));
        // An edit that names another local port keeps the one the host has.
        if let Some(ssh) = tunnelled.ssh.as_mut() {
            ssh.local_port = Some(50000);
        }
        store.save(tunnelled, None).unwrap();
        assert_eq!(store.ssh("dev").unwrap().unwrap().local_port, Some(40123));
    }

    #[test]
    fn hosts_an_ssh_host_needs_its_ssh_and_a_local_port() {
        let scratch = Scratch::new();
        let store = HostStore::load(scratch.file(), Box::new(Memory::default())).unwrap();
        let mut bare = input("dev", "dev");
        bare.kind = HostKind::Ssh;
        assert!(store.save(bare, None).is_err());
        let mut no_port = ssh_input("dev");
        no_port.kind = HostKind::Ssh;
        assert!(store.save(no_port, None).is_err());
        // A url host on an open port keeps its SSH details, for Update and Remove.
        store.save(ssh_input("open"), None).unwrap();
        assert_eq!(store.ssh("open").unwrap().unwrap().reach, Reach::Port);
    }

    #[test]
    fn hosts_a_host_with_ssh_round_trips_through_the_file() {
        let scratch = Scratch::new();
        let store = HostStore::load(scratch.file(), Box::new(Memory::default())).unwrap();
        store.save(ssh_input("dev"), None).unwrap();
        let reloaded = HostStore::load(scratch.file(), Box::new(Memory::default())).unwrap();
        let ssh = reloaded.ssh("dev").unwrap().unwrap();
        assert_eq!(
            (ssh.address.as_str(), ssh.port, ssh.user.as_str()),
            ("server.example", 2222, "me")
        );
        assert_eq!(ssh.key_file.as_deref(), Some("C:/keys/id_ed25519"));
        assert_eq!(reloaded.ssh("dev").unwrap(), store.ssh("dev").unwrap());
        assert!(!scratch.text().contains("password"));
        assert_eq!(store.list()[0].ssh.as_ref().unwrap().port, 2222);
    }

    #[test]
    fn hosts_an_ssh_password_goes_in_the_file_only_by_keep_on_a_failing_vault() {
        let scratch = Scratch::new();
        let store = HostStore::load(scratch.file(), Box::new(Broken)).unwrap();
        store.save(ssh_input("dev"), None).unwrap();
        store.secrets().set_passphrase("dev", "typed passphrase");
        store.secrets().set_password("dev", "typed password");
        assert!(
            !scratch.text().contains("typed"),
            "what was typed is never written"
        );
        let list = store.keep_ssh_password("dev", "kept pw").unwrap();
        assert!(list[0].ssh_password_in_file);
        assert!(scratch.text().contains("\"password\": \"kept pw\""));
        assert!(!scratch.text().contains("typed"));
        assert!(
            !serde_json::to_string(&list).unwrap().contains("kept pw"),
            "never to a window"
        );
        // A save from the form keeps it.
        store.save(ssh_input("dev"), None).unwrap();
        assert_eq!(
            store.ssh_password("dev").unwrap().as_deref(),
            Some("kept pw")
        );
    }

    #[test]
    fn hosts_an_ssh_password_in_the_file_moves_to_the_vault_on_load() {
        let scratch = Scratch::new();
        let broken = HostStore::load(scratch.file(), Box::new(Broken)).unwrap();
        broken.save(ssh_input("dev"), None).unwrap();
        broken.keep_ssh_password("dev", "kept pw").unwrap();
        let store = HostStore::load(scratch.file(), Box::new(Memory::default())).unwrap();
        assert!(!scratch.text().contains("kept pw"));
        assert!(!store.list()[0].ssh_password_in_file);
        assert_eq!(
            store.ssh_password("dev").unwrap().as_deref(),
            Some("kept pw")
        );
    }

    #[test]
    fn hosts_remove_forgets_the_ssh_password_and_what_was_typed() {
        let scratch = Scratch::new();
        let store = HostStore::load(scratch.file(), Box::new(Memory::default())).unwrap();
        store.save(ssh_input("dev"), None).unwrap();
        store.keep_ssh_password("dev", "kept pw").unwrap();
        store.secrets().set_passphrase("dev", "typed");
        store.remove("dev").unwrap();
        assert!(!store.secrets().holds("dev"));
        store.save(ssh_input("dev"), None).unwrap();
        store.keep_ssh_password("dev", "again").unwrap();
        store.forget_ssh_password("dev").unwrap();
        assert_eq!(store.ssh_password("dev").unwrap(), None);
        store.remove("dev").unwrap();
        store.save(ssh_input("dev"), None).unwrap();
        assert_eq!(store.ssh_password("dev").unwrap(), None);
    }

    #[test]
    fn hosts_a_host_needs_an_id_a_name_and_an_address() {
        let scratch = Scratch::new();
        let store = HostStore::load(scratch.file(), Box::new(Memory::default())).unwrap();
        assert!(store.save(input(" ", "Home"), None).is_err());
        assert!(store.save(input("home", ""), None).is_err());
    }
}
