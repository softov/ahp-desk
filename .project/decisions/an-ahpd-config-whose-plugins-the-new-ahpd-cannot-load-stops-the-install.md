---
title: An ahpd config whose plugins the new ahpd cannot load stops the install, and the desk never edits it
status: accepted
date: 2026-10-18
refs:
  - "[code://src-tauri/src/ssh_install.rs](../../src-tauri/src/ssh_install.rs) - `check_ready` and `not_ready`, run in the install's `backends` step"
  - file:///github/ahpd/packages/server/src/plugins.ts - `rootsOf`: the config folder is looked in before `AHPD_PLUGIN_ROOT`, on purpose, for the ahpd image
---

## Context

The desk's install over SSH keeps ahpd's usual config folder, `~/.config/ahpd` ([the decision](the-installed-ahpd-keeps-the-usual-config-folder.md)), and puts its backends under `AHPD_PLUGIN_ROOT`.
A server can hold the config of an ahpd set up before, with plugins in its own `node_modules`, and no ahpd that answers.
ahpd resolves a plugin from the config folder before `AHPD_PLUGIN_ROOT`, so the new ahpd loads those older copies, skips each it cannot load, and exits with no backend.
On a test server this ended the install in a 30-second wait and a journal dump.

## Decision

After `plugin install`, every backend ticked must be ready in the new ahpd's `plugin list --json`, matched by each row's `spec`.
When one is not, the install stops in its `backends` step, before any unit is written, naming each plugin with ahpd's state and reason and saying to update or remove them in `~/.config/ahpd`.
The probe warns when `~/.config/ahpd` is already there.
The desk never edits a config it did not make.
Source: Softov, 2026-10-18, asked "How should the install handle an older ahpd config on the server whose plugins the newest ahpd skips?": "Check plugins, refuse".

## Consequences

A server with an older ahpd's config is refused with what to do, instead of failing at `ready`.
The person cleans that folder by hand before the desk installs there.

## Options

- **The desk gets its own config folder under its install.** Lost: it replaces the decision that a server's own ahpd and the desk's look the same.
- **Warn and install anyway.** Lost: the install still fails when an old copy of a ticked backend wins.
- **ahpd loads from `AHPD_PLUGIN_ROOT` first.** Lost: ahpd orders the roots this way on purpose, for its image, where the config folder cannot be written.
