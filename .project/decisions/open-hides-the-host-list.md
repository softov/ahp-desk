---
title: Opening a host hides the host list
status: accepted
date: 2026-10-11
refs:
  - "[code://src-tauri/src/app_window.rs](../../src-tauri/src/app_window.rs) - `open_app_window` and `show_host_list`"
  - "[code://src-tauri/src/lib.rs](../../src-tauri/src/lib.rs) - the shell window"
---

## Context

After Open, the host list stayed on screen over the app window it had opened.
It cannot close: it holds the one connection per host that gives every window its status dots (decision: the host list window publishes host status).

## Decision

When the host list opens a host, it hides; it keeps running.
It comes back from ahpw's host switcher ("Manage hosts", "+ Add host"), when the last app window closes, and later from the tray.
Source: Softov, asked what the host list does when it opens a host: "Hide it (Recommended)".

## Consequences

One window on screen after Open, and the status dots keep working.
The desk needs a way back to the host list from every app window, which the embedder gives ahpw as `manageHosts()`.

## Options

- **Keep it open.** Lost: two windows on screen, one of them in the way.
- **Close it.** Lost: the status dots, and the desk would quit with no window left.
