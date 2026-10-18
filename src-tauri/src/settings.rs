//! The desk's own settings: `settings.json` in the config folder that holds
//! `hosts.json`, so `AHP_DESK_CONFIG_DIR` covers it too.

use std::fs;
use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, State};
use tauri_plugin_autostart::ManagerExt;

use crate::hosts::write_json;

/// The event each save sends, with the new settings.
pub const SETTINGS_CHANGED: &str = "settings-changed";

/// What closing the host list does when no app window is open.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CloseBehavior {
    /// It hides, and the desk keeps running in the tray.
    #[default]
    MinimizeToTray,
    /// The desk quits.
    Quit,
}

/// How hosts open: a window each, or one window with every host.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Windows {
    /// Each host in a window of its own.
    #[default]
    PerHost,
    /// One window shows every host, and Open on a host shows it there.
    EveryHost,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    #[serde(default)]
    pub close_behavior: CloseBehavior,
    /// Read on each Open, so a change leaves the windows already open as they are.
    #[serde(default)]
    pub windows: Windows,
    /// Whether the first run's "Start with the OS" was applied. The OS's own
    /// login entry is the setting; this only keeps a first run that comes back
    /// from turning on again what the user turned off.
    #[serde(default)]
    pub start_with_os_asked: bool,
}

pub struct SettingsStore {
    path: PathBuf,
    settings: Mutex<Settings>,
}

impl SettingsStore {
    /// Reads `path`; a missing file is the defaults, and a file that is not
    /// settings is an error naming it, as `HostStore::load`.
    pub fn load(path: PathBuf) -> Result<Self, String> {
        let settings = match fs::read_to_string(&path) {
            Ok(text) => serde_json::from_str(&text)
                .map_err(|err| format!("{} is not the desk's settings: {err}", path.display()))?,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Settings::default(),
            Err(err) => return Err(format!("cannot read {}: {err}", path.display())),
        };
        Ok(SettingsStore {
            path,
            settings: Mutex::new(settings),
        })
    }

    fn lock(&self) -> MutexGuard<'_, Settings> {
        self.settings
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    pub fn get(&self) -> Settings {
        *self.lock()
    }

    pub fn save(&self, settings: Settings) -> Result<(), String> {
        let mut held = self.lock();
        write_json(&self.path, &settings)?;
        *held = settings;
        Ok(())
    }
}

#[tauri::command]
pub fn settings(store: State<'_, SettingsStore>) -> Settings {
    store.get()
}

#[tauri::command]
pub fn save_settings(
    app: AppHandle,
    store: State<'_, SettingsStore>,
    settings: Settings,
) -> Result<Settings, String> {
    // Only `set_start_with_os` changes whether the first run's box was applied;
    // a screen that read the settings before it would write the old value back.
    let settings = Settings {
        start_with_os_asked: store.get().start_with_os_asked,
        ..settings
    };
    store.save(settings)?;
    app.emit(SETTINGS_CHANGED, settings)
        .map_err(|err| err.to_string())?;
    Ok(settings)
}

/// "Start with the OS": the login entry as the OS has it, and whether the first run applied its box.
#[derive(Debug, Serialize)]
pub struct StartWithOs {
    pub on: bool,
    pub asked: bool,
}

#[tauri::command]
pub fn start_with_os(
    app: AppHandle,
    store: State<'_, SettingsStore>,
) -> Result<StartWithOs, String> {
    Ok(StartWithOs {
        on: app
            .autolaunch()
            .is_enabled()
            .map_err(|err| err.to_string())?,
        asked: store.get().start_with_os_asked,
    })
}

/// Writes or removes the login entry, and keeps that the box was applied.
#[tauri::command]
pub fn set_start_with_os(
    app: AppHandle,
    store: State<'_, SettingsStore>,
    on: bool,
) -> Result<StartWithOs, String> {
    let launch = app.autolaunch();
    if on {
        launch.enable()
    } else {
        launch.disable()
    }
    .map_err(|err| err.to_string())?;
    let mut settings = store.get();
    settings.start_with_os_asked = true;
    store.save(settings)?;
    app.emit(SETTINGS_CHANGED, settings)
        .map_err(|err| err.to_string())?;
    Ok(StartWithOs {
        on: launch.is_enabled().map_err(|err| err.to_string())?,
        asked: true,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    /// A `settings.json` path in a folder of its own, gone at the end of the test.
    struct Scratch(PathBuf);
    impl Scratch {
        fn new() -> Self {
            static NEXT: AtomicUsize = AtomicUsize::new(0);
            let dir = std::env::temp_dir().join(format!(
                "ahp-desk-settings-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir_all(&dir).unwrap();
            Scratch(dir)
        }
        fn path(&self) -> PathBuf {
            self.0.join("settings.json")
        }
    }
    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn settings_a_missing_file_is_the_defaults() {
        let scratch = Scratch::new();
        let store = SettingsStore::load(scratch.path()).unwrap();
        assert_eq!(store.get().close_behavior, CloseBehavior::MinimizeToTray);
    }

    #[test]
    fn settings_save_round_trips_quit() {
        let scratch = Scratch::new();
        let store = SettingsStore::load(scratch.path()).unwrap();
        store
            .save(Settings {
                close_behavior: CloseBehavior::Quit,
                ..Settings::default()
            })
            .unwrap();
        let text = fs::read_to_string(scratch.path()).unwrap();
        assert!(text.contains(r#""closeBehavior": "quit""#), "{text}");
        let again = SettingsStore::load(scratch.path()).unwrap();
        assert_eq!(again.get().close_behavior, CloseBehavior::Quit);
    }

    #[test]
    fn settings_a_file_with_no_close_field_takes_the_default() {
        let scratch = Scratch::new();
        fs::write(scratch.path(), "{}").unwrap();
        let store = SettingsStore::load(scratch.path()).unwrap();
        assert_eq!(store.get().close_behavior, CloseBehavior::MinimizeToTray);
    }

    #[test]
    fn settings_start_with_os_asked_defaults_to_false() {
        let scratch = Scratch::new();
        fs::write(scratch.path(), r#"{"closeBehavior":"quit"}"#).unwrap();
        let store = SettingsStore::load(scratch.path()).unwrap();
        assert!(!store.get().start_with_os_asked);
        store
            .save(Settings {
                start_with_os_asked: true,
                ..store.get()
            })
            .unwrap();
        let again = SettingsStore::load(scratch.path()).unwrap();
        assert!(again.get().start_with_os_asked);
        assert_eq!(again.get().close_behavior, CloseBehavior::Quit);
    }

    #[test]
    fn settings_a_file_from_before_windows_opens_hosts_per_host() {
        let scratch = Scratch::new();
        fs::write(scratch.path(), r#"{"closeBehavior":"quit","startWithOsAsked":true}"#).unwrap();
        let store = SettingsStore::load(scratch.path()).unwrap();
        assert_eq!(store.get().windows, Windows::PerHost);
    }

    #[test]
    fn settings_save_round_trips_every_host() {
        let scratch = Scratch::new();
        let store = SettingsStore::load(scratch.path()).unwrap();
        store
            .save(Settings {
                windows: Windows::EveryHost,
                ..Settings::default()
            })
            .unwrap();
        let text = fs::read_to_string(scratch.path()).unwrap();
        assert!(text.contains(r#""windows": "every-host""#), "{text}");
        let again = SettingsStore::load(scratch.path()).unwrap();
        assert_eq!(again.get().windows, Windows::EveryHost);
    }

    #[test]
    fn settings_a_broken_file_is_an_error_naming_it() {
        let scratch = Scratch::new();
        fs::write(scratch.path(), "{ not json").unwrap();
        let err = SettingsStore::load(scratch.path()).err().unwrap();
        assert!(err.contains("settings.json"), "{err}");
    }
}
