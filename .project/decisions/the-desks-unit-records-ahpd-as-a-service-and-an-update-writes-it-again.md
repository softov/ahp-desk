---
title: The desk's unit records ahpd as a systemd service, and an update over SSH writes the unit again
status: accepted
date: 2026-10-19
refs:
  - "[code://src-tauri/src/ssh_install.rs](../../src-tauri/src/ssh_install.rs) - `render_unit`, `write_unit` and `update`"
  - file:///github/ahpd/packages/server/src/service.ts - ahpd's own unit, named `ahpd`, at `~/.config/systemd/user/ahpd.service`
  - file:///github/ahpd/packages/server/src/daemon.ts - `SERVICE_ENV` (`AHPD_SERVICE`) and `serviceIn`, which reads `systemd` or `launchd` and ignores anything else
---

## Context

A terminal `ahpd status` on a server did not see the ahpd the desk installed there: `ahpd run` under the desk's unit wrote no record.
ahpd's daemon/31 (unreleased on 2026-10-19) makes a daemon started with `AHPD_SERVICE=systemd` record itself as held by systemd.
ahpd's answer, 2026-10-19: that variable is enough for `ahpd status`; for `ahpd stop` and `ahpd restart` too, the unit must be a systemd user unit named `ahpd.service`, running the daemon in the foreground, with `Restart=on-failure`.
The desk's unit already is all of that, with no `AHPD_SERVICE`.

## Decision

The desk's unit carries `Environment=AHPD_SERVICE=systemd`.
An update over SSH writes the unit an install writes today, with `daemon-reload`, before its restart, so an install made before this takes the line, and a later change to the unit reaches every desk install the same way.
Source: Softov, 2026-10-19, asked "Hosts the desk already installed over SSH have the old unit, without AHPD_SERVICE. Should the desk's Update over SSH also rewrite the unit, so they get it?": "Rewrite on update (Recommended)".

## Consequences

Once the server's ahpd has daemon/31, `ahpd status`, `stop` and `restart` in a terminal reach the desk's ahpd through systemd.
ahpd 0.12.0 and older do not read the variable, so the line changes nothing until then.
The update has a `unit` step between `server` and `start`, and needs the host's reach to write the right `--host`.

## Options

- **New installs only.** Lost: an install made before this keeps a unit a terminal `ahpd` cannot see until it is removed and installed again.
