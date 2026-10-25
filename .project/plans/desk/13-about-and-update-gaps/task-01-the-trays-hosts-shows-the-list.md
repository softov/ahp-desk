---
title: The tray's Hosts shows the host list window on its list
status: todo
depends: []
layer: "src-tauri, packages/shell"
refs:
  - "[code://src-tauri/src/tray.rs#L105](../../../../src-tauri/src/tray.rs#L105) - `MENU_HOSTS => show_host_list(app)`"
  - "[code://src-tauri/src/app_window.rs#L318](../../../../src-tauri/src/app_window.rs#L318) - `HOST_LIST_SETTINGS`, the pattern"
  - "[code://packages/shell/src/settings/ipc.ts#L27](../../../../packages/shell/src/settings/ipc.ts#L27) - `onSettingsAsked`, the listener pattern"
  - "[code://packages/shell/src/App.tsx#L68](../../../../packages/shell/src/App.tsx#L68) - where the screen listeners sit"
---

## Objective

The tray's Hosts shows the host list window and switches it to the list from Settings, About, Add host or Edit host (decision 1).

## Files

- `UPDATE: src-tauri/src/app_window.rs:318` - add `pub const HOST_LIST_HOSTS: &str = "host-list-hosts";` beside `HOST_LIST_SETTINGS`.
- `UPDATE: src-tauri/src/tray.rs:105` - `MENU_HOSTS` shows the list and emits `HOST_LIST_HOSTS`, as `MENU_SETTINGS` does at 106-111.
- `UPDATE: packages/shell/src/hosts/ipc.ts` - `export function onHostsAsked(handler: () => void): Promise<UnlistenFn>`, listening to `host-list-hosts`, as `onAddAsked`.
- `UPDATE: packages/shell/src/App.tsx:68` - `onHostsAsked(() => setScreen({ kind: 'list' }))` beside `onSettingsAsked`.

## Steps

1. In `tray.rs`, `MENU_HOSTS => { show_host_list(app); if let Err(err) = app.emit_to(SHELL_WINDOW, HOST_LIST_HOSTS, ()) { eprintln!("ahp-desk: the tray could not show the host list: {err}"); } }`, importing `HOST_LIST_HOSTS` with `HOST_LIST_SETTINGS`.
2. Leave `app_window::show_host_list` and the single-instance callback alone: a second start keeps the screen it was on (plan, second table).
3. The listener drops a half-filled form, as the form's Back does; nothing asks first.
4. During the first run the screen is not rendered, so the event changes nothing there.

## Validation

- `pnpm --filter @ahp-desk/shell typecheck` and `test`; `cargo clippy --all-targets -- -D warnings`.
- By hand on a release build: tray Settings, then tray Hosts, shows the list; the same from Add host with a typed name.

## Resume
