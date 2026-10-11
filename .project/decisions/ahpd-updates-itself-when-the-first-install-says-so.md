---
title: ahpd updates itself when the first install says so
status: accepted
date: 2026-10-11
refs:
  - file:///github/ahp-desk/.project/decisions/a-local-ahpd-is-found-or-installed-with-the-bundled-node.md - the shell installs ahpd with npm
---

## Context

The shell installs ahpd and its backends with npm.
A newer `@ahpd/server` can come out at any time.

## Decision

The first install of "This computer" has a checkbox "Update ahpd automatically", on by default.
When it is on, the shell updates ahpd when no turn is running.
When it is off, the host row shows "update available", and a click updates.
The checkbox is also in the host's settings.
Source: Softov, asked "When a newer @ahpd/server exists, what does the shell do with the ahpd it installed?": "option on first install, auto update enabled?".

## Consequences

By default the local ahpd stays current with no action.
An update restarts ahpd, so the shell waits until no turn is running.
An ahpd that the shell found and did not install is never updated by the shell.

## Options

- **Show it, and install on a click.** Lost: by default an ahpd falls behind.
- **Never.** Lost: an update needs npm by hand.
