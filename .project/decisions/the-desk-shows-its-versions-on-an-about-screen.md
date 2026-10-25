---
title: The desk shows its versions on an About screen, from the tray and Settings
status: accepted
date: 2026-10-23
refs:
  - "[code://src-tauri/src/tray.rs#L21](../../src-tauri/src/tray.rs#L21) - `TrayWords`, the menu's items"
  - "[code://src-tauri/src/updates.rs#L75](../../src-tauri/src/updates.rs#L75) - `download_desk`, the check a Check now runs"
---

## Context

On 2026-10-23 nothing in the desk showed its own version, the ahpw it carries, or whether it was up to date; the only sign of an update was the notice once one was downloaded.

## Decision

An About screen in the host list window shows the desk's version, the bundled ahpw's version and the bundled Node's version, the desk's update state with a Check now button and Restart when an update is ready, the MIT license, and links to ahpio.com and the GitHub releases.
It opens from a tray item "About ahp-desk" and from a link on the Settings screen.
Source: Softov, asked "Where should the versions show?": "About screen (Recommended)".

## Consequences

The tray menu gains an item, between Settings and Quit.
ahpw itself still shows no update: a new ahpw comes with a desk release ([ahpw reaches the desk with a desk release](ahpw-reaches-the-desk-with-a-desk-release.md)), so About is where its version is read.

## Options

- **Bottom of Settings.** Lost: no screen of its own, and no tray item.
- **In the footer.** Lost: the versions always on screen, with no room for the update state.
