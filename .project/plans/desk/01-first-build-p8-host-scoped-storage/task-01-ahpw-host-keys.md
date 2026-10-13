---
title: ahpw keys a host's pages by the host's address
status: done
depends: []
layer: "ahpw"
refs:
  - file:///github/ahpw/src/embed/index.ts - the contract
---

## Objective

On an embedder, ahpw's tabs, layout and added folders are kept under keys that carry the page's host address; in a browser the keys are unchanged.

## Files

- `UPDATE: ahpw/src/embed/index.ts` - `Embedder` gains `readonly hostUrl: string`.
- `CREATE: ahpw/src/storage-keys.ts` - `hostKey(key)`.
- `CREATE: ahpw/src/storage-keys.test.ts` - the key in a browser and on an embedder.
- `UPDATE: ahpw/src/tabs.ts:7`, `ahpw/src/App.tsx:15`, `ahpw/src/folders/added.ts:8` - the three keys through `hostKey`.

## Steps

1. Add `hostUrl` to `Embedder` (decision 1).
2. Write `hostKey` and its test.
3. Route the three keys through it.

## Validation

- `pnpm typecheck` and `pnpm test` in ahpw.

## Resume

- Done 2026-10-13 on ahpw's `desk`, commit `7d9a79f`; 278 tests pass.
