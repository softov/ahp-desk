---
title: The desk installs ahpd over SSH as a user service, or uses the one it finds, with the backends, folder, port and token, streaming its steps and able to cancel
status: todo
depends: [task-04-node.md]
layer: "src-tauri"
refs:
  - file:///github/opendoop/packages/tauri/src-tauri/src/installer/mod.rs - `InstallControl` and `cancel_install` (lines 24-65), `InstallEvent` with `Step` and `Log` (lines 99-135), `run_install`, the pattern for the steps and the cancel
  - file:///github/opendoop/packages/tauri/src-tauri/src/installer/systemd_node.rs - `write_service_unit` and `render_unit` (lines 499-558), `daemon_reload`, `enable_now`, `wait_health` with the journalctl tail (lines 560-634)
  - file:///github/ahpd/packages/server/src/commands/options.ts - `--host`, `--port` (default 9187) and `--connection-token-file`, at lines 395-440; a missing token file is written owner-only, at lines 1083-1116; `paths`, at lines 991-1001
  - file:///github/ahpd/packages/server/src/commands/run.ts - `ahpd run`, the foreground daemon a systemd unit runs, at line 1286
  - file:///github/ahpd/packages/server/src/install.ts - `pluginRoot()`: `ahpd plugin install` writes to `AHPD_PLUGIN_ROOT` when it is set
  - file:///github/ahpd/packages/server/package.json - the bin `ahpd`, `./dist/main.js`
  - "[code://src-tauri/src/local_ahpd.rs#L169-L225](../../../../src-tauri/src/local_ahpd.rs#L169-L225) - `LocalAhpd::install()` and `add_backends()`: `npm install --prefix`, then `ahpd plugin install` with `AHPD_PLUGIN_ROOT`, the order and the layout this install keeps"
  - "[code://src-tauri/src/local_ahpd.rs#L248-L264](../../../../src-tauri/src/local_ahpd.rs#L248-L264) - `check_backends`, reused for the backends picked"
  - "[code://src-tauri/src/local_ahpd.rs#L763-L769](../../../../src-tauri/src/local_ahpd.rs#L763-L769) - `InstallEvent`, which gains `Step`"
---

## Objective

`install_ssh_ahpd` gets the server an ahpd the desk connects to.
When the probe found one (task 03), it uses it, as decision 16 says: it adds the backends ticked with that ahpd's own `plugin install` and reads how to reach it.
When none was found, it installs `@ahpd/server` and the backends picked (decision 6) under `~/.local/share/ahpd` on the server, with ahpd's config in the usual `~/.config/ahpd` and the backends added through `ahpd plugin install` with `AHPD_PLUGIN_ROOT` (decisions 12 and 13), sets the folder asked (decision 14), writes `~/.config/systemd/user/ahpd.service` on the ahpd port asked (decision 15), enables and starts it with `systemctl --user` (decision 1), waits until it answers, and reads its token from its token file (decision 7).
Each step is sent to the shell over a Tauri `Channel` as it starts and ends, with the lines it writes, and `cancel_ssh_install` stops it between steps or in the middle of one.

## Files

- `UPDATE: src-tauri/src/ssh_install.rs` - `SshInstall` (the request: target, backends, `folder`, `ahpdPort`, reach), `Bind`, `SshInstalled` (the result), `install()`, `use_found()`, `render_unit()`, `wait_ready()`, `InstallControl`, and the commands `install_ssh_ahpd` and `cancel_ssh_install`.
- `UPDATE: src-tauri/src/local_ahpd.rs:763-769` - `InstallEvent` gains `Step { id: String, status: StepStatus }` beside `Line { text }`; `StepStatus` is `running`, `done`, `failed`, `skipped`; the local install sends no steps and is unchanged.
- `UPDATE: src-tauri/src/lib.rs` - `InstallControl` managed; the two commands.
- `UPDATE: src-tauri/build.rs`, `src-tauri/capabilities/shell.json` - the two commands, granted to the shell window.
- `UPDATE: packages/shell/src/hosts/ipc.ts` - `InstallEvent` gains `{ kind: 'step'; id: string; status: ... }`; `installSshAhpd(request, onEvent)` and `cancelSshInstall()`.

## Steps

1. With no ahpd found, `install()` runs these steps, each sent as `Step { id, running }` then `done`, `failed` or `skipped`, with the ids the shell translates (task 07): `connect` (task 01, with task 02's check), `probe` (task 03), `node` (task 04), `server` (`npm install --prefix <prefix> @ahpd/server@latest`, with the Node's `bin` first on `PATH`), `backends`, `folder`, `unit`, `start`, `ready`.
   `<prefix>` is `<home>/.local/share/ahpd`, absolute, from task 01's `remote_home`; ahpd's config folder is left as ahpd's default, `<home>/.config/ahpd`, so a terminal `ahpd status` on the server sees the same ahpd (decision 12).
2. `backends` checks the list with `check_backends` and runs `AHPD_PLUGIN_ROOT=<prefix> <node> <prefix>/node_modules/@ahpd/server/<bin> plugin install <backends>`, with `<bin>` read from that package's `package.json` as `installed_script` does (decisions 12 and 13).
   Every ahpd command the desk runs for its own install on the server carries `AHPD_PLUGIN_ROOT=<prefix>`, as on this computer.
3. `folder` runs `ahpd paths add <folder>` (git://54e4ef26 in ahpd, released after 0.11.0), with the folder from the form, the server user's home by default (decision 14); when the config already names folders, the step is `skipped` and they stay, as on this computer.
   The step waits for the ahpd release with that command, as p3's folder step does.
4. `render_unit(node, script, prefix, bind, ahpd_port, runtime)` is a pure function that answers the unit: `Description=ahpd, installed by ahp-desk`; `After=network-online.target`; `Type=simple`; `WorkingDirectory=%h`, so an ahpd whose config names no folder never serves `/` (decision 14); `ExecStart=<node> <script> run --host <127.0.0.1 or 0.0.0.0> --port <ahpd port> --connection-token-file <prefix>/connection-token`; `Environment=AHPD_PLUGIN_ROOT=<prefix>` (decision 12); `Environment=PATH=<node bin>:/usr/local/bin:/usr/bin:/bin`, since `ahpd plugin install` runs `npm`; `Restart=on-failure`, `RestartSec=2`; `WantedBy=default.target`.
   `Bind` is `Loopback` or `All`, from the reach task 06 picks; the ahpd port is the form's field, 9187 by default (decision 15).
5. `unit` writes the unit with a quoted heredoc, as doop's `write_service_unit`, then `systemctl --user daemon-reload`; `start` runs `systemctl --user enable ahpd.service` and `systemctl --user restart ahpd.service`; each `systemctl --user` runs with the probe's `XDG_RUNTIME_DIR` when it needed one.
6. `ready` asks the server's own Node, so no `curl` is needed, for `http://127.0.0.1:<ahpd port>/` every 500 ms for up to 30 s, and passes on the 426 that ahpd answers a plain GET.
   On a timeout it sends the last 40 lines of `journalctl --user -u ahpd.service --no-pager` as `Line`s and fails naming the port, which may be taken on the server (decision 15).
7. Once ready, `install()` reads `<prefix>/connection-token` with `cat` and the version with `<node> <script> --version`, and answers `SshInstalled { token, version, prefix, node, ahpdPort, byDesk: true }`; the token is returned to the caller and never written to a log line.
8. With an ahpd found (`Probe.found`, task 03), `use_found()` runs `connect`, `probe`, `backends`, `start` and `ready`, and installs nothing (decision 16).
   One older than `OLDEST_SUPPORTED` is refused before any step, with its version and the version needed, as the form shows it.
   `backends` adds only the backends ticked that its `plugin list` does not have ready, with that ahpd's own `plugin install`, the desk's own install with `AHPD_PLUGIN_ROOT=<prefix>`; with none to add it is `skipped`.
   For the desk's own earlier install (`byDesk`), `start` runs `systemctl --user enable` and `restart` on the unit it has, so new backends load, and the token is read from `<prefix>/connection-token` as in step 7.
   For an ahpd the desk did not install, `SshInstalled.byDesk` is false: when it is not running, `start` runs its own `ahpd start` as it is configured and writes no unit (decision 20), when it runs and `backends` added some, `start` runs its own `ahpd restart` so they load, and otherwise is `skipped`.
   Its address comes from its `ahpd status --json` and its token, if it has one, from its `daemon.json`, read with `cat` over SSH (decision 21); `ready` asks that port, and `SshInstalled.ahpdPort` is it.
9. `InstallControl` holds one `AtomicBool` and allows one install at a time, as doop's; `cancel_ssh_install` raises it.
   `install()` checks it before each step; a long step (`node`, `server`, `backends`) runs as `sh -c 'echo $$ > <prefix>/.step.pid; exec ...'`, and a cancel runs `kill "$(cat <prefix>/.step.pid)"` over a second channel, then fails with `cancelled`.
   A cancelled or failed install leaves what it wrote; an install that does not answer `--version` is not found by the next probe (task 03), so the next install writes over it.
10. The token of the desk's install is read from its file after the first start, because ahpd writes a missing token file itself (decision 7); the install never makes one up.

## Validation

- Cargo tests: `render_unit()` for `Loopback` and `All`, with and without `XDG_RUNTIME_DIR`, on 9187 and on another port, quoting a home with a space, with `WorkingDirectory=%h` and `AHPD_PLUGIN_ROOT`; the step ids in order for an install and for a found ahpd; a cancel raised before a step sends no further step and answers `cancelled`.
- Cargo test against task 01's `test_server.rs` with canned answers: the full sequence of commands for a server with its own Node, every ahpd command carrying `AHPD_PLUGIN_ROOT`; a found ahpd with one backend ready gets only the other one ticked and no `npm install` of `@ahpd/server`; a found ahpd too old is refused with no command sent; a `ready` that never answers ends with the journal lines and fails.
- `pnpm test` and `pnpm lint` pass.
- By hand on dev02 only: the ignored test `ssh_a_real_install` installs with `@ahpd/agent-claude` on a port other than 9187, then checks `systemctl --user is-active ahpd`, that `ahpd status --json` run from a login shell with `AHPD_PLUGIN_ROOT` set sees it, and that the token read equals the file.
  A second run finds that install and uses it, adding `@ahpd/bot`, with no `npm install` of `@ahpd/server`; a third run, from a clean server, with Cancel during `server`, stops, and a fourth completes over it.
  The test removes what it installed at the end: `systemctl --user disable --now ahpd.service`, the unit, and `~/.local/share/ahpd`.

## Resume
