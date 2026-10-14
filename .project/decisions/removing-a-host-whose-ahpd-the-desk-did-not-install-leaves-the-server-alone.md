---
title: Removing an SSH host whose ahpd the desk did not install does nothing on the server, and says so
status: accepted
date: 2026-10-14
refs:
  - "[code://packages/shell/src/hosts/HostList.tsx](../../packages/shell/src/hosts/HostList.tsx) - the row's Remove question"
---

## Context

[Removing an SSH host offers to stop and uninstall its ahpd](removing-an-ssh-host-offers-to-uninstall-its-ahpd.md), for an ahpd the desk installed.
[An ahpd found on the server is used](an-ahpd-found-on-the-server-is-used-and-offered-the-backends.md), so a host can point at an ahpd someone else installed.

## Decision

Remove on an SSH host whose ahpd the desk did not install does nothing on the server.
The Remove question says the server's ahpd stays as it is, because the desk did not install it.
Source: Softov, asked "p10: Remove on an SSH host whose ahpd the desk did not install. What happens on the server?": "Nothing, and say so".

## Consequences

An ahpd the person or another client uses is never stopped by the desk.
An ahpd the desk [started because it was not running](a-found-ahpd-not-running-on-the-server-is-started-with-its-own-start.md) keeps running after its host is removed, until it is stopped on the server.

## Options

- **Offer to stop it.** Lost: the desk would stop an ahpd it did not install, which someone else may be using.
