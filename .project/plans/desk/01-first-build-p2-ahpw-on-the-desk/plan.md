---
title: ahpw on the desk
domain: desk
status: active
priority: high
created: 2026-10-11
revalidated: 2026-10-11
requires:
  - plans/desk/01-first-build-p1-shell-and-url-hosts/plan.md
decisions:
  - decisions/ahpw-shows-one-host-at-a-time-on-the-desk.md
  - decisions/ahpw-owns-its-embedding-contract.md
  - decisions/the-host-list-window-publishes-host-status.md
  - decisions/the-app-window-draws-its-own-rounded-frame.md
refs:
  - file:///github/ahpw/src/connection/data.ts - `connect()` at lines 325-335 builds the one host from window.location; `HOST` at line 59
  - file:///github/ahpw/src/connection/socket.ts - `socketUrl` at lines 10-14 takes a location-like object and puts the token in `?tkn=`
  - file:///github/ahpw/src/connection/protocol-versions.ts - `offering()`, which offers every protocol version the SDK supports
  - file:///github/ahpw/src/api.ts - `readServer()` fetches ./ahpw.json at lines 42-51
  - file:///github/ahpw/src/token-provider.ts - `restoreSession()` at lines 82-93 decides the sign-in wall
  - file:///github/ahpw/src/App.tsx - the Limen wall at line 98, and registerShell at lines 55-75
  - file:///github/ahpw/src/shell.tsx - the titlebar mounts at lines 111-149
  - file:///github/ahpw/src/sessions/activity.ts - collides with `Activity.tsx` on a case-insensitive file system
  - file:///github/opendoop/packages/tauri/src-tauri/src/lib.rs - pood's frameless window at lines 97-160
  - file:///github/opendoop/packages/web/src/components/shell/TitleBar.tsx - pood's window controls at lines 468-540
---

## Goal

ahpw, in an app window of the desk, connects to the window's host with the host's token, with no sign-in wall.
The window has no OS border, round corners when not maximized, and ahpw's title bar drags it and has minimize, maximize and close.
The title bar switches the window to another host, each listed with its status dot.
ahpw in a browser behaves as it does today.
The work in ahpw is on its branch `desk`.

## Reconnaissance

The files read and the patterns to reuse are the `refs` above, each with its note.

### Searches performed

- `rg "window.location" ahpw/src` - only `connection/data.ts:329-330`.
- `rg "fetch\(" ahpw/src` - `api.ts:44` for `./ahpw.json`.
- `rg "__TAURI|electron|postMessage" ahpw/src` - `Not found: any embedding hook.`
- `rg "drag-region|startDragging|tauri" scena/dist` - `Not found`: scena's title bar is `div[data-surface="titlebar"]` with no drag support, so ahpw adds it.
- `vite build` in ahpw on Windows - fails with `MISSING_EXPORT "Activity"`: `src/sessions/Activity.tsx` and `activity.ts` differ only in case.

### Runtime path

```
App.tsx restoreSession -> isDesk() -> SIGNED_IN -> registerShell -> ahpProvider.load
  -> connect() -> windowHost() + hostToken() -> MultiHostClient.addHost(HOST, host url + ?tkn=) -> WebSocket
shell window status.ts -> invoke('publish_host_statuses') -> Rust keeps the map -> emit('hosts-status')
  -> ahpw HostSwitcher: hostStatuses() + onHostStatus()
```

### Gaps

- `connect()` reads only `window.location`.
- `restoreSession()` shows the wall when no token is in localStorage.
- The title bar has no drag, no window controls and no host switcher.
- The app window has the OS accent border and square corners.
- ahpw does not build on Windows or macOS.
- The desk's status client offers only protocol 1.0.0, so a host on 0.9.0 shows down.

## Decisions locked in

| # | Decision | Rationale / source |
| --- | --- | --- |
| 1 | [ahpw shows one host at a time on the desk](../../../decisions/ahpw-shows-one-host-at-a-time-on-the-desk.md) | Softov, 2026-10-11 |
| 2 | [ahpw owns its embedding contract, and the desk injects the embedder](../../../decisions/ahpw-owns-its-embedding-contract.md) | Softov, 2026-10-11 |
| 3 | [The host list window publishes each host's status](../../../decisions/the-host-list-window-publishes-host-status.md) | Softov, 2026-10-11 |
| 4 | [The app window draws its own rounded frame](../../../decisions/the-app-window-draws-its-own-rounded-frame.md) | Softov, 2026-10-11 |

| What | Source | Task |
| --- | --- | --- |
| On the desk, `HOST` stays `'daemon'`; a switch of host replaces the window's host and reloads. | Decision 1 | 06, 08 |
| The host switcher lists every host, each with its status dot. | Softov, asked "which hosts does the host switcher list?": "Every host, with its status dot (Recommended)" | 08 |
| `packages/ahpw-embedder` takes `@ahpd/web` through `link:../../../ahpw` until a version with `./embed` is on npm. | Decision 2 | 10 |
| ahpw's `sessions/activity.ts` becomes `sessions/activity-parts.ts`. | (defaulted: the name of what it groups) | 01 |
| The ahpw work is on its branch `desk`, kept up to date with `main`. | Softov: "make a branch on ahpw for those changes." | all |
| The desk's status client offers every protocol version, as ahpw's `offering()` does. | ahpw `connection/protocol-versions.ts` | 04 |

## Proposed architecture

- **Data flow** - When `isDesk()`, ahpw's `connect()` takes the URL from `windowHost()` and the token from `hostToken()` on each connect. Otherwise it keeps `window.location`.
- **Event flow** - `hosts-changed` reconnects ahpw when its host's address changes. `hosts-status` carries every host's status from the host list window to every window.
- **State flow** - Rust keeps the last status map in `HostStatuses`. On the desk, ahpw writes no token to localStorage.
- **Layer responsibilities** - ahp-desk: the frame of the app window, the status relay, the contract · ahpw: the desk branch in `connection/`, `token-provider.ts`, the rounded root, and the title bar.
- **Source-of-truth files** - `ahpw/src/embed/index.ts` for the contract, `ahpw/src/connection/data.ts`, `ahp-desk/packages/ahpw-embedder/src/embedder.ts`.

## Tasks

| Task | Status | Depends on |
| --- | --- | --- |
| [01 - ahpw builds on Windows](task-01-ahpw-builds-on-windows.md) | done | - |
| [02 - ahpw depends on desk-client](task-02-ahpw-depends-on-desk-client.md) | done | 03 |
| [03 - the contract has the frame and the statuses](task-03-contract-frame-and-statuses.md) | done | - |
| [04 - the shell publishes host statuses](task-04-shell-publishes-statuses.md) | done | 03 |
| [05 - the app window has no border and round corners](task-05-app-window-frame.md) | done | - |
| [06 - ahpw connects to the window's host](task-06-ahpw-connects-to-its-host.md) | done | 02 |
| [07 - ahpw skips its sign-in wall on the desk](task-07-ahpw-skips-the-wall.md) | done | 02 |
| [08 - ahpw's frame and title bar on the desk](task-08-ahpw-frame-and-title-bar.md) | done | 02, 04, 05 |
| [09 - end to end from the desk](task-09-end-to-end.md) | doing | 01, 06, 07, 08, 10 |
| [10 - ahpw takes its embedder from the desk](task-10-ahpw-takes-an-injected-embedder.md) | done | 08 |

## Risks and tradeoffs

- The change is in another repository with its own release. ahpw publishes a new `@ahpd/web`, and ahp-desk moves its pin.
- A `link:` dependency makes the desk's typecheck need the ahpw checkout beside it, built, until `@ahpd/web` with `./embed` is published.
- The statuses exist only while the host list window runs; it is hidden, never destroyed.

## Resume state

- **Done so far:** tasks 01 to 08 and 10, 2026-10-11; ahpw's changes are on its branch `desk`.
- **Next action:** publish `@ahpd/web` from ahpw's `desk` branch, move ahp-desk's pin, and run task 09 step 4 with the `npm` source.
- **Open questions:** none.
- **Watch out for:** `packages/ahpw-embedder`'s `link:` to `../../../ahpw` until `@ahpd/web` with `./embed` is on npm.

## Final verification checklist

- [ ] ahpw in an app window connects to a URL host with a token, with `AHP_DESK_AHPW=local` and with `npm`.
- [x] The app window has no accent border, round corners when restored, square when maximized, and working minimize, maximize and close.
- [x] ahpw in a browser through `ahpw serve` still works.
- [x] `pnpm typecheck` and `pnpm test` pass in ahpw and in ahp-desk; `cargo clippy` and `cargo test` pass.
- [x] `plans/index.md` updated.
