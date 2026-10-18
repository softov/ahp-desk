---
title: An SSH-installed ahpd can be updated over the same SSH access, for plan 09's Update
status: done
depends: [task-05-install.md]
layer: "src-tauri"
refs:
  - "[code://src-tauri/src/hosts.rs#L25-L45](../../../../src-tauri/src/hosts.rs#L25-L45) - `Host`, whose `ssh` details say the desk installed it over SSH"
---

## Objective

`ssh_install::update(app, id, version, events)` installs `@ahpd/server@<version>` into the host's `~/.local/share/ahpd` over SSH, restarts `ahpd.service`, waits until it answers, and answers the version now running, streaming its steps as the install does.
It works for a host whose `ssh` details say the desk installed its ahpd (`byDesk`, Answers: a found ahpd), of kind `ssh` or `url`; an ahpd found on the server and not installed by the desk is updated through ahpd, as plan 09's decision says for any other host.
[plan 09](../09-update-alerts/plan.md)'s task 05 makes it the command `update_ssh_ahpd` behind the row's Update, as its decision A host's ahpd update is shown on its row and offered, over SSH or through ahpd says: the newer `@ahpd/server` is installed and the service restarted.
This task adds no command and no shell call; plan 09's task 05 does.

## Files

- `UPDATE: src-tauri/src/ssh_install.rs` - `update()`, which reuses `install()`'s `connect`, `server`, `start` and `ready` steps; `install()`'s `server` step takes the `@ahpd/server` version, `latest` for an install.

## Steps

1. `update()` reads the host's `SshHost` from `HostStore::ssh(id)`, connects with its kept sign-in or what `Secrets` holds (task 01) and the key check (task 02), and refuses a host with no `ssh` details, or with `byDesk` false, before any connection.
2. It finds the install the desk made, `<home>/.local/share/ahpd/node_modules/@ahpd/server`, and the Node the unit names in `ExecStart`, and refuses with what it found when either is missing.
3. Steps `connect`, `server` (`npm install --prefix <prefix> @ahpd/server@<version>`), `start` (`systemctl --user restart ahpd.service`), `ready` on the host's `remotePort`, from task 05, with the same cancel; the unit, its ahpd port, the token file, the folder and the backends under `AHPD_PLUGIN_ROOT` are left as they are, so the host's token and address still hold.
4. It answers the version `--version` prints after the restart; the host list hears the new version from `initialize` when the host reconnects.
5. Telling which sessions are running before the restart is plan 09's task 02, not this one.

## Validation

- Cargo test against task 01's `test_server.rs`: the commands in order, with the version asked; a host with no `ssh` details refused; a host with `byDesk` false refused with no connection; a server with no desk install refused.
- `pnpm test` and `pnpm lint` pass.
- By hand on the test machine only: the ignored test `ssh_a_real_update` installs a pinned older `@ahpd/server`, updates it to the newest, and checks the newer version runs, the service is active, and the token file is unchanged; it removes what it installed at the end.

## Resume

- 2026-10-18: `src-tauri/src/ssh_install.rs` has `update(target, check, keys, version, port, send, cancel)`, which answers the version now running, and `update_host(store, known, id, version, send, cancel)`, which signs in as the host does and refuses a host with no `ssh` details or with `byDesk` false before any connection; plan 09's task 05 makes `update_host` the command `update_ssh_ahpd`, with its `Channel` and `InstallControl`.
- The steps are `connect`, `probe`, `server`, `start` and `ready`: the probe finds the desk's install and the Node its unit's `ExecStart` names, and refuses an ahpd the desk did not install, or none; npm is the one next to that Node.
- `server_step()` is the install's and the update's `server` step, `latest` for an install; a version other than an npm word (letters, digits, `.`, `-`, `+`) is refused before connecting.
- Verified: `cargo test` (122: the update's commands in order with the version asked; no desk install; an ahpd the desk did not install; a version npm would not read; `update_host`'s two refusals with no connection), `cargo clippy --all-targets -- -D warnings`, `cargo fmt`.
- By hand, `ssh_a_real_update` on the test machine Softov named, twice: installed the newest ahpd, updated it to 0.11.0 and back to the newest, each time active, on that version, with the token file's checksum unchanged; it removed what it installed after.
- `ssh_a_real_install` no longer prints the token file it compares, and `ssh_a_real_update` compares a checksum.
