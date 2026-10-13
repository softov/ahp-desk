---
title: A host's ahpd update is shown on its row and offered, over SSH or through ahpd
status: accepted
date: 2026-10-13
refs:
  - file:///github/ahpd/packages/server/src/update.ts - ahpd's own npm check, written to `update.json` and printed in its log only
  - file:///github/ahpd/packages/server/src/commands/restart.ts - ahpd's restart, which an update needs after the install
  - "[code://packages/shell/src/hosts/HostList.tsx](../../packages/shell/src/hosts/HostList.tsx) - the row that shows the host's server and version"
---

## Context

The host list shows each host's server name and version, from its answer to `initialize`.
ahpd knows when npm has a newer `@ahpd/server`, but tells only its own log; no client hears of it.
A remote host's owner has no reason to look at that log.

## Decision

The desk compares each connected ahpd's version with the newest `@ahpd/server` on npm, and the host's row says "ahpd X available" with an Update button.
Update works two ways:

- over SSH, for a host the desk installed over SSH: the newer `@ahpd/server` is installed and the service restarted;
- through a new ahpd admin command, for any other host whose token may administer it: ahpd installs the newer version and restarts itself.

A host the desk can update neither way shows the notice without the button.
The local ahpd keeps its own rule (decision: ahpd updates itself when the first install says so).
Source: Softov, asked what the desk does when a URL host's ahpd is older than npm's: "both. say on the host and offer the update"; asked how Update reaches a remote host: "Both ways (Recommended)".

## Consequences

ahpd needs an update command, over HTTP and AHP, for an administrator; that is ahpd work, done in the ahpd repository.
The desk asks npm once per few hours, not once per host.
An update restarts the host, so the button says which sessions are running and waits for a click.

## Options

- **Over SSH only.** Lost: a host the desk did not install could never be updated from it.
- **The ahpd command only.** Lost: an SSH-installed host whose token is not an administrator's could not be updated, though the desk holds its SSH access.
- **The notice only.** Lost: the person who sees the notice is often the owner, and then has to leave the desk to act on it.
