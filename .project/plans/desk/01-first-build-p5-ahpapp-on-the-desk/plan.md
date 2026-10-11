---
title: ahpapp in a private build
domain: desk
status: draft
priority: medium
created: 2026-10-11
revalidated: 2026-10-11
requires:
  - plans/desk/01-first-build-p1-shell-and-url-hosts/plan.md
  - plans/desk/01-first-build-p2-ahpw-on-the-desk/plan.md
decisions:
  - decisions/a-ui-takes-its-hosts-from-a-command-and-an-event.md
  - decisions/a-ui-bundle-comes-from-the-local-repository-or-from-npm.md
  - decisions/each-ui-is-served-from-its-own-uri-scheme.md
  - decisions/each-desk-build-ships-one-ui.md
refs:
  - file:///github/ahpapp/src/host-context.tsx - the host list at line 650, `register` at lines 583-639, `persist` at lines 820-823
  - file:///github/ahpapp/src/host-rows.ts - `SavedHost` at lines 56-64; `restoredRows` and `durableRows` drop unknown fields
  - file:///github/ahpapp/src/storage.ts - `getToken` at lines 65-107
  - file:///github/ahpapp/src/bots-flag.ts - `EXPO_PUBLIC_BOTS=1` turns the bots on, at lines 16-24
  - file:///github/ahpapp/app.json - no base URL, so the web build uses paths from the root
  - file:///github/ahpapp/.project/working/running-it.md - the web build, and its fallback to index.html
---

## Goal

A private build of the desk ships ahpapp in place of ahpw.
In that build, ahpapp opens in an app window, on every host in the host list.
Its hosts come from the shell, and it writes none of them to its own storage.
The public build does not change, and it holds no code from ahpapp.

## Reconnaissance

The files read and the patterns to reuse are the `refs` above, each with its note.

### Searches performed

- `rg "window.location|__TAURI" ahpapp/src ahpapp/app` - `Not found: no origin is hard-coded.`
- `rg "AsyncStorage|SecureStore" ahpapp/src` - the hosts and the tokens, in `host-context.tsx` and `storage.ts`.

### Runtime path

```
AHP_DESK_UI=ahpapp pnpm tauri build -> private build -> open_app_window -> ahpapp://
ahpapp:// -> HostsProvider -> isDesk() -> hosts() + onHostsChanged() -> register(host) with hostToken() -> RefcountedMultiHostClient
```

### Gaps

- ahpapp's web build has not been checked by a person.
- `HostsProvider` reads only its own storage.
- The shell has one UI, ahpw, fixed in its code.

## Decisions locked in

| # | Decision | Rationale / source |
| --- | --- | --- |
| 1 | [A UI takes its hosts from a command and an event](../../../decisions/a-ui-takes-its-hosts-from-a-command-and-an-event.md) | Softov, 2026-10-11 |
| 2 | [A UI bundle comes from the local repository or from npm](../../../decisions/a-ui-bundle-comes-from-the-local-repository-or-from-npm.md) | Softov, 2026-10-11 |
| 3 | [Each UI is served from its own URI scheme](../../../decisions/each-ui-is-served-from-its-own-uri-scheme.md) | Softov, 2026-10-11 |
| 4 | [Each desk build ships one UI](../../../decisions/each-desk-build-ships-one-ui.md) | Softov, 2026-10-11 |

| What | Source | Task |
| --- | --- | --- |
| ahpapp's only source is `local`, because ahpapp is not on npm. | `ahpapp/package.json` has `private: true` | 02 |
| The desk builds ahpapp with `EXPO_PUBLIC_BOTS=1`. | `bots-flag.ts` | 02 |
| The build setting `AHP_DESK_UI=ahpw|ahpapp` chooses the UI, default `ahpw`. | (defaulted) | 02 |

## Proposed architecture

- **Data flow** - On the desk, `HostsProvider` takes its rows from `hosts()` and its tokens from `hostToken()`.
- **Event flow** - `onHostsChanged` adds, updates and removes hosts with the existing `register` path.
- **State flow** - On the desk, `persist` writes no host and `setToken` writes no token.
- **Layer responsibilities** - ahpapp: the desk branch in `HostsProvider` · ahp-desk: the build setting for the UI, the `ahpapp` scheme, and the private build's identity.
- **Source-of-truth files** - `ahpapp/src/host-context.tsx`.

## Tasks

These task files are written when this plan moves to `planned`.

| Task | Status | Depends on |
| --- | --- | --- |
| 01 - run ahpapp's web build in a browser and fix what breaks | todo | - |
| 02 - the build setting for the UI, the ahpapp scheme and the local source | todo | 01 |
| 03 - HostsProvider takes its hosts from the desk | todo | 02 |
| 04 - title bar with drag and window controls | todo | 03 |
| 05 - the private build's name, identifier, icon and release channel | todo | 02 |
| 06 - end to end with two hosts, in a private build | todo | 03, 04, 05 |

## Risks and tradeoffs

- ahpapp is under a private license. Its build must never publish to the public releases; task 05 keeps the two channels apart.
- The Add host screens of ahpapp must not show on the desk, or there are two host lists.

## Resume state

- **Done so far:** nothing.
- **Next action:** write the task files, then task 01.
- **Open questions:**
  1. What are the private build's product name and identifier? - proposed: "ahp-desk app", `dev.softov.ahpdesk.app`.
  2. Where do private releases go, and how does its updater reach them? A private GitHub repository needs a token, which the updater cannot carry.
- **Watch out for:** `durableRows` drops unknown fields, so a field that marks a desk host stays in memory only.

## Final verification checklist

- [ ] In a build with `AHP_DESK_UI=ahpapp`, ahpapp in an app window lists the sessions of two hosts.
- [ ] A build with `AHP_DESK_UI=ahpw` holds no file from ahpapp.
- [ ] An edit in the desk host list shows in ahpapp with no reload.
- [ ] ahpapp on a phone is not changed.
- [ ] `plans/index.md` updated.
