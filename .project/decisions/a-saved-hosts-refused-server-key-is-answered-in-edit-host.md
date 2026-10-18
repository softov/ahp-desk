---
title: A saved host's refused server key shows on its row and is answered in Edit host
status: accepted
date: 2026-10-18
refs:
  - "[code://src-tauri/src/ssh/tunnel.rs](../../src-tauri/src/ssh/tunnel.rs) - `TunnelState.key`, the key the tunnel refused"
  - "[code://packages/shell/src/hosts/HostForm.tsx](../../packages/shell/src/hosts/HostForm.tsx) - Edit host, which shows `ServerKey` for that key"
---

## Context

A tunnelled host whose server key changed after it was added, or whose key the desk forgot, stops at the key on every try.
The row said only "tunnel down", and its Retry asked the server again with nothing new, so the person had no way to answer the key but removing the host and adding it again.

## Decision

The tunnel's state carries the key it refused, new or changed.
The row says "key changed" or "key not trusted" and offers "Check key" in place of Retry, which opens Edit host.
Edit host shows the same question as Add host (`ServerKey`): Trust for a new key, both fingerprints and "Forget the old key" for a changed one, or `ssh-keygen -R` for one from `~/.ssh/known_hosts`; each answer opens the tunnel again.
Source: Softov, 2026-10-18, asked "When a saved SSH host's server key changes, its row only shows \"tunnel down\" with Retry, which can't fix it. How should the person resolve it?": "Row says it, Edit shows key (Recommended)".

## Consequences

The answer to a refused key is in one place for Add host and Edit host, through one component.
The row needs a wider action column for "Check key".

## Options

- **Only in Edit host.** Lost: the row keeps "tunnel down", and the person has to guess that Edit is where the key is answered.
- **Leave it for the end to end.** Lost: until then the only way out is removing the host and adding it again.
