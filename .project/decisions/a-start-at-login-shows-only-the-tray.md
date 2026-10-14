---
title: A start at login shows only the tray, not the host list
status: accepted
date: 2026-10-14
refs:
  - "[code://.project/decisions/the-desk-starts-with-the-os-by-default.md](../../.project/decisions/the-desk-starts-with-the-os-by-default.md) - \"the pill and the tray are there after a login\""
  - "[code://.project/plans/desk/01-first-build-p4-tray-and-releases/task-05-start-with-the-os.md](../../.project/plans/desk/01-first-build-p4-tray-and-releases/task-05-start-with-the-os.md) - the `--autostart` flag and the hidden host list"
  - "[code://src-tauri/src/lib.rs#L44-L51](../../src-tauri/src/lib.rs#L44-L51) - the host list window, built visible today"
---

## Context

The autostart plugin starts the desk at login with an argument, `--autostart`, that a start by hand does not have.
The decision that the desk starts with the OS by default says the pill and the tray are there after a login; it does not name the host list.
A window that opens at every login is in the way of whatever the user logged in to do.

## Decision

When the OS starts the desk at login, the host list window is built hidden and the desk shows only its tray icon.
A start by hand, and a second start that the running desk receives, always show the host list.
Source: Softov, asked "p4 q5: when the OS starts the desk at login, does the host list show?": "No, tray only".

## Consequences

A login is quiet, and the host list is one click away in the tray.
`run()` reads `--autostart` from the arguments before it builds the host list window.

## Options

- **Yes, as a normal start.** Lost: a window at every login is in the way, and the user closes it each time.
