---
title: The desk's unit records ahpd as a systemd service, and an update writes the unit again
status: done
depends: [task-05-install.md, task-08-update-over-ssh.md]
layer: "src-tauri"
refs:
  - "[code://src-tauri/src/ssh_install.rs](../../../../src-tauri/src/ssh_install.rs) - `render_unit`, `write_unit`, `bind_of`, `update` and `update_host`"
  - file:///github/ahpd/packages/server/src/service.ts - ahpd's own unit, the same name and file
---

## Objective

A terminal `ahpd status`, `ahpd stop` and `ahpd restart` on the server reach the ahpd the desk installed, once the server's ahpd has daemon/31 ([decision](../../../decisions/the-desks-unit-records-ahpd-as-a-service-and-an-update-writes-it-again.md)).

## Files

- `UPDATE: src-tauri/src/ssh_install.rs` - `render_unit` writes `Environment=AHPD_SERVICE=systemd`; `write_unit()` is the unit's heredoc and `daemon-reload`, shared by the install and the update; `bind_of(Reach)` gives the bind for both; `update` takes `bind` and has a `unit` step before `start`; `update_host` passes the host's reach.

## Steps

1. `render_unit`: the line `Environment=AHPD_SERVICE=systemd` after `PATH`.
2. `write_unit(home, unit, runtime_dir)`, taken out of the install's `unit` step.
3. `update`: after `server`, render the unit with the unit's Node, the found script, the prefix, `bind` and `port`, and run `write_unit` as the step `unit`; then `start` restarts it.

## Validation

- `install_the_unit_runs_ahpd_on_its_port_with_its_token_file_and_plugin_root` checks the line.
- `update_installs_the_version_with_the_units_node_writes_the_unit_restarts_and_waits` checks the steps `connect, probe, server, unit, start, ready`, the unit's `ExecStart` on 127.0.0.1 and the port, the line, and `daemon-reload`.
- On the test machine, once ahpd has daemon/31: a terminal `ahpd status --json` sees the desk's ahpd, and `ahpd restart` restarts it through systemd.

## Resume

- 2026-10-19: built as above; `cargo test` (136) and `cargo clippy --all-targets -- -D warnings` pass.
- The shell needed no change: `unit` was already an install step it names.
- Not run: the ignored `ssh_a_real_update` on the test machine, and the terminal check, which waits on an ahpd release with daemon/31; ahpd 0.12.0 does not read the variable.
