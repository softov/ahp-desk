---
title: The app window draws its own rounded frame, as pood's window does
status: accepted
date: 2026-10-11
refs:
  - file:///github/opendoop/packages/tauri/src-tauri/src/lib.rs - `open_web_ui` at lines 97-123: no decorations, transparent, no shadow, then rounded DWM corners at lines 131-160
  - file:///github/opendoop/packages/web/src/app/globals.css - `html.is-tauri #root` rounded at lines 35-48, square when `is-maximized`
  - file:///github/opendoop/packages/web/src/components/shell/TitleBar.tsx - the window controls and the `is-maximized` class at lines 468-540
---

## Context

The app window has no OS frame, so the UI draws its title bar.
With the OS shadow on, Windows draws a 1 px border in the accent colour round the window, a green line beside ahpw's activity bar.
The window has square corners, and the UI has no minimize, maximize or close.

## Decision

The app window is built transparent and with no shadow, and on Windows asks DWM for round corners.
The UI rounds its own root by 10 px while the window is not maximized, and squares it when maximized.
The UI's title bar has minimize, maximize or restore, and close, and a drag on the bar moves the window.
Source: Softov: "like pood we did. remove the border green on the panel. when not maximized border rounded on window the minimize, maximize and close buttons."

## Consequences

The same frame as pood's window, so both desktop apps look alike.
A UI has to draw the frame: ahpw's embedder gives it `window.isMaximized()` and `window.onResized()` (decision: ahpw owns its embedding contract), and ahpw adds the rounded root, the controls and the drag.
A UI that draws no frame shows a window with no edge; ahpapp needs the same work in p5.

## Options

- **The OS frame.** Decorations on. Lost: the title bar would be the OS's, not the UI's, against mockup 03.
- **No frame, shadow on.** Lost: the accent border Softov asked to remove.
