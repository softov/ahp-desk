---
title: The desk asks before it installs its own update
status: accepted
date: 2026-10-13
refs:
  - "[code://src-tauri/tauri.conf.json](../../src-tauri/tauri.conf.json) - where the updater's endpoint and public key go"
---

## Context

Plan p4 adds Tauri's updater, which finds a signed release on GitHub and can install it.
Installing restarts the desk, which closes every app window and its unsent work.

## Decision

The desk downloads an update in the background and then says so, in the host list and in the tray: "ahp-desk X is ready. Restart to update."
It restarts only on that click.
Source: Softov, asked "When the desk itself has an update, what happens?": "Ask, then install (Recommended)".

## Consequences

Nothing restarts under a person's hands; an update can wait days if nobody clicks.
The notice is the desk's, in its three languages.

## Options

- **Install on the next quit.** Lost: a quit that installs is slower and surprising, and the person never chose the moment.
