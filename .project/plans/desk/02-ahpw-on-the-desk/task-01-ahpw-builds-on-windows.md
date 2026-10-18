---
title: ahpw builds on Windows and macOS
status: done
depends: []
layer: "ahpw"
refs:
  - file:///github/ahpw/src/sessions/activity.ts - the grouping of a turn's parts, same name as `Activity.tsx` but for case
  - file:///github/ahpw/src/sessions/Page.tsx - imports both, at lines 21-22
---

## Objective

`vite build` passes in ahpw on a case-insensitive file system, so the desk's `local` source works on Windows.

## Files

- `RENAME: ahpw/src/sessions/activity.ts` to `ahpw/src/sessions/activity-parts.ts`.
- `RENAME: ahpw/src/sessions/activity.test.ts` to `ahpw/src/sessions/activity-parts.test.ts`.
- `UPDATE: ahpw/src/sessions/Activity.tsx:3` - import from `./activity-parts.js`.
- `UPDATE: ahpw/src/sessions/Page.tsx:22` - import `groupParts` from `./activity-parts.js`.
- `UPDATE: ahpw/src/sessions/activity-parts.test.ts:3` - import from `./activity-parts.js`.

## Steps

1. On the branch `desk`, rename both files with `git mv`.
2. Change the three imports.
3. Search for any other import of `./activity.js` and change it.

## Validation

- Run `npx vite build` in ahpw on Windows; it builds `dist/app`.
- Run `pnpm test` and `pnpm typecheck` in ahpw; they pass.

## Resume

- Done 2026-10-11 on ahpw's branch `desk`: `npx vite build` passes on Windows; 37 test files pass.
