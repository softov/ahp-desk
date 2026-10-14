---
title: The desk says what can be updated, and updates it
domain: desk
status: planned
priority: high
created: 2026-10-13
revalidated: 2026-10-14
requires:
  - plans/desk/01-first-build-p4-tray-and-releases/plan.md
  - plans/desk/01-first-build-p3-local-ahpd/plan.md
decisions:
  - decisions/the-desk-asks-before-installing-its-update.md
  - decisions/ahpw-reaches-the-desk-with-a-desk-release.md
  - decisions/a-hosts-ahpd-update-is-shown-and-offered.md
  - decisions/ahpd-updates-itself-when-the-first-install-says-so.md
  - decisions/the-desk-asks-npm-with-reqwest-in-rust.md
  - decisions/update-shows-after-ahpds-check-says-the-token-may.md
  - decisions/the-desk-updates-a-host-through-ahp-x-ahpd-update.md
  - decisions/update-installs-the-version-the-row-showed.md
  - decisions/a-ready-desk-update-is-the-first-tray-item.md
refs:
  - file:///github/ahpd/packages/server/src/update.ts - ahpd's npm check and version comparison, the rules the desk's check follows
  - file:///github/ahpd/packages/server/src/commands/restart.ts - ahpd's restart, which its update command ends with
  - file:///github/ahpd/packages/server/src/commands/plugin.ts - `plugin update`, an admin command over CLI and HTTP, the shape of the new one
  - "[code://packages/shell/src/hosts/status.ts](../../../../packages/shell/src/hosts/status.ts) - each host's server name and version, from `initialize`"
  - "[code://packages/shell/src/hosts/HostList.tsx](../../../../packages/shell/src/hosts/HostList.tsx) - the row that gains the notice and Update"
  - "[code://src-tauri/tauri.conf.json](../../../../src-tauri/tauri.conf.json) - the updater's endpoint and key, from p4"
---

## Goal

The desk says when something it runs or talks to has a newer version: the desk itself, and each host's ahpd.
It updates the desk on a click, and a host's ahpd on a click when it can reach the host to do so.
A new ahpw arrives as part of a desk update.

## Reconnaissance

The files read are the `refs` above.

### Runtime path

```
desk start (p4 task 03), then every 6 h (task 06) -> Tauri updater -> host list notice + first tray item -> click -> install, restart
host connected -> initialize serverInfo (ahpd version) -> npm latest @ahpd/server (reqwest, cached 6 h) -> newer? -> row "ahpd X available"
  -> SSH host (p10): Update -> npm install @ahpd/server@X + systemctl --user restart
  -> other host: AHP x-ahpd/update { check: true } -> may? -> Update -> x-ahpd/update { version: X } -> ahpd installs and restarts
  -> neither: notice only
@ahpd/web release -> desk pin moves -> desk release -> desk updater
```

### Gaps

- ahpd tells no client that it is behind npm, and has no command that updates itself.
- The desk has no updater yet (p4 task 03).

## Decisions locked in

| # | Decision | Rationale / source |
| --- | --- | --- |
| 1 | [The desk asks before it installs its own update](../../../decisions/the-desk-asks-before-installing-its-update.md) | Softov, 2026-10-13 |
| 2 | [A new ahpw reaches the desk with a desk release](../../../decisions/ahpw-reaches-the-desk-with-a-desk-release.md) | Softov, 2026-10-13 |
| 3 | [A host's ahpd update is shown on its row and offered, over SSH or through ahpd](../../../decisions/a-hosts-ahpd-update-is-shown-and-offered.md) | Softov, 2026-10-13 |
| 4 | [ahpd updates itself when the first install says so](../../../decisions/ahpd-updates-itself-when-the-first-install-says-so.md) | Softov, 2026-10-11 |
| 5 | [The desk asks npm for @ahpd/server's latest with reqwest in Rust](../../../decisions/the-desk-asks-npm-with-reqwest-in-rust.md) | Softov, 2026-10-14 |
| 6 | [Update shows on a host's row only after ahpd's update command answers a check that the token may update it](../../../decisions/update-shows-after-ahpds-check-says-the-token-may.md) | Softov, 2026-10-14 |
| 7 | [The desk updates a host's ahpd through the AHP method x-ahpd/update, over the connection the host list holds](../../../decisions/the-desk-updates-a-host-through-ahp-x-ahpd-update.md) | Softov, 2026-10-14 |
| 8 | [Update installs the version the host's row showed, passed by the desk](../../../decisions/update-installs-the-version-the-row-showed.md) | Softov, 2026-10-14 |
| 9 | [A ready desk update is the first item of the tray menu, and a click installs it and restarts](../../../decisions/a-ready-desk-update-is-the-first-tray-item.md) | Softov, 2026-10-14 |

| What | Source | Task |
| --- | --- | --- |
| The desk asks npm for `@ahpd/server`'s latest at most every 6 hours, as ahpd does. | (defaulted: ahpd's `update.ts`) | 01 |
| Update on a host with running sessions names them and waits for a click. | Decision 3 | 02 |
| The desk's update notice, its install command and its tray item are built by p4 task 03, with the check at start; this plan adds the check every 6 hours after it. | [p4's decision](../../../decisions/the-desk-update-notice-is-built-in-p4-task-03.md), Softov, 2026-10-14 | 06 |

## Proposed architecture

- **Data flow** - `src-tauri/src/updates.rs` keeps npm's latest `@ahpd/server` beside the desk's own ready update that p4 task 03 keeps there; the host list reads npm's latest with each host's version.
- **Event flow** - an `updates-changed` event to the host list for npm's latest; p4 task 03's `desk-update-ready` for the desk's own, which task 06's re-check sends too.
- **State flow** - nothing kept on disk but the last npm answer and when it was asked.
- **Layer responsibilities** - `src-tauri`: the checks, the desk install, the SSH update · `packages/shell`: the notices and buttons · ahpd: its update command.
- **Source-of-truth files** - `src-tauri/src/updates.rs`.

## Tasks

| Task | Status | Depends on |
| --- | --- | --- |
| [01 - npm's latest @ahpd/server, checked and kept](task-01-npm-latest-ahpd.md) | todo | - |
| [02 - the host row's notice and Update](task-02-host-row-notice-and-update.md) | todo | 01 |
| [03 - ahpd: an update command for an administrator, over HTTP and AHP (in the ahpd repository)](task-03-ahpd-update-command.md) | todo | - |
| [04 - Update through ahpd's command](task-04-update-through-ahpd.md) | todo | 02, 03 |
| [05 - Update over SSH for an SSH-installed host](task-05-update-over-ssh.md) | todo | 02, p10 task 08 |
| [06 - the desk's own update, checked again every 6 hours](task-06-desk-update-notice.md) | todo | 01, p4 task 03 |
| [07 - the release steps: an ahpw release is followed by a desk release](task-07-release-steps.md) | todo | - |
| [08 - end to end](task-08-end-to-end.md) | todo | 01 to 07 |

## Risks and tradeoffs

- Task 03 is ahpd work, done in the ahpd repository by its own plan; tasks 04 and 08 wait for its release.
- An update restarts a host; a person on another client loses its connection for the restart.

## Resume state

- **Done so far:** the plan, its decisions and its task files, 2026-10-14; the open questions answered as decisions 5 to 9, and task 06 narrowed to the check every 6 hours, 2026-10-14.
- **Next action:** hand task 03 to the ahpd repository as its own plan; then task 01, and task 07, which waits on nothing.
- **Open questions:** none.
- **Watch out for:** the local ahpd follows decision 4 (p3 task 07), not the host-row Update.

## Final verification checklist

- [ ] A host behind npm says so on its row, in each language.
- [ ] Update works over SSH and through ahpd's command, and is absent when neither can reach the host.
- [ ] A desk update is announced and installs only on the click.
- [ ] `plans/index.md` updated.
