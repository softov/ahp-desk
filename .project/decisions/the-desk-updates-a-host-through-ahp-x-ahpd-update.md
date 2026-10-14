---
title: The desk updates a host's ahpd through the AHP method x-ahpd/update, over the connection the host list holds
status: accepted
date: 2026-10-14
refs:
  - "[code://packages/shell/src/hosts/status.ts#L75-L90](../../packages/shell/src/hosts/status.ts#L75-L90) - the one AHP connection per host, through `MultiHostClient`"
  - file:///github/ahpd/packages/sdk/src/recordschannel.ts - the `x-ahpd/<domain>/<verb>` naming of ahpd's own AHP methods, at lines 240-247
  - file:///github/ahpd/packages/sdk/src/host/gate.ts - the AHP methods and the grant each needs
---

## Context

ahpd's admin commands are declared once and reached over the CLI and HTTP, and ahpd's own methods are reached over AHP too.
The host list already holds one AHP connection per host, opened with the host's token, from the host's `ws://` address.
An HTTP call would need ahpd's `/api` turned on and an HTTP address kept beside the `ws://` one.

## Decision

The desk calls ahpd's update command, its `check` and the update itself, as the AHP method `x-ahpd/update`, on the connection the host list holds for that host.
Source: Softov, asked "p9 q3: which ahpd surface does the desk call to update a host?": "AHP x-ahpd/update".

## Consequences

The ahpd request (p9 task 03) asks for the AHP method, with the same params and answer as the command's other surfaces.
The desk adds no address and no setting to a host for the update.
The update's answer arrives before the restart, and the connection dropping after it is the restart, not a failure.

## Options

- **HTTP `POST /api/update`.** Lost: it needs ahpd's `/api` on and an HTTP address beside the host's `ws://` one, for a host the desk already reaches over AHP.
