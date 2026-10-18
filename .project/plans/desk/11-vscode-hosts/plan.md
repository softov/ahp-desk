---
title: VS Code as a host
domain: desk
status: draft
priority: medium
created: 2026-10-13
revalidated: 2026-10-16
requires:
  - plans/desk/03-local-ahpd/plan.md
  - plans/desk/10-ssh-install/plan.md
refs:
  - research/vscode-agent-host-endpoints.md - the CLI, the endpoint registry, the pipe that speaks WebSocket, and the relay that carried AHP
  - research/vscode-remote-tunnel-agent-host.md - a Remote Tunnel's agent host port, its selection handshake, and the ways in
  - "[code://src-tauri/src/local_ahpd.rs](../../../../src-tauri/src/local_ahpd.rs) - how the desk finds, starts and stops a local server; the pattern for `code agent host`"
  - "[code://src-tauri/src/hosts.rs](../../../../src-tauri/src/hosts.rs) - `HostKind`, which gains a VS Code kind"
  - "[code://packages/shell/src/hosts/KindField.tsx](../../../../packages/shell/src/hosts/KindField.tsx) - Add host's Kind"
  - "[code://packages/ahpw-embedder/src/embedder.ts](../../../../packages/ahpw-embedder/src/embedder.ts) - what a window is handed: the host's address and token"
  - file:///github/ahpw/src/bridges/protocol-versions.ts - ahpw offers AHP 0.10.0, VS Code 1.141's version
---

## Goal

VS Code's agent host is a host in the desk, as an ahpd is.
On this computer, the desk reaches the agent hosts that VS Code already runs, so their sessions open in ahpw too.
On a remote machine, the desk reaches the editor's agent host there over SSH.
Through a VS Code Remote Tunnel, the desk reaches the agent host that the tunnel serves.

## Reconnaissance

The files read and the patterns to reuse are the `refs` above, each with its note; the research note holds what was checked by hand.

### Searches performed

- `code agent --help`, `code agent host --help` - the commands and their flags, VS Code 1.141.0.
- `code agent endpoints` - the open editor's host on a named pipe, with its token.
- `rg -i "vs ?code.{0,60}(agent host|websocket|port)" ahpw ahpd ahp-desk` - ahpw already offers AHP 0.10.0; ahpd's research reads VS Code's SSH and tunnel entries.

### Runtime path

```
Add host "VS Code" -> code agent endpoints (here, or over SSH)
  -> the editor's endpoint: a pipe or a socket, and its token
  -> desk bridge: 127.0.0.1:<port> relayed byte for byte to it
  -> Host { kind: vscode, url: ws://127.0.0.1:<port>, token } -> app window -> ahpw (AHP 0.10.0)
or, through a Remote Tunnel: devtunnel connect <tunnel> -> 127.0.0.1:31546
  -> desk bridge: /agent-host/select, the selection handshake, then the frames relayed
  -> Host { kind: vscode, url: ws://127.0.0.1:<port> } -> app window -> ahpw
```

### Gaps

- `Not found: a bridge in the desk - the relay was a test script, src-tauri has none.`
- `Not checked: the registry on Linux and macOS, and code agent endpoints over SSH on a server with VS Code Server.`
- The pipe's name changes when VS Code restarts, so a host cannot keep it.

## Answers

| What | Source | Task |
| --- | --- | --- |
| The scope is VS Code on this computer, the open editor's host and a desk-started one, and VS Code on a remote machine. | Softov, 2026-10-13: "all as a new plan" | - |
| The editor's pipe speaks WebSocket, so the bridge relays bytes and translates nothing. | research/vscode-agent-host-endpoints.md | - |
| When VS Code is closed, the editor's host shows down until VS Code runs again. The desk starts nothing in its place. | Softov, 2026-10-16: "Shows down" | 03 |
| The desk does not start or stop a `code agent host` for now. VS Code controls its own hosts, and the desk connects to the ones it runs. This narrows the scope row above. | Softov, 2026-10-16: "vscode has its own control. we dont need to control code agent host for now. but would be good to connect to it." | 04 |
| On a remote machine over SSH, the desk reaches the VS Code Server editor's agent host, found with `code agent endpoints`. An ahpd on that machine is a separate host, added through plan 10. | Softov, 2026-10-16: "Editor's; ahpd is plan 10" | 06 |
| The bridge is one loopback port per host, kept while the desk runs, so a window reconnects to the same address. | Softov, 2026-10-16: "one bridge per host" | 01 |
| A VS Code Remote Tunnel is a way in, beside this computer and SSH. | Softov, 2026-10-16: "on plan 11 tunnel, ok" | 08 |
| The tunnel is reached first through the `devtunnel` CLI; once that works, the desk moves to the dev-tunnels Rust SDK. | Softov, 2026-10-16: "A first, devtunnel, proved we migrate" | 08, 09 |
| The bridge does a tunnel's selection handshake, so ahpw speaks plain AHP to every host. | Softov, 2026-10-16: "the handshake is by the bridge" | 08 |

## Proposed architecture

- **Data flow** - `src-tauri/src/vscode.rs` runs `code agent endpoints` and `code agent host`; `src-tauri/src/bridge.rs` listens on a loopback port per host and relays each connection to the pipe or socket.
- **Event flow** - the bridge's port is the host's address; `hosts-changed` when it moves.
- **State flow** - `hosts.json` keeps a VS Code host's kind and how it is reached; the token is read from the endpoint each time, never kept.
- **Layer responsibilities** - `src-tauri`: the CLI, the bridge, the SSH forward · `packages/shell`: Add host's VS Code kind · ahpw: nothing new.
- **Source-of-truth files** - `src-tauri/src/vscode.rs`, `src-tauri/src/bridge.rs`.

## Tasks

These task files are written when this plan moves to `planned`.

| Task | Status | Depends on |
| --- | --- | --- |
| 01 - the bridge: a loopback port relayed to a named pipe or a Unix socket | todo | - |
| 02 - find VS Code: `code` on PATH, `code agent endpoints`, the editor's endpoint | todo | - |
| 03 - the editor's host as a host kind, through the bridge, followed across VS Code restarts | todo | 01, 02 |
| 04 - a desk-started `code agent host`, started and stopped as the local ahpd is | dropped | 02 |
| 05 - Add host: the VS Code kind | todo | 03 |
| 06 - remote: `code agent endpoints` over SSH and a forward to its socket | todo | 01, plan 10 |
| 07 - end to end: the editor's sessions in ahpw, here, on dev02, and through a Remote Tunnel | todo | 05, 06, 08 |
| 08 - tunnel: `devtunnel connect` to a VS Code Remote Tunnel, and the selection handshake in the bridge | todo | 01 |
| 09 - tunnel: the dev-tunnels Rust SDK in place of the `devtunnel` CLI | todo | 08 |

## Risks and tradeoffs

- The registry and `code agent endpoints` are VS Code's, not a published contract; the desk reads them as VS Code 1.141 writes them, and a change there breaks the find.
- The bridge holds a loopback port open while the host is used; anything on the machine can dial it, and the token is what keeps it out, as for the local ahpd.
- The editor's host is there only while VS Code runs.
- A tunnel needs a dev-tunnels `connect` token for the owner's account.
  Whether the service accepts one from a desk-owned OAuth app is not known, so task 08 starts with the `devtunnel` CLI's own sign-in.
- The `devtunnel` CLI forwards the agent host on the fixed local port 31546, which two tunnels, or a tunnel and another server, cannot share.

## Resume state

- **Done so far:** the plan and its research, 2026-10-13; the Remote Tunnel research and every open question answered in *Answers*, 2026-10-16.
- **Next action:** write the task files, then move the plan to `planned`.
- **Open questions:** none.
- **Watch out for:** never stop a host with `code agent kill`, which kills the running agent host process tree, the editor's too.

## Final verification checklist

- [ ] The open VS Code's sessions open in ahpw through the desk, and still do after VS Code restarts.
- [ ] On dev02, VS Code's agent host is reached over SSH.
- [ ] Through a VS Code Remote Tunnel, the editor's sessions open in ahpw.
- [ ] `plans/index.md` updated.
