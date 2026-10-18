---
title: The installed ahpd updates itself when its checkbox says so
status: blocked
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
