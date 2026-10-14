---
title: The updater artifacts are made only in the release build
status: accepted
date: 2026-10-14
refs:
  - "[code://src-tauri/tauri.conf.json#L18-L38](../../src-tauri/tauri.conf.json#L18-L38) - the bundle settings, which leave `createUpdaterArtifacts` off"
  - "[code://.project/plans/desk/01-first-build-p4-tray-and-releases/task-04-release-workflow.md](../../.project/plans/desk/01-first-build-p4-tray-and-releases/task-04-release-workflow.md) - creates `src-tauri/tauri.release.conf.json` and passes it with `--config`"
  - https://v2.tauri.app/plugin/updater/ - `createUpdaterArtifacts` and the signing variables it needs
---

## Context

`bundle.createUpdaterArtifacts: true` makes `tauri build` sign the bundles, and the build fails without the private signing key.
ahp-desk is MIT, so anyone can clone it and build an installer, with no key.
The release build in CI has the key as a secret.

## Decision

`createUpdaterArtifacts: true` is set only in `src-tauri/tauri.release.conf.json`, which the release workflow passes to the build with `--config`.
`src-tauri/tauri.conf.json` does not set it.
Source: Softov, asked "p4 q8: where is createUpdaterArtifacts: true set?": "Release config only".

## Consequences

A clone, and a local `pnpm tauri build`, build an installer with no key and no `.sig` file.
A release build without `--config src-tauri/tauri.release.conf.json` makes no `.sig` and so no update; the workflow's publish job checks that `latest.json` names both platforms.

## Options

- **In tauri.conf.json.** Lost: every build, a clone's too, would need the private key or fail.
