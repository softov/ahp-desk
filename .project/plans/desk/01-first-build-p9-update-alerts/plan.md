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
desk start, then every 6 h -> Tauri updater (p4) -> "ahp-desk X is ready. Restart to update." -> click -> install, restart
host connected -> initialize serverInfo (ahpd version) -> npm latest @ahpd/server (cached 6 h) -> newer? -> row "ahpd X available"
  -> Update -> SSH host (p10): npm install + systemctl --user restart
            -> other host, admin token: ahpd update command (HTTP/AHP) -> ahpd installs and restarts
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

| What | Source | Task |
| --- | --- | --- |
| The desk asks npm for `@ahpd/server`'s latest at most every 6 hours, through `npm_config_registry` when set, as ahpd does. | (defaulted: ahpd's `update.ts`) | 01 |
| Update on a host with running sessions names them and waits for a click. | Decision 3 | 02 |

## Proposed architecture

- **Data flow** - `src-tauri/src/updates.rs` keeps npm's latest `@ahpd/server` and the desk's own pending update; the host list reads them with each host's version.
- **Event flow** - an `updates-changed` event to the host list and the tray.
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
| [06 - the desk's own update notice, in the host list and the tray](task-06-desk-update-notice.md) | todo | 01, p4 tasks 01 and 03 |
| [07 - the release steps: an ahpw release is followed by a desk release](task-07-release-steps.md) | todo | - |
| [08 - end to end](task-08-end-to-end.md) | todo | 01 to 07 |

## Risks and tradeoffs

- Task 03 is ahpd work, done in the ahpd repository by its own plan; tasks 04 and 08 wait for its release.
- An update restarts a host; a person on another client loses its connection for the restart.

## Resume state

- **Done so far:** the plan, its decisions and its task files, 2026-10-14.
- **Next action:** answer the open questions below; hand task 03 to the ahpd repository as its own plan; then task 01, and task 07, which waits on nothing.
- **Open questions:**
  1. Task 01: how `updates.rs` asks npm.
     Proposed: `reqwest` with `rustls-tls` and `json`, the client `tauri-plugin-updater` (p4 task 03) already brings.
     Other ways: `npm view @ahpd/server version` run with the bundled Node; the ask made from the host list's webview with `fetch`.
  2. Tasks 03 and 04: how the desk knows, before it shows Update, that ahpd's command will take its token.
     Proposed: the command takes `check` and answers `{ current, latest, may }`, or the refusal a real update would get.
     Other ways: Update on every ahpd that has the command, and a refusal said on the click; ahpd saying it in its `initialize` answer.
  3. Task 04: which of ahpd's surfaces the desk calls.
     Proposed: the AHP method `x-ahpd/update`, over the connection the host list already holds.
     Other way: `POST /api/update`, which needs ahpd's `/api` on and an HTTP address beside the host's `ws://` one.
  4. Tasks 03, 04 and 05: which version Update installs.
     Proposed: the version the row named, passed by the desk.
     Other way: npm's latest at the moment ahpd or the SSH update runs.
  5. Task 06: the tray's form of "ahp-desk X is ready. Restart to update."
     Proposed: the first item of the tray menu, which installs on a click.
     Other ways: an OS notification; the tray icon's tooltip or a badge.
- **Watch out for:** the local ahpd follows decision 4 (p3 task 07), not the host-row Update.

## Final verification checklist

- [ ] A host behind npm says so on its row, in each language.
- [ ] Update works over SSH and through ahpd's command, and is absent when neither can reach the host.
- [ ] A desk update is announced and installs only on the click.
- [ ] `plans/index.md` updated.
