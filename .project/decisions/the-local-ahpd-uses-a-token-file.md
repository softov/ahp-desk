---
title: The local ahpd uses a token file
status: accepted
date: 2026-10-11
refs:
  - file:///github/ahpd/packages/server/src/commands/options.ts - no token on loopback by default, and `--connection-token-file` at lines 1051-1085
---

## Context

ahpd needs no token when it listens on 127.0.0.1.
Then any program or user on the machine can drive its sessions.

## Decision

The shell starts the local ahpd with `--connection-token-file`, in the app data folder.
ahpd writes a new token to that file when it is missing.
The shell reads it and saves it as the host's token.
Source: Softov, asked "Does the local ahpd use a token on loopback?": "Yes, a token file (Recommended)".

## Consequences

Only a program with the token connects to the local ahpd.
An ahpd that the shell finds already on keeps its own setting, and the shell reads its token from `daemon.json`.

## Options

- **No token on loopback.** Lost: anything on the machine can connect.
