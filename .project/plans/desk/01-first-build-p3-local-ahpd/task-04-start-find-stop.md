---
title: The desk starts, finds and stops the local ahpd through its own commands
status: todo
depends: [task-02-find-ahpd.md]
layer: "src-tauri"
refs:
  - file:///github/ahpd/packages/server/src/daemon.ts - daemon.json and its `connectUrl`
---

## Objective

The desk runs `ahpd status --json`, `ahpd start` and `ahpd stop` (decision 4), and takes the address and the token from `daemon.json`.

## Files

- `UPDATE: src-tauri/src/local_ahpd.rs` - `status()`, `start()`, `stop()`, and the read of `daemon.json`.
- `UPDATE: src-tauri/src/hosts.rs` - `startedByDesk`.

## Steps

1. `start()` passes `--connection-token-file <app local data>/ahpd/token` (decision 5) and `--port`; on a failed start, the tail of `daemon.log` is the error.
2. `status()` reads `{"running":false}` as off, and the record's `connectUrl` for the token.
3. `stop()` runs `ahpd stop` only for an ahpd the desk started.
4. For an ahpd the desk installed, each command runs with `AHPD_PLUGIN_ROOT` set (decision 7).

## Validation

- Over a throwaway config folder: start, status and stop; a found ahpd is never stopped.
- The stop is graceful on Windows, which needs the ahpd change found in task 01.

## Resume
