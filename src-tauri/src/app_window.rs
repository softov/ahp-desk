//! The app windows: one host each, showing the build's UI in a window with no
//! OS frame, since the UI draws its own title bar.

use std::collections::HashMap;
use std::sync::{Mutex, MutexGuard};

use tauri::{
    AppHandle, Manager, State, WebviewUrl, WebviewWindow, WebviewWindowBuilder, WindowEvent,
};

use crate::hosts::{HostStore, HostView};
use crate::ui_source::{self, UiSource};
use crate::SHELL_WINDOW;

/// The script that hands ahpw its embedder, `globalThis.ahpwEmbedder`, before
/// the page's own scripts run (decision: ahpw owns its embedding contract).
const EMBEDDER: &str = include_str!(env!("AHP_DESK_EMBEDDER"));

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
        windows.lock().insert(window.label().to_string(), host.id);
        window.set_title(&host.name).map_err(|err| err.to_string())?;
        return window.reload().map_err(|err| err.to_string());
    }

    if let Some(open) = windows.window_of(&host.id).and_then(|label| app.get_webview_window(&label)) {
        return show(&open);
    }

    let label = label_for(&app, &host.id);
    let url = match source.inner() {
        UiSource::DevServer(url) => WebviewUrl::External(url.clone()),
        folder @ UiSource::Folder(_) => WebviewUrl::CustomProtocol(ui_source::ui_url(folder)),
    };
    let opened = WebviewWindowBuilder::new(&app, &label, url)
        .title(&host.name)
        // No OS frame, and no OS shadow, which on Windows brings the accent
        // border; the UI draws a rounded root instead (decision: the app
        // window draws its own rounded frame).
        .decorations(false)
        .transparent(true)
        .shadow(false)
        .initialization_script(EMBEDDER)
        .inner_size(1280.0, 860.0)
        .min_inner_size(600.0, 400.0)
        .build()
        .map_err(|err| err.to_string())?;
    crate::frame::round_corners(&opened);
    windows.lock().insert(label.clone(), host.id);

    let handle = app.clone();
    opened.on_window_event(move |event| {
        if let WindowEvent::Destroyed = event {
            handle.state::<AppWindows>().lock().remove(&label);
        }
    });
    Ok(())
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

/// Shows and focuses the host list.
#[tauri::command]
pub fn show_host_list(app: AppHandle) -> Result<(), String> {
    let shell = app
        .get_webview_window(SHELL_WINDOW)
        .ok_or("the host list window is gone")?;
    show(&shell)
}
