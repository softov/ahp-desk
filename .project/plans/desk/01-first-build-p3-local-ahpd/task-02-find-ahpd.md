---
title: The desk finds an ahpd on PATH or at a set path
status: todo
depends: [task-01-check-ahpd-commands.md]
layer: "src-tauri"
refs:
  - file:///github/opendoop/packages/tauri/src-tauri/src/pood_process.rs - how doop finds Node and its daemon
  - "[code://src-tauri/src/hosts.rs](../../../../src-tauri/src/hosts.rs) - `HostKind`, which gains `Local`"
---

## Objective

`local_ahpd::find()` returns the ahpd the desk uses: the one set on the host, else the one it installed, else one on `PATH`, with its version.

## Files

- `CREATE: src-tauri/src/local_ahpd.rs` - `find()`, and the read of `ahpd --version`.
- `UPDATE: src-tauri/src/hosts.rs` - `HostKind::Local`, and `ahpdPath` on the host.
- `UPDATE: src-tauri/src/lib.rs` - the module, and a `find_local_ahpd` command.

## Steps

1. Look up `ahpd` (`ahpd.cmd` on Windows) on `PATH`, as decision 1 says.
2. Read its version with `ahpd --version`; an ahpd older than the oldest the desk supports is reported, not used.
3. Keep whether the desk installed it, for decision 6.

## Validation

- Cargo tests: the set path wins, then the installed one, then `PATH`; and none found.

## Resume
