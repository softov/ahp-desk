---
title: The shell keeps a host list with each token in the OS keyring
status: todo
depends: [task-01-workspace-scaffold.md]
layer: "src-tauri"
refs:
  - file:///github/opendoop/packages/tauri/src-tauri/src/profile.rs - the atomic write at lines 230-242, and the path at lines 159-162
  - https://crates.io/crates/keyring - `Entry::new`, `set_password`, `get_password`, `delete_credential`, and `mock` for tests
---

## Objective

The commands `hosts`, `save_host`, `remove_host` and `host_token` work from the shell window.
`hosts.json` holds the hosts with no token, and the keyring holds each token.
Every save or remove emits `hosts-changed` with the list.

## Files

- `CREATE: src-tauri/src/hosts.rs` - `Host`, `HostKind`, `HostStore`, the keyring calls, the four commands and their tests.
- `UPDATE: src-tauri/src/lib.rs` - manage `HostStore`, register the four commands.
- `UPDATE: src-tauri/Cargo.toml` - add `keyring = "3"` with the features `windows-native`, `apple-native` and `sync-secret-service`.
- `UPDATE: src-tauri/capabilities/shell.json` - allow the four commands for the `shell` window.

## Steps

1. Define `Host` as `{ id, name, kind, url }` with serde `camelCase`, and `HostKind` as `url` or `local`.
2. Define `HostStore` with a `Mutex<Vec<Host>>` and the path `app_config_dir()/hosts.json`.
3. Load `hosts.json` in setup; treat a missing file as an empty list.
4. Write the file to `hosts.json.tmp` and rename it over `hosts.json`.
5. Write `hosts()`, which returns the list.
6. Write `save_host(host, token: Option<String>)`, which adds or replaces the host by id.
7. In `save_host`, set the token with `Entry::new("ahp-desk", &host.id)` when `token` is `Some`.
8. In `save_host`, delete the token when `token` is `Some("")`.
9. Write `remove_host(id)`, which removes the host and deletes its token.
10. Write `host_token(id)`, which returns the token or `None`.
11. Emit `hosts-changed` with the list after `save_host` and `remove_host`.
12. Return a keyring error as the text "The OS keyring is not available: <reason>".
13. Write the tests with the `keyring` mock: save, replace, remove, token round trip, and a missing file.

## Validation

- Run `cargo test --manifest-path src-tauri/Cargo.toml hosts`; every case passes.
- In `pnpm tauri dev`, call `save_host` from the devtools; `hosts.json` has the host and no token.
- Open the Windows Credential Manager; it lists the entry `ahp-desk` for the host id.

## Resume
