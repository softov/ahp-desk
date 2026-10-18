---
title: The desk finds a signed release, downloads it, and installs it on a click
status: done
depends: [task-01-tray-icon-and-menu.md]
layer: "src-tauri, packages/shell"
refs:
  - file:///github/opendoop/packages/tauri/src-tauri/tauri.conf.json - `createUpdaterArtifacts` at line 29, and `plugins.updater` with `pubkey` and the GitHub endpoint at lines 38-45
  - file:///github/opendoop/packages/tauri/PUBLISH.md - the key pair, `latest.json`, and what a lost key costs
  - https://v2.tauri.app/plugin/updater/ - `UpdaterExt`, `check()`, `download()`, `install()`, and the signing variables
  - "[code://DEVELOPER.md#L119-L134](../../../../DEVELOPER.md#L119-L134) - the signing keys: generated outside git, the public key in `tauri.conf.json`"
  - "[code://src-tauri/tauri.conf.json](../../../../src-tauri/tauri.conf.json) - no `plugins` block yet"
  - "[code://src-tauri/src/lib.rs#L20-L94](../../../../src-tauri/src/lib.rs#L20-L94) - the plugins, the setup, and the stop on quit in `RunEvent::Exit`"
  - "[code://src-tauri/src/local_ahpd.rs#L755](../../../../src-tauri/src/local_ahpd.rs#L755) - `stop_on_quit`, which an install must run as a quit does"
  - "[code://packages/shell/src/hosts/HostList.tsx#L73-L76](../../../../packages/shell/src/hosts/HostList.tsx#L73-L76) - the head of the host list, where the notice goes"
---

## Objective

On start, a release build asks `https://github.com/softov/ahp-desk/releases/latest/download/latest.json` for a newer version, checks its signature against the public key it ships, and downloads it in the background.
Then the host list and the tray say "ahp-desk X is ready. Restart to update.", and the desk installs and restarts only on that click (Answers: asking before the update).
A debug build never checks.

## Before this task: the key pair (the user)

The signing key is the user's, made once, and never in git, in a task file or in a log.

1. The user runs `pnpm tauri signer generate -w .tauri/updater.key` in the repository and gives it a password; `.gitignore` already covers `.tauri/` and `*.key`.
2. The user keeps a copy of `.tauri/updater.key` and its password outside the machine: a lost key leaves every installed desk unable to update (PUBLISH.md, "Lost / rotating the key").
3. The user hands over the public key, the content of `.tauri/updater.key.pub`, for `plugins.updater.pubkey`; the public key is safe in git.
4. The private key and its password become CI secrets in task 04.

## Files

- `UPDATE: src-tauri/Cargo.toml:17-24` - `tauri-plugin-updater = "2"`.
- `UPDATE: src-tauri/tauri.conf.json` - a `plugins.updater` block with `pubkey` (the user's public key) and `endpoints: ["https://github.com/softov/ahp-desk/releases/latest/download/latest.json"]`.
- `CREATE: src-tauri/src/updates.rs` - `DeskUpdate`, `check_on_start()`, `DESK_UPDATE_READY`, and the commands `desk_update` and `install_desk_update`; plan 09 adds npm's `@ahpd/server` to this file, as its architecture says.
- `UPDATE: src-tauri/src/lib.rs` - `mod updates;`, `.plugin(tauri_plugin_updater::Builder::new().build())`, `app.manage(DeskUpdate::default())`, `updates::check_on_start(app.handle())` at the end of the setup, and the two commands.
- `UPDATE: src-tauri/src/tray.rs` - `update: Option<String>` in `TrayWords`, and the item `tray.update` first in the menu when it is set.
- `UPDATE: src-tauri/build.rs:52-68` and `src-tauri/capabilities/shell.json:6-30` - `desk_update`, `install_desk_update`.
- `UPDATE: packages/shell/src/hosts/ipc.ts` - `deskUpdate()`, `installDeskUpdate()`, `onDeskUpdateReady()`, and `update` in `TrayWords`.
- `UPDATE: packages/shell/src/hosts/HostList.tsx:73-76` - the notice and its Restart button above the rows.
- `UPDATE: packages/shell/src/App.tsx` - reads the ready version and sends it in the tray's words.
- `UPDATE: packages/shell/src/i18n.ts` - `update.ready` and `update.restart` in the three dictionaries.

## Steps

1. `plugins.updater` as above; `bundle.createUpdaterArtifacts` is turned on for the release build only, by task 04 (Answers: the updater artifacts), so a clone of this MIT repository still builds an installer with no key.
   The updater's Windows `installMode` stays the default, `passive`.
2. `DeskUpdate(Mutex<Option<Ready>>)` with `struct Ready { version: String, update: tauri_plugin_updater::Update, bytes: Vec<u8> }`.
3. `pub fn check_on_start(app: &AppHandle)`: returns at once when `cfg!(debug_assertions)`; otherwise `tauri::async_runtime::spawn` runs `app.updater()?.check().await`, and for `Some(update)` runs `update.download(|_, _| {}, || {}).await`, keeps `Ready`, and emits `DESK_UPDATE_READY` (`"desk-update-ready"`) with the version.
   A failed check or download (offline, no release yet, a bad signature) is written with `eprintln!` and shows nothing; the desk runs on.
   The check every 6 hours after the start is plan 09's.
4. `#[tauri::command] desk_update(state) -> Option<String>`: the ready version, for a host list that opens after the event.
5. `#[tauri::command] async fn install_desk_update(app, state) -> Result<(), String>`: takes `Ready`, runs `local_ahpd::stop_on_quit(&app)` first, since the installer may end the process without `RunEvent::Exit` (check this on Windows and drop the call if `Exit` runs), then `update.install(bytes)` and `app.restart()`.
6. The notice is this task's, and plan 09 task 06 keeps only the check every 6 hours (Answers: asking before the update, and where the notice is built).
   `HostList` takes `update: string | null` and shows `t('update.ready', { version })` with a primary button `t('update.restart')` that calls `installDeskUpdate()`; a failure shows its message as a `result bad` line.
7. `App.tsx` calls `deskUpdate()` once and `onDeskUpdateReady()`, keeps the version, passes it to `HostList`, and sends `update: t('update.ready', { version })` in the tray's words, so the tray item has the same words.
8. The tray's `tray.update` item calls `install_desk_update`'s body through a shared `pub async fn install(app: AppHandle) -> Result<(), String>` in `updates.rs`.
9. English words: `update.ready` "ahp-desk {version} is ready. Restart to update.", `update.restart` "Restart"; Portuguese and Spanish in the same keys, keeping `{version}`.

## Validation

- `pnpm --filter @ahp-desk/shell test` (the placeholder test in `i18n.test.ts` checks `{version}` in each language), `pnpm typecheck`, `pnpm lint`.
- `cargo test --manifest-path src-tauri/Cargo.toml tray_`: with `update` set, the update item is first.
- A debug desk makes no request to github.com (watch it with the network off: no error is written).
- A release build from `pnpm tauri build`, which needs no key while `createUpdaterArtifacts` is off, starts with no release on GitHub: the failed check is one log line, no notice shows, and the desk works.
- The update from one real release to the next is checked in task 06, once task 04 makes releases.

## Resume

- 2026-10-18: the user made the key pair; `.tauri/updater.key` and `.tauri/updater.key.pub` are both ignored by git, and only the public key was read, into `plugins.updater.pubkey` with the GitHub endpoint.
- `tauri-plugin-updater` 2.14.0 builds on reqwest 0.13 with `rustls-no-provider` and installs rustls's `ring` provider itself, so `cargo tree` still holds no aws-lc.
- `updates.rs` already existed from plan 09 task 01; it now also has `DESK_UPDATE_READY`, `DeskUpdate` with its private `Ready`, `check_on_start()`, the shared `install()`, and the commands `desk_update` and `install_desk_update`.
- Step 5 changed in how, not in what: on Windows the plugin's `install()` starts the installer and calls `std::process::exit(0)`, so `RunEvent::Exit` never runs; the desk's quit (`before_quit()` in `lib.rs`, now shared with `RunEvent::Exit`: the tunnels closed and `stop_on_quit`) runs in the updater's `on_before_exit` hook, set in `check_on_start`'s updater, which the plugin calls only after the download is checked and unpacked, just before the installer starts.
  Calling it before `install()`, as step 5 said, would leave the desk running with its tunnels closed and its ahpd stopped when the install fails.
  On Linux `install()` returns, and `app.restart()` from a command's thread goes through `RunEvent::Exit`.
- A failed install puts the update back, so Restart can be pressed again; the failure shows on the list's error line.
- The tray's item `tray.update` comes first when `TrayWords.update` is set, and runs the same `install()`; a failure there is a log line.
- Verified: `cargo test` (136, with `tray_entries_start_with_a_ready_update`), `cargo clippy --all-targets -- -D warnings` through `pnpm lint`, `pnpm typecheck`, `pnpm test` (42; the placeholder test covers `{version}` in the three languages).
- By hand on a release build from `pnpm tauri build --no-bundle`, with a scratch `AHP_DESK_CONFIG_DIR`, home and app data: with no release on GitHub, each start wrote one line, `ahp-desk: no desk update: Could not fetch a valid release JSON from the remote`, `desk_update` answered `null`, no notice showed, and the desk worked.
- By hand, with `desk-update-ready` sent from the page with `0.2.0`: the notice and Restart showed above the rows, in English at 640x400 and in Portuguese at 480x300, on one line; Restart with nothing downloaded showed "no desk update is ready" on the error line, and the notice stayed.
- Not run: a debug desk with the network watched; `check_on_start` returns on `cfg!(debug_assertions)` before any updater is built.
- Not seen: a real download, install and restart, and whether the hook's quit stops a local ahpd on Windows; both are task 06, once task 04 makes releases.
