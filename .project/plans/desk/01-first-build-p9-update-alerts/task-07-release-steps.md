---
title: The release steps say that an ahpw release is followed by a desk release
status: todo
depends: []
layer: "docs"
refs:
  - "[code://DEVELOPER.md#L54-L64](../../../../DEVELOPER.md#L54-L64) - the two `@ahpd/web` pins, which move together"
  - "[code://DEVELOPER.md#L119-L134](../../../../DEVELOPER.md#L119-L134) - the signing keys, which a release uses"
  - "[code://package.json#L3](../../../../package.json#L3) - the desk's version, one of three places"
  - "[code://package.json#L24](../../../../package.json#L24) - the `@ahpd/web` pin"
  - "[code://packages/ahpw-embedder/package.json#L17](../../../../packages/ahpw-embedder/package.json#L17) - the embedder's `@ahpd/web` pin"
  - "[code://src-tauri/tauri.conf.json#L4](../../../../src-tauri/tauri.conf.json#L4) - the version the updater compares"
  - "[code://src-tauri/Cargo.toml#L3](../../../../src-tauri/Cargo.toml#L3) - the crate's version"
  - "[code://scripts/stage-ui.mjs](../../../../scripts/stage-ui.mjs) - the build copies the pinned `@ahpd/web`'s `dist/app` into the bundle"
---

## Objective

`DEVELOPER.md` has a "Releasing" section: how a desk release is made, and that an ahpw release that matters to the desk is followed by one, with the pins moved, so the desk's updater delivers the new ahpw (Answers: ahpw in a desk release).
The desk never fetches a UI at run time; this section is how a new ahpw reaches it.

## Files

- `UPDATE: DEVELOPER.md` - a "Releasing" section after "Signing keys" (line 134), before "Project records".

## Steps

1. Write the section, one sentence per line, with these steps:
   1. When ahpw is released as a new `@ahpd/web`, move both pins to it: `package.json` and `packages/ahpw-embedder/package.json`; run `pnpm install`.
   2. Run `pnpm test` and `pnpm lint`, then `pnpm tauri dev` and open a host to see the new ahpw in an app window.
   3. Raise the desk's version in `package.json`, `src-tauri/tauri.conf.json` and `src-tauri/Cargo.toml`, the same in all three; the updater compares `tauri.conf.json`'s.
   4. Commit, tag `v<version>`, and push the tag; CI builds the installers and publishes them with `latest.json` (p4 task 04).
   5. An installed desk finds the release within 6 hours, downloads it, and asks before it restarts (p4 task 03, with task 06's check every 6 hours).
2. Say in the section that an ahpw fix reaches people only through these steps, and that a desk release is made for every ahpw release the desk needs.
3. If p4 task 04 has already written a release section, add these steps to it rather than a second one.
4. ahpw's own release steps live in the ahpw repository; a line there naming this follow-up is handed to that repository, not written from here.

## Validation

- By hand: the section reads in order and every file, command and variable it names exists (`rg -n "ahpd/web" package.json packages/ahpw-embedder/package.json`, `rg -n "\"version\"" src-tauri/tauri.conf.json`).
- The first desk release after this follows the steps as written, and any step found missing is added then.

## Resume
