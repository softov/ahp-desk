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

- 2026-10-22, run on a Windows 10 VM as a new standard user, with the published releases v0.1.0 (A) and v0.2.0 (B, tag on `11e35fa`, ahpw 0.6.0):
  - steps 1 to 3: A's `-setup.exe` downloaded from the release page in Edge, whose download list warned "isn't commonly downloaded" (kept with Keep anyway), and SmartScreen's "Windows protected your PC" with Run anyway; the hash matched the release page; the install asked for no admin and went to `%LOCALAPPDATA%\ahp-desk`;
  - step 4: the first run with "Start with the OS" ticked; This computer installed ahpd 0.13.0 and opened in ahpw 0.5.0;
  - step 7: B was released by the workflow in about 9 minutes, `latest.json` with `windows-x86_64`, `windows-x86_64-nsis`, `linux-x86_64`, `-appimage` and `-deb`;
  - step 8: A, started again, said "ahp-desk 0.2.0 is ready. Restart to update." in the host list and as the tray's first item;
  - step 9: Restart started the installer, which stopped at "Error opening file for writing ...\ahp-desk\node\node.exe", since A (0.1.0) has no stop before its installer and the local ahpd was on; after ahpd was stopped by hand, Retry finished, and B came up at 0.2.0 (640x480) with the same host and token, the login entry kept, ahpd started again by "Start when the desktop app starts", and ahpw 0.6.0 on Open;
  - step 10: Windows Settings, Apps, listed ahp-desk 0.2.0; Uninstall asked only "Delete the application data", unticked, and said it completed, but left the local ahpd running on `node\node.exe`, that file, and ahpw 0.5.0's assets under `ui\ahpw\assets`, which the update had written beside 0.6.0's; the Run value, the Start menu and desktop shortcuts were removed, and `%APPDATA%\dev.softov.ahpdesk` and `<app local data>` stayed.
- Fixed for 0.2.1 in `src-tauri/windows/hooks.nsh`, by three decisions: the pre-install hook closes the desk as the template does, ends every process under `$INSTDIR\node` and empties `$INSTDIR\ui` ([the installer stops the local ahpd](../../../decisions/the-installer-stops-the-local-ahpd-before-it-writes.md), [each install carries only its own UI](../../../decisions/each-install-carries-only-its-own-ui.md)); the pre-uninstall hook asks before it ends them, and does not ask under `/UPDATE` or `/S` ([the uninstaller asks](../../../decisions/the-uninstaller-asks-then-stops-the-local-ahpd.md)).
  The hooks compile in the 0.2.1 NSIS build; not yet run by hand, since the VM's console was taken by another session during the run.
- Left: the 0.2.1 hooks by hand (an update from 0.1.0 with ahpd on, run as the updater does with `/P /UPDATE`, and an uninstall with ahpd on, OK and Cancel); then the 0.2.1 release and its update from 0.2.0; step 11 on Debian 13.
