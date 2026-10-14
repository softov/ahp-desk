---
title: Update shows on a host's row only after ahpd's update command answers a check that the token may update it
status: accepted
date: 2026-10-14
refs:
  - file:///github/ahpd/packages/server/src/commands/scopes.ts - `deploymentTokenOnly`, the refusal a person's token gets, at lines 37-47
  - file:///github/ahpd/packages/server/src/commands/plugin.ts - `plugin update`, the admin command the update command follows
  - "[code://packages/shell/src/hosts/HostList.tsx](../../packages/shell/src/hosts/HostList.tsx) - the row that shows Update"
---

## Context

Plan p9's decision 3 offers Update through a new ahpd command for a host whose token may administer it.
ahpd refuses a person's token for an admin command whatever its grants, and may refuse an install it cannot update or a daemon it cannot restart.
The desk holds only a token, so it cannot tell from its own records whether ahpd will take the update.

## Decision

ahpd's update command takes `check`, which changes nothing and answers `{ current, latest, may }`, or the refusal a real update would get.
The desk sends the check once per connection on a host whose row shows the notice, and shows Update only when the answer says `may: true`.
A refusal, or an ahpd with no such command, leaves the notice without the button.
Source: Softov, asked "p9 q2: before showing Update on a host's row, how does the desk know ahpd will accept its token for the update?": "A check call".

## Consequences

The ahpd request (p9 task 03) carries `check` as part of the command's shape.
A button the desk shows is one ahpd has already agreed to, so a click fails only on what changed since the check.
The check costs one request per connection on a host that is behind npm, and none on one that is up to date.

## Options

- **Show Update, and report the refusal on the click.** Lost: a host reached with a person's token would offer a button that always fails.
- **ahpd says it in its `initialize` answer.** Lost: every client would get admin details in the handshake, and the answer would be fixed for the whole connection.
