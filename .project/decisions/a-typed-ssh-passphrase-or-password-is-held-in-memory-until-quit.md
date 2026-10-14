---
title: A typed SSH passphrase, or a password not saved, is held in memory until the desk quits
status: accepted
date: 2026-10-14
refs:
  - file:///github/opendoop/packages/tauri/src-tauri/src/ssh/mod.rs - doop's sign-in, which asks nothing it cannot read from the agent or a key file
  - "[code://packages/shell/src/hosts/HostList.tsx](../../packages/shell/src/hosts/HostList.tsx) - the row where a tunnelled host that needs the person shows \"Sign in\""
---

## Context

A key file with a passphrase, or a password the person did not ask to keep, is typed in the form.
A tunnelled host opens its SSH session again at every launch and after every dropped link, and a key refused is not the only reason it needs the person.
[SSH signs in with the agent, a key file, or a password](ssh-signs-in-with-the-agent-a-key-file-or-a-password.md) never keeps a passphrase, and keeps a password only when the person asks.

## Decision

The desk holds a typed passphrase, and a typed password not saved, in memory for the process, per host, until it quits.
Neither is ever written to disk.
After a restart, a tunnelled host whose sign-in needs one shows down on its row with "Sign in", which asks it again.
Source: Softov, asked "p10 q1: an SSH key passphrase, or a password not saved: how long does the desk hold it?": "In memory until quit".

## Consequences

A dropped tunnel opens again by itself while the desk runs, with no question.
After a restart, such a host stays down until the person signs in on its row.
The desk process holds secrets in memory, so they must never reach a log line or an event.

## Options

- **Ask at every connection.** Lost: each dropped link and each reopened tunnel would stop on a question.
