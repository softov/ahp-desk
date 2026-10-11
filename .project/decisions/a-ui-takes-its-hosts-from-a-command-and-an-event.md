---
title: A UI takes its hosts from a command and an event
status: accepted
date: 2026-10-11
refs:
  - file:///github/ahp-desk/mockups/03-app-window.md - the arrows between the shell and the UI
  - file:///github/ahpw/src/connection/data.ts - ahpw builds its one host from window.location at lines 325-335
  - file:///github/ahpapp/src/host-context.tsx - ahpapp loads its hosts from AsyncStorage at line 650
---

## Context

The shell keeps the host list, and a UI must connect to those hosts.
ahpw takes its host from `window.location`.
ahpapp takes its hosts from its own storage.
Both need one more way to get hosts, from the shell.

## Decision

A UI calls the Tauri command `hosts` to read the host list.
It listens to the Tauri event `hosts-changed` to see edits.
Tokens stay in the OS keyring until a UI asks for them.
Source: Softov, asked "How should the shell give the loaded UI its hosts (the contract in mockup 03)?": "Tauri command + event (Recommended)".

## Consequences

A UI sees an edit to the host list with no reload.
Each UI needs a small desktop host source beside its current one.
The command needs a Tauri capability for the UI's origin.

## Options

- **An injected global.** The shell sets `window.__AHP_DESK__` before the page loads. Lost: it is a snapshot, so a change needs a reload.
- **A local proxy for each host.** ahpw stays as it is. Lost: every host is a separate origin, and many hosts have no single place to live.
