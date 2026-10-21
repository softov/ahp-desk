---
title: The installed ahpd updates itself when its checkbox says so
status: done
depends: [task-03-install-ahpd.md, task-04-start-find-stop.md]
layer: "src-tauri, packages/shell"
refs:
  - file:///github/ahpd/packages/server/src/update.ts - ahpd's npm check, written to update.json
  - "[code://packages/shell/src/hosts/HostList.tsx](../../../../packages/shell/src/hosts/HostList.tsx) - the host row, which can say update available"
---

## Objective

"Update ahpd automatically" (on) is on the first install and in the host's settings; when on, the desk updates while no turn runs, and when off, the row shows "update available" (decision 6).

## Files

- `UPDATE: src-tauri/src/local_ahpd.rs` - `update()`.
- `UPDATE: src-tauri/src/hosts.rs` - `autoUpdate`.
- `UPDATE: packages/shell/src/hosts/HostList.tsx` - "update available".

## Steps

1. Read the newer version from ahpd's `update.json`.
2. Update only an ahpd the desk installed.

## Validation

- By hand on a pinned older `@ahpd/server`: it updates with the box on, and offers the update with it off.

## Resume

- Blocked 2026-10-16: the update goes through ahpd's own update command (plan 09 task 03), not npm run by the desk (Answers: the update command).
- 2026-10-20: built on ahpd 0.13.0's `x-ahpd/update` (plan 09 task 03), with plan 09 task 04's request; the desk runs no npm of its own, so `local_ahpd.rs` is unchanged.
- The newer version is npm's latest `@ahpd/server` that the desk already keeps (plan 09 task 01), not ahpd's `update.json`: it is the version the row names, so the update installs what the row said.
- `hosts.rs`: `OnDesk` has `auto_update` (`autoUpdate`), on by default; only `"autoUpdate": false` is written, so a `hosts.json` from before reads as on.
  `ipc.ts` has `ON_DESK_DEFAULTS`, which `LocalForm.tsx` (the first install) and `HostForm.tsx` (the host's settings) both start from; the checkbox is `form.autoUpdate`, and the settings show `form.autoUpdateHint` under it, in `en`, `ptBR` and `es`.
- `status.ts`: `updateWay` offers `{ kind: 'ahpd' }` for a local host only when `findLocalAhpd` says the desk installed it (`origin === 'installed'`), and the check says may; an ahpd found on `PATH` keeps the notice without Update (decision 9).
- `HostList.tsx`: an effect updates a local host with `autoUpdate` on, whose way is `ahpd`, connected and behind npm, with no turn running and no update under way; once per host and version while the window lives (`tried`), so a failure is shown once and the row keeps its Update.
  `behind` no longer leaves local hosts out, so with the box off the row says "ahpd X available" with Update.
- While an update runs, the row keeps "Updating ahpd…" through the restart and shows no Retry: before, the drop made the row say down with Retry, and Retry would start a second ahpd over the install; the same holds for plan 09 task 05's update over SSH.
- Verified: `cargo test` (137), `cargo clippy --all-targets -- -D warnings` through `pnpm lint`, `pnpm typecheck`, `pnpm test` (53).
- By hand on a Windows 10 VM, as a new standard user, on a release NSIS build of this branch, with a local npm registry holding repacks of 0.13.0 as newer versions (`npm_config_registry` set for the machine):
  - Add host, This computer, showed "Update ahpd automatically", ticked, and installed ahpd 0.13.0;
  - with 0.13.1 published, the desk updated ahpd by itself with no click: ahpd's helper installed, moved the plugins and started the daemon, and the row came back green on 0.13.1 with no notice;
  - with the box unticked in Edit (`"autoUpdate": false` in `hosts.json`) and 0.13.2 published, the row said "ahpd 0.13.2 available" with Update and nothing updated; a click updated, and the row came back green on 0.13.2.
- At the window's 640x400, the new checkbox pushed Add host's "Install and start" and Back below the bottom edge; the host list window now opens at 640x480 (decision 13, `lib.rs`).
  Seen on the VM's release build: the window opens at 640x480; This computer's install form was not seen at that size, since the VM user already has This computer, so task 08's clean user checks it.
- Seen, not fixed here: while the desk starts the local ahpd at launch (about 30 seconds on the VM), the row says down with Retry rather than that ahpd is starting; for task 08 to look at.
- Not seen by hand: a running turn holding the automatic update back, since no backend was signed in.

