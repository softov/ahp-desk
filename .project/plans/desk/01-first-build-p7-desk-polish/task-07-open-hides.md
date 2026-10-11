---
title: Opening a host hides the host list, and the desk brings it back
status: todo
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

- Not started.
