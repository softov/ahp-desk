---
title: ahpw connects to the window's host with its token
status: done
depends: [task-02-ahpw-depends-on-desk-client.md]
layer: "ahpw"
refs:
  - file:///github/ahpw/src/connection/data.ts - `connect()` at lines 325-335
  - file:///github/ahpw/src/connection/socket.ts - `socketUrl` at lines 10-14
---

## Objective

On the desk, ahpw's one host is the window's host: its address from `windowHost()` and its token from `hostToken()`, read again on each connect.

## Files

- `UPDATE: ahpw/src/connection/socket.ts` - `hostSocketUrl(url, token)`: a host's own address with `?tkn=`.
- `UPDATE: ahpw/src/connection/socket.test.ts` - its cases.
- `UPDATE: ahpw/src/connection/data.ts:325-335` - in `connect()`, a desk branch for the `label` and the `transportFactory`.
- `UPDATE: ahpw/src/connection/data.ts` - in `ahpProvider.load`, follow `onHostsChanged`; in `unload`, stop.

## Steps

1. Write `hostSocketUrl(url, token)`, which adds `?tkn=` or `&tkn=` as `socketUrl` does.
2. When `onDesk`, `connect()` labels the host with `windowHost()`'s name.
3. When `onDesk`, the `transportFactory` reads `windowHost()` and `hostToken(id)` and connects to `hostSocketUrl(host.url, token)`, under the same `readingServerInfo(offering(observing(...)))` wrappers.
4. In `load`, follow `onHostsChanged`; when the window's host's address differs from the one connected, call `reconnect()`.
5. Outside the desk, nothing changes.

## Validation

- `socket.test.ts`: a URL with no query, one with a query, and no token.
- Run `pnpm test` in ahpw; it passes.
- In the desk, open a host; ahpw's status bar says connected, and its sessions list loads.

## Resume

- Done 2026-10-11. ahpw on the desk connects to "ahpd 0.10.0" with the host's token and lists its sessions.
