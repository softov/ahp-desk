---
title: Removing an SSH host offers to stop and uninstall its ahpd on the server
status: accepted
date: 2026-10-14
refs:
  - "[code://src-tauri/src/hosts.rs](../../src-tauri/src/hosts.rs) - `HostStore::remove` and the `remove_host` command"
  - file:///github/opendoop/packages/tauri/src-tauri/src/installer/systemd_node.rs - doop's user unit, the pattern the uninstall undoes
---

## Context

The desk installs ahpd on a server as a user service, with its own Node, backends and token file.
Removing the host from the desk alone would leave that ahpd running, listening and holding its token, with no client that knows of it.

## Decision

Remove, on a host the desk installed over SSH, asks whether to also stop and uninstall ahpd on the server.
Yes stops and disables the user unit, and removes the unit file, the desk's install and its token file, then removes the host.
Source: Softov, asked "p10 q11: removing an SSH host from the desk: what happens on the server?": "Offer to uninstall".

## Consequences

A server keeps no ahpd that the person forgot.
The uninstall needs the host's SSH access, so it runs before the host and its kept password are forgotten.
An ahpd the desk found on the server and did not install is not the desk's to uninstall; what Remove offers for it is a question of its own.

## Options

- **Left running.** Lost: an ahpd stays on the server, with its token, after the person removed it from the desk.
