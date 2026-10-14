---
title: Update installs the version the host's row showed, passed by the desk
status: accepted
date: 2026-10-14
refs:
  - "[code://packages/shell/src/hosts/HostList.tsx](../../packages/shell/src/hosts/HostList.tsx) - the row that says \"ahpd X available\" and offers Update"
  - file:///github/ahpd/packages/server/src/update.ts - `newer()`, which the command uses to refuse a version not newer than the running one
---

## Context

A host's row says "ahpd X available", where X is npm's latest as the desk last asked, up to 6 hours before.
Between that answer and the click, npm may publish another version.
The update runs through ahpd's command (p9 task 04) or over SSH (p9 task 05).

## Decision

Update installs the version the row named: the desk passes it to ahpd's `x-ahpd/update` as `version`, and to the SSH update as the `@ahpd/server@<version>` it installs.
Source: Softov, asked "p9 q4: which version does Update install?": "The one the row showed".

## Consequences

What a person agreed to is what is installed.
ahpd's command still refuses a passed version that is not newer than the running one.
A version published after the desk's last ask reaches the row on the next ask, and is installed by the next Update.

## Options

- **npm's latest at the moment the update runs.** Lost: the host may end on a version the row never named, which the person did not see before the click.
