---
title: The shell keeps a host list with each token in the OS keyring, or in hosts.json when there is no keyring
status: done
depends: [task-01-workspace-scaffold.md]
layer: "src-tauri"
refs:
  - file:///github/opendoop/packages/tauri/src-tauri/src/profile.rs - the atomic write at lines 230-242, and the path at lines 159-162
  - https://crates.io/crates/keyring - `Entry::new`, `set_password`, `get_password`, `delete_credential`, and `mock` for tests
---

## Objective

The commands `hosts`, `save_host`, `remove_host` and `host_token` work from the shell window.
The keyring holds each token.
When the keyring is not available, the token is in the host's `token` field in `hosts.json`, and the host carries a warning.
Every save or remove emits `hosts-changed` with the list.

## Files

- `CREATE: src-tauri/src/hosts.rs` - `Host`, `HostKind`, `HostStore`, the four commands and their tests.
- `CREATE: src-tauri/src/tokens.rs` - `TokenStore`: the keyring first, the `token` field in `hosts.json` as the fallback, and the move to the keyring.
- `UPDATE: src-tauri/src/lib.rs` - manage `HostStore`, register the four commands.
- `UPDATE: src-tauri/Cargo.toml` - add `keyring = "3"` with the features `windows-native`, `apple-native` and `sync-secret-service`.
- `UPDATE: src-tauri/capabilities/shell.json` - allow the four commands for the `shell` window.

## Steps

1. Define `Host` as `{ id, name, kind, url, token }` with serde `camelCase`, `token` optional, and `HostKind` as `url` or `local`.
2. Define `HostStore` with a `Mutex<Vec<Host>>` and the path `app_config_dir()/hosts.json`.
3. Load `hosts.json` in setup; treat a missing file as an empty list.
4. Write the file to `hosts.json.tmp` and rename it over `hosts.json`.
5. Write `hosts()`: return the list without `token`, and add `tokenInFile: true` where `token` has a value.
6. Write `save_host(host, token: Option<String>)`, which adds or replaces the host by id.
7. In `TokenStore::set`, write the token with `Entry::new("ahp-desk", &host.id)` and clear the host's `token` field.
8. When the keyring write fails, write the token to the host's `token` field instead.
9. In `TokenStore::get`, read the keyring first, then the host's `token` field.
10. If the keyring works and `token` has a value, move it to the keyring and clear `token`.
11. In `save_host`, call `TokenStore::set` when `token` is `Some`, and delete both copies when it is `Some("")`.
12. Write `remove_host(id)`, which removes the host and deletes the token from the keyring.
13. Write `host_token(id)`, which calls `TokenStore::get`.
14. Emit `hosts-changed` with the list after `save_host` and `remove_host`.
15. Write the tests with the `keyring` mock: save, replace, remove, read order, the fallback, the move, and a missing file.

## Validation

- Run `cargo test --manifest-path src-tauri/Cargo.toml hosts`; every case passes.
- In `pnpm tauri dev`, call `save_host` from the devtools; `hosts.json` has the host and no `token` field.
- Put a `token` field in `hosts.json` by hand and restart; `host_token` returns it, and the field moves to the keyring.
- Open the Windows Credential Manager; it lists the entry `ahp-desk` for the host id.

## Resume

- Done 2026-10-11. 9 `hosts` tests pass. In `pnpm tauri dev`, `hosts.json` holds no token, and the Windows Credential Manager lists `test.ahp-desk` and `other.ahp-desk`.
- The tests use a `Vault` trait in `tokens.rs` with an in-memory vault and a failing one, not the `keyring` mock: the mock keeps nothing between two `Entry` values, so it cannot test a save and a read.
- The move from `hosts.json` to the keyring happens when the list loads, and is covered by a test; it was not tried by hand.
