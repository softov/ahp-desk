---
title: The Linux desk's bundled npm works when ahpd runs it - implemented
date: 2026-10-30
refs:
  - git://2b22a61
  - "[code://scripts/stage-node.mjs](../../../../scripts/stage-node.mjs)"
  - "[code://DEVELOPER.md](../../../../DEVELOPER.md)"
---

On Linux, the first run of "This computer" installs ahpd and every chosen backend.
The desk ships `node/bin/npm`, `npx` and `corepack` as scripts that run their target with the bundled Node, so the `npm` that ahpd finds on `PATH` works in a deb.

## What was built

- [`code://scripts/stage-node.mjs`](../../../../scripts/stage-node.mjs) - on Linux, each symlink in the staged `bin/` becomes a `sh` script; `npm --version` runs through it; the stage stamp is `<name>+bin-scripts`.
- [`code://DEVELOPER.md`](../../../../DEVELOPER.md) - one line on the scripts.

## Verified

- `pnpm typecheck`, `pnpm test` (the Rust 147, the embedder's 16, the shell's 56) and `cargo clippy --all-targets -- -D warnings` on Windows.
- On the Debian 13 test box, 2026-10-30, at e5498ac: `pnpm stage:node` printed npm 10.9.9, and `file` called the three `bin/` entries POSIX shell scripts.
- In the debug deb from `pnpm tauri build --debug --bundles deb`, `node/bin/npm` is the 118-byte script; `npm` and `npx` printed 10.9.9 and `corepack` 0.36.0.
- With that deb installed and no ahpd, desk data or ahpd config on the box, the first run of "This computer" with Claude, pi, cofold and Bots ticked opened the app window on "This computer" with "ahpd 0.14.0 Connected" in about 50 seconds; ahpd's log listed `plugins agent-claude, agent-pi, agent-cofold, bot`, with nothing skipped.

## Departures from the plan

- none.

## Left for later

- Desks up to 0.2.3 on Linux stay broken until they update to the next release.
- The ahpd issue, so that ahpd finds npm beside its own Node on every platform, is Softov's to file.
