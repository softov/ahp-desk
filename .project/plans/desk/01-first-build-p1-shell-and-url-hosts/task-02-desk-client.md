---
title: The desk-client package gives a UI the shell's contract
status: todo
depends: [task-01-workspace-scaffold.md]
layer: "packages/desk-client"
refs:
  - file:///github/ahp-desk/mockups/03-app-window.md - the arrows between the shell and a UI
  - file:///github/opendoop/packages/tauri-shell/src/ipc.ts - doop wraps every invoke in one module
  - npm://@tauri-apps/api@^2 - `invoke`, `listen` and `getCurrentWindow`
---

## Objective

`@ahpd/desk-client` exports the contract that ahpw and ahpapp import.
It builds to ESM with types, and it is ready to publish to npm.

## Files

- `CREATE: packages/desk-client/package.json` - `@ahpd/desk-client`, version `0.1.0`, ESM, `exports` and `types`, peer `@tauri-apps/api@^2`, license MIT.
- `CREATE: packages/desk-client/tsconfig.json` - strict, declarations to `dist`.
- `CREATE: packages/desk-client/src/index.ts` - the contract below.
- `CREATE: packages/desk-client/src/index.test.ts` - the tests, with `@tauri-apps/api` mocked.
- `CREATE: packages/desk-client/README.md` - what a UI calls, and an example.

## Steps

1. Define `DeskHost` as `{ id: string; name: string; kind: 'url' | 'local'; url: string }`.
2. Write `isDesk()`, which returns `'__TAURI_INTERNALS__' in window`.
3. Write `hosts()`, which calls `invoke<DeskHost[]>('hosts')`.
4. Write `windowHost()`, which calls `invoke<DeskHost>('window_host')`.
5. Write `hostToken(id)`, which calls `invoke<string | null>('host_token', { id })`.
6. Write `onHostsChanged(handler)`, which calls `listen<DeskHost[]>('hosts-changed')` and returns the unlisten function.
7. Write `openHost(id, options)` with `options: { replace?: boolean }`, which calls `invoke('open_app_window')`.
8. Write `showHostList()`, which calls `invoke('show_host_list')`.
9. Write `windowControls` with `minimize`, `toggleMaximize`, `close` and `startDragging` over `getCurrentWindow()`.
10. Write a test for each function that checks the command name and the arguments.

## Validation

- Run `pnpm --filter @ahpd/desk-client test`; every case passes.
- Run `pnpm --filter @ahpd/desk-client build`; `dist/index.js` and `dist/index.d.ts` exist.
- Run `npm pack --dry-run` in the package; the list holds only `dist`, `README.md` and `package.json`.

## Resume
