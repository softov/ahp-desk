---
title: The desk gives each app window its host's address
status: done
depends: [task-01-ahpw-host-keys.md]
layer: "src-tauri, packages/ahpw-embedder"
refs:
  - "[code://src-tauri/src/app_window.rs](../../../../src-tauri/src/app_window.rs) - the init script"
  - "[code://packages/ahpw-embedder/src/embedder.ts](../../../../packages/ahpw-embedder/src/embedder.ts) - the embedder"
---

## Objective

Each app window's page address names its host as `?host=<address>`, and the embedder sets `hostUrl` from it.

## Files

- `UPDATE: src-tauri/src/app_window.rs` - `page_url()`, the UI with `?host=`; a switch navigates to the new host's page instead of reloading.
- `UPDATE: packages/ahpw-embedder/src/embedder.ts` - `hostUrlOf(location)`, and `createEmbedder(location)` sets `hostUrl`.
- `UPDATE: packages/ahpw-embedder/src/main.ts` - passes `window.location`.
- `UPDATE: packages/ahpw-embedder/src/embedder.test.ts` - `hostUrl`.

## Steps

1. Open each app window on `page_url()`, and navigate to it on a switch.
2. Read it in `createEmbedder`; fail with a clear error when it is missing.

## Validation

- `pnpm typecheck`, `pnpm test`, `pnpm lint` in ahp-desk.

## Resume

- Done 2026-10-13. The init script was the plan, but Tauri sets it once per window, and a switch keeps the window: it would have named the first host after a switch. The page address changes with the host, so it carries it; the scheme handler reads only the path, and ahpw reads no query.
- Checked on a release build staged from ahpw's `desk`, with two throwaway hosts: each opens on its own tabs, a switch and back restores the first host's tab, the theme is shared.
