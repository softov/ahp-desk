---
title: An install over SSH starts on the first free port, and refuses one in use
status: accepted
date: 2026-10-20
refs:
  - "[code://src-tauri/src/ssh_install.rs](../../src-tauri/src/ssh_install.rs) - `AHPD_PORT`, `PROBE_SCRIPT`, `install_new`"
  - "[code://packages/shell/src/hosts/SshForm.tsx](../../packages/shell/src/hosts/SshForm.tsx) - the form's \"ahpd port\""
---

## Context

The form's "ahpd port" starts at 9187 (`AHPD_PORT`), and the install writes it into the unit without looking.
On 2026-10-20 another ahpd held 9187 on the test machine: the new ahpd stopped on `EADDRINUSE` and systemd restarted it on a loop, and the desk failed at "Wait for ahpd" with `cat: .../connection-token: No such file or directory`, which does not say the port is taken.

## Decision

The probe lists the TCP ports listening on the server, from `/proc/net/tcp` and `/proc/net/tcp6`.
The form's "ahpd port" starts at the first port from 9187 up that nothing listens on, and says why when that is not 9187.
A port that something listens on is refused in the form and again by the install, before anything is installed.
Source: Softov, asked "What should the desk do?": "Propose a free port (Recommended)".

## Consequences

An install next to another service on 9187 goes on, on the next free port, and the host keeps that port as today.
A port taken between the probe and the start still fails at "Wait for ahpd".

## Options

- **Refuse with a clear message.** Lost: the person has to find and type a free port that the probe already knows.
