---
title: Stopping the local ahpd on quit is a host setting
status: accepted
date: 2026-10-11
refs:
  - file:///github/ahp-desk/mockups/02-hosts.md - "Start [x] when the desktop app starts"
  - file:///github/opendoop/packages/tauri/src-tauri/src/pood_process.rs - doop never stops the pood it starts
---

## Context

The shell can start the local ahpd.
Sessions run inside ahpd, so a stop ends the turns that run.
doop leaves its daemon on after it quits.

## Decision

The local host has a checkbox "Stop when the desktop app quits", beside "Start when the desktop app starts".
Source: Softov, asked "When ahp-desk quits, what happens to the local ahpd it started?": "Per-host setting".

## Consequences

The shell keeps a record of whether it started the ahpd, so it does not stop one that it only found.
A stop on quit must be graceful, so that ahpd can save its sessions.

## Options

- **Stop it, unless it was already on.** Lost: no way to keep sessions on after a quit.
- **Leave it on.** Lost: no way to free the machine when the app quits.
