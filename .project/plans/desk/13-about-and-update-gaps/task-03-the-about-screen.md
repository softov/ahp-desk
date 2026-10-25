---
title: An About screen shows the versions and the desk's update state, from the tray and Settings
status: done
depends: [task-01-the-trays-hosts-shows-the-list.md]
layer: "src-tauri, packages/shell"
refs:
  - "[code://src-tauri/src/tray.rs#L21-L39](../../../../src-tauri/src/tray.rs#L21-L39) - `TrayWords` and `entries`"
  - "[code://src-tauri/src/updates.rs#L60-L68](../../../../src-tauri/src/updates.rs#L60-L68) - `check_desk`, which skips a debug build"
  - "[code://src-tauri/src/updates.rs#L131-L136](../../../../src-tauri/src/updates.rs#L131-L136) - `desk_update` and `install_desk_update`"
  - "[code://src-tauri/build.rs#L35](../../../../src-tauri/build.rs#L35) - where the build finds the staged `@ahpd/web`"
  - "[code://src-tauri/src/local_ahpd.rs#L84-L100](../../../../src-tauri/src/local_ahpd.rs#L84-L100) - `BundledNode`"
  - "[code://packages/shell/src/settings/SettingsScreen.tsx#L16](../../../../packages/shell/src/settings/SettingsScreen.tsx#L16) - the screen pattern, a `panel` with Back"
---

## Objective

An About screen shows ahp-desk's, ahpw's and Node's versions, the desk's update state with Check now and Restart, the MIT license and links to ahpio.com and the releases; it opens from the tray's "About ahp-desk" and from a link on Settings (decision 3).

## Files

- `CREATE: src-tauri/src/about.rs` - `#[derive(Serialize)] #[serde(rename_all = "camelCase")] pub struct About { desk: String, ahpw: Option<String>, node: Option<String> }` and `#[tauri::command] pub async fn about(app: AppHandle, local: State<'_, LocalAhpd>) -> Result<About, String>`.
- `UPDATE: src-tauri/build.rs` - read `version` from the staged ahpw's `package.json` and set `cargo:rustc-env=AHP_DESK_AHPW_VERSION=<version>`, empty when none; a local checkout's `package.json` the same way.
- `UPDATE: src-tauri/src/updates.rs` - `#[tauri::command] pub async fn check_desk_update(app: AppHandle) -> Result<Option<String>, String>`: an error in a debug build ("a debug build does not check for updates"), else `download_desk`, then the ready version.
- `UPDATE: src-tauri/src/app_window.rs` - `pub const HOST_LIST_ABOUT: &str = "host-list-about";`.
- `UPDATE: src-tauri/src/tray.rs` - `TrayWords.about`, `MENU_ABOUT = "tray.about"` in `entries` between Settings and Quit, and its event as Settings'.
- `UPDATE: src-tauri/src/lib.rs` - register `about::about` and `updates::check_desk_update`; `mod about;`.
- `UPDATE: src-tauri/build.rs:58` - add `about` and `check_desk_update` to `AppManifest::new().commands(&[...])`.
- `UPDATE: src-tauri/capabilities/shell.json` - `allow-about` and `allow-check-desk-update`, for the host list window only, as its other commands.
- `CREATE: packages/shell/src/about/AboutScreen.tsx` - the screen; `CREATE: packages/shell/src/about/ipc.ts` - `about()`, `checkDeskUpdate()`, `onAboutAsked()`.
- `UPDATE: packages/shell/src/App.tsx` - `Screen` gains `{ kind: 'about' }`; the listener; `tray.about` in `setTrayWords`.
- `UPDATE: packages/shell/src/settings/SettingsScreen.tsx` - a link "About ahp-desk" before Back.
- `UPDATE: packages/shell/src/i18n.ts` - the words below in the three dictionaries.

## Steps

1. `about()` answers `app.package_info().version`, `option_env!("AHP_DESK_AHPW_VERSION")` when not empty, and the bundled Node's `node --version` without its `v`, run off the main thread, `None` when no Node ships or it fails.
2. The screen reads `about()` and `deskUpdate()` on open.
   Update state: "ahp-desk X is ready" with Restart (the host list's Restart path, so a running turn is asked about first) when one is ready; else Check now, which shows "Checking...", then "ahp-desk is up to date" or the ready line, or the error's words.
3. Rows: "ahp-desk" with its version, "ahpw" with its version, "Node" with its version; then "MIT license", and the links ahpio.com and "Releases" (`https://github.com/softov/ahp-desk/releases`), opened outside through the opener as the footer does.
4. Words: `about.title` "About ahp-desk", `about.check` "Check now", `about.checking` "Checking...", `about.upToDate` "ahp-desk is up to date", `about.license` "MIT license", `about.releases` "Releases", `tray.about` "About ahp-desk", `settings.about` "About ahp-desk"; Portuguese and Spanish in the same keys.
5. The tray's Hosts from About lands on the list (task 01).

## Validation

- `cargo test`: `tray_entries_are_hosts_settings_about_then_quit` replaces the order test; `check_desk_update` in a debug build answers its error.
- `pnpm --filter @ahp-desk/shell test` and `typecheck`; `i18n.test.ts` with the new keys.
- By hand on a Windows release build: About from the tray and from Settings shows 0.2.x, ahpw 0.6.0 and Node 22.23.3; Check now says up to date, or ready with Restart after a newer release.

## Resume

Done on 2026-10-25: `about()`, `check_desk_update()`, the tray item, the screen and the Settings link are in.
The host list's update notice and its Restart moved into `DeskUpdateBar`, which the host list and About both show.
The tray's update item is heard by that notice, so it now works on About as on the list.
The capability `shell` lets the opener open the releases page beside ahpio.com.
`build.rs` reads the ahpw version with `serde_json`, now a build dependency.
"Checking…" uses the ellipsis character, as the other words do.
Typecheck, tests and clippy pass.
By hand on 2026-10-25, on a Windows release build: About from the tray and from Settings showed 0.2.1, ahpw 0.6.0 and Node 22.23.3.
Check now said "ahp-desk is up to date", and Releases opened the browser.
The first build showed an empty frame around Check now before a check.
Check now now sits under the versions, and the frame is only for a ready update.
Restart from About after a newer release is not checked by hand: 0.2.1 is the latest release.
