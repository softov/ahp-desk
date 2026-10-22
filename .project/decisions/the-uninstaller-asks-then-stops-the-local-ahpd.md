---
title: The uninstaller asks, then stops the local ahpd, so its Node can be removed
status: accepted
date: 2026-10-22
refs:
  - "[code://src-tauri/windows/hooks.nsh](../../src-tauri/windows/hooks.nsh) - the installer's and uninstaller's hooks"
---

## Context

On 2026-10-22, removing the desk from Windows Settings, Apps, with its local ahpd running (stop on quit off, the default) said the uninstall completed, but left ahpd running on `$INSTDIR\node\node.exe`, that file, and the folder.

## Decision

The uninstaller's pre-uninstall hook looks for processes running from `$INSTDIR\node`.
When one runs, it asks with OK and Cancel, saying the local ahpd stops and its sessions and settings stay; Cancel ends the uninstall with nothing removed, and OK stops it and goes on.
An update's run of the old uninstaller (`/UPDATE`) and a silent one stop it without asking.
Source: Softov, asked "Uninstalling the desk while its local ahpd runs [...] How should uninstall handle it?": "Hook asks first".

## Consequences

An uninstall removes the whole install folder, Node included.
ahpd's own config folder and the desk's data stay, as the uninstaller's "Delete the application data" box decides.

## Options

- **Stop it without asking.** Lost: Softov chose to ask first.
- **Record only.** Lost: the uninstaller would keep saying it succeeded while leaving a running server behind.
