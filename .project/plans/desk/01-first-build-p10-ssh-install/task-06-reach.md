---
title: The desk reaches an SSH-installed ahpd through a tunnel, as host kind ssh, or through its open port, as a URL host
status: todo
depends: [task-05-install.md]
layer: "src-tauri, packages/shell"
refs:
  - file:///github/ahpd/packages/server/src/commands/options.ts - `--host`: 127.0.0.1 unless told otherwise, and any other address needs a token, at lines 401-405 and 1108-1116
  - "[code://src-tauri/src/hosts.rs#L18-L23](../../../../src-tauri/src/hosts.rs#L18-L23) - `HostKind`, which gains `Ssh`"
  - "[code://src-tauri/src/hosts.rs#L274-L308](../../../../src-tauri/src/hosts.rs#L274-L308) - `local()` and `set_local()`, the pattern for a host whose address the desk sets"
  - "[code://src-tauri/src/local_ahpd.rs#L739-L761](../../../../src-tauri/src/local_ahpd.rs#L739-L761) - `start_on_launch` and `stop_on_quit`, the pattern for the tunnels at launch and quit"
  - "[code://src-tauri/src/lib.rs#L67-L94](../../../../src-tauri/src/lib.rs#L67-L94) - where launch and quit call them"
  - "[code://src-tauri/src/app_window.rs#L26-L30](../../../../src-tauri/src/app_window.rs#L26-L30) - `page_url()`, which gives a window its host's `url`"
  - "[code://packages/shell/src/hosts/HostList.tsx](../../../../packages/shell/src/hosts/HostList.tsx) - the row, which shows a tunnel's error while its host is down"
---

## Objective

An install chooses its reach, as decision 5 says.
Through the SSH connection: ahpd stays on the server's 127.0.0.1, and the host is kind `ssh`, whose `url` is a local port the desk forwards to it over SSH while the desk runs.
An open port: ahpd listens on 0.0.0.0 with its token, and the host is kind `url` at `ws://<server>:<port>`, with its SSH details kept for Update (task 08).

## Files

- `CREATE: src-tauri/src/ssh/tunnel.rs` - `Tunnels` (a Tauri state: host id to `Tunnel`), `open()`, `close()`, `forward()`, `open_on_launch()`, `TunnelState`, the event `tunnels-changed`, the command `ssh_tunnel_states`.
- `UPDATE: src-tauri/src/hosts.rs` - `HostKind::Ssh`; `SshHost` gains `reach` (`tunnel` or `port`), `remotePort` and, for a tunnel, `localPort`; `save` refuses kind `ssh` with no `ssh`; the `remove_host` command also closes the host's tunnel.
- `UPDATE: src-tauri/src/ssh_install.rs` - `SshInstall.reach` picks `Bind::Loopback` for a tunnel and `Bind::All` for an open port.
- `UPDATE: src-tauri/src/lib.rs` - `Tunnels` managed; `ssh::tunnel::open_on_launch` after the host list window, beside `local_ahpd::start_on_launch`; the tunnels closed on `RunEvent::Exit`.
- `UPDATE: src-tauri/build.rs`, `src-tauri/capabilities/shell.json` - `ssh_tunnel_states`, and `open_ssh_tunnel` for Retry.
- `UPDATE: packages/shell/src/hosts/ipc.ts` - `kind: 'ssh'`, `SshHost.reach`, `sshTunnelStates()`, `openSshTunnel(id)`, `onTunnelsChanged()`.
- `UPDATE: packages/shell/src/hosts/HostList.tsx` - a down `ssh` host shows its tunnel's error, and Retry opens the tunnel again.
- `UPDATE: packages/shell/src/i18n.ts` - the tunnel's words, in en, pt-BR and es.

## Steps

1. `Tunnel::open(app, id)` reads the host's `SshHost`, connects with task 01's `connect()` and task 02's check, binds `127.0.0.1:<localPort>`, and for each connection it accepts opens a `direct-tcpip` channel to `127.0.0.1:<remotePort>` on the server (`channel_open_direct_tcpip`, check its 0.61 signature) and copies both ways with `tokio::io::copy_bidirectional` over the channel's stream.
   Whether `localPort` is chosen once and kept, or chosen free at each open, is open question 10 in the plan; the proposed answer keeps it, since ahpw keeps a host's pages under its address.
2. The host's `url` is `ws://127.0.0.1:<localPort>`, so `status.ts`, `page_url()` and the embedder connect to it as to any host, with the host's token from the vault.
3. `TunnelState` is `{ on, error }` per host, sent as `tunnels-changed`; while a tunnel is down its host is down, as decision 5 says, and the row shows the error: a server key refused (task 02), a sign-in that needs the person (open question 1), the server unreachable, or the local port taken.
4. A tunnel whose session drops is opened again after 2 s, then doubling to 60 s, and at once on Retry.
   A sign-in that needs the person, or a key refused, is not retried until they act.
5. `open_on_launch(app)` opens every `ssh` host's tunnel off the main thread, after the host list window, as `start_on_launch` does; on `RunEvent::Exit` each tunnel's listener and session are closed.
6. An open port is a `url` host at `ws://<server>:<remotePort>`, with the token `install_ssh_ahpd` read; once saved, the desk connects to it from the desk, and a failure says the port may be closed by a firewall on the server or between, and leaves the host saved and down.
7. The words: the tunnel's note "Reachable while the desk is open." (plan's Risks), and the open port's "The token travels unencrypted unless the server is behind TLS." (decision 5), shown in task 07's form; the row's tunnel errors here.

## Validation

- Cargo tests in `tunnel.rs` against task 01's `test_server.rs`, given a `direct-tcpip` handler that echoes: bytes written to the local port come back; two connections at once; the session closed sends `tunnels-changed` with `on: false` and opens again; the local port taken is the error.
- Cargo tests in `hosts.rs`: kind `ssh` round-trips with `reach`, `remotePort` and `localPort`; kind `ssh` with no `ssh` is refused; a `url` host keeps its `ssh` for Update.
- `pnpm test` and `pnpm lint` pass.
- By hand on dev02 only, with a host list in `AHP_DESK_CONFIG_DIR`: a tunnelled host connects and its app window opens; with the desk closed, nothing on dev02 listens beyond 127.0.0.1 (`ss -ltn`); restarting the desk opens the tunnel again and the window keeps its tabs; an open-port host connects from the desk.

## Resume
