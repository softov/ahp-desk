---
title: A left click on the tray icon shows the host list, and the right click opens the menu
status: accepted
date: 2026-10-14
refs:
  - "[code://.project/plans/desk/01-first-build-p4-tray-and-releases/task-01-tray-icon-and-menu.md](../../.project/plans/desk/01-first-build-p4-tray-and-releases/task-01-tray-icon-and-menu.md) - the tray, its menu and its click handler"
  - file:///github/opendoop/packages/tauri/src-tauri/src/tray.rs - doop's left click, which shows its pill, at lines 8-62
  - https://v2.tauri.app/learn/system-tray/ - `show_menu_on_left_click` and the tray icon events
  - "[code://src-tauri/src/app_window.rs#L228-L239](../../src-tauri/src/app_window.rs#L228-L239) - `show_host_list`, which the left click calls"
---

## Context

On Windows the tray icon gets a left click and a right click, and either can open the menu.
Doop's left click shows its pill, and its right click opens the menu.
On Linux the tray is an appindicator, which sends no click events, so the menu is the only way in there.

## Decision

On Windows a left click on the tray icon shows and focuses the host list, and the right click opens the menu.
On Linux the menu is the only way in.
Source: Softov, asked "p4 q2: on Windows, what does a left click on the tray icon do?": "Show the host list".

## Consequences

The host list is one click away in the tray, where the Hosts item takes two.
The tray is built with `show_menu_on_left_click(false)`, and its icon event handles the left click.

## Options

- **Open the menu.** Lost: the most used action, showing the host list, would take two clicks.
