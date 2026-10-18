//! What can be updated on the desk's hosts: npm's latest `@ahpd/server`,
//! asked at most every 6 hours and kept in `updates.json`, as ahpd's own
//! `update.ts` asks and keeps it. Plan 04's desk update joins it here.

use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard, Once};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager, State};

use crate::hosts::write_json;

/// The event each change sends, with the whole `UpdatesView`.
pub const UPDATES_CHANGED: &str = "updates-changed";
/// The package asked about.
pub const AHPD_PACKAGE: &str = "@ahpd/server";
/// How long an answer is trusted, as ahpd's `MAX_AGE_MS`.
pub const MAX_AGE: Duration = Duration::from_secs(6 * 60 * 60);
/// How long npm gets to answer, as ahpd's `askRegistry`.
const ASK_TIMEOUT: Duration = Duration::from_secs(5);

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
pub async fn ask_npm(registry: &str, name: &str) -> Option<String> {
    tls_provider();
    let client = reqwest::Client::builder()
        .timeout(ASK_TIMEOUT)
        .build()
        .ok()?;
    let answer = client
        .get(format!("{registry}/-/package/{name}/dist-tags"))
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
/// a failed ask leaves the file as it was until the next round.
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
        }
    });
}

#[tauri::command]
pub fn updates(state: State<'_, Updates>) -> UpdatesView {
    state.view()
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
            request.starts_with("GET /-/package/@ahpd/server/dist-tags "),
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
