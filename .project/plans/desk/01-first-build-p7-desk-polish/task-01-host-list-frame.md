---
title: The host list window has no OS frame, round corners and its own title bar
status: done
depends: []
layer: "src-tauri, packages/shell"
---

## Objective

The host list window is frameless, transparent, shadowless and rounded, with a title bar that drags it and ends in minimize, maximize and close.

## Files

- `UPDATE: src-tauri/src/lib.rs` - the shell window built with `decorations(false)`, `transparent(true)`, `shadow(false)`, then `frame::round_corners`; its `CloseRequested` hides it while an app window is open.
- `UPDATE: src-tauri/capabilities/shell.json` - `core:window:allow-minimize`, `allow-toggle-maximize`, `allow-close`, `allow-start-dragging`; `core:default` already has `is-maximized`, and the hide is in Rust.
- `CREATE: packages/shell/src/TitleBar.tsx` - the title, the drag area and the three controls, as ahpw's.
- `CREATE: packages/shell/src/window.ts` - the controls over `getCurrentWindow()`, and `is-maximized` on `<html>`.

## Steps

1. Build the shell window frameless and rounded, as `open_app_window` builds an app window.
2. Draw the title bar: the app name at the start, the controls at the end, 32 px high as ahpw's.
3. A press on the empty bar drags; a double click maximizes.
4. On close, hide when an app window is open; otherwise quit.

## Validation

- No OS border; round corners restored, square maximized.
- Drag, double click, minimize, maximize and close work.

## Resume

- Done 2026-10-11. Frameless, transparent, no shadow, DWM round corners (Windows 11; on Windows 10 the CSS root gives the corner). Close hides while an app window is open.
- Not checked: a drag with a real mouse, and 150% display scale.
