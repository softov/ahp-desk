//! The host list: `hosts.json` in the app config folder, each token in the
//! vault (see `tokens`), and the commands the shell and a UI call.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, State, WebviewWindow};

use crate::app_window::AppWindows;
use crate::statuses::HostStatuses;
use crate::tokens::{self, Vault};

/// The event every save and remove sends, with the whole list.
pub const HOSTS_CHANGED: &str = "hosts-changed";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HostKind {
    Url,
    Local,
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ahpd_path: Option<String>,
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
        }
    }
}

pub struct HostStore {
    path: PathBuf,
    hosts: Mutex<Vec<Host>>,
    vault: Box<dyn Vault>,
}

impl HostStore {
    /// Reads `path`, or starts empty when it does not exist, and moves any
    /// token left in the file into the vault when the vault works.
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
        }
        if moved {
            write(&path, &hosts)?;
        }
        Ok(HostStore {
            path,
            hosts: Mutex::new(hosts),
            vault,
        })
    }

    fn lock(&self) -> MutexGuard<'_, Vec<Host>> {
        // A panic while the list was held leaves it whole: each change is one push or remove.
        self.hosts.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    pub fn list(&self) -> Vec<HostView> {
        self.lock().iter().map(HostView::from).collect()
    }

    pub fn find(&self, id: &str) -> Option<HostView> {
        self.lock().iter().find(|host| host.id == id).map(HostView::from)
    }

    /// Adds the host, or replaces the one with its id. `token`: `None` keeps the
    /// token there is, `Some("")` deletes it, anything else replaces it.
    pub fn save(&self, input: HostInput, token: Option<String>) -> Result<Vec<HostView>, String> {
        for (field, value) in [("id", &input.id), ("name", &input.name), ("url", &input.url)] {
            if value.trim().is_empty() {
                return Err(format!("a host needs a {field}"));
            }
        }
        let mut hosts = self.lock();
        let index = hosts.iter().position(|host| host.id == input.id);
        let mut host = Host {
            token: index.and_then(|i| hosts[i].token.clone()),
            id: input.id,
            name: input.name,
            kind: input.kind,
            url: input.url,
            ahpd_path: input.ahpd_path,
        };
        if let Some(token) = token {
            tokens::set(self.vault.as_ref(), &mut host, &token);
        }
        match index {
            Some(i) => hosts[i] = host,
            None => hosts.push(host),
        }
        write(&self.path, &hosts)?;
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
        write(&self.path, &hosts)?;
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
}

/// Writes the list to a temporary file and renames it over `path`, so a crash
/// mid-write never leaves half a file.
fn write(path: &Path, hosts: &[Host]) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }
    let text = serde_json::to_string_pretty(hosts).map_err(|err| err.to_string())?;
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
) -> Result<(), String> {
    let list = store.save(host, token)?;
    app.emit(HOSTS_CHANGED, list).map_err(|err| err.to_string())
}

/// Removes the host and closes the windows that show it.
#[tauri::command]
pub fn remove_host(
    app: AppHandle,
    store: State<'_, HostStore>,
    windows: State<'_, AppWindows>,
    statuses: State<'_, HostStatuses>,
    id: String,
) -> Result<(), String> {
    let list = store.remove(&id)?;
    windows.close_host(&app, &id);
    statuses.forget(&id);
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
        return Err(format!("window {} may not read the token of {id}", window.label()));
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
        let list = store.save(input("home", "Home"), Some("secret".into())).unwrap();
        assert_eq!(list.len(), 1);
        assert!(!list[0].token_in_file);
        assert!(!scratch.text().contains("secret"));
        assert_eq!(store.token("home").unwrap().as_deref(), Some("secret"));
    }

    #[test]
    fn hosts_save_replaces_by_id_and_keeps_the_token_when_none_is_given() {
        let scratch = Scratch::new();
        let store = HostStore::load(scratch.file(), Box::new(Memory::default())).unwrap();
        store.save(input("home", "Home"), Some("secret".into())).unwrap();
        let list = store.save(input("home", "Home desk"), None).unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].name, "Home desk");
        assert_eq!(store.token("home").unwrap().as_deref(), Some("secret"));
    }

    #[test]
    fn hosts_an_empty_token_deletes_it() {
        let scratch = Scratch::new();
        let store = HostStore::load(scratch.file(), Box::new(Memory::default())).unwrap();
        store.save(input("home", "Home"), Some("secret".into())).unwrap();
        store.save(input("home", "Home"), Some(String::new())).unwrap();
        assert_eq!(store.token("home").unwrap(), None);
    }

    #[test]
    fn hosts_remove_deletes_the_host_and_its_token() {
        let scratch = Scratch::new();
        let store = HostStore::load(scratch.file(), Box::new(Memory::default())).unwrap();
        store.save(input("home", "Home"), Some("secret".into())).unwrap();
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
        let list = store.save(input("home", "Home"), Some("secret".into())).unwrap();
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
        };
        assert_eq!(tokens::get(&vault, &host).as_deref(), Some("from-vault"));
        assert_eq!(tokens::get(&Broken, &host).as_deref(), Some("from-file"));
    }

    #[test]
    fn hosts_a_host_needs_an_id_a_name_and_an_address() {
        let scratch = Scratch::new();
        let store = HostStore::load(scratch.file(), Box::new(Memory::default())).unwrap();
        assert!(store.save(input(" ", "Home"), None).is_err());
        assert!(store.save(input("home", ""), None).is_err());
    }
}
