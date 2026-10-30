---
title: The Linux desk's bundled npm works when ahpd runs it
domain: desk
status: built
priority: high
created: 2026-10-30
revalidated: 2026-10-30
requires:
  - plans/desk/03-local-ahpd/plan.md
decisions:
  - decisions/a-local-ahpd-is-found-or-installed-with-the-bundled-node.md
  - decisions/the-bundled-nodes-npm-runs-from-bin-on-linux.md
refs:
  - "[code://scripts/stage-node.mjs](../../../../scripts/stage-node.mjs) - stages Node's tarball as it unpacks, links and all"
  - "[code://src-tauri/tauri.conf.json#L23](../../../../src-tauri/tauri.conf.json#L23) - `resources/node/` is bundled as `node/`"
  - "[code://src-tauri/src/local_ahpd.rs#L82-L117](../../../../src-tauri/src/local_ahpd.rs#L82-L117) - `BundledNode`: the desk runs `npm-cli.js` itself, and gives ahpd a `PATH` with `node/bin` first"
  - "[code://src-tauri/src/local_ahpd.rs#L201-L237](../../../../src-tauri/src/local_ahpd.rs#L201-L237) - `install`: npm for the server, then ahpd's `plugin install` for the backends"
  - file:///github/ahpd/packages/server/src/plugins.ts - at ahpd acdcd7ae, line 264 runs `npm` by name
  - https://github.com/softov/ahp-desk/releases/tag/v0.2.3 - its deb holds `usr/lib/ahp-desk/node/bin/npm` as a 54-byte file, not a link
---

## Goal

On Linux, the first run of "This computer" installs ahpd and every chosen backend.
Today the deb, and the AppImage built from the same resources, install `@ahpd/server` and then stop with `ahpd plugin install exited with exit status: 2` and `Error: Cannot find module '../lib/cli.js'`.
ahpd's own update of a desk-installed ahpd runs the same `npm`, and is fixed by the same change.

## Reconnaissance

The files read and the patterns to reuse are the `refs` above, each with its note.

### Searches performed

- `tar -tvf data.tar.gz` of `ahp-desk_0.2.3_amd64.deb` - `node/bin/npm` (54 bytes), `npx` and `corepack` are plain files.
- `tar -tvf node-v22.23.3-linux-x64.tar.gz` - `bin/npm`, `bin/npx` and `bin/corepack` are the only symlinks, each to `../lib/node_modules/...`.
- `git grep "'npm'" origin/main -- packages/server/src` in ahpd - `install.ts`, `plugins.ts`, `pluginschannel.ts` and `selfupdate.ts` run `npm` by name; only `install.ts:107-109` looks beside `execPath`, and only on Windows.

### Runtime path

```
first run -> LocalAhpd::install -> node npm-cli.js install @ahpd/server (works)
          -> ahpd plugin install (PATH = node/bin first) -> npm -> node/bin/npm, a copy -> require('../lib/cli.js') fails
```

### Gaps

- `stage-node.mjs` keeps the tarball's symlinks, and Tauri's bundler copies their targets.

## Decisions locked in

| # | Decision | Rationale / source |
| --- | --- | --- |
| 1 | [The bundled Node's npm runs from its bin folder on Linux, through a script the desk stages](../../../decisions/the-bundled-nodes-npm-runs-from-bin-on-linux.md) | Softov, 2026-10-30: "Desk shims + ahpd issue (Recommended)" |

| What | Source | Task |
| --- | --- | --- |
| The fix is recorded as a plan of its own. | Softov, 2026-10-30, asked "Where should the record of this fix go?": "A new plan 15" | - |
| Every symlink in the staged `bin/` becomes a script, not only `npm`. | (defaulted: `npx` and `corepack` are copied the same way and break the same way) | 01 |
| The stage stamp becomes `<name>+bin-scripts`, so a runtime staged before is staged again. | (defaulted: a stale stage would ship the copies) | 01 |

## Proposed architecture

- **Data flow** - `stage-node.mjs` writes each `bin/<tool>` as `exec "$here/node" "$here/<link target>" "$@"`, with `here` from `readlink -f "$0"`.
- **Event flow** - none.
- **State flow** - none.
- **Layer responsibilities** - `scripts/`: the staged layout · `src-tauri`: unchanged.
- **Source-of-truth files** - [`code://scripts/stage-node.mjs`](../../../../scripts/stage-node.mjs).

## Tasks

| Task | Status | Depends on |
| --- | --- | --- |
| [01 - the staged bin holds scripts, not links](task-01-the-staged-bin-holds-scripts-not-links.md) | done | - |

## Risks and tradeoffs

- No Linux machine builds a deb in CI before a release - the check is a local deb build on the Debian box, and the next release.
- Desks up to 0.2.3 stay broken on Linux until they update - the ahpd issue makes a newer ahpd work there too.

## Resume state

- **Done so far:** task 01, with its checks on Windows and on the Debian box, 2026-10-30.
- **Next action:** none; the fix ships in the next release.
- **Open questions:** none.
- **Watch out for:** the scripts are written in the unpack folder before it is renamed into place, so the `npm --version` check runs on what will be staged.

## Final verification checklist

- [x] `pnpm typecheck`, `pnpm test` and `cargo clippy --all-targets -- -D warnings` pass.
- [x] By hand on Debian, with a local deb build of this branch: the first run of "This computer" from zero installs ahpd and all four backends.
- [x] `plans/index.md` updated.
