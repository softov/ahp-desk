---
title: Each host keeps its own pages on the desk - implemented
date: 2026-10-13
refs:
  - git://c466d82
  - npm://@ahpd/web@0.3.1 - ahpw with host-scoped storage, the start page and the session counter fixes
  - npm://@softov/scena@0.5.5 - `surfaces.setEmpty`, and no empty-state words of scena's own
  - "[code://src-tauri/src/app_window.rs](../../../../src-tauri/src/app_window.rs)"
  - "[code://packages/ahpw-embedder/src/embedder.ts](../../../../packages/ahpw-embedder/src/embedder.ts)"
---

On the desk, each host opens on its own tabs, layout and added folders; another host never shows them.
The theme and the other personal choices are the same on every host.
The desk builds on `@ahpd/web` 0.3.1 from npm, which also brings a start page in an empty main and session counters that follow the host.

## What was built

- In ahpw 0.3.1: `Embedder.hostUrl`, and `hostKey()` in `src/storage-keys.ts`, through which the tabs, the layout and the added folders are kept per host address when embedded.
- [`code://src-tauri/src/app_window.rs`](../../../../src-tauri/src/app_window.rs) - `page_url()`: each app window loads the UI with `?host=<address>`, and a switch navigates to the new host's page.
- [`code://packages/ahpw-embedder/src/embedder.ts`](../../../../packages/ahpw-embedder/src/embedder.ts) - `hostUrlOf(location)` and `createEmbedder(location)`, which set `hostUrl`.
- Both `@ahpd/web` pins at 0.3.1 from npm; the embedder no longer links the ahpw checkout.
- Also in ahpw 0.3.1, found while testing this plan: a start page in an empty main (scena 0.5.5's `surfaces.setEmpty`), session counters kept over the AHP SDK's cache, which drops `changes`, and a running row's counts.

## Verified

- ahpw: `pnpm typecheck`, `pnpm test` (282) and `pnpm build` on scena 0.5.5. scena: typecheck, lint, test (260). ahp-desk: `pnpm typecheck`, `pnpm test` (shell 24, embedder 11, cargo 15), `pnpm lint`.
- A release build on `@ahpd/web` 0.3.1 from npm, over CDP, with two throwaway ahpd 0.11.0 hosts: host one keeps its Host tab, host two opens clean on the start page, switching back brings host one's tab, and the theme is shared.
- The session counters, before and after the fix, in a headless Edge on a throwaway ahpd 0.11.0 with a fake model.
- The unscoped `ahpd-web.session.v1`, `ahpd-web.layout.v1` and `ahpd-web.folders-added` were removed from the desk profile, with the stale `ahpd-web.token`.

## Departures from the plan

- Task 02: the host goes in the page address, `?host=`, not in the init script, which Tauri sets once per window while a switch keeps the window.
- Task 03: the release also carried scena 0.5.5 and the session counter fixes; the tag sat on `desk`, and PR #3 merged it into `main` after.

## Left for later

- A host reached by two addresses (an IP and a name) keeps two sets of pages, as the address is the key.
- ahpd 0.11.0 pushes no new file list on a changeset channel after a turn, so the Changes pane lags until its own reload; that is ahpd's, in its own repository.
- The Changes pane's heading is shown twice ("CHANGES" in the section header and again inside); not changed yet.
