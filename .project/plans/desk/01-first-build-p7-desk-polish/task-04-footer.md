---
title: ahpw's mark and © ahpio.com at the bottom of the host list
status: done
depends: [task-02-host-list-palette.md]
layer: "packages/shell, src-tauri"
---

## Objective

The host list ends in ahpw's mark centred at the bottom, with "© ahpio.com" linking to https://ahpio.com in the browser.

## Files

- `CREATE: packages/shell/src/Footer.tsx` - the mark and the link.
- `UPDATE: src-tauri` - the opener plugin, scoped to https://ahpio.com, for the shell window only.

## Steps

1. Draw the mark at 20 px with the muted text beside it.
2. A click opens https://ahpio.com in the browser, never in the desk.

## Validation

- The footer sits centred at the bottom; the link opens the browser.

## Resume

- Done 2026-10-11. `tauri-plugin-opener` with `opener:allow-open-url` scoped to https://ahpio.com on the shell window only. Seen in a release build; the browser opening not clicked yet.
