---
title: The host list window opens at 640 by 480
status: accepted
date: 2026-10-20
refs:
  - "[code://src-tauri/src/lib.rs](../../src-tauri/src/lib.rs) - the host list window's `inner_size` and `min_inner_size`"
  - "[code://packages/shell/src/hosts/LocalForm.tsx](../../packages/shell/src/hosts/LocalForm.tsx) - Add host, This computer, with the settings checkboxes"
---

## Context

The host list window opened at 640x400, with 480x300 as its smallest size, since plan 07, where Softov asked for "less height" than before.
With "Update ahpd automatically" added to Add host (plan 03 task 07), its "Install and start" and Back buttons fall below the window's bottom edge at 640x400, so the user has to scroll to finish adding the host.

## Decision

The host list window opens at 640x480; its smallest size stays 480x300, where the form scrolls.
Source: Softov, asked "At the host-list window's default size (640x400) [...] How should that be fixed?": "Taller default window (Recommended)".

## Consequences

Add host fits without a scroll at the size the window opens at, and the list shows more rows.
A window made smaller by hand still scrolls.

## Options

- **Compact the settings.** Lost: the hints under the checkboxes would move into tooltips, which a person on touch or a keyboard does not see.
- **Leave it scrolling.** Lost: the first thing a new user does would need a scroll to finish.
