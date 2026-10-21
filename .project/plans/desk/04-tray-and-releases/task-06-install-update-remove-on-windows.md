---
title: A released desk installs, updates and is removed cleanly on a clean Windows user
status: todo
depends: [task-04-release-workflow.md, task-05-start-with-the-os.md]
layer: "ahp-desk"
refs:
  - file:///github/opendoop/packages/tauri/PUBLISH.md - "Sanity-check the installer" and "Verify the update flow", the checks this task makes on ahp-desk
  - "[code://DEVELOPER.md#L110-L117](../../../../DEVELOPER.md#L110-L117) - testing on Windows: the installer for the current user, and removal from Apps"
---

## Objective

Every item of the plan's checklist holds on the released installers, the way a user meets them: download, install, tray, login start, update on the click, removal.

## Files

- None. A defect found here is fixed in the task that owns its file, and recorded below.

## Steps

1. Release A: the user tags the version the files say (task 04); the release is published with `latest.json`.
2. On a Windows user that has never had the desk, Node or ahpd, download A's `-setup.exe` from the release page in a browser.
   SmartScreen warns, since the installer has no code signature (plan risk); "Run anyway" goes on.
3. Install: no admin prompt (`installMode: currentUser`); the files land under `%LOCALAPPDATA%`; the Start menu has ahp-desk.
4. First run: "Start with the OS" is checked; Get started; add a URL host and open it; the tray icon is there.
5. Close the host list: the desk stays in the tray; a left click on the icon shows the host list; the right click opens the menu, whose Hosts, Settings and Quit work, in the three languages.
6. Sign out and in: the desk starts in the tray, and the host list does not show (task 05).
7. Release B: bump the version, tag it; wait for its release.
8. Start A, or restart it: within a minute the host list and the tray say "ahp-desk B is ready. Restart to update."; nothing restarts until the click (Answers: asking before the update).
9. Click Restart: the installer runs passively, the desk comes back as B, with the same hosts, tokens, settings and login entry; a local ahpd the desk started was stopped first, whatever "Stop when the desktop app quits" says, and B starts it again by "Start when the desktop app starts" (Answers: a desk update stops the local ahpd).
   With a turn running on that ahpd, Restart names it and waits for a second click; the tray's item opens the host list to ask the same.
10. Remove from Windows Settings, Apps: the install folder, the Start menu entry, the tray icon and the login entry are gone; write down whether the config folder (`%APPDATA%\dev.softov.ahpdesk`) and the OS keyring's tokens stay, and what the uninstaller asked.
11. A smoke check of the Linux bundles on Debian 13: the release's AppImage starts and shows the tray, and the deb installs with `apt install ./ahp-desk_<version>_amd64.deb`.

## Validation

- The plan's final verification checklist, each item ticked with the release versions it was checked on.
- plan.md moves to `built` with `implemented.md` written, and `plans/index.md` with it.

## Resume

- 2026-10-20, found before the run, on a Windows 10 VM: the NSIS installer of a desk build, run over a desk whose local ahpd was on ("Stop when the desktop app quits" off, the default), stopped at "Error opening file for writing ...\ahp-desk\node\node.exe", since that ahpd runs on the desk's bundled Node; Retry went on once ahpd was stopped by hand.
  Fixed by [a desk update stops the local ahpd the desk started](../../../decisions/a-desk-update-stops-the-local-ahpd-the-desk-started.md): `updates.rs`'s `on_before_exit` runs `before_desk_install` (`lib.rs`), which closes the tunnels and stops every local ahpd the desk started (`HostStore::to_stop_for_desk_install`, `local_ahpd::stop_for_desk_install`); the host list's Restart asks first over turns running there (`update.restartQuestion`), and the tray's item opens the host list to ask the same (`host-list-desk-update`).
  Verified: `cargo test` (137, `to_stop_for_desk_install` in `hosts_the_start_and_quit_settings_pick_the_hosts`), `cargo clippy --all-targets -- -D warnings`, `pnpm typecheck`, `pnpm test` (53).
  Not seen by hand: the stop, the question and the tray's item, which need a real ready update; step 9 checks them on the update from A to B.

