---
title: The host list screen adds, edits, tests, removes and opens URL hosts
status: done
depends: [task-02-desk-client.md, task-03-host-store.md, task-05-app-window.md]
layer: "packages/shell"
refs:
  - file:///github/ahp-desk/mockups/02-hosts.md - the host list and Add host
  - file:///github/ahpapp/src/host-context.tsx - `normalizeUrl` at line 100, and the MultiHostClient per host at lines 583-639
  - file:///github/ahpapp/src/transport.ts - `openHostSocket` with a Bearer header and the `?tkn=` fallback, at lines 410-428
  - npm://@microsoft/agent-host-protocol@^1.0.0 - `MultiHostClient` and `WebSocketTransport`
---

## Objective

The `shell` window shows the host list from mockup 02, for the kind URL only.
Each host shows connected, starting or down from a live AHP connection.
Add host saves a name, an address and a token; Test connects once and shows the result.

## Files

- `CREATE: packages/shell/src/hosts/HostList.tsx` - the list, its status dots, Open, and the `⋯` menu with Edit and Remove.
- `CREATE: packages/shell/src/hosts/HostForm.tsx` - Add host and Edit host for a URL host, with Test and Save.
- `CREATE: packages/shell/src/hosts/status.ts` - one `MultiHostClient` over the host list, which publishes each host's state.
- `CREATE: packages/shell/src/hosts/url.ts` - `normalizeUrl`, which adds `ws://` when the address has no scheme.
- `CREATE: packages/shell/src/hosts/url.test.ts` - cases for `normalizeUrl`.
- `UPDATE: packages/shell/src/App.tsx` - show `HostList`.
- `UPDATE: packages/shell/package.json` - depend on `@ahpd/desk-client` through the workspace, and on `@microsoft/agent-host-protocol@^1.0.0`.

## Steps

1. Read the hosts with `hosts()` and follow `onHostsChanged()`.
2. In `status.ts`, add each host to one `MultiHostClient`, with the token from `hostToken(id)`.
3. Remove a host from the client when it leaves the list, and add or reconnect it when it changes.
4. Show the dot from the host's state: `●` connected, `◐` connecting, `○` down.
5. Show `[Open]` for a connected host, and `[Retry]` for a host that is down.
6. Call `openHost(id)` on Open.
7. In `HostForm`, call `save_host` with `kind: 'url'`, the normalized address and the token.
8. On Test, connect once with the typed address and token, and show the server's name and version or the error.
9. Show a warning on a host with `tokenInFile: true`: "The token is in a plain file."
10. Ask before Remove, with the host's name in the question.

## Validation

- Run `pnpm --filter @ahp-desk/shell test`; the `normalizeUrl` cases pass.
- With an ahpd on `ws://127.0.0.1:9187`, add it as a URL host; its dot turns `●`.
- Stop that ahpd; the dot turns `○` and `[Retry]` shows.
- Type a wrong token and press Test; the error shows, and `hosts.json` does not change.
- Open the host; ahpw opens in an app window.
- Check each screen against mockup 02 at 100% and 150% display scale.

## Resume

- Done 2026-10-11 against an ahpd 0.10.0 on 127.0.0.1:9199 with a connection token.
- A wrong token's Test says "Could not connect. Check the address and the token." and `hosts.json` does not change; the right one says "Connected: ahpd 0.10.0".
- Saved, the row turns ● with the server's name; stopped, it turns ○ with Retry; Retry after a restart turns it ● again; Open opens ahpw; Remove asks, then removes the host and closes its window.
- The token goes as `?tkn=`, because a webview's WebSocket cannot set a header; ahpd reads either.
- The desk offers only protocol 1.0.0, as `MultiHostClient` does; an ahpd 0.7.0 answers `-32005`, shown as "This host speaks an older protocol than 1.0.0. Update its server."
- The dots are drawn in CSS: the ● glyph rendered too small in the system font.
- Not checked: 150% display scale, and the "token is in a plain file" warning on screen (the flag is covered by a `hosts` test).
