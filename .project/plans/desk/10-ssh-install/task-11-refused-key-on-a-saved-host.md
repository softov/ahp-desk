---
title: A saved tunnelled host whose server key is refused says so on its row, and Edit host answers the key
status: done
depends: [task-02-server-key.md, task-06-reach.md, task-07-add-host-ssh.md]
layer: "src-tauri, packages/shell"
refs:
  - "[code://src-tauri/src/ssh/tunnel.rs](../../../../src-tauri/src/ssh/tunnel.rs) - `TunnelState` and `supervise`, which keeps the refused key"
  - "[code://packages/shell/src/hosts/HostList.tsx](../../../../packages/shell/src/hosts/HostList.tsx) - the row's state words and its Check key"
  - "[code://packages/shell/src/hosts/HostForm.tsx](../../../../packages/shell/src/hosts/HostForm.tsx) - Edit host, which shows `ServerKey`"
  - "[code://packages/shell/src/hosts/ServerKey.tsx](../../../../packages/shell/src/hosts/ServerKey.tsx) - the key's question, as Add host shows it"
---

## Objective

A tunnelled host whose server key is refused, changed or never trusted, says so on its row and offers "Check key", which opens Edit host with the key's question; each answer opens the tunnel again ([decision](../../../decisions/a-saved-hosts-refused-server-key-is-answered-in-edit-host.md)).

## Files

- `UPDATE: src-tauri/src/ssh/tunnel.rs` - `TunnelState.key`, set by `supervise` from a `ConnectError::NewKey` or `ChangedKey`.
- `UPDATE: packages/shell/src/hosts/ipc.ts` - `RefusedKey`, moved from `ServerKey.tsx`, and `TunnelState.key`.
- `UPDATE: packages/shell/src/hosts/HostList.tsx` - "key changed" or "key not trusted", and Check key in place of Retry.
- `UPDATE: packages/shell/src/hosts/HostForm.tsx` - `ServerKey` for a tunnelled host whose tunnel refused its key; Trust or Forget, then `openSshTunnel`.
- `UPDATE: packages/shell/src/i18n.ts`, `packages/shell/src/app.css` - the words in en, pt-BR and es; the row's action column wide enough for Check key.

## Steps

1. `supervise` keeps the refused key in the state it sends, and clears it once the tunnel is on.
2. The row reads it; Retry is not offered for it, since asking again changes nothing.
3. Edit host shows the question while the key is refused; Trust confirms the offered fingerprint, Forget removes the desk's key, and each opens the tunnel, which asks again or opens.

## Validation

- Cargo test: a tunnel to a server whose key changed has `key` with the changed key; after Forget and a retry, the new key; after Trust and a retry, the tunnel is on with no key and forwards.
- `pnpm typecheck` and `pnpm test` pass.
- By hand on a release build over CDP, with the desk's entry for the server changed to another key: the row shows the changed key and Check key; Edit host shows both fingerprints; Forget leads to Trust; Trust clears the question and the tunnel goes on to its sign-in.

## Resume

- 2026-10-18: `TunnelState.key` (a `ConnectError`, new or changed key) is set by `supervise` and cleared when the tunnel is on; `RefusedKey` lives in `ipc.ts`.
- The row shows `tunnel.keyChanged` or `tunnel.keyNew` with Check key, which opens Edit host; Edit host shows `ServerKey` and opens the tunnel after Trust or Forget.
- The row's action column is 92px with no wrap, so Check key fits in en, pt-BR and es, at 480x300 too.
- Verified: `cargo test` (123, with `tunnel_a_changed_key_is_in_its_state_and_forget_then_trust_opens_it`), `cargo clippy --all-targets -- -D warnings`, `pnpm typecheck`, `pnpm test` (27).
- By hand on a release build over CDP with a scratch `AHP_DESK_CONFIG_DIR`, the desk's entry for the test machine changed to another key: the row read "server key changed" with Check key, Edit host showed both fingerprints, Forget led to Trust, and Trust cleared the question with the tunnel going on to ask the key file's passphrase; the row's words were then shortened to "key changed" and "key not trusted", which fit as "tunnel down" does.
