---
title: An ahpd the desk installs keeps the usual config folder, with its backends under AHPD_PLUGIN_ROOT
status: accepted
date: 2026-10-13
refs:
  - file:///github/ahpd/packages/server/src/plugins.ts - a bare plugin name resolves from the config folder, then from `$AHPD_PLUGIN_ROOT/node_modules`
  - file:///github/ahpd/packages/server/src/install.ts - `pluginRoot()`: `ahpd plugin install` writes to `AHPD_PLUGIN_ROOT` when it is set
  - file:///github/ahpd/packages/server/src/config.ts - the config folder, `$XDG_CONFIG_HOME` or `~/.config`, then `ahpd`
---

## Context

The desk installs `@ahpd/server` and the backends into `<app local data>/ahpd`.
ahpd looks for a plugin in its config folder's `node_modules`, then in `$AHPD_PLUGIN_ROOT/node_modules`.
The config folder also holds `config.json` and `daemon.json`, the record that `ahpd status` reads.

## Decision

An ahpd the desk installs uses the usual config folder, `~/.config/ahpd`.
The desk runs every ahpd command of that install with `AHPD_PLUGIN_ROOT=<app local data>/ahpd`, so the backends are found there and `ahpd plugin install` puts new ones there.
Source: Softov, asked "Which config folder should an ahpd installed by the desk use?": "recommendation): the usual ~/.config/ahpd".

## Consequences

`ahpd status` in a terminal and the desk see the same record, so a user runs one ahpd.
The desk must set the variable on every command it runs for an ahpd it installed; an ahpd it found on `PATH` runs as it is.

## Options

- **A config folder of the desk's own (`XDG_CONFIG_HOME=<app local data>`).** Lost: a terminal `ahpd` does not see the desk's ahpd, and can start a second one beside it.
