---
title: ahpw on the desk
domain: desk
status: draft
priority: high
created: 2026-10-11
revalidated: 2026-10-11
requires:
  - plans/desk/01-first-build-p1-shell-and-url-hosts/plan.md
decisions:
  - decisions/ahpw-shows-one-host-at-a-time-on-the-desk.md
  - decisions/the-shell-contract-is-a-shared-npm-package.md
refs:
  - file:///github/ahpw/src/connection/data.ts - `connect()` at lines 325-335 builds the one host from window.location; `HOST` at line 59
  - file:///github/ahpw/src/connection/socket.ts - `socketUrl` at lines 10-14 takes a location-like object and puts the token in `?tkn=`
  - file:///github/ahpw/src/api.ts - `readServer()` fetches ./ahpw.json at lines 42-51; `serverHoldsToken()` at line 32
  - file:///github/ahpw/src/token-provider.ts - `restoreSession()` at lines 82-93 decides the sign-in wall
  - file:///github/ahpw/src/App.tsx - the Limen wall at line 90, and registerShell at lines 55-75
  - file:///github/ahpw/src/shell.tsx - the titlebar surface at lines 111-149
---

## Goal

ahpw, in an app window of the desk, connects to the window's host with the host's token.
It skips its sign-in wall, because the shell holds the token.
Its title bar drags the window, has the window controls, and switches the host.
ahpw in a browser behaves as it does today.

## Reconnaissance

The files read and the patterns to reuse are the `refs` above, each with its note.

### Searches performed

- `rg "window.location" ahpw/src` - only `connection/data.ts:329-330`.
- `rg "fetch\(" ahpw/src` - `api.ts:44` for `./ahpw.json` and `api.ts:67` for `/api`, which no code reaches at runtime.
- `rg "__TAURI|electron|postMessage" ahpw` - `Not found: any embedding hook - searched those terms in ahpw/src.`

### Runtime path

```
App.tsx restoreSession -> desk branch -> SIGNED_IN -> registerShell -> ahpProvider.load
  -> connect() -> windowHost() + hostToken() -> MultiHostClient.addHost(HOST, url + token) -> WebSocket
```

### Gaps

- `connect()` reads only `window.location`.
- `restoreSession()` shows the wall when no token is in localStorage.
- `readServer()` fetches `./ahpw.json`, which the `ahpw` scheme does not serve.
- The title bar has no drag region and no window controls.

## Decisions locked in

| # | Decision | Rationale / source |
| --- | --- | --- |
| 1 | [ahpw shows one host at a time on the desk](../../../decisions/ahpw-shows-one-host-at-a-time-on-the-desk.md) | Softov, 2026-10-11 |
| 2 | [The shell's contract is a shared npm package](../../../decisions/the-shell-contract-is-a-shared-npm-package.md) | Softov, 2026-10-11 |

| What | Source | Task |
| --- | --- | --- |
| On the desk, `HOST` stays `'daemon'`; a switch of host replaces the window's host and reloads. | Decision 1 | 02, 04 |
| The host switcher lists every host, each with its status dot. | Softov, asked "which hosts does the host switcher list?": "Every host, with its status dot (Recommended)" | 04 |
| ahpw depends on `@ahpd/desk-client`, linked to `../ahp-desk/packages/desk-client` until 0.1.0 is on npm. | (defaulted) | 01 |

## Proposed architecture

- **Data flow** - When `isDesk()`, `connect()` takes the URL from `windowHost()` and the token from `hostToken()`. Otherwise it keeps `window.location`.
- **Event flow** - `onHostsChanged` reconnects when the window's host changes its address or token.
- **State flow** - On the desk, ahpw writes no token to localStorage.
- **Layer responsibilities** - ahpw: the desk branch in `connection/`, `token-provider.ts` and the title bar · ahp-desk: nothing new.
- **Source-of-truth files** - `ahpw/src/connection/data.ts`.

## Tasks

These task files are written when this plan moves to `planned`.

| Task | Status | Depends on |
| --- | --- | --- |
| 01 - depend on desk-client | todo | - |
| 02 - connect to the window's host | todo | 01 |
| 03 - skip the sign-in wall on the desk | todo | 01 |
| 04 - title bar with drag, window controls and the host switcher | todo | 01 |
| 05 - end to end from the desk, with both sources | todo | 02, 03, 04 |

## Risks and tradeoffs

- The change is in another repository with its own release. ahpw publishes a new `@ahpd/web`, and ahp-desk moves its pin.
- `socketUrl` puts the token in the query string. On the desk, a Bearer header is possible only if the SDK transport allows it.

## Resume state

- **Done so far:** nothing.
- **Next action:** write the task files, then task 01.
- **Open questions:** none.
- **Watch out for:** ahpw's dev mode uses the `/ahp` proxy path; on the desk, connect straight to the host's URL.

## Final verification checklist

- [ ] ahpw in an app window connects to a URL host with a token, with `AHP_DESK_AHPW=local` and with `npm`.
- [ ] ahpw in a browser through `ahpw serve` still works.
- [ ] `pnpm typecheck` and `pnpm test` pass in ahpw.
- [ ] `plans/index.md` updated.
