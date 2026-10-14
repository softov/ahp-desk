---
title: The desk's update notice, its install and its tray item are built in p4 task 03
status: accepted
date: 2026-10-14
refs:
  - "[code://.project/plans/desk/01-first-build-p4-tray-and-releases/task-03-updater-with-signing-key.md](../../.project/plans/desk/01-first-build-p4-tray-and-releases/task-03-updater-with-signing-key.md) - the task that builds the notice, the Restart click and the tray item"
  - "[code://.project/plans/desk/01-first-build-p9-update-alerts/task-06-desk-update-notice.md](../../.project/plans/desk/01-first-build-p9-update-alerts/task-06-desk-update-notice.md) - the task that keeps only the check every 6 hours"
  - "[code://packages/shell/src/hosts/HostList.tsx#L73-L76](../../packages/shell/src/hosts/HostList.tsx#L73-L76) - the head of the host list, where the notice goes"
---

## Context

Plan p4 task 03 and plan p9 task 06 both described the desk's own update notice, its install command and its tray item.
Two tasks that build the same notice either build it twice or each wait for the other.
Plan p4's goal and checklist ask for an update a user can install, and the desk asks before it installs its own update ([decision](the-desk-asks-before-installing-its-update.md)).

## Decision

Plan p4 task 03 builds the desk's update notice in the host list, its Restart click that installs the update, and its item in the tray.
Plan p9 task 06 keeps only the check every 6 hours, in its loop.
Source: Softov, asked "p4 q1: p4 task 03 and p9 task 06 both describe the desk's own update notice, its install command and its tray item. Which plan builds it?": "p4 task 03".

## Consequences

Plan p4 ends with an update a user can see and install, and its task 06 checks it on real releases without waiting for p9.
Plan p9 task 06 builds on `updates.rs` and the notice as p4 leaves them, and adds only the later checks.

## Options

- **p9 task 06.** Lost: p4 would ship an updater whose update nobody can install, and its task 06 would wait for p9.
