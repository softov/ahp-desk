---
title: The host list opens Settings from a gear in its title bar, before the language picker
status: accepted
date: 2026-10-14
refs:
  - "[code://.project/plans/desk/01-first-build-p4-tray-and-releases/task-02-close-to-tray-and-setting.md](../../.project/plans/desk/01-first-build-p4-tray-and-releases/task-02-close-to-tray-and-setting.md) - the Settings screen and the gear"
  - "[code://packages/shell/src/TitleBar.tsx#L76-L77](../../packages/shell/src/TitleBar.tsx#L76-L77) - the title bar's controls, where the gear goes before `LanguagePicker`"
---

## Context

Plan p4 adds a Settings screen to the host list window, and the tray's Settings item opens it.
On Linux the tray may be missing or hard to reach, and a user who never opens the tray menu needs a way in from the host list itself.

## Decision

The host list's title bar has a gear button, before the language picker, that opens Settings.
Source: Softov, asked "p4 q4: besides the tray, where does the host list open Settings?": "Gear in the title bar".

## Consequences

Settings is one click away from every screen of the host list window.
`TitleBar` takes an `onSettings` handler, and the gear's title is the word `window.settings`.

## Options

- **Only from the tray.** Lost: a user without a working tray, or who does not look there, never finds Settings.
- **A footer link.** Lost: the host list has no footer, and a link below a long list is out of sight.
