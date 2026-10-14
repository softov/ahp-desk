---
title: The desk installs ahpd over SSH as a user service, with its backends and token file, streaming its steps and able to cancel
status: todo
depends: [task-04-node.md]
layer: "src-tauri"
refs:
  - file:///github/opendoop/packages/tauri/src-tauri/src/installer/mod.rs - `InstallControl` and `cancel_install` (lines 24-65), `InstallEvent` with `Step` and `Log` (lines 99-135), `run_install`, the pattern for the steps and the cancel
  - file:///github/opendoop/packages/tauri/src-tauri/src/installer/systemd_node.rs - `write_service_unit` and `render_unit` (lines 499-558), `daemon_reload`, `enable_now`, `wait_health` with the journalctl tail (lines 560-634)
  - file:///github/ahpd/packages/server/src/commands/options.ts - `--host`, `--port` (default 9187) and `--connection-token-file`, at lines 395-440; a missing token file is written owner-only, at lines 1083-1116
  - file:///github/ahpd/packages/server/src/commands/run.ts - `ahpd run`, the foreground daemon a systemd unit runs, at line 1286
  - file:///github/ahpd/packages/server/package.json - the bin `ahpd`, `./dist/main.js`
  - "[code://src-tauri/src/local_ahpd.rs#L169-L225](../../../../src-tauri/src/local_ahpd.rs#L169-L225) - `LocalAhpd::install()` and `add_backends()`: `npm install --prefix`, then `ahpd plugin install`, the order this install keeps"
  - "[code://src-tauri/src/local_ahpd.rs#L248-L264](../../../../src-tauri/src/local_ahpd.rs#L248-L264) - `check_backends`, reused for the backends picked"
  - "[code://src-tauri/src/local_ahpd.rs#L763-L769](../../../../src-tauri/src/local_ahpd.rs#L763-L769) - `InstallEvent`, which gains `Step`"
---

## Objective

`install_ssh_ahpd` puts `@ahpd/server` and the backends picked (decision 6) into `~/.local/share/ahpd` on the server, writes `~/.config/systemd/user/ahpd.service`, enables and starts it with `systemctl --user` (decision 1), waits until it answers, and reads its token from its token file (decision 7).
Each step is sent to the shell over a Tauri `Channel` as it starts and ends, with the lines it writes, and `cancel_ssh_install` stops it between steps or in the middle of one.

## Files

- `UPDATE: src-tauri/src/ssh_install.rs` - `SshInstall` (the request), `Bind`, `SshInstalled` (the result), `install()`, `render_unit()`, `wait_ready()`, `InstallControl`, and the commands `install_ssh_ahpd` and `cancel_ssh_install`.
- `UPDATE: src-tauri/src/local_ahpd.rs:763-769` - `InstallEvent` gains `Step { id: String, status: StepStatus }` beside `Line { text }`; `StepStatus` is `running`, `done`, `failed`, `skipped`; the local install sends no steps and is unchanged.
- `UPDATE: src-tauri/src/lib.rs` - `InstallControl` managed; the two commands.
- `UPDATE: src-tauri/build.rs`, `src-tauri/capabilities/shell.json` - the two commands, granted to the shell window.
- `UPDATE: packages/shell/src/hosts/ipc.ts` - `InstallEvent` gains `{ kind: 'step'; id: string; status: ... }`; `installSshAhpd(request, onEvent)` and `cancelSshInstall()`.

## Steps

1. `install()` runs these steps, each sent as `Step { id, running }` then `done`, `failed` or `skipped`, with the ids the shell translates (task 07): `connect` (task 01, with task 02's check), `probe` (task 03), `node` (task 04), `server` (`npm install --prefix <prefix> @ahpd/server@latest`, with the Node's `bin` first on `PATH`), `backends`, `unit`, `start`, `ready`.
   `<prefix>` is `<home>/.local/share/ahpd`, absolute, from task 01's `remote_home`.
2. `backends` checks the list with `check_backends` and adds them as the local install does.
   Whether p3's decisions on the config folder with `AHPD_PLUGIN_ROOT` and on `ahpd plugin install` hold here is open question 5 in the plan; the proposed answer is yes, and then the step runs `AHPD_PLUGIN_ROOT=<prefix> <node> <prefix>/node_modules/@ahpd/server/<bin> plugin install <backends>`, with `<bin>` read from that package's `package.json` as `installed_script` does.
3. `render_unit(node, script, prefix, bind, port, runtime)` is a pure function that answers the unit: `Description=ahpd, installed by ahp-desk`; `After=network-online.target`; `Type=simple`; `ExecStart=<node> <script> run --host <127.0.0.1 or 0.0.0.0> --port <port> --connection-token-file <prefix>/connection-token`; `Environment=AHPD_PLUGIN_ROOT=<prefix>` (per open question 5); `Environment=PATH=<node bin>:/usr/local/bin:/usr/bin:/bin`, since `ahpd plugin install` runs `npm`; `Restart=on-failure`, `RestartSec=2`; `WantedBy=default.target`.
   `Bind` is `Loopback` or `All`, from the reach task 06 picks; the folder it serves (`WorkingDirectory`) is open question 6 in the plan, and the port is open question 7.
4. `unit` writes the unit with a quoted heredoc, as doop's `write_service_unit`, then `systemctl --user daemon-reload`; `start` runs `systemctl --user enable ahpd.service` and `systemctl --user restart ahpd.service`, so an install over the desk's own (open question 8) restarts the service it replaced; each `systemctl --user` runs with the probe's `XDG_RUNTIME_DIR` when it needed one.
5. `ready` asks the server's own Node, so no `curl` is needed, for `http://127.0.0.1:<port>/` every 500 ms for up to 30 s, and passes on the 426 that ahpd answers a plain GET.
   On a timeout it sends the last 40 lines of `journalctl --user -u ahpd.service --no-pager` as `Line`s and fails naming the port.
6. Once ready, `install()` reads `<prefix>/connection-token` with `cat` and the version with `<node> <script> --version`, and answers `SshInstalled { token, version, prefix, node }`; the token is returned to the caller and never written to a log line.
7. `InstallControl` holds one `AtomicBool` and allows one install at a time, as doop's; `cancel_ssh_install` raises it.
   `install()` checks it before each step; a long step (`node`, `server`, `backends`) runs as `sh -c 'echo $$ > <prefix>/.step.pid; exec ...'`, and a cancel runs `kill "$(cat <prefix>/.step.pid)"` over a second channel, then fails with `cancelled`.
   A cancelled or failed install leaves what it wrote; the next install writes over it, as open question 8 in the plan proposes for the desk's own earlier install.
8. The token is read from its file after the first start, because ahpd writes a missing token file itself (decision 7); the install never makes one up.

## Validation

- Cargo tests: `render_unit()` for `Loopback` and `All`, with and without `XDG_RUNTIME_DIR`, quoting a home with a space; the step ids in order; a cancel raised before a step sends no further step and answers `cancelled`.
- Cargo test against task 01's `test_server.rs` with canned answers: the full sequence of commands for a server with its own Node; a `ready` that never answers ends with the journal lines and fails.
- `pnpm test` and `pnpm lint` pass.
- By hand on dev02 only: the ignored test `ssh_a_real_install` installs with `@ahpd/agent-claude`, then checks `systemctl --user is-active ahpd` and that the token read equals the file; a second run with Cancel during `server` stops, and a third run completes over it.
  The test removes what it installed at the end: `systemctl --user disable --now ahpd.service`, the unit, and `~/.local/share/ahpd`.

## Resume
