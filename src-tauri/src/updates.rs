//! What can be updated: the desk itself, from its GitHub releases, and npm's
//! latest `@ahpd/server` for the desk's hosts, asked at most every 6 hours and
//! kept in `updates.json`, as ahpd's own `update.ts` asks and keeps it.

use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard, Once};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager, State};

use tauri::ipc::Channel;
use tauri_plugin_updater::UpdaterExt;

use crate::hosts::{write_json, HostStore};
use crate::local_ahpd::InstallEvent;
use crate::ssh::known_keys::KnownKeys;
use crate::ssh_install::{self, InstallControl};

/// The event each change sends, with the whole `UpdatesView`.
pub const UPDATES_CHANGED: &str = "updates-changed";
/// The package asked about.
pub const AHPD_PACKAGE: &str = "@ahpd/server";
/// How long an answer is trusted, as ahpd's `MAX_AGE_MS`.
pub const MAX_AGE: Duration = Duration::from_secs(6 * 60 * 60);
/// How long npm gets to answer, as ahpd's `askRegistry`.
const ASK_TIMEOUT: Duration = Duration::from_secs(5);
/// The event sent with the version once a newer desk is downloaded.
pub const DESK_UPDATE_READY: &str = "desk-update-ready";

/// A newer desk, downloaded and checked against the public key, waiting for Restart.
struct Ready {
    version: String,
    update: tauri_plugin_updater::Update,
    bytes: Vec<u8>,
}

/// The desk's own update, once one is ready.
#[derive(Default)]
pub struct DeskUpdate(Mutex<Option<Ready>>);

impl DeskUpdate {
    fn lock(&self) -> MutexGuard<'_, Option<Ready>> {
        self.0
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

/// A release build asks GitHub for a newer desk at start and downloads it in
/// the background; a debug build never asks. A failure (offline, no release
/// yet, a bad signature) is one log line, and the desk runs on.
pub fn check_on_start(app: &AppHandle) {
    let app = app.clone();
    tauri::async_runtime::spawn(async move { check_desk(&app).await });
}

/// The start check, and the one `watch` makes every 6 hours after it, so a
/// desk left running for days still finds a new release. A debug build never asks.
async fn check_desk(app: &AppHandle) {
    if cfg!(debug_assertions) {
        return;
    }
    if let Err(err) = download_desk(app).await {
        eprintln!("ahp-desk: no desk update: {err}");
    }
}

/// Whether a release found is new to the desk: one already downloaded and
/// waiting for Restart is not fetched or announced again.
fn new_release(ready: Option<&str>, found: &str) -> bool {
    ready != Some(found)
}

async fn download_desk(app: &AppHandle) -> Result<(), String> {
    let quit = app.clone();
    let updater = app
        .updater_builder()
        // On Windows the install ends the process with no RunEvent::Exit, so the
        // desk's quit runs here, after the download is checked and unpacked and
        // just before the installer starts; elsewhere the restart runs it.
        // The installer replaces the bundled Node, so the local ahpd stops too.
        .on_before_exit(move || {
            crate::before_desk_install(&quit);
            quit.cleanup_before_exit();
        })
        .build()
        .map_err(|err| err.to_string())?;
    let Some(update) = updater.check().await.map_err(|err| err.to_string())? else {
        return Ok(());
    };
    let ready = app
        .state::<DeskUpdate>()
        .lock()
        .as_ref()
        .map(|ready| ready.version.clone());
    if !new_release(ready.as_deref(), &update.version) {
        return Ok(());
    }
    let bytes = update
        .download(|_, _| {}, || {})
        .await
        .map_err(|err| err.to_string())?;
    let version = update.version.clone();
    *app.state::<DeskUpdate>().lock() = Some(Ready {
        version: version.clone(),
        update,
        bytes,
    });
    app.emit(DESK_UPDATE_READY, version)
        .map_err(|err| err.to_string())
}

/// Installs the ready desk and restarts into it; the host list's Restart comes
/// here, and the tray's item through it. A failed install keeps it ready for another try.
pub async fn install(app: AppHandle) -> Result<(), String> {
    let ready = app
        .state::<DeskUpdate>()
        .lock()
        .take()
        .ok_or("no desk update is ready")?;
    if let Err(err) = ready.update.install(&ready.bytes) {
        *app.state::<DeskUpdate>().lock() = Some(ready);
        return Err(err.to_string());
    }
    app.restart()
}

/// The ready desk's version, for a host list that opens after the event.
#[tauri::command]
pub fn desk_update(state: State<'_, DeskUpdate>) -> Option<String> {
    state.lock().as_ref().map(|ready| ready.version.clone())
}

#[tauri::command]
pub async fn install_desk_update(app: AppHandle) -> Result<(), String> {
    install(app).await
}

/// What npm said, and when, in seconds since the Unix epoch.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NpmLatest {
    pub name: String,
    pub latest: String,
    pub checked_at: u64,
}

/// What the desk knows can be updated on its hosts. The desk's own update
/// stays in plan 04's `DeskUpdate`.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdatesView {
    pub ahpd: Option<NpmLatest>,
}

/// The view and the file it is kept in.
pub struct Updates {
    file: PathBuf,
    view: Mutex<UpdatesView>,
}

impl Updates {
    /// Reads `updates.json`; a missing, broken or misshapen file is an empty
    /// view, as ahpd's `readUpdate()`.
    pub fn load(file: PathBuf) -> Updates {
        let view = std::fs::read_to_string(&file)
            .ok()
            .and_then(|text| serde_json::from_str(&text).ok())
            .unwrap_or_default();
        Updates {
            file,
            view: Mutex::new(view),
        }
    }

    fn lock(&self) -> MutexGuard<'_, UpdatesView> {
        self.view
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    pub fn view(&self) -> UpdatesView {
        self.lock().clone()
    }

    /// Keeps npm's answer and writes the file; answers whether the view changed.
    pub fn set_ahpd(&self, found: NpmLatest) -> Result<bool, String> {
        let mut view = self.lock();
        let changed = view.ahpd.as_ref().map(|ahpd| (&ahpd.name, &ahpd.latest))
            != Some((&found.name, &found.latest));
        view.ahpd = Some(found);
        write_json(&self.file, &*view)?;
        Ok(changed)
    }
}

/// The registry to ask: `npm_config_registry` without its trailing slashes,
/// else npmjs.org, as ahpd's `registry()`.
pub fn registry(named: Option<String>) -> String {
    match named.filter(|named| !named.is_empty()) {
        Some(named) => named.trim_end_matches('/').to_string(),
        None => "https://registry.npmjs.org".to_string(),
    }
}

/// Whether to ask again: no answer, or one older than `max_age`, as ahpd's `stale()`.
pub fn stale(found: Option<&NpmLatest>, now: u64, max_age: Duration) -> bool {
    match found {
        None => true,
        Some(found) => now.saturating_sub(found.checked_at) > max_age.as_secs(),
    }
}

/// rustls's `ring` provider, once: reqwest's own `rustls` feature uses
/// aws-lc-rs, which needs NASM to build on Windows, as russh's comment says.
fn tls_provider() {
    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        let _ = rustls::crypto::ring::default_provider().install_default();
    });
}

/// npm's `latest` for `name`, from `<registry>/-/package/<name>/dist-tags`.
/// Every failure is `None`: offline, a timeout, an answer that is not `ok`,
/// a body that is not JSON, as ahpd's `askRegistry()`.
///
/// A scoped name's `/` is sent as `%2f`, as npm sends it: npmjs.org takes
/// either, and a registry such as verdaccio answers the bare `/` with 404.
pub async fn ask_npm(registry: &str, name: &str) -> Option<String> {
    tls_provider();
    let client = reqwest::Client::builder()
        .timeout(ASK_TIMEOUT)
        .build()
        .ok()?;
    let answer = client
        .get(format!(
            "{registry}/-/package/{}/dist-tags",
            name.replace('/', "%2f")
        ))
        .header(reqwest::header::ACCEPT, "application/json")
        .send()
        .await
        .ok()?;
    if !answer.status().is_success() {
        return None;
    }
    let tags: serde_json::Value = answer.json().await.ok()?;
    tags["latest"].as_str().map(str::to_string)
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |since| since.as_secs())
}

/// Asks npm when the kept answer is stale, at start and then every 6 hours;
/// a failed ask leaves the file as it was until the next round. After each
/// 6 hours it checks for a newer desk too; the first round leaves that to
/// `check_on_start`, which has just asked.
pub fn watch(app: &AppHandle) {
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        loop {
            let updates = app.state::<Updates>();
            if stale(updates.view().ahpd.as_ref(), now(), MAX_AGE) {
                let asked = registry(std::env::var("npm_config_registry").ok());
                if let Some(latest) = ask_npm(&asked, AHPD_PACKAGE).await {
                    let found = NpmLatest {
                        name: AHPD_PACKAGE.to_string(),
                        latest,
                        checked_at: now(),
                    };
                    match updates.set_ahpd(found) {
                        Ok(true) => {
                            if let Err(err) = app.emit(UPDATES_CHANGED, updates.view()) {
                                eprintln!("ahp-desk: could not send {UPDATES_CHANGED}: {err}");
                            }
                        }
                        Ok(false) => {}
                        Err(err) => eprintln!("ahp-desk: could not keep npm's answer: {err}"),
                    }
                }
            }
            tokio::time::sleep(MAX_AGE).await;
            check_desk(&app).await;
        }
    });
}

#[tauri::command]
pub fn updates(state: State<'_, Updates>) -> UpdatesView {
    state.view()
}

/// Updates the ahpd the desk installed over SSH on host `id` to `version`,
/// the one its row named, with plan 10's update; its steps and lines go to
/// `events`, and it keeps running when the window goes away. One at a time
/// with the installs over SSH, whose Cancel stops it too.
#[tauri::command]
pub async fn update_ssh_ahpd(
    store: State<'_, HostStore>,
    known: State<'_, KnownKeys>,
    control: State<'_, InstallControl>,
    id: String,
    version: String,
    events: Channel<InstallEvent>,
) -> Result<(), String> {
    let _held = control.begin()?;
    let send = move |event: InstallEvent| {
        let _ = events.send(event);
    };
    ssh_install::update_host(&store, &known, &id, &version, &send, control.cancelled())
        .await
        .map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ssh::known_keys::tests::Scratch;
    use std::io::{Read, Write};
    use std::net::TcpListener;

    #[test]
    fn updates_registry_reads_npm_config_registry_without_trailing_slashes() {
        assert_eq!(registry(None), "https://registry.npmjs.org");
        assert_eq!(registry(Some(String::new())), "https://registry.npmjs.org");
        assert_eq!(
            registry(Some("https://mirror.example/npm//".into())),
            "https://mirror.example/npm"
        );
    }

    #[test]
    fn updates_a_desk_release_already_ready_is_not_new() {
        assert!(new_release(None, "0.2.0"));
        assert!(new_release(Some("0.2.0"), "0.3.0"));
        assert!(!new_release(Some("0.2.0"), "0.2.0"));
    }

    #[test]
    fn updates_stale_with_no_answer_or_one_older_than_six_hours() {
        let at = |checked_at| NpmLatest {
            name: AHPD_PACKAGE.into(),
            latest: "0.12.0".into(),
            checked_at,
        };
        let now = 1_000_000;
        assert!(stale(None, now, MAX_AGE));
        assert!(!stale(Some(&at(now - 60)), now, MAX_AGE));
        assert!(!stale(Some(&at(now - 6 * 60 * 60)), now, MAX_AGE));
        assert!(stale(Some(&at(now - 6 * 60 * 60 - 1)), now, MAX_AGE));
    }

    #[test]
    fn updates_load_a_missing_a_broken_and_a_written_file() {
        let scratch = Scratch::new();
        let file = scratch.0.join("updates.json");
        assert_eq!(Updates::load(file.clone()).view(), UpdatesView::default());
        std::fs::write(&file, "{ not json").unwrap();
        assert_eq!(Updates::load(file.clone()).view(), UpdatesView::default());
        std::fs::write(&file, r#"{"ahpd":{"name":1}}"#).unwrap();
        assert_eq!(Updates::load(file.clone()).view(), UpdatesView::default());
        let updates = Updates::load(file.clone());
        let found = NpmLatest {
            name: AHPD_PACKAGE.into(),
            latest: "0.12.0".into(),
            checked_at: 42,
        };
        assert!(updates.set_ahpd(found.clone()).unwrap());
        assert_eq!(Updates::load(file.clone()).view().ahpd, Some(found.clone()));
        // The same version asked again is no change, though the time moves.
        let later = NpmLatest {
            checked_at: 99,
            ..found
        };
        assert!(!updates.set_ahpd(later.clone()).unwrap());
        assert_eq!(Updates::load(file).view().ahpd, Some(later));
    }

    /// A registry on 127.0.0.1 that answers one request with `response`, or
    /// holds it unanswered for `hold`; answers its address and the request it read.
    fn registry_once(
        response: &'static str,
        hold: Duration,
    ) -> (String, std::thread::JoinHandle<String>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = format!("http://{}", listener.local_addr().unwrap());
        let served = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut request = [0; 2048];
            let read = stream.read(&mut request).unwrap();
            std::thread::sleep(hold);
            let _ = stream.write_all(response.as_bytes());
            String::from_utf8_lossy(&request[..read]).to_string()
        });
        (address, served)
    }

    #[tokio::test]
    async fn updates_ask_npm_reads_latest_from_dist_tags() {
        let body = r#"{"latest":"0.12.0","next":"0.13.0-beta.1"}"#;
        let response: &'static str = Box::leak(
            format!(
                "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\n\r\n{body}",
                body.len()
            )
            .into_boxed_str(),
        );
        let (address, served) = registry_once(response, Duration::ZERO);
        assert_eq!(
            ask_npm(&address, AHPD_PACKAGE).await.as_deref(),
            Some("0.12.0")
        );
        let request = served.join().unwrap();
        assert!(
            request.starts_with("GET /-/package/@ahpd%2fserver/dist-tags "),
            "{request}"
        );
        assert!(
            request.to_lowercase().contains("accept: application/json"),
            "{request}"
        );
    }

    #[tokio::test]
    async fn updates_ask_npm_is_none_for_a_404_html_or_no_answer() {
        let (address, _) = registry_once(
            "HTTP/1.1 404 Not Found\r\ncontent-length: 0\r\n\r\n",
            Duration::ZERO,
        );
        assert_eq!(ask_npm(&address, AHPD_PACKAGE).await, None);
        let html = "<html>captive portal</html>";
        let response: &'static str = Box::leak(
            format!(
                "HTTP/1.1 200 OK\r\ncontent-type: text/html\r\ncontent-length: {}\r\n\r\n{html}",
                html.len()
            )
            .into_boxed_str(),
        );
        let (address, _) = registry_once(response, Duration::ZERO);
        assert_eq!(ask_npm(&address, AHPD_PACKAGE).await, None);
        let (address, _) = registry_once("", Duration::from_secs(6));
        let started = std::time::Instant::now();
        assert_eq!(ask_npm(&address, AHPD_PACKAGE).await, None);
        assert!(
            started.elapsed() < Duration::from_secs(6),
            "the timeout is 5 s"
        );
    }
}
