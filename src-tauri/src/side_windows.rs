//! The side windows: small fixed-size pages of the shell's own bundle, opened
//! beside the host list rather than in place of its screen, so the host list
//! keeps what it shows (decision `about-is-a-window-of-its-own`).

use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};

use crate::frame;

/// The label of the About window.
pub const ABOUT_WINDOW: &str = "about";

/// Opens the side window `label` on the shell's page `#view`, or brings it to
/// the front when it is open: frameless, as the host list, and of one size.
fn open_side(app: &AppHandle, label: &str, view: &str, size: (f64, f64)) -> tauri::Result<()> {
    if let Some(window) = app.get_webview_window(label) {
        window.unminimize()?;
        window.show()?;
        return window.set_focus();
    }
    let window = WebviewWindowBuilder::new(
        app,
        label,
        WebviewUrl::App(format!("index.html#{view}").into()),
    )
    .title("ahp-desk")
    .decorations(false)
    .transparent(true)
    .shadow(false)
    .inner_size(size.0, size.1)
    .resizable(false)
    .maximizable(false)
    .center()
    .build()?;
    frame::round_corners(&window);
    Ok(())
}

/// Opens About, as the tray's item does; a failure is only logged, as the tray has nowhere to show it.
pub fn open_about(app: &AppHandle) {
    if let Err(err) = open_side(app, ABOUT_WINDOW, "about", (360.0, 320.0)) {
        eprintln!("ahp-desk: About did not open: {err}");
    }
}

/// Opens About from the host list, as Settings' link and the logo menu do.
// Async: on Windows a window built in a synchronous command deadlocks.
#[tauri::command]
pub async fn open_about_window(app: AppHandle) -> Result<(), String> {
    open_side(&app, ABOUT_WINDOW, "about", (360.0, 320.0)).map_err(|err| err.to_string())
}
