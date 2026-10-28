---
title: The desk's update check is a window of its own
status: accepted
date: 2026-10-27
refs:
  - "[code://src-tauri/src/updates.rs#L131-L150](../../src-tauri/src/updates.rs#L131-L150) - `desk_update`, `install_desk_update` and `check_desk_update`, what the window asks"
  - "[code://packages/shell/src/hosts/DeskUpdateBar.tsx](../../packages/shell/src/hosts/DeskUpdateBar.tsx) - the ready update and its Restart, which the window shows"
---

## Context

Check now and the ready update's Restart sat on the About screen.
With About in a window of its own ([About is a window of its own](about-is-a-window-of-its-own.md)), the check needs a place too.

## Decision

"Check for updates…" opens a small window of its own: Check now, its result (checking, up to date, failed with the reason), the ready version and Restart.
Opening it again while it is open brings that window to the front.
It opens from the title bar's logo menu, the tray menu and Settings.
Source: Softov, 2026-10-27: "the check for update then instead in about could be another window to open, just for that?"; asked "Which menus get 'Check for updates…'?": "Logo menu, Tray menu, Settings".

## Consequences

About shows versions only.
Restart names the turns a local ahpd's stop would end, so this window needs each host's running turns ([the host list publishes each host's running turns](the-host-list-publishes-each-hosts-running-turns.md)).
The host list's own "ready" notice and its Restart stay.

## Options

- **Check now inside the About window.** Lost: Softov asked for a window just for the check.
