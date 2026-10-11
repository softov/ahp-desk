//! The build's one UI and where its files come from.
//!
//! The UI is served from its own URI scheme, `ahpw://`, so its storage and its
//! origin are its own. A debug build reads the published `@ahpd/web` from
//! `node_modules`, or with `AHP_DESK_AHPW=local` loads ahpw's dev server
//! instead. A release build reads the copy bundled in its resources.

use std::path::{Path, PathBuf};

use tauri::http::{header, Request, Response, StatusCode};
use tauri::{AppHandle, Manager, Runtime, Url, UriSchemeContext, UriSchemeResponder};

/// The build's UI, chosen by `AHP_DESK_UI` in `build.rs`. It names the URI scheme too.
pub const UI: &str = env!("AHP_DESK_UI");

/// ahpw's dev server, as its `vite.config.ts` sets it.
const AHPW_DEV_SERVER: &str = "http://127.0.0.1:5180/";

/// Where the UI's pages come from.
#[derive(Clone, Debug)]
pub enum UiSource {
    /// A dev server the window loads directly.
    DevServer(Url),
    /// A built bundle the scheme serves.
    Folder(PathBuf),
}

/// The source for this build, found once when the app starts.
pub fn resolve<R: Runtime>(app: &AppHandle<R>) -> Result<UiSource, String> {
    if cfg!(debug_assertions) {
        if env!("AHP_DESK_AHPW") == "local" {
            return Ok(UiSource::DevServer(Url::parse(AHPW_DEV_SERVER).unwrap()));
        }
        return folder(PathBuf::from(env!("AHP_DESK_AHPW_NPM_DIR")));
    }
    let resources = app.path().resource_dir().map_err(|err| err.to_string())?;
    folder(resources.join("ui").join(UI))
}

fn folder(dir: PathBuf) -> Result<UiSource, String> {
    if !dir.join("index.html").is_file() {
        return Err(format!(
            "{UI} is not at {}: run `pnpm install`, or `pnpm stage:ui` before a release build",
            dir.display()
        ));
    }
    Ok(UiSource::Folder(dir))
}

/// The address an app window loads.
pub fn ui_url(source: &UiSource) -> Url {
    match source {
        UiSource::DevServer(url) => url.clone(),
        // WebView2 and Android serve a custom scheme as http://<scheme>.localhost.
        UiSource::Folder(_) if cfg!(any(windows, target_os = "android")) => {
            Url::parse(&format!("http://{UI}.localhost/")).unwrap()
        }
        UiSource::Folder(_) => Url::parse(&format!("{UI}://localhost/")).unwrap(),
    }
}

/// Answers one request of the scheme from the bundle, off the main thread.
pub fn handle<R: Runtime>(
    source: UiSource,
    _ctx: UriSchemeContext<'_, R>,
    request: Request<Vec<u8>>,
    responder: UriSchemeResponder,
) {
    let path = request.uri().path().to_string();
    tauri::async_runtime::spawn_blocking(move || {
        responder.respond(match &source {
            UiSource::Folder(root) => serve(root, &path),
            UiSource::DevServer(_) => answer(StatusCode::NOT_FOUND, "text/plain", b"Not found".to_vec()),
        });
    });
}

fn serve(root: &Path, path: &str) -> Response<Vec<u8>> {
    let Some(file) = file_for(root, path) else {
        return answer(StatusCode::NOT_FOUND, "text/plain", b"Not found".to_vec());
    };
    match std::fs::read(&file) {
        Ok(body) => answer(StatusCode::OK, content_type(&file), body),
        Err(_) => answer(StatusCode::NOT_FOUND, "text/plain", b"Not found".to_vec()),
    }
}

fn answer(status: StatusCode, content_type: &str, body: Vec<u8>) -> Response<Vec<u8>> {
    Response::builder()
        .status(status)
        .header(header::CONTENT_TYPE, content_type)
        .header(header::X_CONTENT_TYPE_OPTIONS, "nosniff")
        .body(body)
        .unwrap()
}

/// The file a request path names under `root`: the file itself, a folder's
/// `index.html`, or the UI's own `index.html` for a route the page handles.
/// `None` for a path that would leave `root`.
fn file_for(root: &Path, path: &str) -> Option<PathBuf> {
    let mut file = root.to_path_buf();
    for part in path.split('/').filter(|part| !part.is_empty()) {
        let part = decode(part)?;
        if part == "." || part == ".." || part.contains(['/', '\\', ':', '\0']) {
            return None;
        }
        file.push(part);
    }
    if file.is_dir() {
        file.push("index.html");
    }
    if file.is_file() {
        Some(file)
    } else {
        Some(root.join("index.html"))
    }
}

/// `%XX` escapes decoded; `None` when they are broken or not UTF-8.
fn decode(part: &str) -> Option<String> {
    let bytes = part.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' {
            let hex = std::str::from_utf8(bytes.get(i + 1..i + 3)?).ok()?;
            out.push(u8::from_str_radix(hex, 16).ok()?);
            i += 3;
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }
    String::from_utf8(out).ok()
}

fn content_type(file: &Path) -> &'static str {
    match file.extension().and_then(|ext| ext.to_str()).unwrap_or("") {
        "html" => "text/html; charset=utf-8",
        "js" | "mjs" => "text/javascript; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "json" | "map" => "application/json",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        "ico" => "image/x-icon",
        "woff2" => "font/woff2",
        "woff" => "font/woff",
        "wasm" => "application/wasm",
        "txt" => "text/plain; charset=utf-8",
        _ => "application/octet-stream",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn bundle() -> PathBuf {
        let root = std::env::temp_dir().join(format!("ahp-desk-ui-{}", std::process::id()));
        fs::create_dir_all(root.join("assets")).unwrap();
        fs::write(root.join("index.html"), "<html>").unwrap();
        fs::write(root.join("assets/app.js"), "x").unwrap();
        root
    }

    #[test]
    fn ui_source_serves_a_file_and_a_folder_index() {
        let root = bundle();
        assert_eq!(file_for(&root, "/assets/app.js"), Some(root.join("assets/app.js")));
        assert_eq!(file_for(&root, "/"), Some(root.join("index.html")));
        assert_eq!(file_for(&root, "/assets%2Fapp.js"), None);
    }

    #[test]
    fn ui_source_serves_the_page_for_a_route_it_handles() {
        let root = bundle();
        assert_eq!(file_for(&root, "/sessions/42"), Some(root.join("index.html")));
    }

    #[test]
    fn ui_source_never_leaves_the_bundle() {
        let root = bundle();
        assert_eq!(file_for(&root, "/../secret"), None);
        assert_eq!(file_for(&root, "/assets/%2e%2e/%2e%2e/secret"), None);
        assert_eq!(file_for(&root, "/C:%5Cwindows"), None);
        assert_eq!(file_for(&root, "/bad%zz"), None);
    }

    #[test]
    fn ui_source_names_the_content_type() {
        assert_eq!(content_type(Path::new("a.js")), "text/javascript; charset=utf-8");
        assert_eq!(content_type(Path::new("a.unknown")), "application/octet-stream");
    }
}
