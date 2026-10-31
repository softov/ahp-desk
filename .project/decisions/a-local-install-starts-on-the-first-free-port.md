---
title: A local install starts ahpd on the first free port from 9187 up
status: accepted
date: 2026-10-31
refs:
  - "[code://packages/shell/src/hosts/LocalForm.tsx#L150-L153](../../packages/shell/src/hosts/LocalForm.tsx#L150-L153) - the host is saved on `ws://127.0.0.1:9187` until ahpd says its address"
  - "[code://src-tauri/src/local_ahpd.rs#L790-L803](../../src-tauri/src/local_ahpd.rs#L790-L803) - `start_host`, which runs `ahpd start` with no port"
  - file:///github/ahpd/packages/server/src/commands/options.ts - at ahpd acdcd7ae, line 496: `--port`, "Listen here. Default 9187"
---

## Context

With no ahpd found, Add host installs the desk's own and starts it on ahpd's default, 9187.
When something that is not an ahpd listens there, the new ahpd stops at once on `EADDRINUSE`, and the form says only that it did not start.
Over SSH the same case starts on the first free port ([decision](an-ssh-install-starts-on-the-first-free-port.md)).

## Decision

When no ahpd is found, Add host asks the desk for the first port from 9187 up that nothing on `127.0.0.1` listens on.
The host is saved on that port, and the form says why when it is not 9187.
`start_host` passes `--port` with the port of the host's address when that port is not 9187, so every later start keeps it.
Source: Softov, 2026-10-31, asked "When no ahpd is found and something that is not an ahpd holds port 9187, what should a local install do?": "Propose a free port (Recommended)".

## Consequences

The install works on a computer where 9187 is taken.
ahpd's own `ahpd restart` replays the line it was started with, `--port` included.
A port taken between the check and the start still fails the start, with ahpd's words.

## Options

- **Refuse with a clear error.** Lost: the person must find and stop what holds 9187 before the desk works.
