---
title: The host list window publishes each host's status, and reaches older hosts
status: done
depends: [task-03-contract-frame-and-statuses.md]
layer: "src-tauri, packages/shell"
refs:
  - file:///github/ahp-desk/packages/shell/src/hosts/status.ts - the MultiHostClient over every host
  - file:///github/ahpw/src/connection/protocol-versions.ts - `offering()`: rewrites `initialize` to offer every supported version
  - file:///github/ahp-desk/.project/decisions/the-host-list-window-publishes-host-status.md - decision 3
---

## Objective

Every window can read every host's status, and the status follows each change.
A host that speaks protocol 0.9.0 shows connected in the host list.

## Files

- `CREATE: src-tauri/src/statuses.rs` - `HostStatuses` (a `Mutex<HashMap<String, String>>`), the commands `publish_host_statuses` and `host_statuses`, and their tests.
- `UPDATE: src-tauri/src/lib.rs` - manage `HostStatuses`, register the two commands.
- `UPDATE: src-tauri/build.rs` - the two commands in the app manifest.
- `UPDATE: src-tauri/capabilities/shell.json` - allow `publish_host_statuses` and `host_statuses`.
- `UPDATE: src-tauri/capabilities/ui.json` - allow `host_statuses`.
- `UPDATE: packages/shell/src/hosts/status.ts` - publish the map on each change; wrap the transport with an `offering` copy.
- `UPDATE: src-tauri/src/hosts.rs` - `remove_host` drops the host from `HostStatuses`.

## Steps

1. In `statuses.rs`, `publish_host_statuses(map)` keeps the map and emits `hosts-status` with it, only from the `shell` window.
2. `host_statuses()` returns the last map.
3. In `status.ts`, compute the map of every host's `HostStatus['kind']` after each change, and call `publish_host_statuses` when it differs from the last one sent.
4. Copy ahpw's `offering()` into `status.ts` with a comment naming its source, and wrap the transport in it, so `initialize` offers every `SUPPORTED_PROTOCOL_VERSIONS` entry.
5. Use the same wrapper in `testHost`.

## Validation

- Run `cargo test statuses`; publish then read returns the map, and a remove drops its host.
- With the desk running, `invoke('host_statuses')` in an app window returns every host's status.
- Add an ahpd 0.7.0 (protocol 0.9.0) as a host; it shows connected.

## Resume

- Done 2026-10-11. `statuses.rs` with 2 tests; the shell publishes only a map that differs from the last. The `offering` copy also offers 0.10.0, VS Code 1.141's version, as ahpw does. An ahpd 0.7.0 on protocol 0.9.0 shows connected as "ahpd 0.7.0".
