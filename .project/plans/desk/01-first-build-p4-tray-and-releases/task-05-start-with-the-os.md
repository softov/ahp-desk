---
title: The first run and Settings have "Start with the OS", on by default
status: todo
depends: [task-01-tray-icon-and-menu.md, task-02-close-to-tray-and-setting.md]
layer: "src-tauri, packages/shell"
refs:
  - https://v2.tauri.app/plugin/autostart/ - `tauri_plugin_autostart::init`, `ManagerExt::autolaunch()` with `enable`, `disable` and `is_enabled`
  - https://v2.tauri.app/distribute/windows-installer/ - NSIS `installerHooks` and `NSIS_HOOK_POSTUNINSTALL`
  - "[code://src-tauri/src/lib.rs#L20-L70](../../../../src-tauri/src/lib.rs#L20-L70) - the plugins and the host list window, built visible"
  - "[code://src-tauri/tauri.conf.json#L32-L37](../../../../src-tauri/tauri.conf.json#L32-L37) - the NSIS settings"
  - "[code://packages/shell/src/FirstRun.tsx#L32-L42](../../../../packages/shell/src/FirstRun.tsx#L32-L42) - the welcome step, where the checkbox goes"
  - "[code://packages/shell/src/App.tsx#L28-L40](../../../../packages/shell/src/App.tsx#L28-L40) - the first run starts again when the last host is removed"
---

## Objective

The first run's welcome step shows "Start with the OS", checked, and Settings has the same checkbox (decision 1).
The OS's own login entry is the setting: the desk reads and writes it through Tauri's autostart plugin, and keeps only whether the first run's box was applied.
Removing the desk on Windows removes its login entry too.

## Files

- `UPDATE: src-tauri/Cargo.toml:17-24` - `tauri-plugin-autostart = "2"`.
- `UPDATE: src-tauri/src/lib.rs:20-25` - `.plugin(tauri_plugin_autostart::init(MacosLauncher::LaunchAgent, Some(vec!["--autostart"])))`.
- `UPDATE: src-tauri/src/lib.rs:44-51` - the host list window is built with `.visible(false)` when the desk was started by the OS (decision 8 in plan.md).
- `UPDATE: src-tauri/src/settings.rs` - `start_with_os_asked: bool` in `Settings`, and the commands `start_with_os` and `set_start_with_os`.
- `UPDATE: src-tauri/src/lib.rs:71-87`, `src-tauri/build.rs:52-68` and `src-tauri/capabilities/shell.json:6-30` - `start_with_os`, `set_start_with_os`.
- `CREATE: src-tauri/windows/hooks.nsh` - `NSIS_HOOK_POSTUNINSTALL`, which deletes the login entry.
- `UPDATE: src-tauri/tauri.conf.json:33-36` - `"installerHooks": "windows/hooks.nsh"` in `bundle.windows.nsis`.
- `UPDATE: packages/shell/src/settings/ipc.ts` - `StartWithOs`, `startWithOs()`, `setStartWithOs(on)`.
- `UPDATE: packages/shell/src/FirstRun.tsx:32-42` - the checkbox on the welcome step.
- `UPDATE: packages/shell/src/settings/SettingsScreen.tsx` - the same checkbox.
- `UPDATE: packages/shell/src/i18n.ts` - `settings.startWithOs` in the three dictionaries.
- `UPDATE: DEVELOPER.md:110-117` - "Testing on Windows" and the Linux section say where the login entry is, and that a deb removal leaves `~/.config/autostart/ahp-desk.desktop`.

## Steps

1. `Settings.start_with_os_asked`, `#[serde(default)]`, false until the first run's box is applied.
2. `#[tauri::command] start_with_os(app, store) -> Result<StartWithOs, String>` answers `{ on: app.autolaunch().is_enabled()?, asked }`.
3. `#[tauri::command] set_start_with_os(app, store, on: bool) -> Result<StartWithOs, String>` calls `enable()` or `disable()`, sets `start_with_os_asked` and saves, and emits `settings-changed`.
4. `FirstRun` welcome step: a checkbox "Start with the OS", checked when `asked` is false, else as `on` says, so a first run that comes back after the last host is removed does not turn on again what the user turned off; "Get started" calls `setStartWithOs(checked)` and goes on to the next step.
5. `SettingsScreen`: the checkbox shows `on` and calls `setStartWithOs` on each change; a failure shows its message.
6. The plugin starts the desk with `--autostart`; `run()` reads `std::env::args()` for it in the setup.
   With it, the host list is built hidden and the desk shows only its tray icon (decision 8 in plan.md).
   The single-instance callback of task 02 ignores the flag: a start by hand always shows the host list.
7. `hooks.nsh`: `!macro NSIS_HOOK_POSTUNINSTALL` deletes the value from `HKCU\Software\Microsoft\Windows\CurrentVersion\Run` that the plugin wrote.
   Read the value's name in the plugin's source, not from memory (the app's name, `ahp-desk`, is the expected one), and check it with `reg query` after an enable.
   Check in Tauri's NSIS template whether an update runs the old uninstaller; if it does, the hook skips the delete in update mode, or every update would turn the login start off.
8. On Linux the plugin writes `~/.config/autostart/ahp-desk.desktop`; check that its `Exec` is the AppImage's own path (`$APPIMAGE`), not the mount the AppImage runs from, and that a deb install points at `/usr/bin/ahp-desk`.
   A package removal does not touch a user's home, so the entry outlives a deb removal; DEVELOPER.md says so.
9. English word: `settings.startWithOs` "Start with the OS" (the decision's words); Portuguese and Spanish in the same key.

## Validation

- `cargo test --manifest-path src-tauri/Cargo.toml settings_`: `settings_start_with_os_asked_defaults_to_false` and a round trip with it true.
- `pnpm --filter @ahp-desk/shell test`, `pnpm typecheck`, `pnpm lint` pass.
- By hand on Windows, on a release build installed for the user (the plugin writes the installed exe's path):
  - The first run shows the box checked; Get started; `reg query HKCU\Software\Microsoft\Windows\CurrentVersion\Run` lists the desk.
  - Sign out and in: the desk starts, in the tray.
  - Untick in Settings: the value is gone; remove the last host: the first run shows the box unticked.
  - Uninstall from Windows Settings, Apps: the value is gone.
- By hand on Debian 13 with the AppImage: the box on writes `~/.config/autostart/ahp-desk.desktop` with the AppImage's path, and a new login starts the desk.

## Resume
