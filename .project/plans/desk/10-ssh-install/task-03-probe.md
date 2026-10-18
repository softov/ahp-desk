---
title: The desk probes a server for Linux, its CPU, systemctl --user, lingering, Node and an ahpd already there
status: done
depends: [task-01-ssh-session.md]
layer: "src-tauri"
refs:
  - file:///github/opendoop/packages/tauri/src-tauri/src/ssh_probe.rs - `parse_remote_os` and `fetch_os_pretty_name` (in `ssh/mod.rs`), the OS read this probe narrows to Linux
  - file:///github/opendoop/packages/tauri/src-tauri/src/installer/systemd_node.rs - `prereqs`, `systemctl --user --version`, at lines 92-108
  - "[code://src-tauri/src/local_ahpd.rs#L26](../../../../src-tauri/src/local_ahpd.rs#L26) - `OLDEST_SUPPORTED`, the pattern for a version floor read from a version line, and the floor a found ahpd on the server is held to"
  - "[code://src-tauri/src/local_ahpd.rs#L318](../../../../src-tauri/src/local_ahpd.rs#L318) - `installed_script`, the `bin` of an `@ahpd/server` under a prefix"
  - "[code://packages/shell/src/hosts/LocalForm.tsx#L46-L130](../../../../packages/shell/src/hosts/LocalForm.tsx#L46-L130) - what the form shows of a found ahpd: its origin, its version, `supported` and its ready plugins"
  - file:///github/ahpd/packages/server/src/commands/plugin.ts - `ahpd plugin list --json`, each plugin's name and state
  - file:///github/ahpd/packages/server/src/commands/status.ts - `ahpd status --json`, with pid, url and paths, or `{"running":false}`
---

## Objective

`ssh_install::probe()` tells whether the server already has an ahpd, which the desk then uses (Answers: a found ahpd), and, when it has none, whether the server can take ahpd as a user service (Answers: the user systemd service) and where its Node comes from (Answers: the remote Node); `probe_ssh_server` answers it to the shell.

## Files

- `CREATE: src-tauri/src/ssh_install.rs` - `Probe`, `Cpu`, `NodeOnServer`, `FoundOnServer`, `probe()`, `parse_probe()`, `node_version()`, `found_ahpd()`, and the command `probe_ssh_server`.
- `UPDATE: src-tauri/src/lib.rs` - `mod ssh_install;` and the command.
- `UPDATE: src-tauri/build.rs`, `src-tauri/capabilities/shell.json` - `probe_ssh_server`, granted to the shell window.
- `UPDATE: packages/shell/src/hosts/ipc.ts` - `probeSshServer(target)` and the `Probe` and `FoundOnServer` types.

## Steps

1. One `exec_capture` runs a POSIX `sh` script that prints `key=value` lines, so a server answers in one round trip and `parse_probe()` is a pure function: `os` (`uname -s`), `pretty` (`PRETTY_NAME` from `/etc/os-release`), `arch` (`uname -m`), `libc` (`musl` when `ldd --version` says so, else `glibc`), `home` (`$HOME`), `user` (`id -un`), `systemd` (the exit of `systemctl --user show-environment`), `linger` (`loginctl show-user "$(id -un)" -p Linger --value`), `node` (the absolute path of `node` on the server, if any), `nodeVersion` (`node -v`), `npm` (the `npm` beside that `node`), `fetch` (`curl` or `wget`, whichever is there), `tar` (whether `tar` is there), `ahpd` (the absolute path of an `ahpd` found, as step 7 says), `ahpdVersion` (its `--version`), `deskUnit` (whether `~/.config/systemd/user/ahpd.service` exists and its `Description` names ahp-desk).
2. A `systemctl --user` that fails with no user bus is asked again with `XDG_RUNTIME_DIR=/run/user/$(id -u)`; when that works, `Probe.runtime_dir` keeps it, and every later `systemctl --user` and `journalctl --user` (tasks 05 and 08) runs with it.
3. `node` is looked for on the session's `PATH`, then through the login shell (`"$SHELL" -lc 'command -v node'`), since a Node from a version manager is often only there; the unit (task 05) uses its absolute path, so the service never depends on a shell's `PATH`.
4. `Probe` refuses, each with its reason in words a person acts on: not Linux (Answers: the user systemd service); no `systemctl --user` (Answers: the user systemd service); a CPU other than `x86_64` (`x64`), `aarch64` (`arm64`) or `armv7l` when the server has no Node 22 (Answers: the remote Node).
5. `Probe` warns, without refusing: lingering off, with the command an administrator runs, `sudo loginctl enable-linger <user>`, and what it means: ahpd stops at logout (Answers: the user systemd service).
6. `NodeOnServer` is `Own { path, version }` for a Node 22 or later, `Download { cpu }` when there is none or it is older, and `Refused { reason }` when the download cannot work: no `curl` or `wget`, no `tar`, or a musl libc, for which nodejs.org has no official build (Answers: the remote Node).
7. An ahpd is looked for in this order, and the first that answers `--version` is the one found: the desk's own install, `<home>/.local/share/ahpd/node_modules/@ahpd/server` run with the Node its unit names; `ahpd` on the session's `PATH`; `ahpd` through the login shell, as `node` is in step 3.
   An install that does not answer `--version`, as one cancelled half-way, is not found, and task 05 installs over it.
8. When an ahpd is found, `found_ahpd()` runs its `plugin list --json` and `status --json` in one more `exec_capture`, the desk's own install with `AHPD_PLUGIN_ROOT=<home>/.local/share/ahpd` (Answers: the config folder), and `Probe.found` is `FoundOnServer { path, version, supported, plugins, running, url, byDesk }`: `supported` is its version against `OLDEST_SUPPORTED`, `plugins` the ready ones, `running` and `url` from `status --json`, and `byDesk` true for the desk's own install with the desk's unit (Answers: a found ahpd).
   A found ahpd is used, never installed over; one older than `OLDEST_SUPPORTED` is shown with the version needed and is not replaced (Answers: a found ahpd).
   With a found ahpd, the refusals of step 4 that concern the install (no `systemctl --user`, a CPU with no Node build) and `NodeOnServer` do not apply, since the desk installs nothing; not Linux still refuses (Answers: the user systemd service).
   A found ahpd that is not running is started by task 05 with its own `ahpd start`, and the probe's answer says it will not come back after the server restarts (Answers: a found ahpd that is off); a found ahpd is reached on its own port (Answers: how a found ahpd is reached).

## Validation

- Cargo tests for `parse_probe()` on captured outputs: Debian with Node 22 and lingering on; Ubuntu with no Node and lingering off; a Node 20 (`Download`); Alpine (musl, refused when Node is missing); `arm64`; macOS (`Darwin`, refused); no `systemctl --user` (refused); no user bus until `XDG_RUNTIME_DIR` is set; an `ahpd` on `PATH`; the desk's own install with its unit (`byDesk`); an install that does not answer `--version` (not found).
- Cargo tests for `found_ahpd()`'s parse: a running ahpd with `@ahpd/agent-claude` ready; a stopped one with no plugin; a 0.7.0 (`supported: false`).
- Cargo test against task 01's `test_server.rs`: `probe()` sends one command and parses its answer, and a second only when an ahpd is found.
- `pnpm test` and `pnpm lint` pass.
- By hand on dev02 only: the ignored test `ssh_a_real_probe` prints dev02's probe, which is recorded in this file's Resume.

## Resume

- 2026-10-18: `src-tauri/src/ssh_install.rs` has `Probe`, `Cpu`, `NodeOnServer`, `FoundOnServer`, `probe()`, `parse_probe()`, `parse_found()`, `node_version()`, `ahpd_command()` and the command `probe_ssh_server`; `ipc.ts` has `probeSshServer` and the types.
- The probe is one `sh` script after the line `# ahp-desk probe`, printing `key=value` lines; a found ahpd gets one more command, after `# ahp-desk found`, with its `plugin list --json` and `status --json` each after a marker line.
- The desk's own install is run as `<node> <home>/.local/share/ahpd/node_modules/.bin/ahpd`, the Node read from the unit's `ExecStart`, else `<install>/node/bin/node`, else the server's; an `ahpd` found only through the login shell keeps `loginShell` and is run there.
- Refusals and warnings are English sentences from the desk, as the local install's errors are; an ahpd older than `OLDEST_SUPPORTED` is a refusal that names the version needed.
- `probe_ssh_server` takes the typed `passphrase` and `password` and holds them in `Secrets` before it signs in, and rejects with task 02's `SshError`.
- The test server answers canned commands by their start, and keeps the commands it ran.
- Verified: `cargo test` (95: 15 for the probe's parse, 1 against the test server for one command and a second only for a found ahpd), `cargo clippy --all-targets -- -D warnings`, `pnpm typecheck`, `pnpm test`.
- By hand, `ssh_a_real_probe` on the test server Softov named: Debian 13 on x86_64, `systemctl --user` answering, lingering off (warned), no Node 22 (`download`, `x64`), no ahpd found, no refusal.
