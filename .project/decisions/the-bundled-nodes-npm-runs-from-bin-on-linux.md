---
title: The bundled Node's npm runs from its bin folder on Linux, through a script the desk stages
status: accepted
date: 2026-10-30
refs:
  - "[code://scripts/stage-node.mjs](../../scripts/stage-node.mjs) - stages the Node runtime the desk ships"
  - "[code://src-tauri/src/local_ahpd.rs#L106-L117](../../src-tauri/src/local_ahpd.rs#L106-L117) - `path_var`, which puts the bundled Node's folder first on `PATH` for ahpd"
  - file:///github/ahpd/packages/server/src/install.ts - at ahpd acdcd7ae, lines 107-109 look for npm beside its Node on Windows only; elsewhere it runs `npm` from `PATH`
---

## Context

Node's Linux build has `bin/npm`, `bin/npx` and `bin/corepack` as symlinks into `lib/node_modules/`.
Tauri copies a resource through its symlink, so the desk 0.2.3 deb ships `node/bin/npm` as a copy of `npm-cli.js`.
That copy's `require('../lib/cli.js')` looks for `node/lib/cli.js`, which does not exist.
The desk installs `@ahpd/server` by running `npm-cli.js` itself, so that step works.
ahpd's `plugin install` and its self update run `npm` from `PATH`, where the desk puts `node/bin` first, so they fail with `Cannot find module '../lib/cli.js'`.
On Debian, the first run of "This computer" stops at the backends.

## Decision

On Linux, `stage-node.mjs` replaces each symlink in the staged Node's `bin/` with a shell script that runs the link's target with that Node.
The desk ships these scripts, so the `npm` that ahpd finds on `PATH` works in a deb and an AppImage.
ahpd is also asked, with an issue, to find npm beside its own Node on every platform.
Source: Softov, 2026-10-30, asked "The Linux desk ships node/bin/npm as a broken copy, so `ahpd plugin install` fails on the first run. How should I fix it?": "Desk shims + ahpd issue (Recommended)".

## Consequences

Released desks up to 0.2.3 stay broken on Linux until they update to a desk with the scripts.
The scripts need `/bin/sh` and `readlink -f`, which every supported Linux has.
A Node version that adds another link in `bin/` gets a script too, with no change here.

## Options

- **ahpd only.** ahpd finds npm beside `process.execPath` on every platform. Lost: every released desk stays broken until ahpd updates, and the desk's `PATH` still offers a broken `npm` to anything else.
- **A `PATH` folder of the desk's own.** The desk writes scripts at run time into its data folder and puts that first. Lost: a second place for the runtime's layout, written on every start.
