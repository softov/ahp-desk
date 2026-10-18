---
title: How the desk can reach VS Code's agent host through a Remote Tunnel
date: 2026-10-16
refs:
  - https://code.visualstudio.com/docs/remote/tunnels - Remote Tunnels: dev tunnels, sign-in with the same GitHub or Microsoft account
  - https://code.visualstudio.com/docs/agents/remote-agent-sessions - the Agents window reaches remote agent hosts over SSH or a tunnel
  - https://github.com/microsoft/vscode/blob/19fd99357ad/cli/src/constants.rs#L14-L35 - `CONTROL_PORT` 31545, `AGENT_HOST_PORT` 31546, protocol versions 5 and 6
  - https://github.com/microsoft/vscode/blob/19fd99357ad/cli/src/commands/args.rs#L977-L983 - `--delegate-to-editor`: serve only the editor's own agent host
  - https://github.com/microsoft/vscode/blob/19fd99357ad/src/vs/platform/remoteTunnel/node/tunnelProcessCoordinator.ts#L298-L300 - "Turn on Remote Tunnel Access" runs `code tunnel --delegate-to-editor`
  - https://github.com/microsoft/vscode/blob/19fd99357ad/cli/src/tunnels/agent_host.rs#L1549-L1660 - port 31546: `/agent-host/select` goes to the gateway, any other path is proxied with the host's own token
  - https://github.com/microsoft/vscode/blob/19fd99357ad/cli/src/tunnels/agent_host.rs#L1729-L1830 - the selection handshake
  - https://github.com/microsoft/vscode/blob/19fd99357ad/src/vs/platform/agentHost/common/tunnelAgentHostConnector.ts#L111-L131 - the client's `?tkn=`, base64url of SHA-256 of the tunnel id
  - https://github.com/microsoft/vscode/blob/19fd99357ad/src/vs/platform/agentHost/node/tunnelAgentHostService.ts#L83-L93 - a `connect` token, then `TunnelRelayTunnelClient`
  - https://github.com/microsoft/dev-tunnels - the dev-tunnels SDKs, MIT: C#, TypeScript, Java, Go, Rust
  - file:///github/ahpd/packages/tunnel-devtunnel - ahpd published through a dev tunnel with the `devtunnel` CLI, the other way round
---

Read from VS Code's source at commit `19fd99357ad` (main, 2026-10-16) and its docs; nothing here was tried against a live tunnel yet.
Softov runs VS Code with "Remote Tunnel Access" on, for the Agents window.

## What a Remote Tunnel serves

`code tunnel` hosts a Microsoft dev tunnel labelled `vscode-server-launcher`, `protocolv6` and the machine's name.
It forwards two ports: 31545, the VS Code server that vscode.dev opens, and 31546, the agent host.
Only the owner's GitHub or Microsoft account can connect; a client needs a dev-tunnels token with the `connect` scope for that account.
VS Code gets that token through its own first-party OAuth app.

## Port 31546

A WebSocket upgrade to `/agent-host/select` goes to the selection gateway (protocol 6).
Any other path is proxied straight to the agent host (protocol 5), and the CLI adds the host's own token.
With `--delegate-to-editor`, which "Turn on Remote Tunnel Access" uses, the direct path answers `503`, so only the gateway works.

The gateway's handshake, on one WebSocket:

1. The gateway sends an inventory: `endpoints[]` with `type`, `instanceId` and `pid`, never a token.
2. The client answers `{"instanceId": "..."}` or `{"newDedicated": true}`.
3. The gateway dials that endpoint with its own token and answers `{ "ok": true, "selected": { ... } }`.
4. From then on it relays frames, and AHP starts on the same socket.

The client's `?tkn=` is base64url of SHA-256 of the tunnel id, so it is not a secret; the relay's `connect` access is the boundary.

## Ways in for the desk

- **The `devtunnel` CLI.** `devtunnel user login -g` with the account VS Code uses, `devtunnel list --labels vscode-server-launcher --json`, then `devtunnel connect <id>` forwards 31546 to this computer. The bridge does the handshake. It needs the CLI installed, and the local port 31546 is fixed.
- **The dev-tunnels Rust SDK in `src-tauri`.** The cleanest result, and VS Code's own CLI uses it. A token the service accepts is the open part: VS Code's OAuth client is its own, and a desk-owned OAuth app is untested.
- **Anonymous access.** Not taken: anyone with the URL could start agent sessions.

## Not known yet

- Whether `devtunnel connect`, signed in with the same account, opens a tunnel that `code tunnel` made.
- Whether the dev-tunnels service accepts a GitHub token from a third-party OAuth app.
- Whether the VS Code Server license limits a client that is not VS Code.
