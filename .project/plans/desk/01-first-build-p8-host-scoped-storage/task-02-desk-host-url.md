---
title: The desk gives each app window its host's address
status: todo
depends: [task-01-ahpw-host-keys.md]
layer: "src-tauri, packages/ahpw-embedder"
refs:
  - "[code://src-tauri/src/app_window.rs](../../../../src-tauri/src/app_window.rs) - the init script"
  - "[code://packages/ahpw-embedder/src/embedder.ts](../../../../packages/ahpw-embedder/src/embedder.ts) - the embedder"
---

## Objective

Each app window's init script names its host before the embedder runs, and the embedder sets `hostUrl` from it.

## Files

- `UPDATE: src-tauri/src/app_window.rs` - the init script starts with `globalThis.ahpDeskHost = <host as JSON>;`.
- `UPDATE: packages/ahpw-embedder/src/embedder.ts` - `hostUrl` from `globalThis.ahpDeskHost`.
- `UPDATE: packages/ahpw-embedder/src/embedder.test.ts` - `hostUrl`.

## Steps

1. Build the init script per window with the window's `HostView`.
2. Read it in `createEmbedder`; fail with a clear error when it is missing.

## Validation

- `pnpm typecheck`, `pnpm test`, `pnpm lint` in ahp-desk.

## Resume
