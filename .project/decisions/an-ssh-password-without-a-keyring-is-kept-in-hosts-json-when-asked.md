---
title: With no keyring, an SSH password is kept in hosts.json only when the person ticks a box that says so
status: accepted
date: 2026-10-14
refs:
  - "[code://src-tauri/src/tokens.rs](../../src-tauri/src/tokens.rs) - `set`, `get` and `move_to_vault`: the keyring first, the host's field in `hosts.json` when it fails"
  - "[code://src-tauri/src/hosts.rs](../../src-tauri/src/hosts.rs) - `HostView.token_in_file`, the flag the host list shows on a token kept in the plain file"
---

## Context

The desk keeps a password the person asks to keep in the OS keyring, as [plan 10's Answers](../plans/desk/10-ssh-install/plan.md) say.
A Linux desktop with no Secret Service has no keyring.
[Tokens are in the OS keyring, with hosts.json as the fallback](tokens-are-in-the-os-keyring-with-hosts-json-as-the-fallback.md) keeps a token in `hosts.json` there, and the host list flags that host.

## Decision

On a machine with no keyring, the form still offers to keep the password.
Its checkbox says the password is kept unencrypted in the desk's file.
Ticked, the password goes in `hosts.json`, and the host's row is flagged as a token in that file is.
Source: Softov, asked "p10 q4: on a machine with no keyring, can the person still save the SSH password?": "Opt-in, says plain file".
Before answering, Softov asked "a option to save?" when offered to keep it nowhere.

## Consequences

A person with no keyring can still have a tunnel open at launch with no question, and is told where the password sits before choosing it.
The form must know whether the keyring works before the save, not only when the save fails.
A password in `hosts.json` moves to the keyring when the keyring works again, as a token does.

## Options

- **The same checkbox with no warning.** Lost: the password lands in a plain file without the person knowing.
- **Not offered.** Lost: with no keyring, the password is asked at each connection, and a tunnel cannot open at launch.
