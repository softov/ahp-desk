---
title: How the desk can reach VS Code's agent host
date: 2026-10-14
refs:
  - https://github.com/microsoft/vscode/blob/832cf23c588/src/vs/platform/agentHost/common/agentHostEndpointRegistry.ts - the endpoint registry `code agent endpoints` prints
  - https://github.com/microsoft/vscode/blob/832cf23c588/src/vs/platform/agentHost/node/sshRemoteAgentHostHelpers.ts#L286-L291 - a remote command prints `ws://127.0.0.1:PORT[?tkn=TOKEN]`
  - file:///github/ahpd/.project/research/how-vscode-offers-a-dev-container.md - VS Code's SSH, Tunnel and WSL entries, read by ahpd
---

Checked on Softov's Windows machine with VS Code 1.141.0, on 2026-10-14.

## The CLI

`code agent` has `host`, `ps`, `stop`, `kill`, `logs` and `endpoints`.
`code agent host` starts an agent host server: `--host` (default `localhost`), `--port` (0 picks one), `--connection-token`, `--connection-token-file`, `--without-connection-token`, `--server-data-dir`, `--user-data-dir`, `--foreground`, `--new-instance`, `--replace`, `--tunnel`.
`code agent endpoints` prints every live agent host as one JSON document, with each one's connection token; its help says it is meant for an already authenticated transport such as SSH.
`code agent kill` kills "the running agent host process tree", so a test never uses it to stop a host of its own.

## What was seen

- A standalone `code agent host --foreground --new-instance --port 9330 --connection-token-file <f>`, with its data folders in a scratch folder, printed `ws://localhost:9330?tkn=...` and answered AHP `initialize` with protocol `0.10.0`; ahpw's client reached it as a URL host would.
- `code agent endpoints` listed the open editor's own agent host: `type: "editor"`, its `pid`, an `instanceId`, `protocolVersion: "0.10.0"`, a `connectionToken`, and `endpoint: { type: "socket", path: "\\.\pipe\vscode-agent-host-<hash>-<instanceId>" }`.
- That pipe answers an HTTP upgrade with `101 Switching Protocols`: it speaks WebSocket over the pipe.
- A plain byte relay from `127.0.0.1:9331` to the pipe carried AHP through: `initialize` answered `0.10.0`, and `listSessions` listed the editor's 2 sessions.

## What follows

- The editor's own host needs no protocol work in the desk: a local TCP port relayed byte for byte to the pipe is enough, and the client dials `ws://127.0.0.1:<port>/?tkn=<token>`.
- The pipe's name carries the editor's `instanceId`, so it changes when VS Code restarts; the desk reads `code agent endpoints` again rather than keeping the path.
- On Linux and macOS the same registry should name a Unix socket; over SSH, a socket is reached with an SSH stream-local forward, and `code agent endpoints` run over SSH names it. Not checked yet.
