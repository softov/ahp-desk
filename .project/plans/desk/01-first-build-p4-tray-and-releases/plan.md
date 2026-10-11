---
title: The tray, the updater and the releases
domain: desk
status: draft
priority: medium
created: 2026-10-11
revalidated: 2026-10-11
requires:
  - plans/desk/01-first-build-p1-shell-and-url-hosts/plan.md
decisions:
  - decisions/the-desk-starts-with-the-os-by-default.md
refs:
  - file:///github/opendoop/packages/tauri/src-tauri/src/tray.rs - the tray menu and the left click, at lines 8-62
  - file:///github/opendoop/packages/tauri/src-tauri/src/shell_settings.rs - closeBehavior, minimize to tray or quit
  - file:///github/opendoop/packages/tauri/src-tauri/tauri.conf.json - NSIS for the current user, createUpdaterArtifacts, the updater endpoint
  - file:///github/opendoop/packages/tauri/PUBLISH.md - doop's release by hand, and latest.json
  - file:///github/ahp-desk/DEVELOPER.md - the signing keys and the Linux bundles
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

| What | Source | Task |
| --- | --- | --- |
| Close hides to the tray by default; a setting can make it quit. | doop's `closeBehavior` | 02 |
| CI builds the releases with GitHub Actions on a tag `v*`. | (defaulted: DEVELOPER.md speaks of "a CI run") | 04 |
| Windows: NSIS for the current user. Linux: AppImage and deb. | DEVELOPER.md | 04 |

## Proposed architecture

- **Data flow** - `settings.json` in the app config folder holds `closeBehavior`.
- **Event flow** - The tray menu calls the same commands as the shell screens.
- **State flow** - The signing key is a CI secret and is never in git.
- **Layer responsibilities** - `src-tauri`: tray, close, updater · `.github/workflows`: release.
- **Source-of-truth files** - `src-tauri/src/tray.rs`, `.github/workflows/release.yml`.

## Tasks

These task files are written when this plan moves to `planned`.

| Task | Status | Depends on |
| --- | --- | --- |
| 01 - tray icon and menu | todo | - |
| 02 - close to the tray, and the setting | todo | 01 |
| 03 - updater with the signing key | todo | - |
| 04 - release workflow for Windows and Linux | todo | 03 |
| 05 - start with the OS, on the first run and in Settings | todo | 01 |
| 06 - install, update and remove on a clean Windows | todo | 04, 05 |

## Risks and tradeoffs

- The tray on Linux needs libayatana-appindicator. DEVELOPER.md lists it.
- Windows SmartScreen warns about an installer with no code signature. Code signing is not in this plan.

## Resume state

- **Done so far:** nothing.
- **Next action:** write the task files, then task 01.
- **Open questions:** none.
- **Watch out for:** the updater needs `createUpdaterArtifacts: true`, or no `.sig` file is made.

## Final verification checklist

- [ ] A tag builds both installers in CI, and the release has `latest.json`.
- [ ] An older install finds the new release and updates.
- [ ] `plans/index.md` updated.
