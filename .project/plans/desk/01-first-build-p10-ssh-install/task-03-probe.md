---
title: The desk probes a server for Linux, its CPU, systemctl --user, lingering and Node
status: todo
depends: [task-01-ssh-session.md]
layer: "src-tauri"
refs:
  - file:///github/opendoop/packages/tauri/src-tauri/src/ssh_probe.rs - `parse_remote_os` and `fetch_os_pretty_name` (in `ssh/mod.rs`), the OS read this probe narrows to Linux
  - file:///github/opendoop/packages/tauri/src-tauri/src/installer/systemd_node.rs - `prereqs`, `systemctl --user --version`, at lines 92-108
  - "[code://src-tauri/src/local_ahpd.rs#L26](../../../../src-tauri/src/local_ahpd.rs#L26) - `OLDEST_SUPPORTED`, the pattern for a version floor read from a version line"
---

## Objective

`ssh_install::probe()` tells, in one pass over the session, whether the server can take ahpd as a user service (decision 1) and where its Node comes from (decision 4), and `probe_ssh_server` answers it to the shell.

## Files

- `CREATE: src-tauri/src/ssh_install.rs` - `Probe`, `Cpu`, `NodeOnServer`, `probe()`, `parse_probe()`, `node_version()`, and the command `probe_ssh_server`.
- `UPDATE: src-tauri/src/lib.rs` - `mod ssh_install;` and the command.
- `UPDATE: src-tauri/build.rs`, `src-tauri/capabilities/shell.json` - `probe_ssh_server`, granted to the shell window.
- `UPDATE: packages/shell/src/hosts/ipc.ts` - `probeSshServer(target)` and the `Probe` type.

## Steps

1. One `exec_capture` runs a POSIX `sh` script that prints `key=value` lines, so a server answers in one round trip and `parse_probe()` is a pure function: `os` (`uname -s`), `pretty` (`PRETTY_NAME` from `/etc/os-release`), `arch` (`uname -m`), `libc` (`musl` when `ldd --version` says so, else `glibc`), `home` (`$HOME`), `user` (`id -un`), `systemd` (the exit of `systemctl --user show-environment`), `linger` (`loginctl show-user "$(id -un)" -p Linger --value`), `node` (the absolute path of `node` on the server, if any), `nodeVersion` (`node -v`), `npm` (the `npm` beside that `node`), `fetch` (`curl` or `wget`, whichever is there), `tar` (whether `tar` is there), `installed` (whether `~/.local/share/ahpd/node_modules/@ahpd/server` and `~/.config/systemd/user/ahpd.service` exist).
2. A `systemctl --user` that fails with no user bus is asked again with `XDG_RUNTIME_DIR=/run/user/$(id -u)`; when that works, `Probe.runtime_dir` keeps it, and every later `systemctl --user` and `journalctl --user` (tasks 05 and 08) runs with it.
3. `node` is looked for on the session's `PATH`, then through the login shell (`"$SHELL" -lc 'command -v node'`), since a Node from a version manager is often only there; the unit (task 05) uses its absolute path, so the service never depends on a shell's `PATH`.
4. `Probe` refuses, each with its reason in words a person acts on: not Linux (decision 1); no `systemctl --user` (decision 1's consequence); a CPU other than `x86_64` (`x64`), `aarch64` (`arm64`) or `armv7l` when the server has no Node 22 (decision 4).
5. `Probe` warns, without refusing: lingering off, with the command an administrator runs, `sudo loginctl enable-linger <user>`, and what it means: ahpd stops at logout (decision 1's consequence).
6. `NodeOnServer` is `Own { path, version }` for a Node 22 or later, `Download { cpu }` when there is none or it is older, and `Refused { reason }` when the download cannot work: no `curl` or `wget`, no `tar`, or a musl libc, for which nodejs.org has no official build (decision 4's consequence).
7. `installed` is reported, not acted on; what an install does over an existing one is open question 8 in the plan.

## Validation

- Cargo tests for `parse_probe()` on captured outputs: Debian with Node 22 and lingering on; Ubuntu with no Node and lingering off; a Node 20 (`Download`); Alpine (musl, refused when Node is missing); `arm64`; macOS (`Darwin`, refused); no `systemctl --user` (refused); no user bus until `XDG_RUNTIME_DIR` is set.
- Cargo test against task 01's `test_server.rs`: `probe()` sends one command and parses its answer.
- `pnpm test` and `pnpm lint` pass.
- By hand on dev02 only: the ignored test `ssh_a_real_probe` prints dev02's probe, which is recorded in this file's Resume.

## Resume
