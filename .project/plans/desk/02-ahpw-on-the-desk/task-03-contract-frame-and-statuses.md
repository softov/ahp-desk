---
title: The contract has the window's maximized state and every host's status
status: done
depends: []
layer: "packages/desk-client"
refs:
  - file:///github/ahp-desk/packages/desk-client/src/index.ts - the contract, with `windowControls`
  - file:///github/opendoop/packages/web/src/lib/tauri-env.ts - pood's `isMaximized` and `onResized`
  - file:///github/ahp-desk/.project/decisions/the-host-list-window-publishes-host-status.md - decision 3
---

## Objective

A UI can draw its frame and its host switcher from `@ahpd/desk-client` alone.

## Files

- `UPDATE: packages/desk-client/src/index.ts` - `windowControls.isMaximized`, `windowControls.onResized`, `DeskHostStatus`, `hostStatuses()`, `onHostStatus()`.
- `UPDATE: packages/desk-client/src/index.test.ts` - a case for each.
- `UPDATE: packages/desk-client/README.md` - the new calls in the table.

## Steps

1. Add `isMaximized: () => getCurrentWindow().isMaximized()`.
2. Add `onResized(handler)`, which calls `getCurrentWindow().onResized(() => handler())` and returns the unlisten.
3. Define `DeskHostStatus` as `'connected' | 'connecting' | 'down'`.
4. Write `hostStatuses()`, which calls `invoke<Record<string, DeskHostStatus>>('host_statuses')`.
5. Write `onHostStatus(handler)`, which listens to `hosts-status` and hands the handler the whole map.

## Validation

- Run `pnpm --filter @ahpd/desk-client test`; every case passes.
- Run `pnpm build:client`; `dist/index.d.ts` has the new names.

## Resume

- Done 2026-10-11. 9 desk-client tests pass.
