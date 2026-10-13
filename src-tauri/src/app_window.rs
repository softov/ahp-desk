//! The app windows: one host each, showing the build's UI in a window with no
//! OS frame, since the UI draws its own title bar.

use std::collections::HashMap;
use std::sync::{Mutex, MutexGuard};

use tauri::webview::NewWindowResponse;
use tauri::{
    AppHandle, Emitter, Manager, State, Url, WebviewUrl, WebviewWindow, WebviewWindowBuilder,
    WindowEvent,
};
use tauri_plugin_opener::OpenerExt;

use crate::hosts::{HostStore, HostView};
use crate::ui_source::{self, UiSource};
use crate::SHELL_WINDOW;

/// The script that hands ahpw its embedder, `globalThis.ahpwEmbedder`, before
/// the page's own scripts run (decision: ahpw owns its embedding contract).
const EMBEDDER: &str = include_str!(env!("AHP_DESK_EMBEDDER"));

/// The page an app window loads for `host`: the UI with the host's address as
/// `?host=`. Every host shares the UI's origin, so the embedder hands ahpw this
/// address before it runs, and ahpw keeps the host's pages under it (decision:
/// ahpw keeps a host's pages under the host's address).
fn page_url(source: &UiSource, host: &HostView) -> Url {
    let mut url = ui_source::ui_url(source);
    url.query_pairs_mut().clear().append_pair("host", &host.url);
    url
}

/// Which host each app window shows, by window label.
#[derive(Default)]
pub struct AppWindows(Mutex<HashMap<String, String>>);

impl AppWindows {
    fn lock(&self) -> MutexGuard<'_, HashMap<String, String>> {
        self.0.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// The host the window `label` shows.
    pub fn host_of(&self, label: &str) -> Option<String> {
        self.lock().get(label).cloned()
    }

    /// Closes every window that shows `host_id`, as when the host is removed.
    pub fn close_host(&self, app: &AppHandle, host_id: &str) {
        let labels: Vec<String> = self
            .lock()
            .iter()
            .filter(|(_, host)| host.as_str() == host_id)
            .map(|(label, _)| label.clone())
            .collect();
        for label in labels {
            if let Some(window) = app.get_webview_window(&label) {
                // A window that will not close keeps showing a host that is gone; the host itself is removed either way.
                let _ = window.close();
            }
        }
    }

    /// The label of a window that shows `host_id`.
    fn window_of(&self, host_id: &str) -> Option<String> {
        self.lock()
            .iter()
            .find(|(_, host)| host.as_str() == host_id)
            .map(|(label, _)| label.clone())
    }

    /// True while any app window is open.
    pub fn any_open(&self) -> bool {
        !self.lock().is_empty()
    }

    /// True when the window `label` may read `host_id`'s token: the host list, or the host's own window.
    pub fn may_read(&self, label: &str, host_id: &str) -> bool {
        label == SHELL_WINDOW || self.host_of(label).as_deref() == Some(host_id)
    }
}

/// A window label for a host: `app-` and the id, with anything a label cannot hold
/// as `_`, and a number after it when a window that switched hosts holds the name.
fn label_for(app: &AppHandle, host_id: &str) -> String {
    let safe: String = host_id
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .collect();
    let base = format!("app-{safe}");
    let mut label = base.clone();
    let mut n = 2;
    while app.get_webview_window(&label).is_some() {
        label = format!("{base}-{n}");
        n += 1;
    }
    label
}

fn show(window: &WebviewWindow) -> Result<(), String> {
    window.show().map_err(|err| err.to_string())?;
    window.unminimize().map_err(|err| err.to_string())?;
    window.set_focus().map_err(|err| err.to_string())
}

/// Opens `host_id` in a window, or focuses the one that shows it. With `replace`,
/// the calling app window switches to the host instead.
///
/// Async, so the window is built off the main thread; a sync command that builds
/// a window can deadlock on Windows.
#[tauri::command]
pub async fn open_app_window(
    app: AppHandle,
    window: WebviewWindow,
    windows: State<'_, AppWindows>,
    store: State<'_, HostStore>,
    source: State<'_, UiSource>,
    host_id: String,
    replace: bool,
) -> Result<(), String> {
    let host = store.find(&host_id).ok_or_else(|| format!("no host {host_id}"))?;

    if replace && windows.host_of(window.label()).is_some() {
        window.set_title(&host.name).map_err(|err| err.to_string())?;
        // A navigation, not a reload: the page's address names the new host.
        let page = page_url(source.inner(), &host);
        windows.lock().insert(window.label().to_string(), host.id);
        return window.navigate(page).map_err(|err| err.to_string());
    }

    if let Some(open) = windows.window_of(&host.id).and_then(|label| app.get_webview_window(&label)) {
        show(&open)?;
        return hide_if_host_list(&window);
    }

    let label = label_for(&app, &host.id);
    let ui = ui_source::ui_url(source.inner());
    let page = page_url(source.inner(), &host);
    let url = match source.inner() {
        UiSource::DevServer(_) => WebviewUrl::External(page),
        UiSource::Folder(_) => WebviewUrl::CustomProtocol(page),
    };
    let (for_new, for_nav) = (app.clone(), app.clone());
    let opened = WebviewWindowBuilder::new(&app, &label, url)
        .title(&host.name)
        // No OS frame, and no OS shadow, which on Windows brings the accent
        // border; the UI draws a rounded root instead (decision: the app
        // window draws its own rounded frame).
        .decorations(false)
        .transparent(true)
        .shadow(false)
        .initialization_script(EMBEDDER)
        // Tauri's own drop handler replaces WebView2's, and with it on, Windows sends
        // the page no HTML5 drag events: ahpw's explorer could not drag a file into a chat.
        .disable_drag_drop_handler()
        // A web link opens in the browser, as in VS Code: a new window
        // (target=_blank, window.open) never opens inside the desk, and the
        // window itself never leaves the UI for a site.
        .on_new_window(move |url, _| {
            open_outside(&for_new, &url);
            NewWindowResponse::Deny
        })
        .on_navigation(move |url| {
            if url.origin() == ui.origin() {
                return true;
            }
            open_outside(&for_nav, url);
            false
        })
        .inner_size(1280.0, 860.0)
        .min_inner_size(600.0, 400.0)
        .build()
        .map_err(|err| err.to_string())?;
    crate::frame::round_corners(&opened);
    windows.lock().insert(label.clone(), host.id);

    let handle = app.clone();
    opened.on_window_event(move |event| {
        if let WindowEvent::Destroyed = event {
            let windows = handle.state::<AppWindows>();
            windows.lock().remove(&label);
            // The last host closed: the host list comes back, or the desk would
            // run with no window to reach it (decision: open hides the host list).
            if !windows.any_open() {
                if let Some(shell) = handle.get_webview_window(SHELL_WINDOW) {
                    // A host list that will not show leaves the tray; nothing is lost.
                    let _ = show(&shell);
                }
            }
        }
    });
    hide_if_host_list(&window)
}

/// Opens a web or mail link in the OS's own app for it; any other scheme is dropped.
fn open_outside(app: &AppHandle, url: &Url) {
    if !matches!(url.scheme(), "http" | "https" | "mailto") {
        return;
    }
    if let Err(err) = app.opener().open_url(url.as_str(), None::<&str>) {
        eprintln!("opening {url} outside the desk failed: {err}");
    }
}

/// Opening a host from the host list hides the list (decision: open hides the host list).
fn hide_if_host_list(caller: &WebviewWindow) -> Result<(), String> {
    if caller.label() != SHELL_WINDOW {
        return Ok(());
    }
    caller.hide().map_err(|err| err.to_string())
}

/// The host the calling window shows.
#[tauri::command]
pub fn window_host(
    window: WebviewWindow,
    windows: State<'_, AppWindows>,
    store: State<'_, HostStore>,
) -> Result<HostView, String> {
    let id = windows
        .host_of(window.label())
        .ok_or_else(|| format!("window {} shows no host", window.label()))?;
    store.find(&id).ok_or_else(|| format!("no host {id}"))
}

/// The event that asks the host list to open its Add form.
pub const HOST_LIST_ADD: &str = "host-list-add";

/// Shows and focuses the host list; with `add`, on its Add form.
#[tauri::command]
pub fn show_host_list(app: AppHandle, add: Option<bool>) -> Result<(), String> {
    let shell = app
        .get_webview_window(SHELL_WINDOW)
        .ok_or("the host list window is gone")?;
    show(&shell)?;
    if add == Some(true) {
        app.emit_to(SHELL_WINDOW, HOST_LIST_ADD, ())
            .map_err(|err| err.to_string())?;
    }
    Ok(())
}
