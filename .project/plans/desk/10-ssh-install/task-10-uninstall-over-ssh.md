---
title: Remove on an SSH host the desk installed offers to stop and uninstall its ahpd on the server
status: todo
depends: [task-05-install.md, task-06-reach.md, task-07-add-host-ssh.md]
layer: "src-tauri, packages/shell"
refs:
  - "[code://src-tauri/src/hosts.rs#L231-L242](../../../../src-tauri/src/hosts.rs#L231-L242) - `HostStore::remove`, which forgets the host, its token and its kept password"
  - "[code://src-tauri/src/hosts.rs#L339-L352](../../../../src-tauri/src/hosts.rs#L339-L352) - `remove_host`, which the uninstall runs before"
  - "[code://packages/shell/src/hosts/HostList.tsx#L89-L104](../../../../packages/shell/src/hosts/HostList.tsx#L89-L104) - the row's Remove question, which gains the uninstall choice"
  - file:///github/opendoop/packages/tauri/src-tauri/src/installer/systemd_node.rs - doop's user unit and `systemctl --user`, which the uninstall undoes
---

## Objective

Remove, on a host whose `ssh` details say the desk installed its ahpd (`byDesk`), asks whether to also stop and uninstall ahpd on the server, as the Answers row on the uninstall says.
Yes stops and disables the user unit, removes the unit file, the desk's install under `~/.local/share/ahpd` and its token file, streaming its steps, then removes the host.
An ahpd the desk found on the server and did not install is left as it is, and Remove says so (Answers: a host the desk did not install).

## Files

- `UPDATE: src-tauri/src/ssh_install.rs` - `uninstall()`, `uninstall_script()`, and the command `uninstall_ssh_ahpd`.
- `UPDATE: src-tauri/src/lib.rs` - the command.
- `UPDATE: src-tauri/build.rs`, `src-tauri/capabilities/shell.json` - `uninstall_ssh_ahpd`, granted to the shell window.
- `UPDATE: packages/shell/src/hosts/ipc.ts` - `uninstallSshAhpd(id, onEvent)`.
- `UPDATE: packages/shell/src/hosts/HostList.tsx:89-104` - the Remove question of an SSH host: for `byDesk`, it asks whether to also stop and uninstall ahpd on the server, with "Remove and uninstall", "Remove from the desk only" and Cancel; while it runs, the steps in `InstallSteps`; on a failure, why, with "Remove from the desk only" and Cancel (Answers: an unreachable server); for a host with `byDesk` false, the line that the server's ahpd stays (Answers: a host the desk did not install).
- `UPDATE: packages/shell/src/i18n.ts`, `packages/shell/src/app.css` - the words in en, pt-BR and es, and the styles.

## Steps

1. `uninstall(app, id, events)` reads the host's `SshHost` from `HostStore::ssh(id)` and refuses one with `byDesk` false before any connection (Answers: a found ahpd); it connects with the kept sign-in or what `Secrets` holds (task 01) and the key check (task 02).
   A sign-in that needs the person, or a key refused, fails with that error, as task 06 shows it.
2. It runs these steps, each sent as `Step { id, running }` then `done`, `failed` or `skipped`, with the same `Channel` and `InstallEvent` as task 05: `connect`; `stop` (`systemctl --user disable --now ahpd.service`, `skipped` when the unit is not there); `unit` (removes `<home>/.config/systemd/user/ahpd.service`, then `systemctl --user daemon-reload`); `files` (removes `<prefix>`, `<home>/.local/share/ahpd`, with the downloaded Node, `@ahpd/server`, the backends and `connection-token` in it).
   Each `systemctl --user` runs with the probe's `XDG_RUNTIME_DIR` when it needed one (task 03).
3. `uninstall_script()` is a pure function that answers the commands of step 2, every path absolute from task 01's `remote_home` and quoted with `quote()`; `files` checks the folder is `<home>/.local/share/ahpd` and that its `node_modules/@ahpd/server` or `connection-token` is there before `rm -rf`, so a wrong home never removes another folder.
4. `~/.config/ahpd` on the server is left as it is: no step touches it (Answers: the config folder on uninstall), and the Remove question says it stays.
5. Once every step is `done` or `skipped`, the shell calls `removeHost(id)`, which closes the tunnel (task 06), forgets the token and the kept password (task 01), and drops the host's `Secrets`; the uninstall runs first because it needs the host's SSH access.
6. "Remove from the desk only" removes the host, as Remove does today, and the server's ahpd keeps running (Answers: the uninstall is offered, not forced).
7. A failure in any step, including a server that cannot be reached, a sign-in that needs the person or a refused key, stops the uninstall and keeps the host; the row shows why, with "Remove from the desk only", which removes the host as step 6 does, and Cancel, which keeps it (Answers: an unreachable server).
8. On a host with `byDesk` false, the Remove question offers no uninstall and says the server's ahpd stays as it is, because the desk did not install it; Remove then forgets the host and does nothing on the server (Answers: a host the desk did not install).

## Validation

- Cargo tests: `uninstall_script()` with a home with a space; the guard refuses a prefix outside `<home>/.local/share/ahpd`; the step ids in order; `stop` is `skipped` with no unit.
- Cargo test against task 01's `test_server.rs` with canned answers: the commands in order, none touching `~/.config/ahpd`; a host with `byDesk` false refused with no connection; a failing `systemctl --user disable` stops before `files` and the host is kept; a server that does not answer fails at `connect` and the host is kept.
- `pnpm typecheck`, `pnpm test` and `pnpm lint` pass; `i18n.test.ts` passes with the new words in en, pt-BR and es.
- By hand on dev02 only: the ignored test `ssh_a_real_uninstall` installs with task 05's code, uninstalls, and checks that `systemctl --user status ahpd` knows no unit, that `~/.local/share/ahpd` and the unit file are gone, and that nothing listens on the ahpd port (`ss -ltn`).
- By hand on the debug desk over CDP, against dev02 only: "Remove and uninstall" on a tunnelled host shows the steps and ends with the host gone from the list; "Remove from the desk only" leaves the server's ahpd running; the question in en, pt-BR and es at the window's smallest size.

## Resume
