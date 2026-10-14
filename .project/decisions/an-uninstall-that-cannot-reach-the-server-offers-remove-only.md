---
title: An uninstall that cannot reach the server shows why, with Remove from the desk only and Cancel
status: accepted
date: 2026-10-14
refs:
  - "[code://packages/shell/src/hosts/HostList.tsx](../../packages/shell/src/hosts/HostList.tsx) - the row's Remove question, where the failure is shown"
---

## Context

[Removing an SSH host offers to stop and uninstall its ahpd](removing-an-ssh-host-offers-to-uninstall-its-ahpd.md), and the uninstall needs the host's SSH access.
A server can be down, unreachable, refuse the sign-in, or offer a changed key.

## Decision

When the uninstall cannot reach the server, or fails there, the host is kept and the Remove question shows why, with "Remove from the desk only" and Cancel.
Source: Softov, asked "p10: uninstall is asked but the server cannot be reached. What then?": "Error + remove-only button".

## Consequences

The person decides, knowing the server's ahpd stays when they remove the host from the desk only.
Cancel keeps the host and its SSH access, so the uninstall can be asked again later.

## Options

- **Remove anyway.** Lost: the host and its SSH access are gone while the server's ahpd keeps running, and the person may not notice.
