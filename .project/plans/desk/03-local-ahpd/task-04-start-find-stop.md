---
title: The desk starts, finds and stops the local ahpd through its own commands
status: done
depends: [task-02-find-ahpd.md]
layer: "src-tauri"
refs:
  - file:///github/ahpd/packages/server/src/daemon.ts - daemon.json and its `connectUrl`
---

## Objective

The desk runs `ahpd status --json`, `ahpd start` and `ahpd stop` (decision 4), and takes the address and the token from `daemon.json`.

## Files

- `UPDATE: src-tauri/src/local_ahpd.rs` - `status()`, `start()`, `stop()`, and the read of `daemon.json`.
- `UPDATE: src-tauri/src/hosts.rs` - `startedByDesk`, `HostStore::local()` and `set_local()`.
- `UPDATE: src-tauri/src/lib.rs`, `src-tauri/build.rs`, `src-tauri/capabilities/shell.json` - `local_ahpd_status`, `start_local_ahpd`, `stop_local_ahpd`.
- `UPDATE: packages/shell/src/hosts/ipc.ts` - `localAhpdStatus`, `startLocalAhpd`, `stopLocalAhpd`.

## Steps

1. `start()` passes `--connection-token-file <app local data>/ahpd/token` (decision 5) and `--port`; on a failed start, the tail of `daemon.log` is the error.
2. `status()` reads `{"running":false}` as off, and the record's `connectUrl` for the token.
3. `stop()` runs `ahpd stop` only for an ahpd the desk started.
4. For an ahpd the desk installed, each command runs with `AHPD_PLUGIN_ROOT` set (decision 7).

## Validation

- Over a throwaway config folder: start, status and stop; a found ahpd is never stopped.
- The stop is graceful on Windows, which needs the ahpd change found in task 01.

## Resume

Done 2026-10-14.

- `status()`, `start()` and `stop()` run the found ahpd; the token is the `tkn` of `daemon.json`'s `connectUrl`, read only when the record names the same pid.
- `start()` runs in the home folder, which ahpd serves when its config names no folder (decision 10), with `--connection-token-file <app local data>/ahpd/connection-token`; it makes that folder first, since ahpd writes the file but not its folder.
  A failed start answers ahpd's words and the last lines of `daemon.log`.
- The commands save the address and the token on the host and send `hosts-changed`; `startedByDesk` is set by a start the desk made, and a save from the form keeps it.
- `stop_local_ahpd` stops only an ahpd the desk started; for one it found on, it refuses, and for one already off it answers off.
- Departure: no `--port`; the port is the one in ahpd's config, else ahpd's 9187, as `ahpd start` in a terminal uses.
- Verified: unit tests (status on, off and broken through a fake ahpd; the record's token; the config folder); the ignored `local_ahpd_a_real_start_and_stop` on ahpd main 69ed7790; and the debug desk over CDP on a scratch host list: start, status, the token equal to the file's, stop, a second stop, and an ahpd started from a terminal taken but not stopped.
- The stop is graceful on ahpd main (`ahpd stop --json` answers `"killed": false`); on 0.11.0 it is still a hard kill on Windows.
