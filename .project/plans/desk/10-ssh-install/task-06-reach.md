---
title: The desk reaches an ahpd over SSH through a tunnel on a kept local port, as host kind ssh, or through its open port, as a URL host
status: done
depends: [task-05-install.md]
layer: "src-tauri, packages/shell"
refs:
  - file:///github/ahpd/packages/server/src/commands/options.ts - `--host`: 127.0.0.1 unless told otherwise, and any other address needs a token, at lines 401-405 and 1108-1116
  - "[code://src-tauri/src/hosts.rs#L18-L23](../../../../src-tauri/src/hosts.rs#L18-L23) - `HostKind`, which gains `Ssh`"
  - "[code://src-tauri/src/hosts.rs#L274-L308](../../../../src-tauri/src/hosts.rs#L274-L308) - `local()` and `set_local()`, the pattern for a host whose address the desk sets"
  - "[code://src-tauri/src/local_ahpd.rs#L739-L761](../../../../src-tauri/src/local_ahpd.rs#L739-L761) - `start_on_launch` and `stop_on_quit`, the pattern for the tunnels at launch and quit"
  - "[code://src-tauri/src/lib.rs#L67-L94](../../../../src-tauri/src/lib.rs#L67-L94) - where launch and quit call them"
  - "[code://src-tauri/src/app_window.rs#L26-L30](../../../../src-tauri/src/app_window.rs#L26-L30) - `page_url()`, which gives a window its host's `url`"
  - "[code://packages/shell/src/hosts/HostList.tsx](../../../../packages/shell/src/hosts/HostList.tsx) - the row, which shows a tunnel's error while its host is down, and \"Sign in\" when the tunnel needs the person"
---

## Objective

An install chooses its reach, as the Answers row on the reach says.
Through the SSH connection: ahpd stays on the server's 127.0.0.1, and the host is kind `ssh`, whose `url` is a local port, chosen once and kept on the host (Answers: the local port), that the desk forwards to it over SSH while the desk runs.
An open port: ahpd listens on 0.0.0.0 with its token, and the host is kind `url` at `ws://<server>:<ahpd port>`, with its SSH details kept for Update (task 08) and Remove (task 10).

## Files

- `CREATE: src-tauri/src/ssh/tunnel.rs` - `Tunnels` (a Tauri state: host id to `Tunnel`), `open()`, `close()`, `forward()`, `open_on_launch()`, `pick_local_port()`, `TunnelState`, the event `tunnels-changed`, the commands `ssh_tunnel_states` and `ssh_sign_in`.
- `UPDATE: src-tauri/src/hosts.rs` - `HostKind::Ssh`; `SshHost` gains `reach` (`tunnel` or `port`), `remotePort` (Answers: the ahpd port), `folder` (Answers: the folder), `byDesk` (Answers: a found ahpd) and, for a tunnel, `localPort`; `save` refuses kind `ssh` with no `ssh` or no `localPort`; the `remove_host` command also closes the host's tunnel.
- `UPDATE: src-tauri/src/ssh_install.rs` - `SshInstall.reach` picks `Bind::Loopback` for a tunnel and `Bind::All` for an open port.
- `UPDATE: src-tauri/src/lib.rs` - `Tunnels` managed; `ssh::tunnel::open_on_launch` after the host list window, beside `local_ahpd::start_on_launch`; the tunnels closed on `RunEvent::Exit`.
- `UPDATE: src-tauri/build.rs`, `src-tauri/capabilities/shell.json` - `ssh_tunnel_states`, `open_ssh_tunnel` for Retry, `ssh_sign_in`, and `pick_ssh_local_port` for task 07.
- `UPDATE: packages/shell/src/hosts/ipc.ts` - `kind: 'ssh'`, `SshHost.reach`, `localPort`, `remotePort`, `folder`, `byDesk`, `sshTunnelStates()`, `openSshTunnel(id)`, `sshSignIn(id, secret)`, `pickSshLocalPort()`, `onTunnelsChanged()`.
- `UPDATE: packages/shell/src/hosts/HostList.tsx` - a down `ssh` host shows its tunnel's error, and Retry opens the tunnel again; one whose tunnel needs the person shows "Sign in", which asks the passphrase or the password in the row and opens the tunnel.
- `UPDATE: packages/shell/src/i18n.ts` - the tunnel's words, in en, pt-BR and es.

## Steps

1. `pick_local_port()` binds `127.0.0.1:0`, reads the port the OS gave, and closes it; the command `pick_ssh_local_port` gives it to task 07 once, when the host is added, and the host keeps it as `localPort` (Answers: the local port).
   It is never picked again for that host: Edit host leaves it as it is.
2. `Tunnel::open(app, id)` reads the host's `SshHost`, connects with task 01's `connect()` and task 02's check, binds `127.0.0.1:<localPort>`, and for each connection it accepts opens a `direct-tcpip` channel to `127.0.0.1:<remotePort>` on the server (`channel_open_direct_tcpip`, check its 0.61 signature) and copies both ways with `tokio::io::copy_bidirectional` over the channel's stream.
   A `localPort` taken by another program is the tunnel's error; the tunnel never moves to another port (Answers: the local port).
3. The host's `url` is `ws://127.0.0.1:<localPort>`, the same at every launch, so `status.ts`, `page_url()` and the embedder connect to it as to any host, with the host's token from the vault, and ahpw opens the host's own tabs and layout (Answers: the local port).
4. `TunnelState` is `{ on, error, needs }` per host, sent as `tunnels-changed`; while a tunnel is down its host is down, as the Answers row on the reach says, and the row shows the error: a server key refused (task 02), the server unreachable, or the local port taken.
   `needs` is `passphrase` or `password` when task 01's `connect()` failed with `SignInError::NeedsPerson`, as after a restart for a key with a passphrase or a password not kept (Answers: secrets held until quit); the row then shows "Sign in".
5. `ssh_sign_in(id, secret)` puts what the person typed in task 01's `Secrets` for that host, held until the desk quits and never on disk (Answers: secrets held until quit), and opens the tunnel; a secret refused is dropped and the row asks again.
6. A tunnel whose session drops is opened again after 2 s, then doubling to 60 s, and at once on Retry; it signs in with what `Secrets` holds, so it asks nothing while the desk runs (Answers: secrets held until quit).
   A sign-in that needs the person, or a key refused, is not retried until they act.
7. `open_on_launch(app)` opens every `ssh` host's tunnel off the main thread, after the host list window, as `start_on_launch` does; on `RunEvent::Exit` each tunnel's listener and session are closed, and `Secrets` goes with the process.
8. An open port is a `url` host at `ws://<server>:<remotePort>`, with the token `install_ssh_ahpd` read; once saved, the desk connects to it from the desk, and a failure says the port may be closed by a firewall on the server or between, and leaves the host saved and down.
   For an ahpd found on the server and not installed by the desk, the reach follows how it listens (Answers: how a found ahpd is reached): the tunnel, to its own port, is always offered; the open port only when its `ahpd status --json` address is beyond 127.0.0.1, and then the host's `url` is that address with the server's name.
9. The words: the tunnel's note "Reachable while the desk is open." (plan's Risks), and the open port's "The token travels unencrypted unless the server is behind TLS." (Answers: the reach), shown in task 07's form; the row's tunnel errors and "Sign in" here.

## Validation

- Cargo tests in `tunnel.rs` against task 01's `test_server.rs`, given a `direct-tcpip` handler that echoes: bytes written to the local port come back; two connections at once; the session closed sends `tunnels-changed` with `on: false` and opens again on the same local port, asking nothing when `Secrets` holds the passphrase; the local port taken is the error and no other port is used; with `Secrets` empty and a key with a passphrase, the state has `needs: passphrase`, and `ssh_sign_in` with it opens the tunnel.
- Cargo tests in `hosts.rs`: kind `ssh` round-trips with `reach`, `remotePort`, `localPort`, `folder` and `byDesk`; kind `ssh` with no `ssh` or no `localPort` is refused; an edit keeps `localPort`; a `url` host keeps its `ssh` for Update.
- `pnpm test` and `pnpm lint` pass.
- By hand on dev02 only, with a host list in `AHP_DESK_CONFIG_DIR`: a tunnelled host connects and its app window opens; with the desk closed, nothing on dev02 listens beyond 127.0.0.1 (`ss -ltn`); restarting the desk opens the tunnel again on the same local port and the window keeps its tabs; a host signed in with a key with a passphrase shows "Sign in" after the restart, and opens once it is typed; an open-port host connects from the desk.

## Resume

- 2026-10-18: `src-tauri/src/ssh/tunnel.rs` has `Tunnels` (`open`, `retry`, `sign_in`, `close`, `close_all`, `states`), `TunnelSpec`, `spec_of()`, `TunnelState`, `pick_local_port()`, `open_on_launch()`, `manage()`, the event `tunnels-changed`, and the commands `ssh_tunnel_states`, `open_ssh_tunnel`, `ssh_sign_in` and `pick_ssh_local_port`.
- `HostKind::Ssh` and `Reach` (`tunnel`, `port`); `SshHost` and `SshHostView` have `reach`, `remotePort`, `folder`, `byDesk` and `localPort`; an edit keeps the host's `localPort`; `remove_host` closes the host's tunnel; `SshInstall.reach` replaces `bind`.
- The tunnel's core takes a callback for each change, so it runs under tests against the test server; `manage()` makes that callback emit `tunnels-changed`.
- `Secrets` is an `Arc` in `HostStore`, so a tunnel keeps signing in with what was typed.
- A tunnel retries a dropped session after 2 s, doubling to 60 s, or at once on Retry; a sign-in that needs the person, a refused key and a refused sign-in wait for them.
- The row of a down `ssh` host shows `tunnel down` with its error as the title, Retry opening the tunnel, or Sign in, which asks the passphrase or the password in the row (`tunnel.*` words in three languages); `useTunnels()` keeps the states.
- The test server echoes a `direct-tcpip` channel, and `drop_sessions` shuts each connection's socket, since `run_stream` runs the session in the background.
- Step 8's open port is saved by task 07's form as a `url` host with its `ssh` details; the choice for a found ahpd, by how it listens, is task 07's form too.
- Verified: `cargo test` (117: 4 for the tunnel, a forward with two connections, a dropped session reopened on the same port asking nothing, the local port taken, a passphrase needed and Sign in; 2 in `hosts.rs`), the suite five times in a row, `cargo clippy --all-targets -- -D warnings`, `pnpm typecheck`, `pnpm test`.
- By hand, `ssh_a_real_tunnel` on the test server Softov named: a tunnel to the server's own SSH port read its `SSH-2.0-` banner, with only the key Softov matched trusted.
- The window checks of this task (a tunnelled host's app window, the same port after a restart, Sign in after a restart, an open-port host) need a host added from the form, and run with task 07's.
