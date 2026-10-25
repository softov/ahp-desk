---
title: Tray and settings
status: current
covers:
  - src-tauri/src/tray.rs
  - src-tauri/src/settings.rs
  - src-tauri/src/lib.rs
  - src-tauri/windows/hooks.nsh
  - packages/shell/src/settings/**
refs:
  - "[code://src-tauri/src/tray.rs#L42](../../src-tauri/src/tray.rs#L42) - `install` and `set_tray_words`, the icon and its menu"
  - "[code://src-tauri/src/tray.rs#L60](../../src-tauri/src/tray.rs#L60) - `shown`, whether the desktop shows the tray"
  - "[code://src-tauri/src/settings.rs#L85](../../src-tauri/src/settings.rs#L85) - `save_settings`, which keeps the stored `startWithOsAsked`"
  - "[code://src-tauri/src/settings.rs#L125](../../src-tauri/src/settings.rs#L125) - `set_start_with_os`, the OS's login entry"
  - "[code://src-tauri/src/lib.rs#L34](../../src-tauri/src/lib.rs#L34) - the single-instance plugin, first"
  - "[code://src-tauri/src/lib.rs#L92](../../src-tauri/src/lib.rs#L92) - the host list built hidden on a start at login, unless the desktop shows no tray"
  - "[code://src-tauri/src/lib.rs#L102](../../src-tauri/src/lib.rs#L102) - the host list's close, on `closeBehavior` and the tray"
  - "[code://src-tauri/windows/hooks.nsh#L7](../../src-tauri/windows/hooks.nsh#L7) - the uninstaller removes the login entry, an update keeps it"
  - https://github.com/tauri-apps/tauri/blob/tauri-cli-v2.12.1/crates/tauri-bundler/src/bundle/windows/nsis/installer.nsi - an update runs the old uninstaller with `/UPDATE`
  - git://4c89294 - the deleted decision this spec was written from: the-desk-starts-with-the-os-by-default
---

The desk lives in the tray: closing the host list hides it, and the tray icon brings it back.
On a desktop that shows no tray, such as GNOME without its AppIndicator extension, closing the host list quits the desk.
`settings.json`, beside `hosts.json`, holds the desk's own settings, set on the Settings screen of the host list window.
The desk starts with the OS by default, through the OS's own login entry.
A second start of the desk shows the running one.
The code is in [`code://src-tauri/src/tray.rs`](../../src-tauri/src/tray.rs) and [`code://src-tauri/src/settings.rs`](../../src-tauri/src/settings.rs).

## Scenarios

### The tray icon

WHEN the desk runs
THEN a tray icon with the tooltip `ahp-desk` is there
AND its menu is Hosts, Settings and Quit, in the host list's language, and changes with a pick
AND on Windows a left click shows the host list; on Linux the menu is the only way in
AND Hosts shows the host list window on its list, from any screen, and drops a half-filled form
AND Settings shows the host list window on its Settings screen
AND Quit exits as a close does, so the local ahpd's stop on quit runs.

### Closing the host list

WHEN the host list is closed
THEN with an app window open it hides
AND with none open, it hides to the tray when `closeBehavior` is `minimize-to-tray`, the default, and the desktop shows a tray
AND the desk quits when `closeBehavior` is `quit`, or when the desktop shows no tray.

### Whether the desktop shows a tray

WHEN the desk asks whether the desktop shows its tray icon, at each close of the host list and at a start at login
THEN on Linux it asks the session bus's `org.kde.StatusNotifierWatcher` for `IsStatusNotifierHostRegistered`, and no watcher, or no session bus, is no tray
AND on Windows and macOS the tray is always there.

### The settings file

WHEN the desk reads `settings.json`
THEN a missing file is the defaults
AND a field the file does not have takes its default, so a file from before `windows` opens hosts per host
AND a file that is not settings is an error naming it.

### Settings

WHEN Settings opens, from the gear in the title bar or the tray
THEN it shows "When I close the host list", with "Hide it in the tray" and "Quit ahp-desk", and "Start with the OS"
AND "Hosts open in", with "A window for each host", the default, and "One window with every host"
AND each change is saved at once
AND the gear is not there during the first run.

### A second start

WHEN the desk is started while it runs
THEN the running desk shows its host list, and the second start exits.

### Start with the OS

WHEN "Start with the OS" is set
THEN the desk writes or removes the OS's login entry: the value `ahp-desk` under `HKCU\Software\Microsoft\Windows\CurrentVersion\Run` on Windows, `~/.config/autostart/ahp-desk.desktop` on Linux
AND the first run shows the box ticked the first time
AND a first run that comes back after the last host is removed shows the login entry as it is
AND a start at login, with `--autostart`, builds the host list hidden and shows only the tray icon
AND on a desktop that shows no tray, that start shows the host list.

### Removing the desk

WHEN the desk is removed on Windows
THEN the uninstaller removes the login entry, and the one Task Manager keeps beside it
AND an update, which runs the old uninstaller, keeps them.

## Known gaps

- Unticking "Start with the OS" leaves Task Manager's value until the desk is removed.
- A desktop with only the older XEmbed tray, which libayatana-appindicator can fall back to, counts as no tray.
- A tray host that registers a moment after a start at login counts as no tray for that start, so the host list shows once.
