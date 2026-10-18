---
title: The app window has no accent border and asks for round corners
status: done
depends: []
layer: "src-tauri"
refs:
  - file:///github/ahp-desk/src-tauri/src/app_window.rs - `open_app_window`, the window builder
  - file:///github/opendoop/packages/tauri/src-tauri/src/lib.rs - pood's builder at lines 97-112, and `apply_round_window_corners` at lines 131-163
  - file:///github/ahp-desk/.project/specs/app-window.md - decision 4
---

## Objective

An app window is built transparent, with no shadow, and on Windows with DWM's round corners, so the OS draws no accent border.

## Files

- `UPDATE: src-tauri/src/app_window.rs` - `.transparent(true)` and `.shadow(false)` on the builder; call `round_corners(&window)` after the build.
- `CREATE: src-tauri/src/frame.rs` - `round_corners`, the DWM call on Windows and nothing elsewhere, from pood's `apply_round_window_corners`.
- `UPDATE: src-tauri/src/lib.rs` - `mod frame;`.

## Steps

1. Add `.transparent(true)` and `.shadow(false)` to the builder in `open_app_window`.
2. Write `round_corners` in `frame.rs` with `DWMWA_WINDOW_CORNER_PREFERENCE` set to `DWMWCP_ROUND`, as pood does; a no-op off Windows.
3. Call it on the new window.

## Validation

- `cargo clippy -- -D warnings` passes.
- Open a host; the window has no green line on its edge. ahpw's own rounded root comes in task 08.

## Resume

- Done 2026-10-11. The window is transparent with no shadow; the full-height accent line is gone. The DWM call answers an error on Windows 10, where corners come from ahpw's rounded root only.
