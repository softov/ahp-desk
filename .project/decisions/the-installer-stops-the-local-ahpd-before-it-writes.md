---
title: The installer stops the local ahpd before it writes the desk's files
status: accepted
date: 2026-10-22
refs:
  - "[code://src-tauri/windows/hooks.nsh](../../src-tauri/windows/hooks.nsh) - the installer's and uninstaller's hooks"
  - "[code://src-tauri/src/updates.rs](../../src-tauri/src/updates.rs) - the desk's own stop before its installer, from decision a-desk-update-stops-the-local-ahpd-the-desk-started"
---

## Context

The desk stops its local ahpd before it runs an update's installer ([a desk update stops the local ahpd the desk started](a-desk-update-stops-the-local-ahpd-the-desk-started.md)), but only from 0.2.0 on.
On 2026-10-22, v0.1.0 on a Windows VM found 0.2.0, and its Restart, with This computer running, stopped at "Error opening file for writing ...\ahp-desk\node\node.exe"; Retry went on only once ahpd was stopped by hand.
Every 0.1.0 desk with a local ahpd running meets this, and a person does not know what to stop.

## Decision

The installer's pre-install hook ends every process that runs from `$INSTDIR\node`, whatever desk started the installer, after the template's check that closes the desk.
The new desk starts the local ahpd again by its host's start setting.
Shipped in 0.2.1, so an update from 0.1.0 to 0.2.1 or later goes through.
Source: Softov, asked "Every v0.1.0 desk with This computer running hits 'Error opening file for writing ...node.exe' when it updates to 0.2.0 [...] What to do?": "Installer stops it too (Recommended)".

## Consequences

An install over a running local ahpd stops it with no question; the desk's own Restart has already asked over running turns from 0.2.0 on.
The 0.1.0 to 0.2.0 update keeps the error; 0.2.0 is skipped by a 0.1.0 desk only once 0.2.1 is the latest release.

## Options

- **Release note only.** Lost: a note on the release page is not read from the desk's notice.
- **Nothing.** Lost: the update fails for every 0.1.0 desk with This computer on, the default.
