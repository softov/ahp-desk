---
title: Tokens are in the OS keyring, with hosts.json as the fallback
status: accepted
date: 2026-10-11
supersedes: decisions/tokens-are-kept-in-the-os-keyring.md
refs:
  - file:///github/ahp-desk/mockups/02-hosts.md - "Token [...] in the OS keyring"
  - https://crates.io/crates/keyring - the Rust crate for the Windows Credential Manager, the macOS Keychain and the Secret Service
---

## Context

A host's token gives full access to its sessions.
The OS keyring keeps it encrypted and out of plain files.
A Linux machine with no Secret Service has no keyring, so a save there fails.

## Decision

The shell saves each token in the OS keyring, under the service `ahp-desk` and the host id.
With no keyring, it saves the token in the host's `token` field in `hosts.json`.
The host list then shows a warning on that host.
To read a token, the shell asks the keyring first.
When the keyring has no token and `hosts.json` has one, the shell uses the token from `hosts.json`.
Source: Softov, asked "Tokens in the OS keyring, or in hosts.json?": "ok can be os keyring. default to file if no token on keyring and has token on file? keyring with the fallback.".

## Consequences

A token is in a plain file only on a machine with no keyring, and the user sees that.
A token that is in `hosts.json` moves to the keyring when the keyring works again, and the field is cleared (defaulted: Softov may erase this).
A person can put a token in `hosts.json` by hand, and the shell uses it.

## Options

- **The keyring only.** Lost: no token can be saved on a machine with no keyring.
- **`hosts.json` only.** Lost: every token is in a plain file.
