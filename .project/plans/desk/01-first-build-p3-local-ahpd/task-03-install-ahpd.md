---
title: The desk ships Node and installs ahpd with the chosen backends
status: todo
depends: [task-01-check-ahpd-commands.md]
layer: "src-tauri, build"
refs:
  - file:///github/opendoop/packages/tauri/src-tauri/src/pood_process.rs - doop ships Node as a resource
  - file:///github/ahpd/packages/server/src/install.ts - `pluginRoot()` and `AHPD_PLUGIN_ROOT`
---

## Objective

With no ahpd, the desk installs `@ahpd/server` and the backends picked in Add host with npm from its bundled Node 22, into `<app local data>/ahpd`, and streams npm's steps.

## Files

- `UPDATE: src-tauri/src/local_ahpd.rs` - `install(backends, channel)`.
- `UPDATE: src-tauri/tauri.conf.json` - the Node runtime as a resource.
- `CREATE: scripts/fetch-node.mjs` - fetches the official Node 22 build for the OS being built, checked against its SHASUMS.

## Steps

1. Fetch the Node 22 build at build time, as decision 1 says.
2. Install into `<app local data>/ahpd` with `npm install --prefix`, streaming each line over a Tauri `Channel`.
3. Run every command of that install with `AHPD_PLUGIN_ROOT=<app local data>/ahpd`, keeping `~/.config/ahpd` as its config folder (decision 7).

## Validation

- On a user with no Node: the install ends with `ahpd --version` from the installed folder, and a start loads the picked backends.

## Resume
