---
title: Uninstalling the desk's ahpd leaves the server's ~/.config/ahpd
status: accepted
date: 2026-10-14
refs:
  - file:///github/ahpd/packages/server/src/config.ts - the config folder, `$XDG_CONFIG_HOME` or `~/.config`, then `ahpd`
---

## Context

[An ahpd the desk installs keeps the usual config folder](the-installed-ahpd-keeps-the-usual-config-folder.md), so on a server the desk's install shares `~/.config/ahpd` with any ahpd the person runs there.
[Removing an SSH host offers to uninstall its ahpd](removing-an-ssh-host-offers-to-uninstall-its-ahpd.md): the unit, the desk's install and its token file.

## Decision

The uninstall leaves `~/.config/ahpd` on the server as it is.
Source: Softov, asked "p10: when the desk uninstalls its ahpd, what happens to ~/.config/ahpd?": "Leave it".

## Consequences

An ahpd the person installs later on that server finds its config, folders and records.
Its `plugins` may name backends that were under the desk's install and are now gone, which that ahpd reports when it starts.

## Options

- **Remove it too.** Lost: the folder is ahpd's usual one, which another ahpd on the server may use.
- **Ask in the dialog.** Lost: one more choice in Remove for a folder most people never look at.
