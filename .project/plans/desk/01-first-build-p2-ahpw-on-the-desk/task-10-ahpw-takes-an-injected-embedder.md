---
title: ahpw takes its embedder from the desk, with no desk code in ahpw
status: done
depends: [task-08-ahpw-frame-and-title-bar.md]
layer: "ahpw, packages/ahpw-embedder, packages/shell, src-tauri"
refs:
  - file:///github/ahpw/src/embed/index.ts - ahpw's contract, `Embedder`, read from `globalThis.ahpwEmbedder`
  - "[code://packages/ahpw-embedder/src/embedder.ts](../../../../packages/ahpw-embedder/src/embedder.ts) - the desk's embedder"
  - "[code://src-tauri/src/app_window.rs](../../../../src-tauri/src/app_window.rs) - `initialization_script(EMBEDDER)`"
---

## Objective

ahpw reaches the desk only through `globalThis.ahpwEmbedder`, which the desk injects before ahpw runs; ahpw depends on neither `@ahpd/desk-client` nor `@tauri-apps/api`.

## Files

- `CREATE: ahpw/src/embed/index.ts` - `Embedder`, `EmbedHost`, `EmbedHostStatus`, `EmbedWindow`, `Unsubscribe`, and `embedder`.
- `CREATE: ahpw/tsconfig.embed.json` - builds the contract into `dist/embed`, exported as `@ahpd/web/embed`.
- `UPDATE: ahpw/src/desk/` - moved to `src/embed/`; the frame, the controls and the switcher take the embedder; classes `is-framed` and `embed-*`; the dots use the theme's success and warning colours.
- `UPDATE: ahpw/src/connection/data.ts`, `src/token-provider.ts`, `src/main.tsx`, `src/shell.tsx` - read `embedder`.
- `UPDATE: ahpw/package.json` - the `./embed` export; no `@ahpd/desk-client`, no `@tauri-apps/api`.
- `UPDATE: packages/desk-client` - moved to `packages/ahpw-embedder`, private, built by Vite into `dist/ahpw-embedder.js`.
- `UPDATE: packages/shell/src/hosts/ipc.ts` - the host list's own Tauri calls, in place of desk-client.
- `UPDATE: src-tauri/build.rs`, `src-tauri/src/app_window.rs` - the script read in with `include_str!` and injected into each app window.

## Validation

- ahpw: `pnpm typecheck`, 259 tests, `pnpm build` with `dist/embed`.
- ahp-desk: `pnpm typecheck`, 7 embedder tests, 10 shell tests, 15 Rust tests, `cargo clippy -D warnings`.
- A release build staged from ahpw's `desk` branch opens Desk test from the host list: `ahpwEmbedder` is set, no sign-in wall, the frame and its three controls, and the switcher lists the three hosts with their dots.

## Resume

- Done 2026-10-11.
