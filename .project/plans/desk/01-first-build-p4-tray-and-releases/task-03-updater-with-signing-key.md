---
title: The desk finds a signed release, downloads it, and installs it on a click
status: todo
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
Then the host list and the tray say "ahp-desk X is ready. Restart to update.", and the desk installs and restarts only on that click (decision 3).
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
- `CREATE: src-tauri/src/updates.rs` - `DeskUpdate`, `check_on_start()`, `DESK_UPDATE_READY`, and the commands `desk_update` and `install_desk_update`; plan p9 adds npm's `@ahpd/server` to this file, as its architecture says.
- `UPDATE: src-tauri/src/lib.rs` - `mod updates;`, `.plugin(tauri_plugin_updater::Builder::new().build())`, `app.manage(DeskUpdate::default())`, `updates::check_on_start(app.handle())` at the end of the setup, and the two commands.
- `UPDATE: src-tauri/src/tray.rs` - `update: Option<String>` in `TrayWords`, and the item `tray.update` first in the menu when it is set.
- `UPDATE: src-tauri/build.rs:52-68` and `src-tauri/capabilities/shell.json:6-30` - `desk_update`, `install_desk_update`.
- `UPDATE: packages/shell/src/hosts/ipc.ts` - `deskUpdate()`, `installDeskUpdate()`, `onDeskUpdateReady()`, and `update` in `TrayWords`.
- `UPDATE: packages/shell/src/hosts/HostList.tsx:73-76` - the notice and its Restart button above the rows.
- `UPDATE: packages/shell/src/App.tsx` - reads the ready version and sends it in the tray's words.
- `UPDATE: packages/shell/src/i18n.ts` - `update.ready` and `update.restart` in the three dictionaries.

## Steps

1. `plugins.updater` as above; `bundle.createUpdaterArtifacts` is turned on for the release build only, by task 04 (open question 8 in plan.md), so a clone of this MIT repository still builds an installer with no key.
   The updater's Windows `installMode` stays the default, `passive`.
2. `DeskUpdate(Mutex<Option<Ready>>)` with `struct Ready { version: String, update: tauri_plugin_updater::Update, bytes: Vec<u8> }`.
3. `pub fn check_on_start(app: &AppHandle)`: returns at once when `cfg!(debug_assertions)`; otherwise `tauri::async_runtime::spawn` runs `app.updater()?.check().await`, and for `Some(update)` runs `update.download(|_, _| {}, || {}).await`, keeps `Ready`, and emits `DESK_UPDATE_READY` (`"desk-update-ready"`) with the version.
   A failed check or download (offline, no release yet, a bad signature) is written with `eprintln!` and shows nothing; the desk runs on.
   The check every 6 hours after the start is plan p9's.
4. `#[tauri::command] desk_update(state) -> Option<String>`: the ready version, for a host list that opens after the event.
5. `#[tauri::command] async fn install_desk_update(app, state) -> Result<(), String>`: takes `Ready`, runs `local_ahpd::stop_on_quit(&app)` first, since the installer may end the process without `RunEvent::Exit` (check this on Windows and drop the call if `Exit` runs), then `update.install(bytes)` and `app.restart()`.
6. The notice (decision 3; open question 1 in plan.md asks whether it is this task's or p9 task 06's): `HostList` takes `update: string | null` and shows `t('update.ready', { version })` with a primary button `t('update.restart')` that calls `installDeskUpdate()`; a failure shows its message as a `result bad` line.
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
