---
title: "@ahpd/web 0.3.1 is released and the desk takes it from npm"
status: todo
depends: [task-01-ahpw-host-keys.md, task-02-desk-host-url.md]
layer: "ahpw, ahp-desk"
refs:
  - "[code://packages/ahpw-embedder/package.json](../../../../packages/ahpw-embedder/package.json) - `@ahpd/web` by `link:`"
---

## Objective

`@ahpd/web` 0.3.1 is on npm with the host keys, the start page and the README without the administration screens, on scena 0.5.5, and the desk takes it from npm in both places.

## Files

- `UPDATE: ahpw/CHANGELOG.md`, `ahpw/package.json` - 0.3.1, and `@softov/scena` ^0.5.5 for `surfaces.setEmpty`.
- `UPDATE: package.json` and `packages/ahpw-embedder/package.json` in ahp-desk - `@ahpd/web` 0.3.1 from npm.

## Steps

0. scena 0.5.5 is released with `surfaces.setEmpty` and no empty-state words of its own (scena commit `438926b`).
1. On ahpw's `desk`: the CHANGELOG's Unreleased under 0.3.1, the version, `release: @ahpd/web v0.3.1`; a PR to `main` for Softov to merge.
2. After the merge, the `v0.3.1` tag; Softov approves the release on npmjs.
3. Move both desk pins to 0.3.1.

## Validation

- `npm view @ahpd/web version` is 0.3.1; the desk's checks pass on it.

## Resume
