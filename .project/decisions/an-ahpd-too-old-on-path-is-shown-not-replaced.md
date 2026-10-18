---
title: An ahpd on PATH too old for the desk is shown, not replaced
status: accepted
date: 2026-10-14
refs:
  - "[code://src-tauri/src/local_ahpd.rs](../../src-tauri/src/local_ahpd.rs) - `OLDEST_SUPPORTED`, 0.8.0, and `FoundView.supported`"
---

## Context

The desk drives ahpd through `ahpd status --json`, which came in 0.8.0.
An older ahpd can be the one on `PATH`, as a global npm install.
An ahpd the desk installs would share `~/.config/ahpd` with it ([the installed ahpd keeps the usual config folder](the-installed-ahpd-keeps-the-usual-config-folder.md)), so the two would read and write the same `daemon.json` and `config.json`.

## Decision

When the ahpd found on `PATH` or at the set path is older than 0.8.0, Add host says so, with its version, the version needed and `npm i -g @ahpd/server`, and a Check again.
It installs nothing in its place.
Source: Softov, asked "What does Add host do when the ahpd on PATH is older than 0.8.0?": "ok, go for A".

## Consequences

A user keeps one ahpd, seen the same from a terminal and from the desk.
A user with an old global ahpd updates it by hand before "This computer" works.

## Options

- **Install the desk's own ahpd beside it.** Lost: two ahpds share one config folder and fight over its record.
- **Offer both.** Lost: the second button leads to the same two ahpds.
