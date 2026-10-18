---
title: The desk's icon in ahpw's mark
status: done
depends: []
layer: "src-tauri/icons"
---

## Objective

The desk's icon is ahpw's mark in ahpw's colours.

## Files

- `RENAME: src-tauri/icons/placeholder.svg -> src-tauri/icons/icon.svg` - ahpw's mark from `ahpw/public/ahpw.svg`, on its dark tile.
- `UPDATE: src-tauri/icons/*` - regenerated with `pnpm tauri icon`.

## Steps

1. Take ahpw's mark as the source image.
2. Regenerate every size with `pnpm tauri icon`.

## Validation

- The title bar, the taskbar and the installer show ahpw's mark.

## Resume

- Done 2026-10-13. `icon.svg` is ahpw's `public/ahpw.svg` at 1024 px; `pnpm tauri icon` regenerated the set, and its Android and iOS output was removed as in plan 01 task 01. The exe and, with `installerIcon`, the installer carry the mark (task 09).
