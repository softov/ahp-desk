---
title: The desk opens an SSH session with the agent, a key file with its passphrase, or a password, and holds what was typed until it quits
status: todo
depends: []
layer: "src-tauri, packages/shell"
refs:
  - file:///github/opendoop/packages/tauri/src-tauri/src/ssh/mod.rs - doop's `open_session`, `authenticate` (agent, then key file), `try_agent_auth` (OpenSSH pipe, then Pageant), `exec_capture`, `expand_remote_path`; its `AcceptAllHandler` is not copied
  - file:///github/opendoop/packages/tauri/src-tauri/src/ssh_probe.rs - `shell_escape_basic`, the quoting every remote command uses
  - file:///github/opendoop/packages/tauri/Cargo.toml - `russh = "0.61"`, the version doop builds with
  - "[code://src-tauri/Cargo.toml](../../../../src-tauri/Cargo.toml) - gains `russh`"
  - "[code://src-tauri/src/tokens.rs#L7-L43](../../../../src-tauri/src/tokens.rs#L7-L43) - `Vault` and `Keyring`, where a kept password goes"
  - "[code://src-tauri/src/tokens.rs#L45-L79](../../../../src-tauri/src/tokens.rs#L45-L79) - `set`, `get` and `move_to_vault`, the fallback to `hosts.json` that a kept password follows when there is no keyring"
  - "[code://src-tauri/src/hosts.rs#L109-L135](../../../../src-tauri/src/hosts.rs#L109-L135) - `HostView.token_in_file`, the flag that a password in `hosts.json` gets beside it"
  - "[code://packages/shell/src/hosts/HostList.tsx#L116-L118](../../../../packages/shell/src/hosts/HostList.tsx#L116-L118) - the row's warning on a token in the plain file, which a password in it shows too"
  - "[code://src-tauri/src/hosts.rs#L25-L45](../../../../src-tauri/src/hosts.rs#L25-L45) - `Host`, which gains its SSH details"
  - "[code://src-tauri/src/hosts.rs#L231-L242](../../../../src-tauri/src/hosts.rs#L231-L242) - `HostStore::remove`, which also removes a kept password"
  - "[code://src-tauri/src/local_ahpd.rs#L269-L306](../../../../src-tauri/src/local_ahpd.rs#L269-L306) - `run_streaming`, the line-by-line pattern the remote exec follows"
---

## Objective

`ssh::connect()` opens a `russh` session to a server and signs in, trying the SSH agent, then a key file (asking its passphrase when it has one), then a password, as decision 3 says.
A password the person asks to keep is in the OS keyring, or, on a machine with no keyring and only when the person ticked the box that says so, in `hosts.json` (decision 11); it is removed with its host.
A passphrase, or a password not kept, that the person typed is held in memory until the desk quits, never on disk (decision 8).
`exec_capture()` and `exec_lines()` run a command on the server, the second sending each line as it comes.

## Files

- `UPDATE: src-tauri/Cargo.toml` - `russh = "0.61"`, as doop.
- `CREATE: src-tauri/src/ssh/mod.rs` - `SshTarget`, `SignIn`, `Secrets`, `connect()`, `sign_in()`, `try_agent()`, `exec_capture()`, `exec_lines()`, `remote_home()`, `quote()`.
- `CREATE: src-tauri/src/ssh/password.rs` - `keyring_works()`, `keep()`, `kept()`, `forget()`, `move_to_vault()`: the password in the vault under `ssh-password:<host id>`, or in the host's `ssh.password` when there is no keyring; and the command `ssh_keyring_works`.
- `CREATE: src-tauri/src/ssh/test_server.rs` - `#[cfg(test)]` only: an in-process `russh::server` on `127.0.0.1:0` with a generated host key, one password and one public key it accepts, and canned answers to `exec`.
- `UPDATE: src-tauri/src/hosts.rs` - `SshHost` (`address`, `port`, `user`, `signIn`, `keyFile`, and `password` only on a machine with no keyring) as `Host.ssh` and `HostInput.ssh`, absent on a host with none; `HostView.ssh` without any secret, and `HostView.ssh_password_in_file`; `HostStore::ssh(id)` answers a host's SSH details; `HostStore::remove` also calls `ssh::password::forget` and drops the host's `Secrets`; the load moves a password in the file to the vault when the vault works, as it does a token.
- `UPDATE: src-tauri/src/lib.rs` - `mod ssh;`, `ssh::Secrets` managed, and `ssh_keyring_works`.
- `UPDATE: src-tauri/build.rs`, `src-tauri/capabilities/shell.json` - `ssh_keyring_works`, granted to the shell window.
- `UPDATE: packages/shell/src/hosts/ipc.ts` - `SshHost` on `Host`, `sshPasswordInFile` on the host view, and `sshKeyringWorks()`.
- `UPDATE: packages/shell/src/hosts/HostList.tsx` - a host with `sshPasswordInFile` shows the row's plain-file warning, as a token in the file does.
- `UPDATE: packages/shell/src/i18n.ts` - `hosts.passwordInFile`, in en, pt-BR and es.

## Steps

1. `SshTarget { address, port, user, key_file: Option<PathBuf> }` names the server; `port` is 22 unless the address says `host:port`.
2. `connect(target, check, secrets)` builds the `client::Config` with `keepalive_interval` set, so a tunnel (task 06) notices a dead link, and hands the server key to `check`, the handler task 02 writes; until then `check` is a parameter the tests fill, never a handler that accepts any key (decision 2).
3. `sign_in()` tries, in order, as decision 3 says: the agent (`try_agent()`: `$SSH_AUTH_SOCK` on Unix; the OpenSSH pipe `\\.\pipe\openssh-ssh-agent`, then Pageant, on Windows, as doop's `try_agent_auth`); the key file, loaded with `load_secret_key(path, None)`, and on an encrypted key `Err(SignInError::Passphrase)` so the form asks it; the password, from `Secrets` or from `password::kept(id)`.
   It answers `SignIn::Agent`, `SignIn::KeyFile` or `SignIn::Password`, which the host keeps as `signIn`, and fails naming each way it tried.
   When the host's sign-in needs a passphrase or a password that neither `Secrets` nor the kept password holds, it fails with `SignInError::NeedsPerson { what }`, `what` being `passphrase` or `password`, which task 06 shows as "Sign in" on a tunnelled host's row (decision 8).
4. `Secrets` is a Tauri state holding, per host id, for this process only, what the person typed: a passphrase, and a password not kept (decision 8).
   It keeps them until the desk quits, so a tunnel that drops opens again with no question.
   It never writes them to disk, never puts them in an event or a log line, and drops a host's entry when the host is removed.
   A typed passphrase that does not decrypt the key, or a password the server refuses, is dropped from `Secrets` at once, so the next attempt asks again.
5. `password::keyring_works(vault)` answers whether the vault can be used: `vault.get("ssh-keyring-check")` answers `Ok`, with or without an entry.
   The command `ssh_keyring_works` gives it to the form before the save, so the checkbox says where the password will go (decision 11, task 07).
   `password::keep(vault, host, password)` writes it to the `Keyring` (`tokens.rs`) under `ssh-password:<id>`, beside the host's token under `<id>`, and clears `host.ssh.password`.
   When the vault fails, it writes it to `host.ssh.password` in `hosts.json`, as `tokens::set` does a token; the form calls it only when the person ticked the checkbox that names the plain file (decision 11).
   `kept` reads the vault first, then `host.ssh.password`; `forget` deletes both, and deleting what is not there is not an error; `move_to_vault` moves a password in the file into the vault when the vault works, as `tokens::move_to_vault` does.
6. `exec_capture(session, command)` answers the exit code and stdout and stderr together, as doop's.
   `exec_lines(session, command, line, cancel)` sends each line of both as it comes, as `run_streaming` does for the local install, and fails with the exit code; `cancel` is the `AtomicBool` task 05 raises.
7. `remote_home(session)` reads `$HOME` once with `printf %s "$HOME"`; every remote path is made absolute with it, as doop's `expand_remote_path`, because a quoted `~` is not expanded.
8. `quote(s)` is doop's `shell_escape_basic`; every path and name in a remote command goes through it.

## Validation

- Cargo tests in `src-tauri/src/ssh/mod.rs` against `test_server.rs`: a password signs in and answers `SignIn::Password`; a wrong password fails naming the ways tried; an unencrypted key file signs in; an encrypted key file without its passphrase answers `SignInError::Passphrase`, and with it signs in; `exec_capture` answers the exit code and both streams; `exec_lines` sends lines in order and fails with a non-zero exit.
- Cargo tests in `src-tauri/src/ssh/mod.rs`: a typed passphrase held in `Secrets` signs in again on a second `connect()` with no question; with `Secrets` empty, as after a restart, the same host answers `SignInError::NeedsPerson`; a refused password is dropped from `Secrets`.
- Cargo tests in `src-tauri/src/ssh/password.rs` with `tokens::test_vaults::Memory` and a vault that always fails: kept in the vault and not in the host; with the failing vault, kept in `host.ssh.password`; read from either; forgotten from both; moved to the vault when it works; `keyring_works` answers false on the failing vault; `HostStore::remove` forgets it and drops the host's `Secrets`.
- Cargo tests in `hosts.rs`: a host with `ssh` round-trips through `hosts.json`; a password is written there only by `keep` on a failing vault, and then `ssh_password_in_file` is true; a passphrase is never written there.
- `pnpm test` and `pnpm lint` pass.
- By hand, from this Windows machine to dev02 only: the ignored test `ssh_a_real_sign_in`, with `AHP_DESK_SSH_TEST=<user>@dev02`, signs in with the OpenSSH agent.
  Its `check` accepts only the fingerprint in `AHP_DESK_SSH_TEST_KEY`, taken from `ssh-keygen -lf` on dev02's host key; the ignored tests of tasks 03 to 05 and 08 use the same check.

## Resume
