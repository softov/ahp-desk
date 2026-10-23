---
title: The tray, the updater and the releases - implemented
date: 2026-10-23
refs:
  - git://616d948 - main when the plan was closed; the Linux fixes are on the branch desk/no-tray-quits
  - "[code://src-tauri/src/tray.rs](../../../../src-tauri/src/tray.rs) - the icon, its menu, and `shown`"
  - "[code://src-tauri/src/settings.rs](../../../../src-tauri/src/settings.rs) - `settings.json` and the login entry"
  - "[code://src-tauri/src/updates.rs](../../../../src-tauri/src/updates.rs) - the desk's own update"
  - "[code://src-tauri/windows/hooks.nsh](../../../../src-tauri/windows/hooks.nsh) - the installer's and uninstaller's hooks"
  - "[code://.github/workflows/release.yml](../../../../.github/workflows/release.yml) - the release on a tag"
---

ahp-desk lives in the tray on Windows and Linux, starts with the OS, has a Settings screen for its close and its windows, updates itself from signed GitHub releases, and a tag builds and publishes the Windows installer, the AppImage and the deb with `latest.json`; v0.1.0, v0.2.0 and v0.2.1 were released this way.

## What was built

- [`code://src-tauri/src/tray.rs`](../../../../src-tauri/src/tray.rs) - the icon, its menu in three languages, and `shown`, which asks Linux's StatusNotifier watcher whether a tray is there.
- [`code://src-tauri/src/settings.rs`](../../../../src-tauri/src/settings.rs) - `settings.json`: the close, the windows, and the login entry through the autostart plugin.
- [`code://src-tauri/src/lib.rs`](../../../../src-tauri/src/lib.rs) - the host list's close and the start at login, both on `tray::shown`.
- [`code://src-tauri/src/updates.rs`](../../../../src-tauri/src/updates.rs) - the check, the download, the notice and the Restart, with the local ahpd stopped first.
- [`code://src-tauri/windows/hooks.nsh`](../../../../src-tauri/windows/hooks.nsh) - the install stops the local ahpd and clears `ui\`; the uninstall asks, then stops it; the login entry goes with an uninstall and stays through an update.
- [`code://src-tauri/src/app_window.rs`](../../../../src-tauri/src/app_window.rs) - `is_ui`, so an app window loads the UI on Linux.
- [`code://packages/shell/src/window.ts`](../../../../packages/shell/src/window.ts) - `pointer-away`, so a window that comes back shows no old hover.
- [`code://.github/workflows/release.yml`](../../../../.github/workflows/release.yml) - the version check, the draft, the builds on Windows and Ubuntu 22.04, and the publish.

## Verified

- `cargo test`: 144 pass on Windows; 141 and clippy with `-D warnings` pass on Debian 13 before `is_ui`'s three tests were added; the shell's 53 tests and its typecheck pass.
- By hand on a Windows 10 VM, 2026-10-21 and 22: tasks 01, 02 and 05 on a release build; task 06's install of v0.1.0, its update to 0.2.0, the 0.2.1 hooks with the updater's flags, the uninstall's OK and Cancel, and the Restart from 0.2.0 to 0.2.1.
- By hand on Debian 13 with GNOME 48, 2026-10-23: task 06's step 11 and the Debian checks of tasks 01, 02 and 05, on a deb built from the branch with the Linux fixes.

## Departures from the plan

- [With no tray to come back from, closing the host list quits](../../../decisions/with-no-tray-closing-the-host-list-quits.md) and [with no tray, a start at login shows the host list](../../../decisions/with-no-tray-a-start-at-login-shows-the-host-list.md) - stock GNOME shows no tray, which the plan did not foresee.
- Three installer decisions of 2026-10-22, from task 06's run on Windows.

## Left for later

- A release that carries the Linux fixes; v0.2.1's Linux bundles cannot open a host - see [deferred.md](deferred.md).
