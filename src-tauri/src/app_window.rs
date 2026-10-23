//! The app windows: one host each, or with the setting "One window with every
//! host" one window for them all, showing the build's UI in a window with no OS
//! frame, since the UI draws its own title bar.

use std::collections::HashMap;
use std::sync::{Mutex, MutexGuard};

use tauri::webview::NewWindowResponse;
use tauri::{
    AppHandle, Emitter, Manager, State, Url, WebviewUrl, WebviewWindow, WebviewWindowBuilder,
    WindowEvent,
};
use tauri_plugin_opener::OpenerExt;

use crate::hosts::{HostStore, HostView};
use crate::settings::{SettingsStore, Windows};
use crate::ui_source::{self, UiSource};
use crate::SHELL_WINDOW;

/// The script that hands ahpw its embedder, `globalThis.ahpwEmbedder`, before
/// the page's own scripts run (decision `ahpw-owns-its-embedding-contract`).
const EMBEDDER: &str = include_str!(env!("AHP_DESK_EMBEDDER"));

/// The page an app window loads for `host`: the UI with the host's address as
/// `?host=`. Every host shares the UI's origin, so the embedder hands ahpw this
/// address before it runs, and ahpw keeps the host's pages under it (decision
/// `ahpw-keeps-a-hosts-pages-under-the-hosts-address`).
fn page_url(source: &UiSource, host: &HostView) -> Url {
    let mut url = ui_source::ui_url(source);
    url.query_pairs_mut().clear().append_pair("host", &host.url);
    url
}

/// The label of the window with every host: `app-` so the UI's capability
/// covers it, and a `:` that `label_for` never writes, so no host's window
/// takes it.
pub const EVERY_WINDOW: &str = "app-:every";

/// The event that asks the window with every host to show one host, with its id.
pub const FOCUS_HOST: &str = "focus-host";

/// The page of the window with every host: the UI with `?hosts=every`, and the
/// host it opens on as `focus=`, since the event would come before the page listens.
fn every_url(source: &UiSource, focus: &str) -> Url {
    let mut url = ui_source::ui_url(source);
    url.query_pairs_mut()
        .clear()
        .append_pair("hosts", "every")
        .append_pair("focus", focus);
    url
}

/// Which host each app window shows, by window label; the window with every
/// host is listed with the host it was opened on.
#[derive(Default)]
pub struct AppWindows(Mutex<HashMap<String, String>>);

impl AppWindows {
    fn lock(&self) -> MutexGuard<'_, HashMap<String, String>> {
        self.0
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// The host the window `label` shows; for the window with every host, the one it was opened on.
    pub fn host_of(&self, label: &str) -> Option<String> {
        self.lock().get(label).cloned()
    }

    /// True while the window with every host is open.
    fn every_open(&self) -> bool {
        self.lock().contains_key(EVERY_WINDOW)
    }

    /// Closes every window that shows only `host_id`, as when the host is
    /// removed; the window with every host stays, and drops it from its list.
    pub fn close_host(&self, app: &AppHandle, host_id: &str) {
        let labels: Vec<String> = self
            .lock()
            .iter()
            .filter(|(label, host)| host.as_str() == host_id && label.as_str() != EVERY_WINDOW)
            .map(|(label, _)| label.clone())
            .collect();
        for label in labels {
            if let Some(window) = app.get_webview_window(&label) {
                // A window that will not close keeps showing a host that is gone; the host itself is removed either way.
                let _ = window.close();
            }
        }
    }

    /// The label of a window that shows only `host_id`.
    fn window_of(&self, host_id: &str) -> Option<String> {
        self.lock()
            .iter()
            .find(|(label, host)| host.as_str() == host_id && label.as_str() != EVERY_WINDOW)
            .map(|(label, _)| label.clone())
    }

    /// True while any app window is open, the window with every host among them.
    pub fn any_open(&self) -> bool {
        !self.lock().is_empty()
    }

    /// True when the window `label` may read `host_id`'s token: the host list,
    /// the window with every host, or the host's own window.
    pub fn may_read(&self, label: &str, host_id: &str) -> bool {
        label == SHELL_WINDOW
            || (label == EVERY_WINDOW && self.every_open())
            || self.host_of(label).as_deref() == Some(host_id)
    }
}

/// A window label for a host: `app-` and the id, with anything a label cannot hold
/// as `_`, and a number after it when a window that switched hosts holds the name.
fn label_for(app: &AppHandle, host_id: &str) -> String {
    let safe: String = host_id
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
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
/// the calling app window switches to the host instead. With "One window with
/// every host", it shows the window with every host, which then shows `host_id`.
///
/// Async, so the window is built off the main thread; a sync command that builds
/// a window can deadlock on Windows.
#[allow(clippy::too_many_arguments)]
#[tauri::command]
pub async fn open_app_window(
    app: AppHandle,
    window: WebviewWindow,
    windows: State<'_, AppWindows>,
    store: State<'_, HostStore>,
    source: State<'_, UiSource>,
    settings: State<'_, SettingsStore>,
    host_id: String,
    replace: bool,
) -> Result<(), String> {
    let host = store
        .find(&host_id)
        .ok_or_else(|| format!("no host {host_id}"))?;

    // The window with every host never becomes one host's: it shows the host instead.
    let every = window.label() == EVERY_WINDOW;
    if every || (!replace && settings.get().windows == Windows::EveryHost) {
        if let Some(open) = app.get_webview_window(EVERY_WINDOW) {
            show(&open)?;
            app.emit_to(EVERY_WINDOW, FOCUS_HOST, &host.id)
                .map_err(|err| err.to_string())?;
            return hide_if_host_list(&window);
        }
        let page = every_url(source.inner(), &host.id);
        build_app_window(&app, source.inner(), EVERY_WINDOW, page, "ahp-desk")?;
        windows.lock().insert(EVERY_WINDOW.to_string(), host.id);
        return hide_if_host_list(&window);
    }

    if replace && windows.host_of(window.label()).is_some() {
        window
            .set_title(&host.name)
            .map_err(|err| err.to_string())?;
        // A navigation, not a reload: the page's address names the new host.
        let page = page_url(source.inner(), &host);
        windows.lock().insert(window.label().to_string(), host.id);
        return window.navigate(page).map_err(|err| err.to_string());
    }

    if let Some(open) = windows
        .window_of(&host.id)
        .and_then(|label| app.get_webview_window(&label))
    {
        show(&open)?;
        return hide_if_host_list(&window);
    }

    let label = label_for(&app, &host.id);
    let page = page_url(source.inner(), &host);
    build_app_window(&app, source.inner(), &label, page, &host.name)?;
    windows.lock().insert(label, host.id);
    hide_if_host_list(&window)
}

/// Builds an app window on `page`; when it is gone, it leaves `AppWindows`,
/// and the last one gone shows the host list again.
fn build_app_window(
    app: &AppHandle,
    source: &UiSource,
    label: &str,
    page: Url,
    title: &str,
) -> Result<(), String> {
    let ui = ui_source::ui_url(source);
    let url = match source {
        UiSource::DevServer(_) => WebviewUrl::External(page),
        UiSource::Folder(_) => WebviewUrl::CustomProtocol(page),
    };
    let (for_new, for_nav) = (app.clone(), app.clone());
    let opened = WebviewWindowBuilder::new(app, label, url)
        .title(title)
        // No OS frame, and no OS shadow, which on Windows brings the accent
        // border; the UI draws a rounded root instead.
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
            if is_ui(url, &ui) {
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

    let handle = app.clone();
    let label = label.to_string();
    opened.on_window_event(move |event| {
        if let WindowEvent::Destroyed = event {
            let windows = handle.state::<AppWindows>();
            windows.lock().remove(&label);
            // The last host closed: the host list comes back, or the desk would
            // run with no window to reach it.
            if !windows.any_open() {
                if let Some(shell) = handle.get_webview_window(SHELL_WINDOW) {
                    // A host list that will not show leaves the tray; nothing is lost.
                    let _ = show(&shell);
                }
            }
        }
    });
    Ok(())
}

/// Whether `url` is a page of the UI: its scheme, host and port.
/// Not `Url::origin`, which is opaque for a custom scheme such as Linux's
/// `ahpw://localhost/`, and two opaque origins are never equal, so the
/// window refused even its first page and stayed empty.
fn is_ui(url: &Url, ui: &Url) -> bool {
    url.scheme() == ui.scheme()
        && url.host_str() == ui.host_str()
        && url.port_or_known_default() == ui.port_or_known_default()
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

/// Opening a host from the host list hides the list; it keeps running, with every host's status.
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

/// The event that asks the host list to open a host's form, with the host's id.
pub const HOST_LIST_EDIT: &str = "host-list-edit";

/// The event that asks the host list to open Settings, as the tray's Settings item does.
pub const HOST_LIST_SETTINGS: &str = "host-list-settings";

/// The event that asks the host list to install the ready desk update, as the
/// tray's item does: the host list asks first over turns running on a local ahpd.
pub const HOST_LIST_DESK_UPDATE: &str = "host-list-desk-update";

/// Shows and focuses the host list; with `add`, on its Add form, and with
/// `edit`, on that host's form, as ahpw's Sign in on a host that refuses its token.
#[tauri::command]
pub fn show_host_list(
    app: AppHandle,
    add: Option<bool>,
    edit: Option<String>,
) -> Result<(), String> {
    let shell = app
        .get_webview_window(SHELL_WINDOW)
        .ok_or("the host list window is gone")?;
    show(&shell)?;
    if add == Some(true) {
        app.emit_to(SHELL_WINDOW, HOST_LIST_ADD, ())
            .map_err(|err| err.to_string())?;
    } else if let Some(id) = edit {
        app.emit_to(SHELL_WINDOW, HOST_LIST_EDIT, id)
            .map_err(|err| err.to_string())?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn url(text: &str) -> Url {
        Url::parse(text).unwrap()
    }

    #[test]
    fn is_ui_takes_a_page_of_the_linux_scheme() {
        let ui = url("ahpw://localhost/");
        assert!(is_ui(
            &url("ahpw://localhost/?host=ws%3A%2F%2F127.0.0.1%3A9187"),
            &ui
        ));
        assert!(is_ui(&url("ahpw://localhost/assets/index.js"), &ui));
    }

    #[test]
    fn is_ui_takes_a_page_of_the_webview2_address() {
        let ui = url("http://ahpw.localhost/");
        assert!(is_ui(
            &url("http://ahpw.localhost/?hosts=every&focus=a"),
            &ui
        ));
    }

    #[test]
    fn is_ui_refuses_any_other_site() {
        let ui = url("ahpw://localhost/");
        assert!(!is_ui(&url("https://localhost/"), &ui));
        assert!(!is_ui(&url("ahpw://other/"), &ui));
        let ui = url("http://ahpw.localhost/");
        assert!(!is_ui(&url("http://ahpw.localhost.example.com/"), &ui));
        assert!(!is_ui(&url("http://ahpw.localhost:8080/"), &ui));
        assert!(!is_ui(&url("https://ahpw.localhost/"), &ui));
    }

    fn windows(open: &[(&str, &str)]) -> AppWindows {
        let windows = AppWindows::default();
        for (label, host) in open {
            windows.lock().insert(label.to_string(), host.to_string());
        }
        windows
    }

    #[test]
    fn may_read_the_host_list_reads_every_token() {
        assert!(windows(&[]).may_read(SHELL_WINDOW, "a"));
    }

    #[test]
    fn may_read_the_window_with_every_host_reads_every_token() {
        let open = windows(&[(EVERY_WINDOW, "a")]);
        assert!(open.may_read(EVERY_WINDOW, "a"));
        assert!(open.may_read(EVERY_WINDOW, "b"));
    }

    #[test]
    fn may_read_a_window_of_one_host_reads_only_its_own() {
        let open = windows(&[("app-a", "a"), (EVERY_WINDOW, "a")]);
        assert!(open.may_read("app-a", "a"));
        assert!(!open.may_read("app-a", "b"));
    }

    #[test]
    fn may_read_the_every_label_reads_nothing_while_that_window_is_closed() {
        assert!(!windows(&[]).may_read(EVERY_WINDOW, "a"));
    }

    #[test]
    fn window_of_skips_the_window_with_every_host() {
        let open = windows(&[(EVERY_WINDOW, "a")]);
        assert_eq!(open.window_of("a"), None);
        assert!(open.any_open());
        assert_eq!(open.host_of(EVERY_WINDOW).as_deref(), Some("a"));
    }

    #[test]
    fn every_window_label_is_one_no_host_window_takes() {
        assert!(EVERY_WINDOW.starts_with("app-"));
        assert!(EVERY_WINDOW.contains(':'));
    }
}
