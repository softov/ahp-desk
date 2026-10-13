---
title: An SSH server's key is confirmed once and remembered
status: accepted
date: 2026-10-13
refs:
  - file:///github/opendoop/packages/tauri/src-tauri/src/ssh/mod.rs - doop's `AcceptAllHandler`, which trusts any key
---

## Context

doop's SSH client accepts any server key, so a man in the middle would see the install and its token.

## Decision

The first connection to a server shows its key's fingerprint to confirm.
The desk keeps the confirmed key and refuses a different one with a warning that names both fingerprints.
A key already in `~/.ssh/known_hosts` for that host counts as confirmed.
Source: Softov, asked "How does the desk trust a server's SSH key the first time?": "Show the fingerprint, then remember (Recommended)".

## Consequences

The desk keeps its own list of confirmed keys beside `hosts.json`.
A server whose key really changed needs its old key removed in the desk first.

## Options

- **`known_hosts` only.** Lost: a new server needs an `ssh` from a terminal before the desk can reach it.
- **Accept any key.** Lost: the install and its token are open to a man in the middle.
