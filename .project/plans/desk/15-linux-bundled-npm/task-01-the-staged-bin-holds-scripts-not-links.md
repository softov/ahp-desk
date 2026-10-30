---
title: The staged Node's bin holds scripts, not links, on Linux
status: doing
depends: []
layer: "scripts"
refs:
  - "[code://scripts/stage-node.mjs](../../../../scripts/stage-node.mjs) - where the runtime is unpacked and renamed into place"
  - "[code://DEVELOPER.md](../../../../DEVELOPER.md) - says what `pnpm stage:node` stages"
---

## Objective

On Linux, `src-tauri/resources/node/bin/` holds `npm`, `npx` and `corepack` as scripts that run their target with the bundled `node`.
A deb or AppImage built from it ships a `bin/npm` that works when ahpd runs `npm` from `PATH`.

## Files

- `UPDATE: scripts/stage-node.mjs` - after the unpack check, on Linux, each symlink in `bin/` becomes a script; `npm --version` runs through the new script; the stamp is `<name>+bin-scripts`.
- `UPDATE: DEVELOPER.md:105` - one line on the scripts in `bin/` (decision `the-bundled-nodes-npm-runs-from-bin-on-linux`).

## Steps

1. In `stage-node.mjs`, between the `did not unpack` check and `rmSync(to, ...)`, when `os === 'linux'`: for each entry of `<unpack>/<name>/bin` that `lstatSync` says is a symlink, read its target, remove it, and write `#!/bin/sh`, `here=$(dirname "$(readlink -f "$0")")` and `exec "$here/node" "$here/<target>" "$@"` with mode `0755`.
2. Run `bin/npm --version` there, so a broken script stops the stage.
3. Set the stamp to `${name}+bin-scripts`, read and written through one `staged` const.

## Validation

- `pnpm typecheck`, `pnpm test` (with `CARGO_TARGET_DIR=F:/github/ahp-desk/src-tauri/target`) and `cargo clippy --all-targets -- -D warnings` in `src-tauri` pass on Windows, where the Linux branch does not run and the runtime is staged again for the new stamp.
- By hand on the Debian box: `pnpm stage:node` prints npm's version; `file src-tauri/resources/node/bin/npm` says "POSIX shell script".
- By hand on the Debian box: `pnpm tauri build --debug`, install the deb, remove `~/.local/share/dev.softov.ahpdesk`, `~/.config/dev.softov.ahpdesk` and `~/.config/ahpd`, and walk the first run of "This computer" with all four backends: it ends with ahpd running and the host online.

## Resume

2026-10-30: the code and the DEVELOPER.md line are written; the checks on Windows run; the Debian checks are left.
