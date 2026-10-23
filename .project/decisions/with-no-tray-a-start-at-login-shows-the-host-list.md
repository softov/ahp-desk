---
title: With no tray, a start at login shows the host list
status: accepted
date: 2026-10-22
refs:
  - "[code://src-tauri/src/lib.rs](../../src-tauri/src/lib.rs) - the host list window's `visible` at start"
  - "[code://src-tauri/src/tray.rs](../../src-tauri/src/tray.rs) - `shown`"
---

## Context

A start at login (`--autostart`) shows only the tray icon, and the host list stays hidden.
On a desktop that shows no tray, such as stock GNOME (see [with no tray, closing the host list quits](with-no-tray-closing-the-host-list-quits.md)), that start leaves the desk running with nothing on screen until it is started again.

## Decision

A start at login shows the host list when the desktop shows no tray, by the same `tray::shown` check as the close.
Source: Softov, asked "A start at login (--autostart) shows only the tray icon and no window [...] Should a login start with no tray show the host list?": "Show the list".

## Consequences

On stock GNOME the host list opens at each login while "Start with the OS" is on.
A tray host that registers a moment after the desk starts at login counts as no tray for that start, so the list shows once.

## Options

- **Leave it.** Lost: the desk would run at login with no window and no icon, its hosts connected and its local ahpd started, out of sight.
