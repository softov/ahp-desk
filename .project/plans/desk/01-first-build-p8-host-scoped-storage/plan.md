---
title: Each host keeps its own pages on the desk
domain: desk
status: active
priority: high
created: 2026-10-13
revalidated: 2026-10-13
requires:
  - plans/desk/01-first-build-p2-ahpw-on-the-desk/plan.md
decisions:
  - decisions/ahpw-keeps-a-hosts-pages-under-the-hosts-address.md
  - decisions/ahpw-owns-its-embedding-contract.md
refs:
  - file:///github/ahpw/src/embed/index.ts - `Embedder`, which gains `hostUrl`
  - file:///github/ahpw/src/tabs.ts - `ahpd-web.session.v1` at line 7
  - file:///github/ahpw/src/App.tsx - `ahpd-web.layout.v1` at line 15
  - file:///github/ahpw/src/folders/added.ts - `ADDED_KEY` at line 8
  - file:///github/ahpw/README.md - the administration screens, removed while ahpw is AHP only
  - "[code://src-tauri/src/app_window.rs](../../../../src-tauri/src/app_window.rs) - `page_url()`, the window's host in the page address"
  - "[code://packages/ahpw-embedder/src/embedder.ts](../../../../packages/ahpw-embedder/src/embedder.ts) - the desk's embedder, which sets `hostUrl`"
  - "[code://packages/ahpw-embedder/package.json](../../../../packages/ahpw-embedder/package.json) - `@ahpd/web` by `link:`, to move to npm"
---

## Goal

On the desk, a host's open tabs, layout and added folders belong to that host: opening another host never shows them.
The theme and the other personal choices stay the same on every host.

## Reconnaissance

The files read are the `refs` above.

### Searches performed

- `rg "localStorage|LocalStorage" ahpw/src` - seven keys; three hold a host's own data (tabs, layout, added folders), the rest are personal choices.
- The desk profile's `ahpd-web.layout.v1` holds `activeContainerKey` and window bounds keyed by session, so the layout is a host's too.

### Runtime path

```
app_window.rs open_app_window -> page ?host=<address> -> embedder hostUrl -> ahpw module load -> hostKey(key) -> localStorage
```

### Gaps

- Every host shares `http://ahpw.localhost`, so the three keys are shared.
- `Embedder.host()` is async, and the keys are read when ahpw's modules load.

## Decisions locked in

| # | Decision | Rationale / source |
| --- | --- | --- |
| 1 | [ahpw keeps a host's pages under the host's address](../../../decisions/ahpw-keeps-a-hosts-pages-under-the-hosts-address.md) | Softov, 2026-10-13 |
| 2 | [ahpw owns its embedding contract](../../../decisions/ahpw-owns-its-embedding-contract.md) | Softov, 2026-10-11 |

| What | Source | Task |
| --- | --- | --- |
| ahpw's README stops listing the administration screens. | Softov: "remove from read-me from now." | 03 |
| The unscoped keys in the desk profile, and the stale `ahpd-web.token`, are removed once by hand. | Softov: "yes, remove the old ahpd-web.token"; decision 1 | 04 |
| ahpw's session counters follow the host: it keeps what the SDK's session cache drops (`changes` and more), so a relist sticks (ahpw `08e7518`). | Softov: "Counters does not match. Only after some time" | 03 |
| A running session's row shows its counts above its activity, as ahpapp does (ahpw `3677719`). | Softov: "ok" to the proposed three-line row | 03 |
| scena shows no words of its own in an empty surface, tree or list; an app gives a surface an empty view with `surfaces.setEmpty`, and ahpw gives `main` a start page. | Softov, 2026-10-13: "Nothing (Recommended)" and "App-provided empty view (Recommended)" | 03 |

## Proposed architecture

- **Data flow** - Rust opens the window on the UI with `?host=<address>`; the embedder exposes it as `hostUrl`; `hostKey()` appends it to each host key.
- **Event flow** - `switchHost` navigates the window to the new host's page.
- **State flow** - three `localStorage` entries per host address on the desk; one each in a browser.
- **Layer responsibilities** - ahpw: the contract and the keys · `packages/ahpw-embedder`: `hostUrl` · `src-tauri`: the host in the page address.
- **Source-of-truth files** - ahpw's `src/embed/index.ts` and `src/storage-keys.ts`.

## Tasks

| Task | Status | Depends on |
| --- | --- | --- |
| [01 - ahpw keys a host's pages by its address](task-01-ahpw-host-keys.md) | done | - |
| [02 - the desk gives the window's host address](task-02-desk-host-url.md) | done | 01 |
| [03 - release @ahpd/web 0.3.1 and move the pins](task-03-release.md) | done | 01, 02 |
| [04 - end to end](task-04-end-to-end.md) | todo | 03 |

## Risks and tradeoffs

- The contract change needs both sides: a desk on 0.3.1 with an old embedder has no `hostUrl`; the desk moves its embedder and its pins in the same change.

## Resume state

- **Done so far:** the plan, tasks 01 to 03, 2026-10-13.
- **Next action:** [task-04-end-to-end.md](task-04-end-to-end.md), on a release build of the npm pins.
- **Open questions:** none.
- **Watch out for:** ahpw's changes go on its branch `desk`, then a PR to `main` that Softov merges; the npm release is approved by Softov.

## Final verification checklist

- [ ] Two hosts on the desk each open on their own tabs and layout; the theme is the same on both.
- [ ] ahpw in a browser keeps its keys unchanged.
- [ ] `pnpm typecheck`, `pnpm test`, `pnpm lint` pass in ahp-desk; `pnpm typecheck` and `pnpm test` in ahpw.
- [ ] `plans/index.md` updated.
