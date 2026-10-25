---
title: The About screen, the tray's Hosts, and the update of a local ahpd the desk did not install
domain: desk
status: active
priority: medium
created: 2026-10-23
revalidated: 2026-10-23
requires:
  - plans/desk/04-tray-and-releases/plan.md
  - plans/desk/09-update-alerts/plan.md
changes:
  - specs/tray-and-settings.md
  - specs/host-list.md
decisions:
  - decisions/the-trays-hosts-always-shows-the-host-list.md
  - decisions/a-local-ahpd-the-desk-did-not-install-updates-as-a-remote-one.md
  - decisions/the-desk-shows-its-versions-on-an-about-screen.md
refs:
  - "[code://src-tauri/src/tray.rs#L96-L114](../../../../src-tauri/src/tray.rs#L96-L114) - `on_menu_event`: Settings switches the screen, Hosts only shows the window"
  - "[code://src-tauri/src/app_window.rs#L318](../../../../src-tauri/src/app_window.rs#L318) - `HOST_LIST_SETTINGS`, the event pattern a new `HOST_LIST_HOSTS` and `HOST_LIST_ABOUT` follow"
  - "[code://packages/shell/src/App.tsx#L29](../../../../packages/shell/src/App.tsx#L29) - `Screen`, and the listeners at line 68"
  - "[code://packages/shell/src/hosts/status.ts#L242-L248](../../../../packages/shell/src/hosts/status.ts#L242-L248) - `updateWay`, which returns no way for a local ahpd the desk did not install"
  - "[code://packages/shell/src/hosts/HostList.tsx#L436-L455](../../../../packages/shell/src/hosts/HostList.tsx#L436-L455) - the row's notice and Update"
  - "[code://src-tauri/src/updates.rs#L75-L111](../../../../src-tauri/src/updates.rs#L75-L111) - `download_desk`, which a Check now runs"
  - "[code://src-tauri/src/local_ahpd.rs#L84-L100](../../../../src-tauri/src/local_ahpd.rs#L84-L100) - `BundledNode`, the bundled Node whose version About shows"
  - "[code://scripts/stage-ui.mjs](../../../../scripts/stage-ui.mjs) - where the bundled ahpw comes from, npm's `@ahpd/web` or a local checkout"
---

## Goal

Three gaps found on 2026-10-23 on a v0.2.1 desk are closed.
The tray's Hosts brings the host list back to the list from any screen.
A local ahpd the desk did not install can be updated from its row when ahpd allows it, and its row says how otherwise.
An About screen, from the tray and Settings, shows the desk's, ahpw's and Node's versions and the desk's update state, with Check now.
The desk's update notice inside ahpw's windows is not in this plan.

## Reconnaissance

The files read and the patterns to reuse are the `refs` above, each with its note.

### Searches performed

- `rg "HOST_LIST_" src-tauri/src` - the add, edit, settings and desk-update events; none for the list itself.
- `rg "getVersion|package_info|version" packages/shell/src` - nothing shows a version of the desk.
- ahpd's `CHANGELOG.md` at `701942c7` - `ahpd update`, `POST /api/update` and `x-ahpd/update` came in 0.13.0.

### Runtime path

```
tray Hosts -> on_menu_event -> show_host_list -> (new) emit host-list-hosts -> App.tsx -> screen list
host row -> statuses.updateWay -> local not by desk -> (new) ahpd's check -> Update, else the how-to words
tray About / Settings link -> (new) screen about -> about() + desk_update() -> Check now -> check_desk_update() -> download_desk -> desk-update-ready
```

### Gaps

- No event switches the host list window to its list.
- `updateWay` answers `null` for every local ahpd the desk did not install, even one whose own update command would work.
- No command answers the desk's, ahpw's or Node's version, and no command runs the desk's update check on demand.

## Decisions locked in

| # | Decision | Rationale / source |
| --- | --- | --- |
| 1 | [The tray's Hosts always shows the host list itself](../../../decisions/the-trays-hosts-always-shows-the-host-list.md) | Softov, 2026-10-23: "Always go to the list (Recommended)" |
| 2 | [A local ahpd the desk did not install updates as a remote one, else its row says how](../../../decisions/a-local-ahpd-the-desk-did-not-install-updates-as-a-remote-one.md) | Softov, 2026-10-23: "Both (Recommended)" |
| 3 | [The desk shows its versions on an About screen, from the tray and Settings](../../../decisions/the-desk-shows-its-versions-on-an-about-screen.md) | Softov, 2026-10-23: "About screen (Recommended)" |

| What | Source | Task |
| --- | --- | --- |
| The desk's update notice stays in the host list and the tray; ahpw's windows do not show it. | Softov, 2026-10-23, asked "The desk's update notice inside ahpw windows?": "Not now (Recommended)" | - |
| A second start of the desk shows the host list window where it was; only the tray's Hosts switches it to the list. | Softov, 2026-10-23, asked "Should it also switch to the list, as the tray's Hosts will?": "Stay where it was (Recommended)" | 01 |
| The tray menu is the update item while one is ready, then Hosts, Settings, About ahp-desk, Quit ahp-desk. | (defaulted: About before Quit, as most apps) | 03 |
| The bundled Node's version is read by running it with `--version`, so About shows what ships. | (defaulted: no second copy of the version to keep in step) | 03 |

## Proposed architecture

- **Data flow** - `about()` answers `{ desk, ahpw, node }`: the desk's `package_info().version`, ahpw's version baked in at build from the staged `@ahpd/web`'s `package.json` as `AHP_DESK_AHPW_VERSION`, and `node --version` of `BundledNode`, or `null` when none ships.
- **Event flow** - The tray emits `host-list-hosts` and `host-list-about` to the host list window as it emits `host-list-settings`; `check_desk_update()` runs `download_desk` and answers the ready version or `null`, and `desk-update-ready` still reaches the list and the tray.
- **State flow** - The About screen holds the check's state (idle, checking, up to date, ready, failed) only while it shows.
- **Layer responsibilities** - `src-tauri`: the events, `about`, `check_desk_update`, the tray item · `packages/shell`: the screens, the row's update way and words.
- **Source-of-truth files** - [`code://src-tauri/src/tray.rs`](../../../../src-tauri/src/tray.rs), [`code://src-tauri/src/updates.rs`](../../../../src-tauri/src/updates.rs), [`code://packages/shell/src/App.tsx`](../../../../packages/shell/src/App.tsx), [`code://packages/shell/src/hosts/status.ts`](../../../../packages/shell/src/hosts/status.ts).

## Tasks

| Task | Status | Depends on |
| --- | --- | --- |
| [01 - the tray's Hosts shows the list](task-01-the-trays-hosts-shows-the-list.md) | done | - |
| [02 - a local ahpd the desk did not install can be updated](task-02-a-local-ahpd-the-desk-did-not-install-can-be-updated.md) | todo | - |
| [03 - the About screen](task-03-the-about-screen.md) | done | 01 |

## Risks and tradeoffs

- ahpd's update of an ahpd installed with `npm i -g` writes where that npm installs; a global prefix that needs admin rights makes it fail - the failure is shown on the row with ahpd's words, as for a remote host.
- A Check now in a debug build finds nothing, as the start check does - About says so rather than "up to date".

## Resume state

- **Done so far:** tasks 01 and 03 here, task 02 in its own change; the by-hand checks wait for the final checklist.
- **Next action:** the by-hand checks of the final checklist on a Windows release build with all three tasks; then `implemented.md`.
- **Open questions:** none.
- **Watch out for:** a second start and the tray's Hosts share `show_host_list`; the new event goes in `on_menu_event`, not in `show_host_list`, so a second start keeps the screen it was on.

## Final verification checklist

- [ ] `pnpm typecheck`, `pnpm test` and `pnpm lint` pass.
- [ ] By hand on Windows on a release build: the tray's Hosts from Settings, About and Add host lands on the list; About shows the three versions and Check now; a local ahpd 0.12.0 the desk did not install shows the how-to words, and one at 0.13.0 shows Update.
- [ ] The specs in `changes` corrected.
- [ ] `plans/index.md` updated.
