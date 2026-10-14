---
title: A found ahpd on the server is reached on its own port, through a tunnel, or through an open port only when it already listens beyond loopback
status: accepted
date: 2026-10-14
refs:
  - file:///github/ahpd/packages/server/src/commands/status.ts - `ahpd status --json`, with pid, url and paths, or `{"running":false}`
  - file:///github/ahpd/packages/server/src/commands/options.ts - `--host`, 127.0.0.1 unless told otherwise
---

## Context

[An SSH-installed ahpd is reached through the SSH connection or an open port](an-ssh-installed-ahpd-is-reached-by-tunnel-or-open-port.md), chosen per install, because the desk writes the unit and picks the address.
An ahpd the desk found on the server and did not install already has its address, port and token.

## Decision

The desk reads a found ahpd's address from its `ahpd status --json` and its token from its `daemon.json`, as p3 reads a found ahpd's on this computer.
The ahpd port field shows its port, fixed.
The tunnel is always offered; the open port is offered only when that ahpd already listens beyond 127.0.0.1.
Source: Softov, asked "p10: how does the desk reach an ahpd it found on the server (one it did not install)?": "Its own port, via tunnel".

## Consequences

The desk never changes how a found ahpd listens.
A found ahpd on 127.0.0.1 is reachable from the desk only while the desk holds its tunnel.

## Options

- **Tunnel or open port as asked.** Lost: an open port asked for an ahpd on 127.0.0.1 means restarting it on another address, changing an ahpd the desk did not install.
