---
title: The first run adds a URL host; this computer comes later
status: accepted
date: 2026-10-11
refs:
  - file:///github/ahp-desk/.project/plans/desk/01-first-build-p3-local-ahpd/plan.md - the local ahpd, which brings "This computer"
  - "[code://packages/shell/src/hosts/HostForm.tsx](../../packages/shell/src/hosts/HostForm.tsx) - the form the first run reuses"
---

## Context

With no host, the host list opened on an empty table.
A first run should walk the user to a working host, as an installer does.

## Decision

With no host, the host list opens on a first run: a welcome, then the choice of where the host is, then the address, name and token with Test, then Open.
"A host on the internet or my network" is the one live choice; "This computer" is shown as coming soon until p3 brings the local ahpd.
Source: Softov, asked what the first run offers before the local ahpd exists: "URL now, local later (Recommended)".

## Consequences

A new user ends the first run in an ahpw window on a host that answered Test.
p3 turns "This computer" on; the first run's steps stay.

## Options

- **Wait for p3 and offer both.** Lost: the first run waits on the local ahpd.
