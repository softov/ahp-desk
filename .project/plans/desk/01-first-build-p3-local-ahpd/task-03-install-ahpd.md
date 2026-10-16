---
title: The desk ships Node and installs ahpd with the chosen backends
status: done
depends: [task-01-check-ahpd-commands.md]
layer: "src-tauri, build"
refs:
  - file:///github/opendoop/packages/tauri/src-tauri/src/pood_process.rs - doop ships Node as a resource
  - file:///github/ahpd/packages/server/src/install.ts - `pluginRoot()` and `AHPD_PLUGIN_ROOT`
---

## Objective

With no ahpd, the desk installs `@ahpd/server` and the backends picked in Add host with npm from its bundled Node 22, into `<app local data>/ahpd`, and streams npm's steps.

## Files

- `UPDATE: src-tauri/src/local_ahpd.rs` - `BundledNode`, `LocalAhpd::install()`, `install_local_ahpd`.
- `UPDATE: src-tauri/tauri.conf.json` - the Node runtime as a resource, and `pnpm stage:all` before dev and build.
- `CREATE: scripts/stage-node.mjs` - fetches the official Node 22 build for the OS being built, checked against its SHASUMS256.
- `UPDATE: package.json`, `.gitignore`, `src-tauri/build.rs`, `src-tauri/capabilities/shell.json`, `DEVELOPER.md`.

## Steps

1. Fetch the Node 22 build at build time, as decision 1 says.
2. Install into `<app local data>/ahpd` with `npm install --prefix`, streaming each line over a Tauri `Channel`.
3. Run every command of that install with `AHPD_PLUGIN_ROOT=<app local data>/ahpd`, keeping `~/.config/ahpd` as its config folder (decision 7).

## Validation

- On a user with no Node: the install ends with `ahpd --version` from the installed folder, and a start loads the picked backends.

## Resume

Started 2026-10-13.

- Done: `scripts/stage-node.mjs` stages Node v22.23.3 (win-x64 zip, linux tar.gz), checked against SHASUMS256 and cached in `.cache/node/`; the build bundles it as the `node/` resource.
- Done: `LocalAhpd::install()` runs the bundled `npm-cli.js` on the bundled Node, `npm install --prefix <app local data>/ahpd @ahpd/server@latest`, streaming each line over a Tauri `Channel`; one install at a time.
- Done: the installed ahpd runs on the bundled Node with `AHPD_PLUGIN_ROOT` and that Node first on `PATH`.
- Done: `find_local_ahpd` and `install_local_ahpd` are permissions, granted to the shell window; task 02 had left `find_local_ahpd` out.
- Verified: the ignored test `local_ahpd_a_real_install` installs `@ahpd/server` 0.11.0 from npm into a scratch prefix and reads its version on the bundled Node.
- Done: the backends, through the installed ahpd's `plugin install` (decision 8), after a check that each is an `@ahpd/` package and that there is at least one; `installLocalAhpd(backends, onEvent)` in `packages/shell/src/hosts/ipc.ts`.
- Found: on `@ahpd/server` 0.11.0 the real install failed at `ahpd plugin install`, which ran `spawn('npm')` with no shell (ENOENT on Windows).
- Verified on 2026-10-16 with `@ahpd/server` 0.12.0: the ignored test `local_ahpd_a_real_install` passes on Windows.
  It installs ahpd and `@ahpd/agent-claude` 0.12.0 with the bundled Node, and the backend is in the prefix and in ahpd's config.
  `HOME`, `USERPROFILE`, `TEMP` and `XDG_CONFIG_HOME` pointed at scratch folders.
