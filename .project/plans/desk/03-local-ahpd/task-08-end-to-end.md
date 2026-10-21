---
title: This computer works end to end on a clean Windows user
status: done
depends: [task-05-start-quit-settings.md, task-06-add-this-computer.md, task-07-auto-update.md]
layer: "ahp-desk"
---

## Objective

Every item of the plan's checklist holds on a release build.

## Files

- None.

## Steps

1. On a Windows user with no Node: Add host "This computer" installs ahpd and connects.
2. With ahpd on `PATH`: it is used and nothing is installed.
3. Quit with each stop setting.

## Validation

- The plan's checklist.

## Resume

- Checked by hand on a Windows 10 VM, as a new standard user with no Node and no ahpd, on a release NSIS build of the branch of plan 09 task 04 (PR #1), 2026-10-21:
  - Add host, This computer, found no ahpd, offered the install with Claude ticked, and fitted the 640x480 window with its buttons in view; "Install and start" installed ahpd 0.13.0 with the bundled Node into `<app local data>/ahpd`, and the row came up green on "ahpd 0.13.0"; Open showed ahpw on the host;
  - with "Stop when the desktop app quits" off, Quit in the tray ended the desk and left ahpd running; with it on (`"stopOnQuit": true` in `hosts.json`), Quit ended both, and ahpd's log ends `stopping`, `stopped`;
  - with the desk's install and host list removed and `@ahpd/server` 0.13.0 installed globally on the user's `PATH`, Add host said "ahpd 0.13.0, on PATH" with Claude "installed" and "Save and start"; nothing was written under `<app local data>/ahpd` but the token file, and the desk started that ahpd, which read the token from there.
- Found and fixed in the run:
  - while the desk starts the local ahpd at launch, the row said down with Retry; it now says "starting" with the connecting dot and no Retry (`LOCAL_STARTING` and `local_starting` in `local_ahpd.rs`, `useLocalStarting.ts`, `status.starting`), seen for the few seconds before it turned green;
  - Add host offered "Update ahpd automatically" for an ahpd found on `PATH`, which never updates itself (decision 6); the box now shows only when the desk installs ahpd (`LocalForm.tsx`).
