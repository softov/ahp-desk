---
title: The desk opens an SSH session with the agent, a key file with its passphrase, or a password kept in the keyring
status: todo
depends: []
layer: "src-tauri"
refs:
  - file:///github/opendoop/packages/tauri/src-tauri/src/ssh/mod.rs - doop's `open_session`, `authenticate` (agent, then key file), `try_agent_auth` (OpenSSH pipe, then Pageant), `exec_capture`, `expand_remote_path`; its `AcceptAllHandler` is not copied
  - file:///github/opendoop/packages/tauri/src-tauri/src/ssh_probe.rs - `shell_escape_basic`, the quoting every remote command uses
  - file:///github/opendoop/packages/tauri/Cargo.toml - `russh = "0.61"`, the version doop builds with
  - "[code://src-tauri/Cargo.toml](../../../../src-tauri/Cargo.toml) - gains `russh`"
  - "[code://src-tauri/src/tokens.rs#L7-L43](../../../../src-tauri/src/tokens.rs#L7-L43) - `Vault` and `Keyring`, where a kept password goes"
  - "[code://src-tauri/src/hosts.rs#L25-L45](../../../../src-tauri/src/hosts.rs#L25-L45) - `Host`, which gains its SSH details"
  - "[code://src-tauri/src/hosts.rs#L231-L242](../../../../src-tauri/src/hosts.rs#L231-L242) - `HostStore::remove`, which also removes a kept password"
  - "[code://src-tauri/src/local_ahpd.rs#L269-L306](../../../../src-tauri/src/local_ahpd.rs#L269-L306) - `run_streaming`, the line-by-line pattern the remote exec follows"
---

## Objective

`ssh::connect()` opens a `russh` session to a server and signs in, trying the SSH agent, then a key file (asking its passphrase when it has one), then a password, as decision 3 says.
A password the person asks to keep is in the OS keyring and is removed with its host.
`exec_capture()` and `exec_lines()` run a command on the server, the second sending each line as it comes.

## Files

- `UPDATE: src-tauri/Cargo.toml` - `russh = "0.61"`, as doop.
- `CREATE: src-tauri/src/ssh/mod.rs` - `SshTarget`, `SignIn`, `Secrets`, `connect()`, `sign_in()`, `try_agent()`, `exec_capture()`, `exec_lines()`, `remote_home()`, `quote()`.
- `CREATE: src-tauri/src/ssh/password.rs` - `keep()`, `kept()`, `forget()`: the password in the vault under `ssh-password:<host id>`.
- `CREATE: src-tauri/src/ssh/test_server.rs` - `#[cfg(test)]` only: an in-process `russh::server` on `127.0.0.1:0` with a generated host key, one password and one public key it accepts, and canned answers to `exec`.
- `UPDATE: src-tauri/src/hosts.rs` - `SshHost` (`address`, `port`, `user`, `signIn`, `keyFile`) as `Host.ssh` and `HostInput.ssh`, absent on a host with none; `HostView.ssh` without any secret; `HostStore::ssh(id)` answers a host's SSH details; `HostStore::remove` also calls `ssh::password::forget`.
- `UPDATE: src-tauri/src/lib.rs` - `mod ssh;`, and `ssh::Secrets` managed.
- `UPDATE: packages/shell/src/hosts/ipc.ts` - `SshHost` on `Host`.

## Steps

1. `SshTarget { address, port, user, key_file: Option<PathBuf> }` names the server; `port` is 22 unless the address says `host:port`.
2. `connect(target, check, secrets)` builds the `client::Config` with `keepalive_interval` set, so a tunnel (task 06) notices a dead link, and hands the server key to `check`, the handler task 02 writes; until then `check` is a parameter the tests fill, never a handler that accepts any key (decision 2).
3. `sign_in()` tries, in order, as decision 3 says: the agent (`try_agent()`: `$SSH_AUTH_SOCK` on Unix; the OpenSSH pipe `\\.\pipe\openssh-ssh-agent`, then Pageant, on Windows, as doop's `try_agent_auth`); the key file, loaded with `load_secret_key(path, None)`, and on an encrypted key `Err(SignInError::Passphrase)` so the form asks it; the password, from `Secrets` or from `password::kept(id)`.
   It answers `SignIn::Agent`, `SignIn::KeyFile` or `SignIn::Password`, which the host keeps as `signIn`, and fails naming each way it tried.
4. `Secrets` is a Tauri state holding, for this process only, what the form typed: a password not kept, and a passphrase.
   How long a passphrase-protected key stays usable is open question 1 in the plan; until it is answered, `Secrets` drops the passphrase once `sign_in()` returns.
5. `password::keep(vault, id, password)` writes it to the `Keyring` (`tokens.rs`) under `ssh-password:<id>`, beside the host's token under `<id>`; `forget` deletes it, and deleting what is not there is not an error.
   What happens when the keyring cannot keep it is open question 4 in the plan.
6. `exec_capture(session, command)` answers the exit code and stdout and stderr together, as doop's.
   `exec_lines(session, command, line, cancel)` sends each line of both as it comes, as `run_streaming` does for the local install, and fails with the exit code; `cancel` is the `AtomicBool` task 05 raises.
7. `remote_home(session)` reads `$HOME` once with `printf %s "$HOME"`; every remote path is made absolute with it, as doop's `expand_remote_path`, because a quoted `~` is not expanded.
8. `quote(s)` is doop's `shell_escape_basic`; every path and name in a remote command goes through it.

## Validation

- Cargo tests in `src-tauri/src/ssh/mod.rs` against `test_server.rs`: a password signs in and answers `SignIn::Password`; a wrong password fails naming the ways tried; an unencrypted key file signs in; an encrypted key file without its passphrase answers `SignInError::Passphrase`, and with it signs in; `exec_capture` answers the exit code and both streams; `exec_lines` sends lines in order and fails with a non-zero exit.
- Cargo tests in `src-tauri/src/ssh/password.rs` with `tokens::test_vaults::Memory`: kept, read, forgotten; `HostStore::remove` forgets it.
- Cargo tests in `hosts.rs`: a host with `ssh` round-trips through `hosts.json`, and no password is ever written there.
- `pnpm test` and `pnpm lint` pass.
- By hand, from this Windows machine to dev02 only: the ignored test `ssh_a_real_sign_in`, with `AHP_DESK_SSH_TEST=<user>@dev02`, signs in with the OpenSSH agent.
  Its `check` accepts only the fingerprint in `AHP_DESK_SSH_TEST_KEY`, taken from `ssh-keygen -lf` on dev02's host key; the ignored tests of tasks 03 to 05 and 08 use the same check.

## Resume
