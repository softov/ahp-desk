---
title: ahpw depends on @ahpd/desk-client
status: done
depends: [task-03-contract-frame-and-statuses.md]
layer: "ahpw"
refs:
  - file:///github/ahpw/package.json - the dependencies, and `@softov/scena` at line 63
  - file:///github/ahp-desk/packages/desk-client/package.json - the package, built to `dist`
---

## Objective

ahpw imports `@ahpd/desk-client` and `@tauri-apps/api`, and builds and tests with them.

## Files

- `UPDATE: ahpw/package.json` - add `"@ahpd/desk-client": "link:../ahp-desk/packages/desk-client"` and `"@tauri-apps/api": "^2.12.3"` to `dependencies`.
- `UPDATE: ahpw/pnpm-lock.yaml` - from `pnpm install`.
- `CREATE: ahpw/src/desk.ts` - `onDesk`, the one place ahpw asks `isDesk()`, re-exported for the rest of ahpw.

## Steps

1. Build desk-client in ahp-desk with `pnpm build:client`.
2. Add the two dependencies and run `pnpm install` in ahpw.
3. In `src/desk.ts`, export `const onDesk = isDesk();`, read once at load: the answer never changes for a page.
4. Note in ahpw's `DEVELOPER.md` that the desk build needs `../ahp-desk` beside it until desk-client is on npm.

## Validation

- Run `pnpm typecheck` and `npx vite build` in ahpw; they pass.
- In a browser, `onDesk` is false.

## Resume

- Done 2026-10-11. Both went to `devDependencies`, where ahpw keeps every library it bundles into the page. `src/desk/index.ts` holds `onDesk`; the README, not a DEVELOPER.md (ahpw has none), says to clone ahp-desk beside it.
