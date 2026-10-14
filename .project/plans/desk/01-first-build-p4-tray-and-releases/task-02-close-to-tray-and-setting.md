---
title: Closing the host list hides the desk to the tray, unless Settings says quit
status: todo
depends: [task-01-tray-icon-and-menu.md]
layer: "src-tauri, packages/shell"
refs:
  - file:///github/opendoop/packages/tauri/src-tauri/src/shell_settings.rs - `CloseBehavior` with `MinimizeToTray` as the default and `Quit`, at lines 76-82, and the atomic save at lines 114-127
  - https://v2.tauri.app/plugin/single-instance/ - a second start hands its arguments to the running desk
  - "[code://src-tauri/src/lib.rs#L30-L70](../../../../src-tauri/src/lib.rs#L30-L70) - the config folder, the host list window and its close handler"
  - "[code://src-tauri/src/hosts.rs#L150-L172](../../../../src-tauri/src/hosts.rs#L150-L172) - `HostStore::load`, the pattern for a store on a JSON file"
  - "[code://src-tauri/src/hosts.rs#L311-L321](../../../../src-tauri/src/hosts.rs#L311-L321) - the write to a temporary file and rename"
  - "[code://src-tauri/src/app_window.rs#L176-L189](../../../../src-tauri/src/app_window.rs#L176-L189) - the last app window closed shows the host list"
  - "[code://src-tauri/src/app_window.rs#L225-L239](../../../../src-tauri/src/app_window.rs#L225-L239) - `HOST_LIST_ADD` and `show_host_list`, the pattern for the Settings event"
  - "[code://packages/shell/src/App.tsx#L13-L98](../../../../packages/shell/src/App.tsx#L13-L98) - the `Screen` union and the screens"
  - "[code://packages/shell/src/TitleBar.tsx#L76-L77](../../../../packages/shell/src/TitleBar.tsx#L76-L77) - the title bar's controls, where the Settings button goes"
  - "[code://DEVELOPER.md#L89](../../../../DEVELOPER.md#L89) - `AHP_DESK_CONFIG_DIR` replaces the app config folder"
---

## Objective

`settings.json` in the app config folder holds `closeBehavior`, `minimize-to-tray` by default or `quit`.
With the default, closing the host list hides it and the desk keeps running in the tray; with `quit`, closing it with no app window open quits the desk, as today.
A Settings screen in the host list window sets it, reached from the tray's Settings item and from the host list.
A second start of the desk shows the running desk's host list instead of starting another desk.

## Files

- `CREATE: src-tauri/src/settings.rs` - `CloseBehavior`, `Settings`, `SettingsStore`, `SETTINGS_CHANGED`, and the commands `settings` and `save_settings`, with their tests.
- `UPDATE: src-tauri/src/lib.rs:1-7` - `mod settings;`.
- `UPDATE: src-tauri/src/lib.rs:20-25` - `tauri_plugin_single_instance::init(...)` as the first plugin.
- `UPDATE: src-tauri/src/lib.rs:33-37` - `app.manage(SettingsStore::load(config.join("settings.json"))?);` beside the host list.
- `UPDATE: src-tauri/src/lib.rs:53-66` - the close handler reads `closeBehavior`.
- `UPDATE: src-tauri/src/lib.rs:71-87` - `settings::settings` and `settings::save_settings`.
- `UPDATE: src-tauri/src/app_window.rs:225` - `pub const HOST_LIST_SETTINGS: &str = "host-list-settings";` beside `HOST_LIST_ADD`.
- `UPDATE: src-tauri/src/tray.rs` - `settings` in `TrayWords`, and the Settings item between Hosts and Quit.
- `UPDATE: src-tauri/Cargo.toml:17-24` - `tauri-plugin-single-instance = "2"`.
- `UPDATE: src-tauri/build.rs:52-68` and `src-tauri/capabilities/shell.json:6-30` - `settings`, `save_settings`.
- `CREATE: packages/shell/src/settings/ipc.ts` - `CloseBehavior`, `Settings`, `getSettings()`, `saveSettings()`, `onSettingsAsked()`.
- `CREATE: packages/shell/src/settings/SettingsScreen.tsx` - the Settings screen.
- `UPDATE: packages/shell/src/App.tsx:13` - `{ kind: 'settings' }` in `Screen`, its branch in the render, and `onSettingsAsked` with the listeners at line 42.
- `UPDATE: packages/shell/src/TitleBar.tsx:76-77` - a gear button that opens Settings, before the language picker (Answers: the Settings gear).
- `UPDATE: packages/shell/src/i18n.ts` - the new keys in the three dictionaries.
- `UPDATE: packages/shell/src/hosts/ipc.ts` - `settings` in `TrayWords`; `App.tsx` sends `t('tray.settings')`.

## Steps

1. In `settings.rs`: `#[serde(rename_all = "kebab-case")] pub enum CloseBehavior { #[default] MinimizeToTray, Quit }`, as doop's.
2. `#[serde(rename_all = "camelCase")] pub struct Settings { #[serde(default)] pub close_behavior: CloseBehavior }`, `Default`; plan p6 adds the pill's fields to this same struct and file.
3. `pub struct SettingsStore { path: PathBuf, settings: Mutex<Settings> }` with `load(path) -> Result<Self, String>`: a missing file is the defaults, a file that is not settings is an error that names the path, as `HostStore::load` (no silent fallback, unlike doop's `load`).
   `get() -> Settings` and `save(settings) -> Result<(), String>` with the same temporary file and rename as `hosts.rs` `write()`.
4. Commands: `settings(store) -> Settings` and `save_settings(app, store, settings) -> Result<Settings, String>`, which saves and emits `SETTINGS_CHANGED` (`"settings-changed"`) with the new settings.
5. The file sits in `config`, the folder `hosts.json` sits in, so `AHP_DESK_CONFIG_DIR` covers it as DEVELOPER.md line 89 says.
6. The host list's close handler (lib.rs:57-65): with an app window open it hides, as now (decision: open hides the host list); with none open, `MinimizeToTray` prevents the close and hides the window, and `Quit` lets the close go on, so the desk exits through `RunEvent::Exit` as it does today.
   The last app window closed still shows the host list (app_window.rs:182-188), so a hidden desk always has its host list to come back to.
7. `tauri_plugin_single_instance::init(|app, _args, _cwd| ...)` calls `app_window::show_host_list(app.clone(), None)`; it is registered before any other plugin, as its docs ask.
   With the desk in the tray, a start from the Start menu would otherwise make a second desk on the same `hosts.json` and the same local ahpd.
8. The tray's Settings item calls `show_host_list(app.clone(), None)` and then `app.emit_to(SHELL_WINDOW, HOST_LIST_SETTINGS, ())`.
9. `settings/ipc.ts`: `export type CloseBehavior = 'minimize-to-tray' | 'quit'`, `export interface Settings { closeBehavior: CloseBehavior }`, `getSettings()` invoking `settings`, `saveSettings(settings)` invoking `save_settings`, and `onSettingsAsked(handler)` listening to `host-list-settings`, as `onAddAsked`.
10. `SettingsScreen({ onDone })`: a `panel` like the host form, with the title, a radio group "When I close the host list" with "Hide it in the tray" and "Quit ahp-desk", saved on each change, and a Back button to the list.
11. `App.tsx`: the Settings screen renders for `screen.kind === 'settings'` (not during the first run); `TitleBar` takes `onSettings` and shows a gear button with the title `window.settings`.
12. English words: `window.settings` "Settings", `settings.title` "Settings", `settings.close` "When I close the host list", `settings.closeTray` "Hide it in the tray", `settings.closeQuit` "Quit ahp-desk", `tray.settings` "Settings"; Portuguese and Spanish in the same keys.

## Validation

- `cargo test --manifest-path src-tauri/Cargo.toml settings_`: `settings_a_missing_file_is_the_defaults` (close is `minimize-to-tray`), `settings_save_round_trips_quit`, `settings_a_file_with_no_close_field_takes_the_default`, `settings_a_broken_file_is_an_error_naming_it`.
- `tray_entries_are_hosts_settings_then_quit` replaces task 01's order test.
- `pnpm --filter @ahp-desk/shell test`, `pnpm typecheck`, `pnpm lint` pass.
- By hand on the debug desk with a scratch `AHP_DESK_CONFIG_DIR`, on Windows and on Debian 13:
  - Close the host list with no host open: the window goes, the tray icon stays, and a running local ahpd stays on; Hosts brings the list back.
  - Settings, from the gear and from the tray, in the three languages; pick "Quit ahp-desk": `settings.json` holds `"closeBehavior": "quit"`; close the list: the desk exits and the stop on quit runs.
  - In both settings: open a host, the list hides; close the app window, the list comes back.
  - With the desk hidden, start it again from its shortcut: the first desk's host list shows, and there is one tray icon.

## Resume
