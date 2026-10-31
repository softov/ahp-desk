---
title: A local ahpd found through its record or npm's prefix is used as one on PATH
status: accepted
date: 2026-10-31
refs:
  - "[code://src-tauri/src/local_ahpd.rs#L726-L745](../../src-tauri/src/local_ahpd.rs#L726-L745) - `launcher_for`, which every local command goes through"
  - "[code://packages/shell/src/hosts/LocalForm.tsx#L184-L188](../../packages/shell/src/hosts/LocalForm.tsx#L184-L188) - the line naming the ahpd found, and the too-old note"
---

## Context

Add host shows a found ahpd with its version and where it came from, and offers to install the desk's own only when none is found.
Plan 16 adds two places an ahpd is found ([decision](a-local-ahpd-off-path-is-found-through-its-record-or-npms-prefix.md)).

## Decision

An ahpd found through its record or npm's prefix is shown and used as one found on `PATH`.
Add host names it with its version and where it was found, applies the too-old rule ([decision](an-ahpd-too-old-on-path-is-shown-not-replaced.md)), and offers no install while it is there.
The desk starts, stops, asks the status of and updates it through its own commands ([decision](the-local-ahpd-runs-through-its-own-start-and-stop.md), [decision](a-local-ahpd-the-desk-did-not-install-updates-as-a-remote-one.md)).
Source: Softov, 2026-10-31, asked "What does Add host do with a local ahpd found that way?": "Use it as one on PATH (Recommended)".

## Consequences

One ahpd per config folder: the desk never installs a second over a found one.
A person who wants the desk's own ahpd removes or stops the other first, or sets a path with Change.

## Options

- **Offer both.** Lost: the two share `~/.config/ahpd` and its port, so the second would fight the first.
