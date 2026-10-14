---
title: A server's key is checked against known_hosts and the desk's confirmed keys; a new one is refused until Trust, and a changed one refused with Forget
status: todo
depends: [task-01-ssh-session.md]
layer: "src-tauri"
refs:
  - file:///github/opendoop/packages/tauri/src-tauri/src/ssh/mod.rs - `AcceptAllHandler`, at lines 33-49, which trusts any key; decision 2 replaces it and it is not copied
  - "[code://src-tauri/src/lib.rs#L31-L37](../../../../src-tauri/src/lib.rs#L31-L37) - the config folder, `AHP_DESK_CONFIG_DIR` or the app's, where `known_keys.json` sits beside `hosts.json`"
  - "[code://src-tauri/src/hosts.rs#L311-L321](../../../../src-tauri/src/hosts.rs#L311-L321) - `write`, the temporary file and rename that `known_keys.json` is written with too"
---

## Objective

Every SSH connection checks the server's key, as decision 2 says: a key in `~/.ssh/known_hosts` for that host, or one the person confirmed before, passes.
A new one refuses the connection with its fingerprint, and once the person trusts it the desk connects again (decision 9).
A different one is refused with both fingerprints, and the refusal can forget the desk's old key (decision 10).

## Files

- `CREATE: src-tauri/src/ssh/known_keys.rs` - `KnownKeys` (load, `check`, `confirm`, `forget`), `KeyCheck`, `ServerKeyHandler`, `fingerprint()`.
- `UPDATE: src-tauri/src/ssh/mod.rs` - `connect()` takes the `KnownKeys` and answers `ConnectError::NewKey` or `ConnectError::ChangedKey`.
- `UPDATE: src-tauri/src/lib.rs` - `KnownKeys::load(config.join("known_keys.json"))` managed; the commands `confirm_ssh_key` and `forget_ssh_key`.
- `UPDATE: src-tauri/build.rs`, `src-tauri/capabilities/shell.json` - the two commands, granted to the shell window.
- `UPDATE: packages/shell/src/hosts/ipc.ts` - `confirmSshKey`, `forgetSshKey`, and the `NewKey` and `ChangedKey` errors as the shell reads them.

## Steps

1. `fingerprint(key)` is `SHA256:<base64, no padding>`, as `ssh-keygen -l` and OpenSSH's own question print it, with the key type beside it (`ED25519`, `RSA`, ...).
2. `KnownKeys` is `known_keys.json` in the desk's config folder: one entry per `host:port` with the key type, the key in OpenSSH form, the fingerprint and the date confirmed.
3. `KnownKeys::check(address, port, key)` answers `KeyCheck::Known` when `~/.ssh/known_hosts` (`%USERPROFILE%\.ssh\known_hosts` on Windows) holds that key for the host, read with `russh`'s known_hosts functions (check their 0.61 names first), or when `known_keys.json` holds it.
   It answers `KeyCheck::Changed { known, offered }` when either holds a different key for the host, and `KeyCheck::New { offered }` when neither holds one.
4. `ServerKeyHandler`, the `client::Handler` that task 01's `connect()` uses, answers `check_server_key` from `KnownKeys::check` at once, never waiting on the person: `Known` passes; `New` and `Changed` refuse, and the handler keeps what it saw so `connect()` answers `ConnectError::NewKey { fingerprint, key }` or `ConnectError::ChangedKey { known, offered, inKnownHosts }`.
   The refusal comes before any sign-in, so nothing is asked of the person on a refused connection (decision 9).
5. `KnownKeys` holds the last key refused per `host:port`, in memory; `confirm_ssh_key(address, port, fingerprint)` records that key, only when its fingerprint equals the one the person was shown.
   The shell then connects again, and that connection passes as `Known` (decision 9).
6. `forget_ssh_key(address, port)` removes the desk's entry for that server, so the next connection answers `NewKey` and asks to trust the new key as a first one (decision 10).
   It never edits `~/.ssh/known_hosts`: `ChangedKey` carries `inKnownHosts: true` when the old key came from there, and the refusal then names `ssh-keygen -R <host>` instead of a Forget button.
7. The words for the confirmation and the refusal, with its "Forget the old key" button, are task 07's; this task answers the errors with both fingerprints and where the old key came from.

## Validation

- Cargo tests in `known_keys.rs`: a key in a scratch `known_hosts` passes; a key in `known_keys.json` passes; a new key answers `New`; a different key in either answers `Changed` with both fingerprints, and says which file held the old one; `confirm` with a fingerprint that is not the one seen is refused; `forget` makes the next check `New`; a missing `known_hosts` is not an error.
- Cargo tests against task 01's `test_server.rs`: the first connection answers `NewKey` and the server sees no sign-in attempt; after `confirm_ssh_key` a second connection signs in; with the server restarted on another host key it answers `ChangedKey` with both fingerprints; after `forget_ssh_key` the next connection answers `NewKey`, and after Trust it signs in.
- `pnpm test` and `pnpm lint` pass.

## Resume
