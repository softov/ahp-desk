---
title: A local ahpd the desk did not install updates as a remote one, else its row says how
status: accepted
date: 2026-10-23
refs:
  - "[code://packages/shell/src/hosts/status.ts#L242](../../packages/shell/src/hosts/status.ts#L242) - `updateWay`, which answers no way for a local ahpd the desk did not install"
  - "[code://packages/shell/src/hosts/HostList.tsx#L436](../../packages/shell/src/hosts/HostList.tsx#L436) - the row's notice and its Update"
  - https://github.com/softov/ahpd/blob/main/CHANGELOG.md - 0.13.0 added `ahpd update` and `x-ahpd/update`
---

## Context

Plan 09 offers Update on a host's row over SSH for an ahpd the desk installed there, or through ahpd's own update command when the host's token may use it.
A local host was left out: the desk updates the ahpd it installed as plan 03 says, and no other.
On 2026-10-23, a "This computer" host on an ahpd 0.12.0 the person installed with npm showed "ahpd 0.13.0 available" with no Update and no word on how to update it.

## Decision

A local ahpd the desk did not install gets the row's Update through ahpd's own update command, when ahpd's check says this token may update it, as a remote host does.
When it cannot, as for an ahpd older than 0.13.0, which has no update command, the row says how to update it where it was installed: `npm i -g @ahpd/server`.
The ahpd the desk installed keeps plan 03's update.
Source: Softov, asked "A local ahpd the desk did not install, with a newer ahpd on npm: what should its row offer?": "Both (Recommended)".

## Consequences

Plan 09's row "The local ahpd updates as plan 03's Answers say" now holds only for the ahpd the desk installed.
An ahpd 0.12.0 shows the words alone; from 0.13.0 on it shows Update.
"Update ahpd automatically" stays for the ahpd the desk installed; one it did not install updates only on a click.

## Options

- **ahpd's own command only.** Lost: an ahpd with no update command would keep a notice with no way forward.
- **Say how only.** Lost: an ahpd that can update itself would still need a terminal.
