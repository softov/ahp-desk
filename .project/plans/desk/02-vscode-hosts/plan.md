---
title: VS Code as a host
domain: desk
status: draft
priority: medium
created: 2026-10-14
revalidated: 2026-10-14
requires:
  - plans/desk/01-first-build-p3-local-ahpd/plan.md
  - plans/desk/01-first-build-p10-ssh-install/plan.md
refs:
  - research/vscode-agent-host-endpoints.md - the CLI, the endpoint registry, the pipe that speaks WebSocket, and the relay that carried AHP
  - "[code://src-tauri/src/local_ahpd.rs](../../../../src-tauri/src/local_ahpd.rs) - how the desk finds, starts and stops a local server; the pattern for `code agent host`"
  - "[code://src-tauri/src/hosts.rs](../../../../src-tauri/src/hosts.rs) - `HostKind`, which gains a VS Code kind"
  - "[code://packages/shell/src/hosts/KindField.tsx](../../../../packages/shell/src/hosts/KindField.tsx) - Add host's Kind"
  - "[code://packages/ahpw-embedder/src/embedder.ts](../../../../packages/ahpw-embedder/src/embedder.ts) - what a window is handed: the host's address and token"
  - file:///github/ahpw/src/connection/protocol-versions.ts - ahpw offers AHP 0.10.0, VS Code 1.141's version
---

## Goal

VS Code's agent host is a host in the desk, as an ahpd is.
On this computer, the desk reaches the agent host inside the open VS Code, so its sessions open in ahpw too, and it can start a `code agent host` of its own.
On a remote machine, the desk reaches VS Code's agent host there over SSH.

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
or: code agent host --port <p> --connection-token-file <f> -> ws://127.0.0.1:<p>
```

### Gaps

- `Not found: a bridge in the desk - the relay was a test script, src-tauri has none.`
- `Not checked: the registry on Linux and macOS, and code agent endpoints over SSH on a server with VS Code Server.`
- The pipe's name changes when VS Code restarts, so a host cannot keep it.

## Decisions locked in

No decision is locked yet; the open questions below come first.

| What | Source | Task |
| --- | --- | --- |
| The scope is VS Code on this computer, the open editor's host and a desk-started one, and VS Code on a remote machine. | Softov, 2026-10-14: "all as a new plan" | - |
| The editor's pipe speaks WebSocket, so the bridge relays bytes and translates nothing. | research/vscode-agent-host-endpoints.md | - |

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
| 04 - a desk-started `code agent host`, started and stopped as the local ahpd is | todo | 02 |
| 05 - Add host: the VS Code kind | todo | 03, 04 |
| 06 - remote: `code agent endpoints` over SSH and a forward to its socket | todo | 01, p10 |
| 07 - end to end: the editor's sessions in ahpw, here and on dev02 | todo | 05, 06 |

## Risks and tradeoffs

- The registry and `code agent endpoints` are VS Code's, not a published contract; the desk reads them as VS Code 1.141 writes them, and a change there breaks the find.
- The bridge holds a loopback port open while the host is used; anything on the machine can dial it, and the token is what keeps it out, as for the local ahpd.
- The editor's host is there only while VS Code runs.

## Resume state

- **Done so far:** the plan and its research, 2026-10-14.
- **Next action:** answer the open questions, then write the task files.
- **Open questions:**
  1. When VS Code is closed, what does the editor's host do? - proposed: it shows down until VS Code runs again; the desk does not start anything in its place.
  2. Does the desk-started `code agent host` stay a host of its own, apart from the editor's? - proposed: yes, a second kind, started and stopped like the local ahpd.
  3. On a remote machine, which host does the desk reach? - proposed: the VS Code Server editor's, through `code agent endpoints` over SSH and a stream-local forward; else a `code agent host` it starts there, through a port forward.
  4. Is the bridge one port per host for as long as the desk runs, or opened per window? - proposed: one per host, while the desk runs, so a window reconnects to the same address.
- **Watch out for:** never stop a host with `code agent kill`, which kills the running agent host process tree, the editor's too.

## Final verification checklist

- [ ] The open VS Code's sessions open in ahpw through the desk, and still do after VS Code restarts.
- [ ] A desk-started `code agent host` starts, connects and stops.
- [ ] On dev02, VS Code's agent host is reached over SSH.
- [ ] `plans/index.md` updated.
