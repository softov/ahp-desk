---
title: A found ahpd not running on the server is started with its own ahpd start, with no unit, and the form says it will not survive a reboot
status: accepted
date: 2026-10-14
refs:
  - file:///github/ahpd/packages/server/src/commands/start.ts - `ahpd start`, which runs ahpd in the background and writes `daemon.json`
  - "[code://src-tauri/src/local_ahpd.rs](../../src-tauri/src/local_ahpd.rs) - how p3 starts a found ahpd on this computer with its own `start`"
---

## Context

[An ahpd found on the server is used](an-ahpd-found-on-the-server-is-used-and-offered-the-backends.md), and the desk installs nothing beside it.
An ahpd installed by hand may not be running, and the desk wrote no systemd unit for it.

## Decision

When the ahpd found on the server is not running, the desk starts it with that ahpd's own `ahpd start`, as it is configured.
The desk writes no systemd unit for it.
The form says it will not come back after the server restarts.
Source: Softov, asked "p10: the server already has ahpd, but it is not running. What does the desk do?": "Start it, say it won't survive reboot".

## Consequences

An ahpd the person set up by hand stays as they set it up, with no unit of the desk's beside it.
After a server restart that ahpd is down until someone starts it, and the host shows down; the person was told so in the form.

## Options

- **Start it and add a unit.** Lost: the desk writes a service for an ahpd it did not install.
- **Only say it is off.** Lost: the person leaves the desk to start it before the host works.
