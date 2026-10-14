---
title: A server key never seen before refuses the first connection, and Trust connects again
status: accepted
date: 2026-10-14
refs:
  - file:///github/opendoop/packages/tauri/src-tauri/src/ssh/mod.rs - doop's `AcceptAllHandler`, the `check_server_key` that answers during the handshake
---

## Context

[An SSH server's key is confirmed once and remembered](an-ssh-server-key-is-confirmed-once-and-remembered.md) shows a new server key's fingerprint for the person to confirm.
`russh` asks the client's handler about the server key during the handshake, and the person's answer can take minutes.

## Decision

A server key never seen before refuses that first connection, naming the fingerprint.
When the person clicks Trust, the desk records the key and connects again, and that connection passes the check as any known key does.
Source: Softov, asked "p10 q2: a server key never seen before: what happens to that first connection?": "Refuse, reconnect".

## Consequences

The key check stays a plain lookup with no waiting inside the handshake, and one path checks every connection.
A first connection to a server is made twice: once refused at the key, before any sign-in, and once after Trust.
A sign-in that asks something, as a passphrase, comes after Trust, on the second connection.

## Options

- **Hold the handshake.** Lost: the handshake waits on a person, against the server's own sign-in timeout.
