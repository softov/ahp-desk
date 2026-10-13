---
title: SSH signs in with the agent, a key file, or a password
status: accepted
date: 2026-10-13
refs:
  - file:///github/opendoop/packages/tauri/src-tauri/src/ssh/mod.rs - doop's agent-first, then key-file sign-in
  - "[code://src-tauri/src/tokens.rs](../../src-tauri/src/tokens.rs) - the OS keyring the desk keeps host tokens in"
---

## Context

doop signs in with the SSH agent, then a key file.
Some servers, and some people, have only a password.

## Decision

The desk tries the SSH agent (Pageant or the OpenSSH agent) first, then a key file the person picks, asking its passphrase when it has one, then a password.
A password the person asks to keep goes in the OS keyring, as host tokens do; a passphrase is never kept.
Source: Softov, asked "How does the desk sign in over SSH?": "Also a password".

## Consequences

The SSH form has a password field and a "Remember" checkbox.
A kept password is removed with its host.

## Options

- **The agent, then a key file, only.** Lost: a server with password sign-in only could not be installed from the desk.
