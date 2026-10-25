---
title: The tray's Hosts always shows the host list itself
status: accepted
date: 2026-10-23
refs:
  - "[code://src-tauri/src/tray.rs#L105](../../src-tauri/src/tray.rs#L105) - Hosts only shows the window today"
  - "[code://packages/shell/src/App.tsx#L68](../../packages/shell/src/App.tsx#L68) - the screen the shell switches to on the tray's Settings"
---

## Context

On 2026-10-23, on Windows, the tray's Settings switched the host list window to Settings, and the tray's Hosts then showed the window still on Settings.
Hosts only shows the window; nothing tells the shell which screen to show.

## Decision

The tray's Hosts shows the window and switches it to the list, from any screen: Settings, About, Add host or Edit host.
A half-filled form is dropped, as the form's own Back drops it.
Source: Softov, asked "Tray \"Hosts\" while the window shows another screen: what should it do?": "Always go to the list (Recommended)".

## Consequences

Hosts in the tray always ends on the list.
Typed fields of an open form are lost on that click.

## Options

- **Not from a form.** From Settings or About it would go to the list, and during Add host or Edit host it would only show the window, so typed fields stayed. Lost: Softov chose the list from every screen.
