---
title: Each install carries only its own UI files
status: accepted
date: 2026-10-22
refs:
  - "[code://src-tauri/windows/hooks.nsh](../../src-tauri/windows/hooks.nsh) - the installer's and uninstaller's hooks"
  - "[code://src-tauri/tauri.conf.json](../../src-tauri/tauri.conf.json) - `resources/ui/` bundled as `ui/`"
---

## Context

ahpw's build names its assets by hash, so each ahpw release ships different file names under `ui\ahpw\assets`.
On 2026-10-22, after v0.1.0 updated to 0.2.0 and 0.2.0 was removed, the install folder still held ahpw 0.5.0's assets: the update wrote 0.6.0's files beside them, and the uninstaller removes only the files its own build shipped.

## Decision

The installer's pre-install hook removes `$INSTDIR\ui` before the files are written, after the template's check that closes the desk.
Source: Softov, asked "An update leaves the previous release's ahpw asset files [...] How to fix?": "Installer clears ui\ first (Recommended)".

## Consequences

Each install or update has only its own UI, and an uninstall then removes all of it.
Files left by an update to 0.2.0 are removed by the next install, 0.2.1 or later.

## Options

- **Record only.** Lost: every update would add a whole old ahpw build to the folder, kept after an uninstall.
