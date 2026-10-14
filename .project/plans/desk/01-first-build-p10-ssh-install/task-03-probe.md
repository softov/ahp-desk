---
title: The desk probes a server for Linux, its CPU, systemctl --user, lingering, Node and an ahpd already there
status: todo
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

`ssh_install::probe()` tells whether the server already has an ahpd, which the desk then uses (decision 16), and, when it has none, whether the server can take ahpd as a user service (decision 1) and where its Node comes from (decision 4); `probe_ssh_server` answers it to the shell.

## Files

- `CREATE: src-tauri/src/ssh_install.rs` - `Probe`, `Cpu`, `NodeOnServer`, `FoundOnServer`, `probe()`, `parse_probe()`, `node_version()`, `found_ahpd()`, and the command `probe_ssh_server`.
- `UPDATE: src-tauri/src/lib.rs` - `mod ssh_install;` and the command.
- `UPDATE: src-tauri/build.rs`, `src-tauri/capabilities/shell.json` - `probe_ssh_server`, granted to the shell window.
- `UPDATE: packages/shell/src/hosts/ipc.ts` - `probeSshServer(target)` and the `Probe` and `FoundOnServer` types.

## Steps

1. One `exec_capture` runs a POSIX `sh` script that prints `key=value` lines, so a server answers in one round trip and `parse_probe()` is a pure function: `os` (`uname -s`), `pretty` (`PRETTY_NAME` from `/etc/os-release`), `arch` (`uname -m`), `libc` (`musl` when `ldd --version` says so, else `glibc`), `home` (`$HOME`), `user` (`id -un`), `systemd` (the exit of `systemctl --user show-environment`), `linger` (`loginctl show-user "$(id -un)" -p Linger --value`), `node` (the absolute path of `node` on the server, if any), `nodeVersion` (`node -v`), `npm` (the `npm` beside that `node`), `fetch` (`curl` or `wget`, whichever is there), `tar` (whether `tar` is there), `ahpd` (the absolute path of an `ahpd` found, as step 7 says), `ahpdVersion` (its `--version`), `deskUnit` (whether `~/.config/systemd/user/ahpd.service` exists and its `Description` names ahp-desk).
2. A `systemctl --user` that fails with no user bus is asked again with `XDG_RUNTIME_DIR=/run/user/$(id -u)`; when that works, `Probe.runtime_dir` keeps it, and every later `systemctl --user` and `journalctl --user` (tasks 05 and 08) runs with it.
3. `node` is looked for on the session's `PATH`, then through the login shell (`"$SHELL" -lc 'command -v node'`), since a Node from a version manager is often only there; the unit (task 05) uses its absolute path, so the service never depends on a shell's `PATH`.
4. `Probe` refuses, each with its reason in words a person acts on: not Linux (decision 1); no `systemctl --user` (decision 1's consequence); a CPU other than `x86_64` (`x64`), `aarch64` (`arm64`) or `armv7l` when the server has no Node 22 (decision 4).
5. `Probe` warns, without refusing: lingering off, with the command an administrator runs, `sudo loginctl enable-linger <user>`, and what it means: ahpd stops at logout (decision 1's consequence).
6. `NodeOnServer` is `Own { path, version }` for a Node 22 or later, `Download { cpu }` when there is none or it is older, and `Refused { reason }` when the download cannot work: no `curl` or `wget`, no `tar`, or a musl libc, for which nodejs.org has no official build (decision 4's consequence).
7. An ahpd is looked for in this order, and the first that answers `--version` is the one found: the desk's own install, `<home>/.local/share/ahpd/node_modules/@ahpd/server` run with the Node its unit names; `ahpd` on the session's `PATH`; `ahpd` through the login shell, as `node` is in step 3.
   An install that does not answer `--version`, as one cancelled half-way, is not found, and task 05 installs over it.
8. When an ahpd is found, `found_ahpd()` runs its `plugin list --json` and `status --json` in one more `exec_capture`, the desk's own install with `AHPD_PLUGIN_ROOT=<home>/.local/share/ahpd` (decision 12), and `Probe.found` is `FoundOnServer { path, version, supported, plugins, running, url, byDesk }`: `supported` is its version against `OLDEST_SUPPORTED`, `plugins` the ready ones, `running` and `url` from `status --json`, and `byDesk` true for the desk's own install with the desk's unit (decision 16).
   A found ahpd is used, never installed over; one older than `OLDEST_SUPPORTED` is shown with the version needed and is not replaced (decision 16).
   With a found ahpd, the refusals of step 4 that concern the install (no `systemctl --user`, a CPU with no Node build) and `NodeOnServer` do not apply, since the desk installs nothing; not Linux still refuses (decision 1).
   How a found ahpd that is not running is started, and how it is reached, are open questions 1 and 2 in the plan.

## Validation

- Cargo tests for `parse_probe()` on captured outputs: Debian with Node 22 and lingering on; Ubuntu with no Node and lingering off; a Node 20 (`Download`); Alpine (musl, refused when Node is missing); `arm64`; macOS (`Darwin`, refused); no `systemctl --user` (refused); no user bus until `XDG_RUNTIME_DIR` is set; an `ahpd` on `PATH`; the desk's own install with its unit (`byDesk`); an install that does not answer `--version` (not found).
- Cargo tests for `found_ahpd()`'s parse: a running ahpd with `@ahpd/agent-claude` ready; a stopped one with no plugin; a 0.7.0 (`supported: false`).
- Cargo test against task 01's `test_server.rs`: `probe()` sends one command and parses its answer, and a second only when an ahpd is found.
- `pnpm test` and `pnpm lint` pass.
- By hand on dev02 only: the ignored test `ssh_a_real_probe` prints dev02's probe, which is recorded in this file's Resume.

## Resume
