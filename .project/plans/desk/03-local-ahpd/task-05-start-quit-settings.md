---
title: The local host has its start and quit settings
status: done
depends: [task-04-start-find-stop.md]
layer: "src-tauri, packages/shell"
refs:
  - "[code://packages/shell/src/hosts/HostForm.tsx](../../../../packages/shell/src/hosts/HostForm.tsx) - the host form, which gains the two checkboxes"
---

## Objective

"Start when the desktop app starts" (on) and "Stop when the desktop app quits" (off) are on the local host, and the desk acts on them (decision 3).

## Files

- `UPDATE: src-tauri/src/hosts.rs` - `OnDesk` (`startOnLaunch`, `stopOnQuit`), `to_start_on_launch()`, `to_stop_on_quit()`.
- `UPDATE: src-tauri/src/local_ahpd.rs` - `start_host()`, `stop_host()`, `start_on_launch()`, `stop_on_quit()`.
- `UPDATE: packages/shell/src/hosts/ipc.ts`, `packages/shell/src/app.css`.
- `UPDATE: src-tauri/src/lib.rs` - the start at launch and the stop at quit.
- `UPDATE: packages/shell/src/hosts/HostForm.tsx` - the two checkboxes.
- `UPDATE: packages/shell/src/i18n.ts` - their words, in en, pt-BR and es.

## Steps

1. At launch, start the local ahpd when `startOnLaunch` is on and it is off.
2. At quit, stop it when `stopOnQuit` and `startedByDesk` are both on.

## Validation

- Cargo tests for the two rules; by hand, a quit with each setting.

## Resume

Done 2026-10-14.

- `OnDesk` is on the host: `startOnLaunch` on and `stopOnQuit` off by default, written to `hosts.json` only when they differ; the shell reads and sends both as `onDesk`, on a local host only, and a save without it keeps them.
- At launch, after the host list window, each local host set to start is started off the main thread; a failed start leaves the host down.
- At quit, `RunEvent::Exit` stops each local host set to stop whose ahpd the desk started, before the process ends.
- The host form opens a local host with its name and the two checkboxes, and a hint that only an ahpd the desk started is stopped; its address and token come from its ahpd, so they are not in the form, and nor is Test.
- Verified: cargo tests for the two rules and the defaults; the debug desk over CDP on a scratch host list and ahpd main 69ed7790: the ahpd started at launch and the row showed it connected, the form in en, pt-BR and es, Save wrote `stopOnQuit`, a quit with it on stopped the ahpd and cleared `startedByDesk`, and a quit with it off left the ahpd on.
