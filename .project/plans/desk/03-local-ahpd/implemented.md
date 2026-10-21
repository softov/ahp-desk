---
title: The local ahpd - implemented
date: 2026-10-21
refs:
  - "[code://src-tauri/src/local_ahpd.rs](../../../../src-tauri/src/local_ahpd.rs)"
  - "[code://src-tauri/src/hosts.rs](../../../../src-tauri/src/hosts.rs)"
  - "[code://packages/shell/src/hosts/LocalForm.tsx](../../../../packages/shell/src/hosts/LocalForm.tsx)"
  - "[code://packages/shell/src/hosts/status.ts](../../../../packages/shell/src/hosts/status.ts)"
  - "[code://packages/shell/src/hosts/HostList.tsx](../../../../packages/shell/src/hosts/HostList.tsx)"
---

The desk adds "This computer" as a host: it uses an ahpd it finds on `PATH` or at a set path, or installs one with the Node its installer ships, with the backends picked in Add host.
It starts that ahpd when it starts, stops it on quit when asked, says "starting" while it comes up, and keeps an ahpd it installed up to date through ahpd's own update command.

## What was built

- [`code://src-tauri/src/local_ahpd.rs`](../../../../src-tauri/src/local_ahpd.rs) - finding ahpd, the install into `<app local data>/ahpd` with the bundled Node, start, status and stop through ahpd's own commands, the start at launch with `local-starting`, and the stop on quit and before a desk install.
- [`code://src-tauri/src/hosts.rs`](../../../../src-tauri/src/hosts.rs) - a local host's settings: `startOnLaunch`, `stopOnQuit`, `autoUpdate`.
- [`code://packages/shell/src/hosts/LocalForm.tsx`](../../../../packages/shell/src/hosts/LocalForm.tsx) - Add host, This computer: the ahpd found or the install, the folder, the backends, and the settings.
- [`code://packages/shell/src/hosts/status.ts`](../../../../packages/shell/src/hosts/status.ts) and [`code://packages/shell/src/hosts/HostList.tsx`](../../../../packages/shell/src/hosts/HostList.tsx) - the row's state, and the update through `x-ahpd/update` for an ahpd the desk installed.

## Verified

- `cargo test` (141), `cargo clippy --all-targets -- -D warnings`, `pnpm typecheck`, `pnpm test` (53).
- By hand on a Windows 10 VM, as a new standard user, on release NSIS builds: the install and connect, the found ahpd on `PATH` used with nothing installed, Quit with each stop setting, the automatic update and the Update click on ahpd 0.13.x (tasks 07 and 08).

## Departures from the plan

- Task 07 updates through ahpd's own `x-ahpd/update` (ahpd 0.13.0) and reads the newer version from the desk's npm check, not from ahpd's `update.json`.
- The host list window opens at 640x480 so Add host fits with the settings (decision 13).
- The row says "starting" while the desk starts its ahpd at launch, found in task 08.

## Left for later

- None in this plan; the desk's own update with a local ahpd running is plan 04 task 06.
