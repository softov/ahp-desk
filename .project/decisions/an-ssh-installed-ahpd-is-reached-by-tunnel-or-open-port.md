---
title: An SSH-installed ahpd is reached through the SSH connection or an open port, chosen per install
status: accepted
date: 2026-10-13
refs:
  - file:///github/ahpd/packages/server/src/commands/options.ts - `--host`, which binds 127.0.0.1 unless told otherwise, and needs a token for anything else
---

## Context

ahpd listens on the server's 127.0.0.1 by default.
The desk has to reach it from another machine.

## Decision

The installer offers both, with the tunnel chosen first:

- **Through the SSH connection**: ahpd stays on 127.0.0.1; the desk forwards a local port over SSH and connects to it. Nothing new is open on the server, and the host is reachable while the desk holds the session.
- **An open port with its token**: ahpd listens on 0.0.0.0 with its connection token, and the desk adds it as a URL host. ahpapp and browsers can reach it too; the token travels in cleartext without TLS, which the form says.

Source: Softov, asked "How does the desk reach an ahpd it installed over SSH?": "Ask per install".

## Consequences

A tunnelled host is a new host kind, `ssh`, whose address is the server and whose socket is local; the desk opens the tunnel before it connects, and its status is down while the tunnel is.
An open-port host is a URL host with its SSH details kept for Update.

## Options

- **The tunnel only.** Lost: ahpapp and browsers could never reach the host.
- **The open port only.** Lost: every install opens a port, with a token in cleartext.
