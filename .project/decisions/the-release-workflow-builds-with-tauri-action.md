---
title: The release workflow builds and publishes with tauri-apps/tauri-action
status: accepted
date: 2026-10-14
refs:
  - https://github.com/tauri-apps/tauri-action - builds, uploads the bundles and their `.sig`, and merges `latest.json` into a release
  - file:///github/opendoop/packages/tauri/PUBLISH.md - doop's release by hand, and the shape of `latest.json`
  - "[code://.project/plans/desk/01-first-build-p4-tray-and-releases/task-04-release-workflow.md](../../.project/plans/desk/01-first-build-p4-tray-and-releases/task-04-release-workflow.md) - the workflow"
---

## Context

A tag `v*` makes CI build the Windows and Linux installers, sign the updater bundles, and publish them with `latest.json`.
`latest.json` names a signature and a URL for each platform, and each runner builds one platform.

## Decision

Each build job of `.github/workflows/release.yml` runs `tauri-apps/tauri-action` with the draft release's id; the action uploads its bundles with their `.sig` files and merges its platform into `latest.json`.
Source: Softov, asked "p4 q6: how does the release workflow build and publish?": "tauri-apps/tauri-action".

## Consequences

The workflow has no script of its own for the uploads or for `latest.json`, whose shape follows the updater the action is made for.
The workflow depends on the action's version and inputs, which a Tauri upgrade may change.

## Options

- **Own steps and a script.** Lost: the workflow would carry a script for the uploads and the shape of `latest.json` that the action already keeps.
