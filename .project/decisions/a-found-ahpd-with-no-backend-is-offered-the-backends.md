---
title: A found ahpd with no backend is offered the backends in Add host
status: accepted
date: 2026-10-14
refs:
  - file:///github/ahpd/packages/server/src/commands/plugin.ts - `ahpd plugin list --json`, each plugin's name and state, and `plugin install`
  - "[code://src-tauri/src/local_ahpd.rs](../../src-tauri/src/local_ahpd.rs) - the found ahpd, and the install"
---

## Context

Add host offers the backends only when the desk installs ahpd (decision 2).
An ahpd found on `PATH` may name no backend in its config, and then it refuses to start: Softov's global ahpd 0.11.0 did, on 2026-10-14.
`ahpd plugin list --json` names each plugin and its state, but not whether it is a backend.

## Decision

For a found ahpd, Add host shows the backends too: those its `plugin list` has ready are ticked and fixed, and those ticked here are added with that ahpd's own `plugin install` before it starts.
Save and start needs at least one backend: one of those offered, ticked or ready, or a ready plugin the desk does not offer.
Source: Softov, asked "Add host found an ahpd on PATH whose config names no backend, so it cannot start. What should Add host do in that case?": "Offer the backends (Recommended)".

## Consequences

A found ahpd with no backend starts from Add host, with no terminal.
The desk changes the plugins of an ahpd it did not install, and only when the person ticks a backend.
A ready plugin the desk does not offer counts as a backend, since the list cannot tell; a config with only such a plugin that is not a backend still fails at the start, with ahpd's words.

## Options

- **Say so, and stop.** Lost: the person runs `ahpd configure` in a terminal first.
- **Install the desk's own.** Lost: two ahpds share `~/.config/ahpd` (decision 9).
