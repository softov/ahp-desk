---
title: The local host has its start and quit settings
status: todo
depends: [task-04-start-find-stop.md]
layer: "src-tauri, packages/shell"
refs:
  - "[code://packages/shell/src/hosts/HostForm.tsx](../../../../packages/shell/src/hosts/HostForm.tsx) - the host form, which gains the two checkboxes"
---

## Objective

"Start when the desktop app starts" (on) and "Stop when the desktop app quits" (off) are on the local host, and the desk acts on them (decision 3).

## Files

- `UPDATE: src-tauri/src/hosts.rs` - `startOnLaunch` and `stopOnQuit`.
- `UPDATE: src-tauri/src/lib.rs` - the start at launch and the stop at quit.
- `UPDATE: packages/shell/src/hosts/HostForm.tsx` - the two checkboxes.
- `UPDATE: packages/shell/src/i18n.ts` - their words, in en, pt-BR and es.

## Steps

1. At launch, start the local ahpd when `startOnLaunch` is on and it is off.
2. At quit, stop it when `stopOnQuit` and `startedByDesk` are both on.

## Validation

- Cargo tests for the two rules; by hand, a quit with each setting.

## Resume
