---
title: A changed server key is forgotten from its refusal
status: accepted
date: 2026-10-14
refs:
  - "[code://packages/shell/src/hosts/HostForm.tsx](../../packages/shell/src/hosts/HostForm.tsx) - Edit host, which does not carry the Forget"
---

## Context

[An SSH server's key is confirmed once and remembered](an-ssh-server-key-is-confirmed-once-and-remembered.md) refuses a server whose key changed, naming both fingerprints, and says a key that really changed needs the old one removed in the desk first.

## Decision

The refusal shows both fingerprints and a "Forget the old key" button.
The button removes the desk's confirmed key for that server, and the next connection asks to trust the new key as a first one does.
A changed key found in `~/.ssh/known_hosts` is not the desk's to forget: the refusal names `ssh-keygen -R <host>` instead.
Source: Softov, asked "p10 q3: when a server's key changed, where can the person forget the old one?": "On the refusal".

## Consequences

The person acts where the problem is shown, with the two fingerprints in front of them.
One click and one Trust replace the old key, so the words on the refusal must say plainly that a key that changed can mean someone in the middle.

## Options

- **Only in Edit host.** Lost: the person leaves the refusal, finds the host and its Edit, and no longer sees the fingerprints there.
