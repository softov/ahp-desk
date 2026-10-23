---
title: With no tray to come back from, closing the host list quits
status: accepted
date: 2026-10-22
refs:
  - "[code://src-tauri/src/tray.rs](../../src-tauri/src/tray.rs) - `shown`, which asks the StatusNotifier watcher on Linux"
  - "[code://src-tauri/src/lib.rs](../../src-tauri/src/lib.rs) - the host list's close"
---

## Context

On 2026-10-22, plan 04 task 06's step 11 ran both Linux bundles of 0.2.1 on Debian 13 with stock GNOME 48.
Stock GNOME has no StatusNotifier host, so the tray icon did not show anywhere, though the AppImage carries libayatana-appindicator and the deb pulls it in.
Closing the host list hid it to that tray, as Settings' default says; the desk kept running with no window and no icon.
Starting the desk again showed the list, but Quit is only in the tray's menu, so the desk could not be quit from itself.

## Decision

When the desktop shows no tray, closing the host list, with no app window open, quits the desk, as Settings' "quit" does.
On Linux the desk asks the session bus's `org.kde.StatusNotifierWatcher` for `IsStatusNotifierHostRegistered` at each close; no watcher, or no session bus, counts as no tray.
Windows and macOS always show it.
Source: Softov, asked "Without a tray (stock GNOME), the host list hides on close, a second start shows it again, but Quit lives only in the tray menu, so there's no way to quit from the desk. What should happen?": "Close quits without a tray".

## Consequences

On stock GNOME the desk behaves as Settings' "quit" does; with GNOME's AppIndicator extension, or on a desktop with its own StatusNotifier host, nothing changes.
The check is made at each close, so an extension turned on while the desk runs takes effect at the next close.
A desktop with only the older XEmbed tray, which libayatana-appindicator can fall back to, counts as no tray, so there the close quits too.

## Options

- **README note only.** Lost: the desk would still run with no window and no way to quit it from itself.
- **Quit in the host list.** Lost: a second way to quit on every platform, where the tray already has one.
