---
title: The tray, the updater and the releases
domain: desk
status: active
priority: medium
created: 2026-10-11
revalidated: 2026-10-14
requires:
  - plans/desk/01-shell-and-url-hosts/plan.md
decisions:
  - decisions/ahp-desk-is-mit-and-ahpapp-stays-private.md
  - decisions/updater-artifacts-are-made-only-in-the-release-build.md
  - decisions/the-linux-release-builds-on-ubuntu-22-04.md
  - decisions/a-desk-update-stops-the-local-ahpd-the-desk-started.md
  - decisions/the-installer-stops-the-local-ahpd-before-it-writes.md
  - decisions/the-uninstaller-asks-then-stops-the-local-ahpd.md
  - decisions/each-install-carries-only-its-own-ui.md
refs:
  - file:///github/opendoop/packages/tauri/src-tauri/src/tray.rs - the tray menu and the left click, at lines 8-62
  - file:///github/opendoop/packages/tauri/src-tauri/src/shell_settings.rs - closeBehavior, minimize to tray or quit
  - file:///github/opendoop/packages/tauri/src-tauri/tauri.conf.json - NSIS for the current user, createUpdaterArtifacts, the updater endpoint
  - file:///github/opendoop/packages/tauri/PUBLISH.md - doop's release by hand, and latest.json
  - "[code://DEVELOPER.md](../../../../DEVELOPER.md) - the signing keys and the Linux bundles"
---

## Goal

ahp-desk has a tray icon with its menu, and closing the last window hides the app to the tray.
It checks GitHub releases for an update and installs a signed one.
A tag in git makes CI build the Windows and Linux installers and publish them with `latest.json`.

## Reconnaissance

The files read and the patterns to reuse are the `refs` above, each with its note.

### Searches performed

- `ls opendoop/.github` - `Not found: .github - doop has no CI.`

### Runtime path

```
tray icon -> left click (Windows) -> host list
tray icon -> menu (Hosts, Settings, Quit) -> shell commands
start at login (--autostart) -> host list hidden, tray icon only
start -> updater check -> https://github.com/softov/ahp-desk/releases/latest/download/latest.json -> download -> notice in the host list and the tray -> Restart click -> install
git tag v* -> GitHub Actions -> tauri-action with --config src-tauri/tauri.release.conf.json on windows-latest and ubuntu-22.04 -> release with latest.json
```

### Gaps

- No CI in any of these repositories builds a Tauri app.

## Answers

| What | Source | Task |
| --- | --- | --- |
| [The desk starts with the OS by default](../../../specs/tray-and-settings.md) | Softov, 2026-10-11 | 05 |
| [ahp-desk is MIT, and ahpapp stays private](../../../decisions/ahp-desk-is-mit-and-ahpapp-stays-private.md) | Softov, 2026-10-11 | 03, 04 |
| The desk downloads its own update in the background, then says "ahp-desk X is ready. Restart to update." in the host list and the tray, and restarts only on that click. | Softov, 2026-10-13, asked "When the desk itself has an update, what happens?": "Ask, then install (Recommended)" | 03 |
| This plan builds the desk's update notice in the host list, its Restart click that installs the update, and its tray item; plan 09 task 06 keeps only the check every 6 hours. | Softov, 2026-10-14: "p4 task 03" | 03 |
| [A desk update stops the local ahpd the desk started, and the new desk starts it again](../../../decisions/a-desk-update-stops-the-local-ahpd-the-desk-started.md) | Softov, 2026-10-20: "Stop ahpd for the install (Recommended)" | 03, 06 |
| [The installer stops the local ahpd before it writes the desk's files](../../../decisions/the-installer-stops-the-local-ahpd-before-it-writes.md) | Softov, 2026-10-22: "Installer stops it too (Recommended)" | 06 |
| [The uninstaller asks, then stops the local ahpd, so its Node can be removed](../../../decisions/the-uninstaller-asks-then-stops-the-local-ahpd.md) | Softov, 2026-10-22: "Hook asks first" | 06 |
| [Each install carries only its own UI files](../../../decisions/each-install-carries-only-its-own-ui.md) | Softov, 2026-10-22: "Installer clears ui\ first (Recommended)" | 06 |
| On Windows a left click on the tray icon shows and focuses the host list, and the right click opens the menu; on Linux the menu is the only way in. | Softov, 2026-10-14: "Show the host list" | 01 |
| The tray menu is Hosts, Settings and Quit, with no "Show pill"; plan 06 adds "Show pill" when it builds the pill. | Softov, 2026-10-14: "No, p6 adds it" | 01 |
| A gear in the host list's title bar, before the language picker, opens Settings. | Softov, 2026-10-14: "Gear in the title bar" | 02 |
| A start at login (`--autostart`) builds the host list hidden and shows only the tray icon; a start by hand, or a second start, shows the host list. | Softov, 2026-10-14: "No, tray only" | 05 |
| Each build job of the release workflow runs `tauri-apps/tauri-action`, which uploads the bundles with their `.sig` files and merges its platform into `latest.json`. | Softov, 2026-10-14: "tauri-apps/tauri-action" | 04 |
| [The Linux release builds on ubuntu-22.04](../../../decisions/the-linux-release-builds-on-ubuntu-22-04.md) | Softov, 2026-10-14: "ubuntu-22.04" | 04 |
| [The updater artifacts are made only in the release build](../../../decisions/updater-artifacts-are-made-only-in-the-release-build.md) | Softov, 2026-10-14: "Release config only" | 03, 04 |
| Close hides to the tray by default; a setting can make it quit. | doop's `closeBehavior` | 02 |
| CI builds the releases with GitHub Actions on a tag `v*`. | (defaulted: DEVELOPER.md speaks of "a CI run") | 04 |
| Windows: NSIS for the current user. Linux: AppImage and deb. | DEVELOPER.md | 04 |
| The release workflow builds the public build only, with `AHP_DESK_UI=ahpw` and `AHP_DESK_AHPW=npm`. | [ahp-desk is MIT, and ahpapp stays private](../../../decisions/ahp-desk-is-mit-and-ahpapp-stays-private.md) | 04 |

## Proposed architecture

- **Data flow** - `settings.json` in the app config folder holds `closeBehavior`; the OS's login entry is the start-with-the-OS setting.
- **Event flow** - The tray menu calls the same commands as the shell screens; the host list sends the tray its words in its language.
- **State flow** - The signing key is a CI secret and is never in git.
- **Layer responsibilities** - `src-tauri`: tray, close, settings, autostart, updater · `packages/shell`: the Settings screen, the first run's checkbox, the update notice · `.github/workflows`: release.
- **Source-of-truth files** - `src-tauri/src/tray.rs`, `src-tauri/src/settings.rs`, `src-tauri/src/updates.rs`, `.github/workflows/release.yml`.

## Tasks

| Task | Status | Depends on |
| --- | --- | --- |
| [01 - tray icon and menu](task-01-tray-icon-and-menu.md) | doing | - |
| [02 - close to the tray, and the setting](task-02-close-to-tray-and-setting.md) | doing | 01 |
| [03 - updater with the signing key](task-03-updater-with-signing-key.md) | done | 01 |
| [04 - release workflow for Windows and Linux](task-04-release-workflow.md) | done | 03 |
| [05 - start with the OS, on the first run and in Settings](task-05-start-with-the-os.md) | doing | 01, 02 |
| [06 - install, update and remove on a clean Windows](task-06-install-update-remove-on-windows.md) | todo | 04, 05 |

## Risks and tradeoffs

- The tray on Linux needs libayatana-appindicator. DEVELOPER.md lists it.
- Windows SmartScreen warns about an installer with no code signature. Code signing is not in this plan.
- A lost updater key leaves every installed desk unable to update; the user keeps a copy outside the machine (task 03).

## Resume state

- **Done so far:** tasks 01, 02 and 05 are built and checked on the debug desk, 2026-10-16.
  Each one's Resume lists the checks left by hand.
  Task 03 is built and checked on a release build, and task 04 made the v0.1.0 release, 2026-10-18.
  Tasks 01, 02 and 05 checked by hand on Windows on a release build, 2026-10-21; each keeps Debian 13, and task 01 the menu's words in Portuguese and Spanish.
  Task 06 ran on v0.1.0 and v0.2.0, 2026-10-22, and found three installer defects, now in `hooks.nsh` for 0.2.1.
  The 0.2.1 hooks checked by hand and v0.2.1 released, 2026-10-22.
- **Next action:** task 06's update to 0.2.1 from the desk's own Restart, on 0.1.0 and on 0.2.0; step 11 on Debian 13.
  The Debian 13 checks of tasks 01, 02 and 05 can run beside it.
- **Open questions:** none.
- **Watch out for:** the release build needs `--config src-tauri/tauri.release.conf.json`, which turns on `createUpdaterArtifacts` (Answers: the updater artifacts), or no `.sig` file is made; an update must not remove the login entry that the uninstaller hook removes (task 05).

## Final verification checklist

- [x] A tag builds both installers in CI, and the release has `latest.json`.
- [ ] An older install finds the new release and updates.
- [ ] `plans/index.md` updated.
