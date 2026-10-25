---
title: The desk says what can be updated, and updates it
domain: desk
status: active
priority: high
created: 2026-10-13
revalidated: 2026-10-14
requires:
  - plans/desk/04-tray-and-releases/plan.md
  - plans/desk/03-local-ahpd/plan.md
decisions:
  - decisions/ahpw-reaches-the-desk-with-a-desk-release.md
refs:
  - file:///github/ahpd/packages/server/src/update.ts - ahpd's npm check and version comparison, the rules the desk's check follows
  - file:///github/ahpd/packages/server/src/commands/restart.ts - ahpd's restart, which its update command ends with
  - file:///github/ahpd/packages/server/src/commands/plugin.ts - `plugin update`, an admin command over CLI and HTTP, the shape of the new one
  - "[code://packages/shell/src/hosts/status.ts](../../../../packages/shell/src/hosts/status.ts) - each host's server name and version, from `initialize`"
  - "[code://packages/shell/src/hosts/HostList.tsx](../../../../packages/shell/src/hosts/HostList.tsx) - the row that gains the notice and Update"
  - "[code://src-tauri/tauri.conf.json](../../../../src-tauri/tauri.conf.json) - the updater's endpoint and key, from plan 04"
---

## Goal

The desk says when something it runs or talks to has a newer version: the desk itself, and each host's ahpd.
It updates the desk on a click, and a host's ahpd on a click when it can reach the host to do so.
A new ahpw arrives as part of a desk update.

## Reconnaissance

The files read are the `refs` above.

### Runtime path

```
desk start (plan 04 task 03), then every 6 h (task 06) -> Tauri updater -> host list notice + first tray item -> click -> install, restart
host connected -> initialize serverInfo (ahpd version) -> npm latest @ahpd/server (reqwest, cached 6 h) -> newer? -> row "ahpd X available"
  -> SSH host (plan 10): Update -> npm install @ahpd/server@X + systemctl --user restart
  -> other host: AHP x-ahpd/update { check: true } -> may? -> Update -> x-ahpd/update { version: X } -> ahpd installs and restarts
  -> neither: notice only
@ahpd/web release -> desk pin moves -> desk release -> desk updater
```

### Gaps

- ahpd tells no client that it is behind npm, and has no command that updates itself.
- The desk has no updater yet (plan 04 task 03).

## Answers

| What | Source | Task |
| --- | --- | --- |
| The desk downloads its own update in the background, then says "ahp-desk X is ready. Restart to update." in the host list and the tray, and restarts only on that click. | Softov, 2026-10-13, asked "When the desk itself has an update, what happens?": "Ask, then install (Recommended)" | 06 |
| [A new ahpw reaches the desk with a desk release](../../../decisions/ahpw-reaches-the-desk-with-a-desk-release.md) | Softov, 2026-10-13 | 07 |
| A host's ahpd update is shown on its row and offered: the desk compares each connected ahpd's version with the newest `@ahpd/server` on npm, and the row says "ahpd X available" with Update. Update goes over SSH for a host the desk installed over SSH, else through ahpd's own update command for a host whose token may administer it; a host the desk can update neither way shows the notice alone. | Softov, 2026-10-13, asked what the desk does when a URL host's ahpd is older than npm's: "both. say on the host and offer the update"; asked how Update reaches a remote host: "Both ways (Recommended)" | 02, 04, 05 |
| The local ahpd the desk installed updates as plan 03's *Answers* say, not through the host-row Update; one it did not install follows [a local ahpd the desk did not install updates as a remote one](../../../decisions/a-local-ahpd-the-desk-did-not-install-updates-as-a-remote-one.md). | Softov, 2026-10-11; 2026-10-23 | plan 03 task 07; plan 13 task 02 |
| The desk asks npm for `@ahpd/server`'s latest with `reqwest` in Rust, over rustls, and reads `npm_config_registry` first, as ahpd's `registry()` does. | Softov, 2026-10-14: "reqwest in Rust" | 01 |
| Update shows on a host's row only after ahpd's update command, called with `check`, answers that the token may update it; a refusal, or no such command, leaves the notice without the button. | Softov, 2026-10-14: "A check call" | 03, 04 |
| The desk calls ahpd's update command as the AHP method `x-ahpd/update`, on the connection the host list holds for that host. | Softov, 2026-10-14: "AHP x-ahpd/update" | 03, 04 |
| Update installs the version the host's row named: the desk passes it as `version` to `x-ahpd/update`, and as `@ahpd/server@<version>` to the SSH update. | Softov, 2026-10-14: "The one the row showed" | 03, 04, 05 |
| While a desk update is ready, the first item of the tray menu says so with its version, and a click installs it and restarts; plan 04 task 03 builds it. | Softov, 2026-10-14: "First menu item" | 06, plan 04 task 03 |
| The desk asks npm for `@ahpd/server`'s latest at most every 6 hours, as ahpd does. | (defaulted: ahpd's `update.ts`) | 01 |
| Update on a host with running sessions names them and waits for a click. | A host's ahpd update is shown on its row and offered | 02 |
| The desk's update notice, its install command and its tray item are built by plan 04 task 03, with the check at start; this plan adds the check every 6 hours after it. | Softov, 2026-10-14: "p4 task 03", in [plan 04's Answers](../04-tray-and-releases/plan.md#answers) | 06 |

## Proposed architecture

- **Data flow** - `src-tauri/src/updates.rs` keeps npm's latest `@ahpd/server` beside the desk's own ready update that plan 04 task 03 keeps there; the host list reads npm's latest with each host's version.
- **Event flow** - an `updates-changed` event to the host list for npm's latest; plan 04 task 03's `desk-update-ready` for the desk's own, which task 06's re-check sends too.
- **State flow** - nothing kept on disk but the last npm answer and when it was asked.
- **Layer responsibilities** - `src-tauri`: the checks, the desk install, the SSH update · `packages/shell`: the notices and buttons · ahpd: its update command.
- **Source-of-truth files** - `src-tauri/src/updates.rs`.

## Tasks

| Task | Status | Depends on |
| --- | --- | --- |
| [01 - npm's latest @ahpd/server, checked and kept](task-01-npm-latest-ahpd.md) | done | - |
| [02 - the host row's notice and Update](task-02-host-row-notice-and-update.md) | done | 01 |
| [03 - ahpd: an update command for an administrator, over HTTP and AHP (in the ahpd repository)](task-03-ahpd-update-command.md) | done | - |
| [04 - Update through ahpd's command](task-04-update-through-ahpd.md) | done | 02, 03 |
| [05 - Update over SSH for an SSH-installed host](task-05-update-over-ssh.md) | done | 02, plan 10 task 08 |
| [06 - the desk's own update, checked again every 6 hours](task-06-desk-update-notice.md) | doing | 01, plan 04 task 03 |
| [07 - the release steps: an ahpw release is followed by a desk release](task-07-release-steps.md) | done | - |
| [08 - end to end](task-08-end-to-end.md) | todo | 01 to 07 |

## Risks and tradeoffs

- Task 03 is ahpd work, released in ahpd 0.13.0.
- An update restarts a host; a person on another client loses its connection for the restart.

## Resume state

- **Done so far:** the plan, its decisions and its task files, 2026-10-14.
  The open questions are answered in *Answers*, and task 06 is narrowed to the check every 6 hours, 2026-10-14.
  Tasks 01, 02, 05 and 07, 2026-10-18.
  Task 03 released in ahpd 0.13.0, and task 04 on it, checked by hand on Windows, 2026-10-20.
  Task 06 is built; its by-hand check waits for a newer signed desk release than v0.1.0.
- **Next action:** task 06's by-hand check with plan 04 task 06, once Softov tags the next release; then task 08.
- **Open questions:** none.
- **Watch out for:** the local ahpd follows plan 03's update answer (task 07), not the host-row Update.

## Final verification checklist

- [ ] A host behind npm says so on its row, in each language.
- [ ] Update works over SSH and through ahpd's command, and is absent when neither can reach the host.
- [ ] A desk update is announced and installs only on the click.
- [ ] `plans/index.md` updated.
