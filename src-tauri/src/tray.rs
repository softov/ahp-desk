//! The tray icon and its menu. The words live in the shell's dictionaries, so
//! the menu is set when the host list sends them, and again on each language pick.

use serde::Deserialize;
use tauri::menu::{Menu, MenuEvent, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter};

use crate::app_window::{HOST_LIST_HOSTS, HOST_LIST_SETTINGS};
use crate::side_windows;
use crate::SHELL_WINDOW;

pub const TRAY_ID: &str = "ahp-desk";
const MENU_UPDATE: &str = "tray.update";
const MENU_HOSTS: &str = "tray.hosts";
const MENU_SETTINGS: &str = "tray.settings";
const MENU_CHECK: &str = "tray.check";
const MENU_ABOUT: &str = "tray.about";
const MENU_QUIT: &str = "tray.quit";

/// The menu's words, in the language the host list shows.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrayWords {
    pub hosts: String,
    pub settings: String,
    pub check: String,
    pub about: String,
    pub quit: String,
    /// The desk update's words, while one is ready.
    pub update: Option<String>,
}

/// The menu's items, in order, as ids and words.
fn entries(words: &TrayWords) -> Vec<(&'static str, String)> {
    let update = words.update.iter().map(|text| (MENU_UPDATE, text.clone()));
    update
        .chain([
            (MENU_HOSTS, words.hosts.clone()),
            (MENU_SETTINGS, words.settings.clone()),
            (MENU_CHECK, words.check.clone()),
            (MENU_ABOUT, words.about.clone()),
            (MENU_QUIT, words.quit.clone()),
        ])
        .collect()
}

/// The icon, with no menu until the host list sends its words.
pub fn install(app: &AppHandle) -> tauri::Result<()> {
    let mut tray = TrayIconBuilder::with_id(TRAY_ID)
        .tooltip("ahp-desk")
        .show_menu_on_left_click(false)
        .on_menu_event(on_menu_event)
        .on_tray_icon_event(on_tray_icon_event);
    if let Some(icon) = app.default_window_icon() {
        tray = tray.icon(icon.clone());
    }
    tray.build(app)?;
    Ok(())
}

/// Whether the desktop shows the tray icon now.
/// On Linux the icon is a StatusNotifierItem, shown only while a host such as
/// GNOME's AppIndicator extension is registered with the watcher; stock GNOME
/// has none, so no watcher answers, which is a "no", as is a session bus that
/// cannot be reached, since the icon needs it too.
pub fn shown() -> bool {
    #[cfg(target_os = "linux")]
    return status_notifier_host_registered().unwrap_or(false);
    #[cfg(not(target_os = "linux"))]
    true
}

#[cfg(target_os = "linux")]
fn status_notifier_host_registered() -> zbus::Result<bool> {
    let bus = zbus::blocking::Connection::session()?;
    let watcher = zbus::blocking::Proxy::new(
        &bus,
        "org.kde.StatusNotifierWatcher",
        "/StatusNotifierWatcher",
        "org.kde.StatusNotifierWatcher",
    )?;
    watcher.get_property("IsStatusNotifierHostRegistered")
}

/// Sets the menu in the host list's language. Only the host list may call it.
#[tauri::command]
pub fn set_tray_words(app: AppHandle, words: TrayWords) -> Result<(), String> {
    let items = entries(&words)
        .into_iter()
        .map(|(id, text)| MenuItem::with_id(&app, id, text, true, None::<&str>))
        .collect::<tauri::Result<Vec<_>>>()
        .map_err(|err| err.to_string())?;
    let refs: Vec<&dyn tauri::menu::IsMenuItem<_>> = items
        .iter()
        .map(|item| item as &dyn tauri::menu::IsMenuItem<_>)
        .collect();
    let menu = Menu::with_items(&app, &refs).map_err(|err| err.to_string())?;
    let tray = app.tray_by_id(TRAY_ID).ok_or("the tray icon is gone")?;
    tray.set_menu(Some(menu)).map_err(|err| err.to_string())
}

/// Quits the desk, as the tray's Quit does: the title bar's logo menu asks for it.
#[tauri::command]
pub fn quit_desk(app: AppHandle) {
    app.exit(0);
}

fn on_menu_event(app: &AppHandle, event: MenuEvent) {
    match event.id().as_ref() {
        // The Updates window shows the ready update, and its Restart names the
        // turns a local ahpd's stop would end.
        MENU_UPDATE | MENU_CHECK => side_windows::open_updates(app),
        MENU_HOSTS => {
            show_host_list(app);
            if let Err(err) = app.emit_to(SHELL_WINDOW, HOST_LIST_HOSTS, ()) {
                eprintln!("ahp-desk: the tray could not show the host list: {err}");
            }
        }
        MENU_SETTINGS => {
            show_host_list(app);
            if let Err(err) = app.emit_to(SHELL_WINDOW, HOST_LIST_SETTINGS, ()) {
                eprintln!("ahp-desk: the tray could not open Settings: {err}");
            }
        }
        // Beside the host list, which keeps its screen.
        MENU_ABOUT => side_windows::open_about(app),
        // Through RunEvent::Exit, so the local ahpd's stop on quit runs as on a close.
        MENU_QUIT => app.exit(0),
        _ => {}
    }
}

/// A left click shows the host list on Windows; the right click opens the
/// menu. Linux's appindicator sends no clicks, so there the menu is the way in.
fn on_tray_icon_event(tray: &TrayIcon, event: TrayIconEvent) {
    if let TrayIconEvent::Click {
        button: MouseButton::Left,
        button_state: MouseButtonState::Up,
        ..
    } = event
    {
        show_host_list(tray.app_handle());
    }
}

fn show_host_list(app: &AppHandle) {
    if let Err(err) = crate::app_window::show_host_list(app.clone(), None, None) {
        eprintln!("ahp-desk: the tray could not show the host list: {err}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tray_entries_are_hosts_settings_check_about_then_quit() {
        let words = TrayWords {
            hosts: "Hosts".into(),
            settings: "Settings".into(),
            check: "Check for updates…".into(),
            about: "About ahp-desk".into(),
            quit: "Quit ahp-desk".into(),
            update: None,
        };
        assert_eq!(
            entries(&words),
            [
                (MENU_HOSTS, "Hosts".to_string()),
                (MENU_SETTINGS, "Settings".to_string()),
                (MENU_CHECK, "Check for updates…".to_string()),
                (MENU_ABOUT, "About ahp-desk".to_string()),
                (MENU_QUIT, "Quit ahp-desk".to_string())
            ]
        );
    }

    #[test]
    fn tray_entries_start_with_a_ready_update() {
        let words = TrayWords {
            hosts: "Hosts".into(),
            settings: "Settings".into(),
            check: "Check for updates…".into(),
            about: "About ahp-desk".into(),
            quit: "Quit ahp-desk".into(),
            update: Some("ahp-desk 0.2.0 is ready. Restart to update.".into()),
        };
        let ids: Vec<_> = entries(&words).into_iter().map(|(id, _)| id).collect();
        assert_eq!(
            ids,
            [
                MENU_UPDATE,
                MENU_HOSTS,
                MENU_SETTINGS,
                MENU_CHECK,
                MENU_ABOUT,
                MENU_QUIT
            ]
        );
    }
}
