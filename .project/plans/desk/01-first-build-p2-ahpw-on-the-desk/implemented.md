---
title: ahpw on the desk - implemented
date: 2026-10-13
refs:
  - git://6e98095
  - npm://@ahpd/web@0.3.0 - ahpw with the embedding contract, published from its branch `desk`
  - "[code://packages/ahpw-embedder/src/embedder.ts](../../../../packages/ahpw-embedder/src/embedder.ts)"
  - "[code://src-tauri/src/app_window.rs](../../../../src-tauri/src/app_window.rs)"
---

ahpw runs in a desk app window on the window's host, with the host's token and no sign-in wall.
The window has no OS border and round corners, ahpw draws its title bar, and the title bar switches hosts, each with its status dot.
ahpw in a browser is unchanged.

## What was built

- In ahpw, released as `@ahpd/web` 0.3.0: the embedding contract `Embedder` in `src/embed/index.ts`, exported as `@ahpd/web/embed`; the connection to the embedder's host; no wall when embedded; the frame, the title bar and the host switcher.
- [`code://packages/ahpw-embedder/src/embedder.ts`](../../../../packages/ahpw-embedder/src/embedder.ts) - the desk's `Embedder` over its Tauri commands, built into one script.
- [`code://src-tauri/src/app_window.rs`](../../../../src-tauri/src/app_window.rs) - the frameless app window with the embedder injected by `initialization_script`.
- [`code://src-tauri/src/statuses.rs`](../../../../src-tauri/src/statuses.rs) - the host statuses the host list publishes for every window.
- The desk pins `@ahpd/web` 0.3.0.

## Verified

- `local`: `pnpm tauri dev` on ahpw's dev server, and a release build staged from ahpw's `desk`, open a host with no wall (2026-10-11).
- `npm`: `pnpm tauri dev` serving `node_modules/@ahpd/web/dist/app`, and a release build on the same package, open a throwaway ahpd 0.7.0 with its token and no wall (2026-10-13).
- `ahpw serve` in a browser shows the wall and signs in with the token.
- `pnpm typecheck` and `pnpm test` in ahpw and ahp-desk; `cargo clippy` and `cargo test`.

## Departures from the plan

- Decision `ahpw-owns-its-embedding-contract` replaced the shared `@ahpd/desk-client` package (task 10): ahpw reads an injected `globalThis.ahpwEmbedder`.

## Left for later

- `packages/ahpw-embedder` still takes `@ahpd/web` by `link:` to the ahpw checkout; it moves to npm with the next ahpw release, in [p8](../01-first-build-p8-host-scoped-storage/plan.md).
- A second desk process cannot start while one runs with other WebView2 arguments; single-instance is p4's.
