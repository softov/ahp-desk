---
title: The probe finds a running ahpd through its daemon.json, and an install starts on the first free port
status: doing
depends: [task-03-probe.md, task-05-install.md, task-07-add-host-ssh.md]
layer: "src-tauri, packages/shell"
refs:
  - "[code://src-tauri/src/ssh_install.rs](../../../../src-tauri/src/ssh_install.rs) - `PROBE_SCRIPT`, `parse_probe`, `first_free_port`, `ahpd_command` and `install`'s probe step"
  - "[code://packages/shell/src/hosts/SshForm.tsx](../../../../packages/shell/src/hosts/SshForm.tsx) - the form's \"ahpd port\""
  - "[code://packages/shell/src/hosts/ipc.ts](../../../../packages/shell/src/hosts/ipc.ts) - `Probe` and `FoundOnServer`"
---

## Objective

The two defects the pending check found on 2026-10-20 (the plan's Resume) are fixed.
A running ahpd that neither `PATH` nor the login shell shows is found through its `daemon.json` ([decision](../../../decisions/the-probe-finds-a-running-ahpd-through-its-record.md)).
An install starts "ahpd port" on the first free port and refuses one in use ([decision](../../../decisions/an-ssh-install-starts-on-the-first-free-port.md)).

## Files

- `UPDATE: src-tauri/src/ssh_install.rs` - the probe says `listening`, and as its last try the running ahpd (`ahpdNode`, `ahpdRunning`); `FoundOnServer.desk_install`; `Probe.listening` and `free_port`, from `first_free_port`; `ahpd_command` gives the plugin root only to the desk's install; `install`'s probe step refuses a port in use.
- `UPDATE: packages/shell/src/hosts/ipc.ts` - `Probe.listening`, `Probe.freePort`, `FoundOnServer.deskInstall`.
- `UPDATE: packages/shell/src/hosts/SshForm.tsx` - "ahpd port" moves to `freePort` when the typed one is in use; `ssh.portInUse` refuses one in use, `ssh.portMoved` says why the port is not 9187.
- `UPDATE: packages/shell/src/i18n.ts` - `ssh.portInUse` and `ssh.portMoved` in `en`, `ptBR` and `es`.

## Steps

1. The probe prints `listening=` with the ports in state `0A` in `/proc/net/tcp` and `/proc/net/tcp6`, before any early exit.
2. After the login shell's try, the probe reads the pid in `daemon.json`; when `/proc/<pid>/cmdline` runs a program named `ahpd` or one under `@ahpd/server`, it prints `ahpd`, `ahpdNode`, `ahpdVersion` and `ahpdRunning=yes`.
3. `parse_probe` keeps `ahpdNode` for the desk's install and for a running ahpd, with `desk_install` only for the first.
4. `ahpd_command` runs a found ahpd with a Node as `PATH=<its folder>:"$PATH" <node> <script>`, with `AHPD_PLUGIN_ROOT` only for the desk's install.
5. `install`'s probe step refuses, when no ahpd is found, a requested port in `listening`: "port X is in use on the server; Y is free".
6. The form: after a check with no ahpd found, a typed port in `listening` becomes `freePort`; a port in `listening` shows `ssh.portInUse` and blocks the install; a port moved off 9187 shows `ssh.portMoved`.

## Validation

- `probe_a_running_ahpd_found_through_its_record_runs_with_its_own_node`, `probe_the_listening_ports_give_the_first_free_one` and `install_on_a_port_in_use_is_refused_with_the_free_one_and_no_install_command` pass, with the rest of `cargo test`, `cargo clippy --all-targets -- -D warnings`, `pnpm typecheck` and `pnpm test`.
- On the test machine, the probe's new lines under its `sh` find the running global ahpd and name 9187 in `listening`.
- By hand, Add host's check on a server with a running global ahpd shows that ahpd, and on a server where 9187 is taken and no ahpd runs, "ahpd port" starts on the next free one with `ssh.portMoved`.

## Resume

- 2026-10-20: built as above.
  The probe's own test in `install_with_a_found_ahpd_adds_only_the_backends_it_lacks_and_installs_nothing` now leaves the probe command out of "nothing installed", since the probe's `case` names `@ahpd/server`.
- Verified: `cargo test` (140), `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check`, `pnpm typecheck`, `pnpm test` (53).
- On the test machine, the probe's new lines run alone under `sh` (dash) printed `listening=22 631 9187 631 22`, and the running global ahpd at `~/.npm-global/bin/ahpd` with `/usr/local/bin/node` and 0.13.0.
- By hand on a release build of this branch with a scratch config, Add host's check (nothing saved or installed) against the test machine said "ahpd 0.13.0 at /home/softov/.npm-global/bin/ahpd", its own port 9187 and its installed backends, with "Save and connect" and no install; the test machine was left as it was.
- Still `doing`: the form on a server where 9187 is taken and no ahpd runs, which the test machine is not while another session's ahpd runs there.
