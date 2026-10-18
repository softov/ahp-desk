---
title: The desk finds an ahpd on PATH or at a set path
status: done
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

Done 2026-10-13.

- `find()` takes the path set on the host, else `LocalAhpd::installed()`, else the first `ahpd.exe` or `ahpd.cmd` (`ahpd` elsewhere) on `PATH`; a set path that is gone is an error, never a reason to use another.
- `OLDEST_SUPPORTED` is 0.8.0, the first release with `status --json` (ahpd aecb628f); an older ahpd is found with `supported: false`.
- The installed ahpd is run as `<bundled node> <prefix>/node_modules/@ahpd/server/<bin>`, not through npm's shim, which looks for `node` on `PATH`.
  `LocalAhpd.node` is `None` until task 03 ships Node, so until then nothing counts as installed.
- Every ahpd command runs with `CREATE_NO_WINDOW` on Windows, so `ahpd.cmd` flashes no console.
- `findLocalAhpd(ahpdPath)` in `packages/shell/src/hosts/ipc.ts` calls it; `ahpdPath` is on the host in `hosts.json`.
- On this machine it finds the global ahpd 0.7.0 on `PATH`, unsupported.
- Not decided: what Add host does with an ahpd on `PATH` that is too old; task 06 needs it.
