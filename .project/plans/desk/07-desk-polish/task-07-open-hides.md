---
title: Opening a host hides the host list, and the desk brings it back
status: done
depends: []
layer: "src-tauri, packages/shell"
---

## Objective

Open from the host list hides it; `show_host_list` shows it, with its Add form when asked; the last app window's close shows it.

## Files

- `UPDATE: src-tauri/src/app_window.rs` - `open_app_window` hides the shell when the shell called it; `show_host_list` takes `add: Option<bool>` and emits `host-list-add`; the last app window's `Destroyed` shows the shell.
- `UPDATE: packages/shell/src/App.tsx` - `host-list-add` opens the Add form.
- `UPDATE: packages/ahpw-embedder/src/embedder.ts` - `manageHosts({ add })`.

## Steps

1. Hide the shell after it opens a host.
2. Show it, with the Add form on `add`.
3. Show it when the last app window closes.

## Validation

- Open leaves one window; closing it brings the host list back.

## Resume

- Done 2026-10-11. `open_app_window` hides the host list when the host list called it, for a new window and for one already open; `show_host_list` takes `add` and emits `host-list-add`, which opens the shell's Add form; the last app window's `Destroyed` shows the host list. The host list window also opens at 640 x 400 (min 480 x 300). Seen in a release build: the window size and the footer; Open, the last close and the Add form not clicked yet (task 09).
