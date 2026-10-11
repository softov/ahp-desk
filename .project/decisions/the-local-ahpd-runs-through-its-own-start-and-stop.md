---
title: The local ahpd runs through its own start and stop
status: accepted
date: 2026-10-11
refs:
  - file:///github/ahpd/packages/server/src/daemon.ts - `ahpd start`, `stop` and `status`, with daemon.json and its connectUrl
  - file:///github/ahpd/packages/server/src/commands/run.ts - the line "ahpd on ws://..." on stdout, and SIGTERM for a graceful stop
  - file:///github/opendoop/packages/tauri/src-tauri/src/pood_process.rs - doop spawns its daemon as a child and never stops it
---

## Context

ahpd has `ahpd start`, which runs it in the background and writes `daemon.json`.
`ahpd stop` stops it with a graceful signal, and `ahpd status --json` reports it.
On Windows, a kill from the shell is not graceful.

## Decision

The shell runs `ahpd status --json` to find a local ahpd that is on.
It runs `ahpd start` to start one, and `ahpd stop` to stop one.
It takes the address and the token from `daemon.json`.
Source: Softov, asked "The shell drives the local ahpd through ahpd start, stop and status --json. Keep it?": "Keep (Recommended)".

## Consequences

An ahpd that the shell started outlives a crash of the shell, and the next start finds it.
The shell depends on the output of `ahpd status --json` as a contract.
The plan for the local ahpd checks that `ahpd start` takes the flags that the shell needs.

## Options

- **A child process, as doop does.** Lost: no graceful stop on Windows, and a crash of the shell leaves an ahpd that no one knows.
