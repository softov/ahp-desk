---
title: The embedder gives the mode, each host's token and the host to show
status: done
depends: [task-03-the-token-rule.md]
layer: "packages/ahpw-embedder"
refs:
  - "[code://packages/ahpw-embedder/src/embedder.ts#L34-L52](../../../../packages/ahpw-embedder/src/embedder.ts#L34-L52) - `createEmbedder`: `host()` from `?host=`, and `token` of that host only"
  - file:///github/ahpw/.project/plans/hosts/02-a-window-with-every-host/task-01-the-embedder-says-its-mode.md - the contract this task implements
---

## Objective

In the window with every host, the embedder says `mode: 'multi'`, gives any host's token, and passes on `focus-host`.
A window of one host gets the embedder it has now.

## Files

- `UPDATE: packages/ahpw-embedder/src/embedder.ts:34-52` - `mode` from `?hosts=every`; `token(id)` asks `host_token` for `id`, or the window's host without one; `onFocusHost` listens to `focus-host`; `manageHosts` passes `edit`.
- `UPDATE: src-tauri/src/app_window.rs` - `show_host_list` takes `edit: Option<String>` and opens the host list on that host's form.
- `UPDATE: packages/ahpw-embedder/src/embedder.test.ts` - multi and mono cases.
- `UPDATE: package.json`, `packages/ahpw-embedder/package.json` - the `@ahpd/web` pin that has the new contract, both together.

## Steps

1. Move both `@ahpd/web` pins to the release with ahpw hosts plan 02 task 01.
2. Read `?hosts=every` into `mode`, and in multi answer `hostUrl` as `*`.
3. Give `token` its `id`, and add `onFocusHost` and `edit`.
4. Add `edit` to `show_host_list`.

## Validation

- `pnpm test` passes, with the embedder's new cases.
- `pnpm lint` passes.

## Resume

- 2026-10-16: the embedder reads `?hosts=every` as `mode: 'multi'` with `hostUrl` `*`, gives `token(id)`, and in multi `onFocusHost`.
  It gives the `focus=` host once, then drops it from the address, so a reload keeps the person's filter.
- `manageHosts({ edit })` reaches `show_host_list`, which emits `host-list-edit`; the host list opens that host's form.
- Verified against ahpw's local contract: `tsc` with `@ahpd/web/embed` mapped to ahpw's build, and the embedder's 16 tests.
- Done 2026-10-18: both `@ahpd/web` pins on 0.5.0, the release with ahpw hosts plan 02, and the local ahpw link in `packages/ahpw-embedder/node_modules` gone.
- Verified on 0.5.0 from npm: `pnpm typecheck`, `pnpm test` (the embedder's 16 and the shell's 24) and `pnpm lint` pass.
