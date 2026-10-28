---
title: Check for updates opens a window of its own
status: doing
depends: [task-01-every-window-reads-running-turns.md, task-02-the-about-window.md]
layer: "src-tauri and packages/shell"
refs:
  - "[code://src-tauri/src/updates.rs#L131-L150](../../../../src-tauri/src/updates.rs#L131-L150) - `desk_update`, `install_desk_update`, `check_desk_update`"
  - "[code://src-tauri/src/tray.rs#L22-L42](../../../../src-tauri/src/tray.rs#L22-L42) - `TrayWords` and `entries`; `on_menu_event` at line 105 and the menu tests at line 175"
  - "[code://src-tauri/src/app_window.rs#L329](../../../../src-tauri/src/app_window.rs#L329) - `HOST_LIST_DESK_UPDATE`, which the tray's update item emits today"
  - "[code://packages/shell/src/hosts/DeskUpdateBar.tsx](../../../../packages/shell/src/hosts/DeskUpdateBar.tsx) - the ready update and Restart's question"
  - "[code://packages/shell/src/TitleBar.tsx#L52-L105](../../../../packages/shell/src/TitleBar.tsx#L52-L105) - `MenuWays` and the logo menu"
  - "[code://packages/shell/src/settings/SettingsScreen.tsx#L82-L84](../../../../packages/shell/src/settings/SettingsScreen.tsx#L82-L84) - the About link the check sits beside"
  - "[code://packages/shell/src/App.tsx](../../../../packages/shell/src/App.tsx) - `setTrayWords`, with the tray's words"
  - "[code://.project/specs/tray-and-settings.md](../../../specs/tray-and-settings.md) - the tray, Settings and About as specified"
---

## Objective

"Check for updates…" opens a fixed-size frameless window of 400 by 260, labelled `updates`, that loads `index.html#updates` (decision 2).
It shows the running desk's version, Check now and its result, and with an update ready the ready notice and Restart, whose running-turns question shows in this window (decision 3).
It opens from the logo menu, the tray and Settings, and the tray's "Restart to update" item opens it too.

## Files

- `UPDATE: src-tauri/src/side_windows.rs` - `UPDATES_WINDOW`, `open_updates`, and the command `open_updates_window`.
- `CREATE: src-tauri/capabilities/updates.json` - the `updates` window's capability.
- `UPDATE: src-tauri/src/lib.rs` and `src-tauri/build.rs` - `open_updates_window`.
- `UPDATE: src-tauri/capabilities/shell.json` - `"allow-open-updates-window"`.
- `UPDATE: src-tauri/src/tray.rs` - `MENU_CHECK`, `TrayWords::check`, the menu order, `MENU_UPDATE` and `MENU_CHECK` calling `side_windows::open_updates(app)`, and the menu tests.
- `UPDATE: src-tauri/src/app_window.rs:327-329` - `HOST_LIST_DESK_UPDATE` goes.
- `CREATE: packages/shell/src/updates/UpdatesWindow.tsx` - the Updates window's page.
- `CREATE: packages/shell/src/updates/ipc.ts` - `openUpdatesWindow()`.
- `UPDATE: packages/shell/src/main.tsx` - `#updates` renders `UpdatesWindow`.
- `UPDATE: packages/shell/src/hosts/DeskUpdateBar.tsx` - the `onDeskUpdateAsked` listener goes.
- `UPDATE: packages/shell/src/hosts/ipc.ts` - `TrayWords.check`; `onDeskUpdateAsked` goes.
- `UPDATE: packages/shell/src/TitleBar.tsx` - `MenuWays.onCheck` and its item.
- `UPDATE: packages/shell/src/settings/SettingsScreen.tsx` - a "Check for updates…" button beside the About link.
- `UPDATE: packages/shell/src/App.tsx` - the tray's `check` word; `onCheck` for the logo menu and Settings.
- `UPDATE: packages/shell/src/i18n.ts` - `tray.check` and the window's words, in the three languages.
- `UPDATE: packages/shell/src/app.css` - the Updates window's layout.
- `UPDATE: .project/specs/tray-and-settings.md` - the tray's menu, Settings, About and the Updates window as built, and the title bar's logo menu, "Launch at startup" and no gear from PRs #13 and #15.

## Steps

1. In `side_windows.rs`, add `pub const UPDATES_WINDOW: &str = "updates";`, `pub fn open_updates(app: &AppHandle)` calling `open_side(app, UPDATES_WINDOW, "updates", (400.0, 260.0))` and logging a failure, and `#[tauri::command] pub fn open_updates_window(app: AppHandle) -> Result<(), String>`.
2. Write `capabilities/updates.json`: `"identifier": "updates"`, `"windows": ["updates"]`, permissions `core:default`, `core:window:allow-minimize`, `core:window:allow-close`, `core:window:allow-start-dragging`, `core:window:allow-set-title`, `allow-about`, `allow-hosts`, `allow-host-running`, `allow-local-ahpd-status`, `allow-desk-update`, `allow-check-desk-update`, `allow-install-desk-update`.
3. In `tray.rs`, add `const MENU_CHECK: &str = "tray.check";` and `pub check: String` to `TrayWords`; `entries` gives the update item while one is ready, then Hosts, Settings, the check, About, Quit; `MENU_UPDATE` and `MENU_CHECK` call `side_windows::open_updates(app)`; update the menu tests at line 175 and line 193 for the new item and order.
4. Remove `HOST_LIST_DESK_UPDATE` from `app_window.rs` and `tray.rs`, `onDeskUpdateAsked` from `hosts/ipc.ts`, and its listener from `DeskUpdateBar.tsx`.
5. Write `UpdatesWindow`: `WindowBar` with `t('updates.title')`; on mount `about()` for the desk's version, `deskUpdate()` and `onDeskUpdateReady` for a ready one, `listHosts()` and `onHostsChanged` for the hosts, `hostRunning()` and `onHostRunning` for the turns.
   It shows "ahp-desk {version}", then with an update ready `DeskUpdateBar` with `running={(id) => running[id] ?? []}`, otherwise Check now with "Checking…", "ahp-desk is up to date" or the error, as `AboutScreen` did.
6. In `main.tsx`, render `UpdatesWindow` when `location.hash === '#updates'`.
7. Add `onCheck` to `MenuWays` and an item `t('tray.check')` before About in the logo menu; add `onCheck` to `SettingsScreen`'s props and a link button `t('tray.check')` beside the About link; in `App.tsx` both call `openUpdatesWindow()`, logging a failure, and `setTrayWords` sends `check: t('tray.check')`.
8. In `i18n.ts`, add `tray.check` ("Check for updates…", "Verificar atualizações…", "Buscar actualizaciones…") and `updates.title` ("Check for updates", "Verificar atualizações", "Buscar actualizaciones"); rename `about.check`, `about.checking` and `about.upToDate` to `updates.check`, `updates.checking` and `updates.upToDate`.
9. Correct `specs/tray-and-settings.md`: the tray's menu and its items, Settings without the gear and with "Launch at startup", "About ahp-desk" and "Check for updates…", an About window section and an Updates window section, and the title bar's logo menu.

## Validation

- `tray.rs` tests: the menu with and without an update, in the new order with the check.
- `pnpm typecheck`, `pnpm test`, and `cargo clippy --all-targets -- -D warnings`.
- By hand on a Windows release build: the window opens from the logo menu, the tray and Settings, and a second open focuses it; Check now answers "up to date" or the ready notice; with an update ready and a turn running on a local ahpd the desk started, Restart names the turn in the Updates window and a second Restart installs; the tray's "Restart to update" opens the window.

## Resume

- Built: `UPDATES_WINDOW`, `open_updates` and the async command `open_updates_window`; `capabilities/updates.json`; the tray's `MENU_CHECK`, with `MENU_UPDATE` opening the window too; `UpdatesWindow` and `updates/ipc.ts`; the logo menu's and Settings' entries; the words in three languages; the spec corrected. `HOST_LIST_DESK_UPDATE` and `onDeskUpdateAsked` are gone.
- Beyond the steps: `checkDeskUpdate` moved from `about/ipc.ts` to `updates/ipc.ts`, as About no longer checks; Settings' two links sit together at the start of the actions, and Back keeps the end.
- Checked: `pnpm typecheck`, `pnpm test` (the tray's tests in the new order), `cargo clippy --all-targets -- -D warnings` pass.
- Next: the by-hand checks in Validation, with task 01's (the Restart names a running turn), then `status: done` and the plan closed.

