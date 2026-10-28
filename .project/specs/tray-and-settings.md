---
title: Tray and settings
status: current
covers:
  - src-tauri/src/tray.rs
  - src-tauri/src/settings.rs
  - src-tauri/src/lib.rs
  - src-tauri/windows/hooks.nsh
  - src-tauri/src/side_windows.rs
  - packages/shell/src/settings/**
  - packages/shell/src/about/**
  - packages/shell/src/updates/**
  - packages/shell/src/TitleBar.tsx
  - packages/shell/src/WindowBar.tsx
refs:
  - "[code://src-tauri/src/tray.rs#L49](../../src-tauri/src/tray.rs#L49) - `install` and `set_tray_words`, the icon and its menu"
  - "[code://src-tauri/src/tray.rs#L67](../../src-tauri/src/tray.rs#L67) - `shown`, whether the desktop shows the tray"
  - "[code://src-tauri/src/side_windows.rs](../../src-tauri/src/side_windows.rs) - `open_side`, the About and Updates windows"
  - "[code://packages/shell/src/TitleBar.tsx](../../packages/shell/src/TitleBar.tsx) - the host list's title bar and its logo menu"
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
About and the update check open in small windows of their own, beside the host list, which keeps its screen.
The desk starts with the OS by default, through the OS's own login entry.
A second start of the desk shows the running one.
The code is in [`code://src-tauri/src/tray.rs`](../../src-tauri/src/tray.rs) and [`code://src-tauri/src/settings.rs`](../../src-tauri/src/settings.rs).

## Scenarios

### The tray icon

WHEN the desk runs
THEN a tray icon with the tooltip `ahp-desk` is there
AND its menu is Hosts, Settings, Check for updates…, About ahp-desk and Quit, in the host list's language, and changes with a pick
AND while a desk update is ready, its item comes first, and opens the Updates window
AND on Windows a left click shows the host list; on Linux the menu is the only way in
AND Hosts shows the host list window on its list, from any screen, and drops a half-filled form
AND Settings shows the host list window on its Settings screen
AND Check for updates… opens the Updates window, and About ahp-desk the About window, or brings it to the front
AND Quit exits as a close does, so the local ahpd's stop on quit runs.

### The title bar's logo menu

WHEN the logo at the start of the host list's title bar is clicked
THEN its menu is Hosts, Add host, Settings, a line, Check for updates…, About ahp-desk, a line, and Quit
AND during the first run it has only Quit
AND the title bar has no gear; the logo menu is the way to Settings.

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

WHEN Settings opens, from the logo menu or the tray
THEN it shows "When I close the host list", with "Hide it in the tray" and "Quit ahp-desk", and "Launch at startup"
AND "Hosts open in", with "A window for each host", the default, and "One window with every host"
AND each change is saved at once
AND the links "About ahp-desk" and "Check for updates…", away from Back, open the About and Updates windows.

### About

WHEN About opens, from the tray, the logo menu or Settings
THEN a window of 360 by 320, which cannot resize or maximize, opens centred, or the open one comes to the front
AND its title bar shows the logo, "About ahp-desk", minimize and close
AND it shows the versions of ahp-desk, of the ahpw it carries and of its bundled Node
AND the ahpw version is baked in at build from the staged `@ahpd/web`, and the Node version is `node --version` of the bundled Node
AND it shows "MIT license" and the links ahpio.com and Releases, opened in the browser
AND the host list keeps its screen.

### The Updates window

WHEN Check for updates… is picked, from the tray, the logo menu or Settings, or the tray's ready update
THEN a window of 400 by 260, which cannot resize or maximize, opens centred, or the open one comes to the front
AND its title bar shows the logo, "Check for updates", minimize and close
AND it shows the running desk's version
AND with a desk update ready, it shows the update notice and its Restart, which names the turns running on a local ahpd the desk started, from the host list's published map, before a second Restart installs
AND otherwise Check now asks for a newer desk at once, and shows "Checking…"
AND then it shows "ahp-desk is up to date", the ready notice, or the error
AND a debug build answers Check now with "a debug build does not check for updates".

### The side windows' language

WHEN a language is picked in the host list
THEN an open About or Updates window changes to it.

### A second start

WHEN the desk is started while it runs
THEN the running desk shows its host list, and the second start exits.

### Launch at startup

WHEN "Launch at startup" is set
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

- Unticking "Launch at startup" leaves Task Manager's value until the desk is removed.
- A desktop with only the older XEmbed tray, which libayatana-appindicator can fall back to, counts as no tray.
- A tray host that registers a moment after a start at login counts as no tray for that start, so the host list shows once.
