---
title: The port the remote ahpd listens on is a field, 9187 by default
status: accepted
date: 2026-10-14
refs:
  - file:///github/ahpd/packages/server/src/commands/options.ts - `--port`, 9187 by default, at lines 395-440
  - "[code://mockups/02-hosts.md](../../mockups/02-hosts.md) - Add host, kind SSH: Host, Port and User on one line"
---

## Context

ahpd listens on 9187 unless told otherwise.
A server may already use that number, for another service or another person's ahpd.

## Decision

Add host, "A server over SSH", has an "ahpd port" field with 9187 filled in, apart from the SSH Port.
The unit starts ahpd on that port; a tunnel forwards to it, and an open-port host connects to it.
Source: Softov, asked "p10 q7: which port does the remote ahpd listen on?": "A field, default 9187".

## Consequences

A server where 9187 is taken still takes the install, on another port.
The form has two port fields, so their labels must keep them apart.
The host keeps that port, and Update leaves it as it is.

## Options

- **Always 9187.** Lost: a server with 9187 in use cannot take the desk's install.
