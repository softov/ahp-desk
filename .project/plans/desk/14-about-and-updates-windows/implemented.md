---
title: About and the desk's update check in windows of their own - implemented
date: 2026-10-28
refs:
  - git://c1452c1
  - "[code://src-tauri/src/side_windows.rs](../../../../src-tauri/src/side_windows.rs) - `open_side`, the About and Updates windows"
  - "[code://src-tauri/src/statuses.rs](../../../../src-tauri/src/statuses.rs) - `HostRunning` and the `hosts-running` relay"
  - "[code://packages/shell/src/main.tsx](../../../../packages/shell/src/main.tsx) - the page by hash"
---

About and "Check for updates…" open as small fixed-size windows beside the host list, which keeps its screen, so a host form with a problem and About fit in one screenshot.
The Updates window holds Check now, the ready update and its Restart, whose question about running turns reads each host's running turns that the host list now publishes to every window.

## What was built

- [`code://src-tauri/src/side_windows.rs`](../../../../src-tauri/src/side_windows.rs) - `open_side` opens or focuses a frameless, non-resizable window on `index.html#<view>`; `open_about`, `open_updates` and their async commands.
- [`code://src-tauri/capabilities/about.json`](../../../../src-tauri/capabilities/about.json) and [`code://src-tauri/capabilities/updates.json`](../../../../src-tauri/capabilities/updates.json) - each window's commands only.
- [`code://src-tauri/src/statuses.rs`](../../../../src-tauri/src/statuses.rs) - `HostRunning`, `publish_host_running`, `host_running`; `remove_host` forgets a removed host's turns.
- [`code://src-tauri/src/tray.rs`](../../../../src-tauri/src/tray.rs) - "Check for updates…" between Settings and About; it, About and "Restart to update" open the side windows.
- [`code://packages/shell/src/main.tsx`](../../../../packages/shell/src/main.tsx) - `#about` and `#updates` pick the page; only `App` starts the desk's work.
- [`code://packages/shell/src/WindowBar.tsx`](../../../../packages/shell/src/WindowBar.tsx) - a side window's bar: the logo, the title, minimize and close.
- [`code://packages/shell/src/about/AboutWindow.tsx`](../../../../packages/shell/src/about/AboutWindow.tsx) and [`code://packages/shell/src/updates/UpdatesWindow.tsx`](../../../../packages/shell/src/updates/UpdatesWindow.tsx) - the two pages.
- [`code://packages/shell/src/hosts/status.ts`](../../../../packages/shell/src/hosts/status.ts) - `publishRunning`, on connection and session events.
- [`code://packages/shell/src/i18n.ts`](../../../../packages/shell/src/i18n.ts) - a language picked in one window reaches the others through the `storage` event.
- [`code://.project/specs/tray-and-settings.md`](../../../specs/tray-and-settings.md) - corrected for the logo menu, "Launch at startup" and both windows.

## Verified

- `pnpm typecheck`, `pnpm test` (shell 56, embedder 16, Rust 147 with the new `statuses.rs` test and the tray's menu tests in the new order) and `cargo clippy --all-targets -- -D warnings` pass at each task.
- By hand on the debug desk, by Softov: the About window and the Updates window open from their entries, and PRs #17 and #19 were merged after it.

## Departures from the plan

- Task 01 - `status.ts` publishes the running turns on the client's session events too, since `hostEvents()` carries only connection changes.
- Task 02 - `about.json` also grants the opener for the two links; `startFrame` takes `{ maximizes }`, so a side window's bar does not maximize on a double click.
- Task 03 - `checkDeskUpdate` moved to `updates/ipc.ts`; Settings' two links sit together at the start of the actions.

## Left for later

- Restart naming a running turn in the Updates window, checked by hand - see [deferred.md](deferred.md).
