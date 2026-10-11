---
title: Tokens are kept in the OS keyring
status: superseded
date: 2026-10-11
superseded-by: decisions/tokens-are-in-the-os-keyring-with-hosts-json-as-the-fallback.md
refs:
  - file:///github/ahp-desk/mockups/02-hosts.md - "Token [...] in the OS keyring"
  - file:///github/opendoop/packages/tauri/src-tauri/src/profile.rs - doop keeps only a token reference, which no code reads
  - https://crates.io/crates/keyring - the Rust crate for the Windows Credential Manager, the macOS Keychain and the Secret Service
---

## Context

A host's token gives full access to its sessions.
doop's profile file holds a reference to a token, and no code reads it.
The host list is a file in the app's config folder.

## Decision

`hosts.json` holds every field of a host except its token.
Each token is in the OS keyring, under the service `ahp-desk` and the host id.
Source: (defaulted: mockup 02 says "in the OS keyring"; Softov may erase this).

## Consequences

A copy of `hosts.json` holds no secret.
On Linux the keyring needs a Secret Service, such as GNOME Keyring or KWallet.
A machine with no Secret Service cannot save a token, and Add host says so.

## Options

- **The token in `hosts.json`.** Lost: a secret in a plain file.
