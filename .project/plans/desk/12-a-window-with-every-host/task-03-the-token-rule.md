---
title: The window with every host may read every token
status: done
depends: [task-02-the-window-with-every-host.md]
layer: "src-tauri"
refs:
  - "[code://src-tauri/src/app_window.rs#L77-L80](../../../../src-tauri/src/app_window.rs#L77-L80) - `may_read`: the host list, or the host's own window"
---

## Objective

The host list and the window with every host may read any host's token.
A window of one host still reads only its own host's token.

## Files

- `UPDATE: src-tauri/src/app_window.rs:77-80` - `may_read` is true for `SHELL_WINDOW`, for the label in `every`, and for a one-host window on its own host.
- `UPDATE: src-tauri/src/app_window.rs` tests - the three cases, and a one-host window refused another host's token.
- `UPDATE: .project/specs/app-window.md` - the scenario for the token rule.

## Steps

1. Add the window with every host to `may_read`.
2. Write the four test cases.
3. Correct the token scenario in the spec.

## Validation

- `cargo test` in `src-tauri` passes, with the four cases.

## Resume

- Done 2026-10-16: `may_read` is true for the host list, for `app-:every` while it is open, and for a window on its own host.
- Verified: four cases in `app_window.rs`, a window of one host refused another's token among them; `specs/app-window.md` has the rule.
