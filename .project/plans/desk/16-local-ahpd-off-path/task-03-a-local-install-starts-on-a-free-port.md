---
title: A local install starts ahpd on the first free port
status: todo
depends: []
layer: "src-tauri, packages/shell"
refs:
  - "[code://src-tauri/src/local_ahpd.rs#L555-L587](../../../../src-tauri/src/local_ahpd.rs#L555-L587) - `start`"
  - "[code://src-tauri/src/local_ahpd.rs#L790-L803](../../../../src-tauri/src/local_ahpd.rs#L790-L803) - `start_host`"
  - "[code://src-tauri/src/hosts.rs#L494-L504](../../../../src-tauri/src/hosts.rs#L494-L504) - `HostStore::local`"
  - "[code://src-tauri/src/ssh_install.rs#L208-L213](../../../../src-tauri/src/ssh_install.rs#L208-L213) - `first_free_port`"
  - "[code://packages/shell/src/hosts/LocalForm.tsx#L150-L153](../../../../packages/shell/src/hosts/LocalForm.tsx#L150-L153) - the host saved on 9187"
---

## Objective

With no ahpd found, Add host saves the host on the first port from 9187 up that nothing on `127.0.0.1` listens on, says why when it is not 9187, and every start of that host's ahpd passes that port (decision 3).

## Files

- `UPDATE: src-tauri/src/local_ahpd.rs` - `local_free_port`, `start`'s `port` and `start_host`'s port.
- `UPDATE: src-tauri/src/hosts.rs:494-504` - `HostStore::local` also answers the port of the host's `url`.
- `UPDATE: src-tauri/src/lib.rs` - `local_free_port` in the invoke handler, beside `find_local_ahpd`.
- `UPDATE: packages/shell/src/hosts/ipc.ts` - `localFreePort(): Promise<number>`.
- `UPDATE: packages/shell/src/hosts/LocalForm.tsx` - the port the host is saved on, and the note.
- `UPDATE: packages/shell/src/i18n.ts` - `local.portMoved` in `en`, `ptBR` and `es`.

## Steps

1. Add `#[tauri::command] pub fn local_free_port() -> u16`: the first port from `ssh_install::AHPD_PORT` up for which `std::net::TcpListener::bind(("127.0.0.1", port))` succeeds; the listener drops at once.
2. `HostStore::local` answers `(Option<String>, bool, u16)`, the third the port of `url`, 9187 when `url` has none; its callers take the third where they need it.
3. `start` takes `port: Option<u16>` and adds `--port <port>` after `start` when it is `Some`; `start_host` passes `Some(port)` only when the port is not 9187.
4. `LocalForm`: when `found` is `null`, it asks `localFreePort()` with the find; the host is saved on `ws://127.0.0.1:<port>`; when the port is not 9187, the form shows `local.portMoved` under the ahpd line.
5. `en`: `'local.portMoved': 'Port 9187 is in use on this computer, so ahpd will listen on {port}.'`; `ptBR`: `'A porta 9187 está em uso neste computador, então o ahpd vai escutar na {port}.'`; `es`: `'El puerto 9187 está en uso en este equipo, así que ahpd escuchará en el {port}.'`.

## Validation

- A Rust test: with a listener held on 127.0.0.1:9187 in the test, `local_free_port()` answers a port above 9187; skipped when 9187 cannot be bound by the test itself.
- A Rust test: `HostStore::local` answers 9324 for a host on `ws://127.0.0.1:9324` and 9187 for one with no port.
- `pnpm typecheck`, `pnpm test` and `cargo clippy --all-targets -- -D warnings` pass.

## Resume

