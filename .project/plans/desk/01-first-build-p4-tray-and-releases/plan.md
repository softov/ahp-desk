---
title: The tray, the updater and the releases
domain: desk
status: planned
priority: medium
created: 2026-10-11
revalidated: 2026-10-14
requires:
  - plans/desk/01-first-build-p1-shell-and-url-hosts/plan.md
decisions:
  - decisions/the-desk-starts-with-the-os-by-default.md
  - decisions/ahp-desk-is-mit-and-ahpapp-stays-private.md
  - decisions/the-desk-asks-before-installing-its-update.md
  - decisions/the-desk-update-notice-is-built-in-p4-task-03.md
  - decisions/a-left-click-on-the-tray-icon-shows-the-host-list.md
  - decisions/the-tray-menu-gets-show-pill-with-the-pill.md
  - decisions/the-host-list-opens-settings-from-a-gear-in-its-title-bar.md
  - decisions/a-start-at-login-shows-only-the-tray.md
  - decisions/the-release-workflow-builds-with-tauri-action.md
  - decisions/the-linux-release-builds-on-ubuntu-22-04.md
  - decisions/updater-artifacts-are-made-only-in-the-release-build.md
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

## Decisions locked in

| # | Decision | Rationale / source |
| --- | --- | --- |
| 1 | [The desk starts with the OS by default](../../../decisions/the-desk-starts-with-the-os-by-default.md) | Softov, 2026-10-11 |
| 2 | [ahp-desk is MIT, and ahpapp stays private](../../../decisions/ahp-desk-is-mit-and-ahpapp-stays-private.md) | Softov, 2026-10-11 |
| 3 | [The desk asks before it installs its own update](../../../decisions/the-desk-asks-before-installing-its-update.md) | Softov, 2026-10-13 |
| 4 | [The desk's update notice, its install and its tray item are built in p4 task 03](../../../decisions/the-desk-update-notice-is-built-in-p4-task-03.md) | Softov, 2026-10-14 |
| 5 | [A left click on the tray icon shows the host list, and the right click opens the menu](../../../decisions/a-left-click-on-the-tray-icon-shows-the-host-list.md) | Softov, 2026-10-14 |
| 6 | [The tray menu gets "Show pill" with the pill, in plan p6](../../../decisions/the-tray-menu-gets-show-pill-with-the-pill.md) | Softov, 2026-10-14 |
| 7 | [The host list opens Settings from a gear in its title bar, before the language picker](../../../decisions/the-host-list-opens-settings-from-a-gear-in-its-title-bar.md) | Softov, 2026-10-14 |
| 8 | [A start at login shows only the tray, not the host list](../../../decisions/a-start-at-login-shows-only-the-tray.md) | Softov, 2026-10-14 |
| 9 | [The release workflow builds and publishes with tauri-apps/tauri-action](../../../decisions/the-release-workflow-builds-with-tauri-action.md) | Softov, 2026-10-14 |
| 10 | [The Linux release builds on ubuntu-22.04](../../../decisions/the-linux-release-builds-on-ubuntu-22-04.md) | Softov, 2026-10-14 |
| 11 | [The updater artifacts are made only in the release build](../../../decisions/updater-artifacts-are-made-only-in-the-release-build.md) | Softov, 2026-10-14 |

| What | Source | Task |
| --- | --- | --- |
| Close hides to the tray by default; a setting can make it quit. | doop's `closeBehavior` | 02 |
| CI builds the releases with GitHub Actions on a tag `v*`. | (defaulted: DEVELOPER.md speaks of "a CI run") | 04 |
| Windows: NSIS for the current user. Linux: AppImage and deb. | DEVELOPER.md | 04 |
| The release workflow builds the public build only, with `AHP_DESK_UI=ahpw` and `AHP_DESK_AHPW=npm`. | Decision 2 | 04 |

## Proposed architecture

- **Data flow** - `settings.json` in the app config folder holds `closeBehavior`; the OS's login entry is the start-with-the-OS setting.
- **Event flow** - The tray menu calls the same commands as the shell screens; the host list sends the tray its words in its language.
- **State flow** - The signing key is a CI secret and is never in git.
- **Layer responsibilities** - `src-tauri`: tray, close, settings, autostart, updater · `packages/shell`: the Settings screen, the first run's checkbox, the update notice · `.github/workflows`: release.
- **Source-of-truth files** - `src-tauri/src/tray.rs`, `src-tauri/src/settings.rs`, `src-tauri/src/updates.rs`, `.github/workflows/release.yml`.

## Tasks

| Task | Status | Depends on |
| --- | --- | --- |
| [01 - tray icon and menu](task-01-tray-icon-and-menu.md) | todo | - |
| [02 - close to the tray, and the setting](task-02-close-to-tray-and-setting.md) | todo | 01 |
| [03 - updater with the signing key](task-03-updater-with-signing-key.md) | todo | 01 |
| [04 - release workflow for Windows and Linux](task-04-release-workflow.md) | todo | 03 |
| [05 - start with the OS, on the first run and in Settings](task-05-start-with-the-os.md) | todo | 01, 02 |
| [06 - install, update and remove on a clean Windows](task-06-install-update-remove-on-windows.md) | todo | 04, 05 |

## Risks and tradeoffs

- The tray on Linux needs libayatana-appindicator. DEVELOPER.md lists it.
- Windows SmartScreen warns about an installer with no code signature. Code signing is not in this plan.
- A lost updater key leaves every installed desk unable to update; the user keeps a copy outside the machine (task 03).

## Resume state

- **Done so far:** the task files, and decisions 4 to 11 from Softov's answers, 2026-10-14.
- **Next action:** [task 01](task-01-tray-icon-and-menu.md). Before task 03, the user makes the updater key pair; before task 04's first tag, the user adds the two CI secrets.
- **Open questions:** none.
- **Watch out for:** the release build needs `--config src-tauri/tauri.release.conf.json`, which turns on `createUpdaterArtifacts` (decision 11), or no `.sig` file is made; an update must not remove the login entry that the uninstaller hook removes (task 05).

## Final verification checklist

- [ ] A tag builds both installers in CI, and the release has `latest.json`.
- [ ] An older install finds the new release and updates.
- [ ] `plans/index.md` updated.
