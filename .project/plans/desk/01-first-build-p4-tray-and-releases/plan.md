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
tray icon -> menu (Hosts, Settings, Show pill, Quit) -> shell commands
start -> updater check -> https://github.com/softov/ahp-desk/releases/latest/download/latest.json -> download -> install
git tag v* -> GitHub Actions -> tauri build on windows-latest and ubuntu-latest -> release with latest.json
```

### Gaps

- No CI in any of these repositories builds a Tauri app.

## Decisions locked in

| # | Decision | Rationale / source |
| --- | --- | --- |
| 1 | [The desk starts with the OS by default](../../../decisions/the-desk-starts-with-the-os-by-default.md) | Softov, 2026-10-11 |
| 2 | [ahp-desk is MIT, and ahpapp stays private](../../../decisions/ahp-desk-is-mit-and-ahpapp-stays-private.md) | Softov, 2026-10-11 |
| 3 | [The desk asks before it installs its own update](../../../decisions/the-desk-asks-before-installing-its-update.md) | Softov, 2026-10-13 |

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

- **Done so far:** the task files, 2026-10-14.
- **Next action:** [task 01](task-01-tray-icon-and-menu.md). Before task 03, the user makes the updater key pair; before task 04's first tag, the user adds the two CI secrets.
- **Open questions:** each has a proposed answer, which the task files follow until it is answered.
  1. Which task shows the desk's update notice and its Restart click: p4 task 03, or [p9 task 06](../01-first-build-p9-update-alerts/task-06-desk-update-notice.md), whose file has it too? Proposed: p4 task 03, with the line in the host list and the item in the tray, since this plan's goal and checklist ask for an update a user can install; p9 task 06 then keeps only the check every 6 hours, moved into its loop. With the other answer, task 03 keeps its steps 1 to 4, and task 06 here waits for p9 task 06.
  2. What does a left click on the tray icon do on Windows? Proposed: show the host list, and the right click opens the menu, as doop's left click shows its pill; on Linux the menu is the only way in.
  3. Is "Show pill" in this plan's menu, before the pill exists? Proposed: no; p4's menu is Hosts, Settings and Quit, and p6 task 01 adds Show pill.
  4. Where does the host list open Settings, besides the tray? Proposed: a gear button in the host list's title bar, before the language picker.
  5. When the OS starts the desk at login, does the host list show? Proposed: no; the desk starts in the tray, as decision 1's "the pill and the tray are there after a login" reads, and the host list shows on a start by hand.
  6. Does the release workflow build with `tauri-apps/tauri-action`, or with its own steps and a script for `latest.json`? Proposed: `tauri-action`, which signs, uploads and merges `latest.json` per platform.
  7. Which Linux runner builds the release: `ubuntu-latest` (24.04), as the runtime path says, or `ubuntu-22.04`? Proposed: `ubuntu-22.04`, so the AppImage runs on distributions with an older glibc.
  8. Is `createUpdaterArtifacts: true` in `tauri.conf.json`, or only in the release build? Proposed: only in the release build, through `src-tauri/tauri.release.conf.json` and `--config`, so a clone of this MIT repository builds an installer with no key.
- **Watch out for:** the release build needs `createUpdaterArtifacts: true`, or no `.sig` file is made; an update must not remove the login entry that the uninstaller hook removes (task 05).

## Final verification checklist

- [ ] A tag builds both installers in CI, and the release has `latest.json`.
- [ ] An older install finds the new release and updates.
- [ ] `plans/index.md` updated.
