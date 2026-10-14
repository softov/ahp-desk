---
title: An ahpd found on the server is used and offered the backends, and the desk installs only where none is found
status: accepted
date: 2026-10-14
refs:
  - "[code://packages/shell/src/hosts/LocalForm.tsx#L46-L130](../../packages/shell/src/hosts/LocalForm.tsx#L46-L130) - the found ahpd on this computer: its version, one too old shown, the backends it has ticked and fixed"
  - "[code://src-tauri/src/local_ahpd.rs](../../src-tauri/src/local_ahpd.rs) - `OLDEST_SUPPORTED`, 0.8.0, and the found ahpd's `supported` and `plugins`"
  - file:///github/ahpd/packages/server/src/commands/plugin.ts - `ahpd plugin list --json` and `plugin install`
---

## Context

A server can already have an ahpd: one installed by hand, a global npm install, or the desk's own earlier install.
On this computer, Add host uses a found ahpd: it shows its version, [shows one too old without replacing it](an-ahpd-too-old-on-path-is-shown-not-replaced.md), and [offers the backends it lacks](a-found-ahpd-with-no-backend-is-offered-the-backends.md).
A server's ahpd and the desk's install share `~/.config/ahpd` ([the usual config folder](the-installed-ahpd-keeps-the-usual-config-folder.md)), so a second ahpd beside a found one would fight over its record.

## Decision

When Add host over SSH finds an ahpd on the server, it uses it, as Add host does on this computer.
It shows its version, offers the backends it does not have, adds the ones ticked with that ahpd's own `plugin install`, and connects to it.
An ahpd older than the desk supports is shown with the version needed, not replaced.
The desk installs ahpd only on a server where none is found.
Source: Softov, asked "p10 q8: Add host over SSH finds ahpd already installed on the server. What does the desk do?": "Use it, offer backends".

## Consequences

A server keeps one ahpd, seen the same from its terminal and from the desk.
The desk's own earlier install is found like any other and is used, not installed over.
The probe must find an ahpd on the server and read its version, its plugins and whether it runs, and the host must record whether the desk installed it, for Update and Remove.
An ahpd the desk did not install runs under no unit the desk wrote, so how the desk starts it when it is not running, and how it is reached, are questions of their own.

## Options

- **Install over it.** Lost: two ahpds, or one replaced that the person installed and configured.
- **Only the desk's own installs.** Lost: a server whose owner already runs ahpd cannot be added over SSH.
