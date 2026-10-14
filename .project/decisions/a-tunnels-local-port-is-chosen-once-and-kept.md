---
title: A tunnelled host's local port is chosen once and kept on the host
status: accepted
date: 2026-10-14
refs:
  - "[code://src-tauri/src/hosts.rs](../../src-tauri/src/hosts.rs) - `Host`, whose `url` is the address ahpw keys a host's pages by"
---

## Context

A tunnelled SSH host is reached at `ws://127.0.0.1:<local port>` on the desk's machine.
[ahpw keeps a host's pages under the host's address](ahpw-keeps-a-hosts-pages-under-the-hosts-address.md), so the address must stay the same for the host's tabs and layout to come back.

## Decision

The desk picks a free local port once, when the host is added, and keeps it on the host.
Every later tunnel for that host listens on that port.
Source: Softov, asked "p10 q10: a tunnelled SSH host's local port: fixed or new each time?": "Chosen once, kept".

## Consequences

A tunnelled host opens on its own tabs and layout after every restart.
A port taken by another program when the tunnel opens shows as the tunnel's error on the host's row.

## Options

- **A free port each time.** Lost: the host's address changes at each open, and ahpw loses its pages.
