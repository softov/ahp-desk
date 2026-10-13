---
title: The local ahpd
domain: desk
status: active
priority: high
created: 2026-10-11
revalidated: 2026-10-13
requires:
  - plans/desk/01-first-build-p1-shell-and-url-hosts/plan.md
decisions:
  - decisions/a-local-ahpd-is-found-or-installed-with-the-bundled-node.md
  - decisions/the-backends-are-picked-in-add-host.md
  - decisions/stopping-the-local-ahpd-on-quit-is-a-host-setting.md
  - decisions/the-local-ahpd-runs-through-its-own-start-and-stop.md
  - decisions/the-local-ahpd-uses-a-token-file.md
  - decisions/ahpd-updates-itself-when-the-first-install-says-so.md
  - decisions/the-installed-ahpd-keeps-the-usual-config-folder.md
refs:
  - file:///github/ahpd/packages/server/src/daemon.ts - `ahpd start`, `stop` and `status`, with daemon.json, at lines 10-37, 189-196 and 436-439
  - file:///github/ahpd/packages/server/src/commands/options.ts - the token from `--connection-token-file`, at lines 1051-1085
  - file:///github/ahpd/packages/server/src/commands/run.ts - the exit with no backend at lines 848-852, and the ready line at lines 958-1003
  - file:///github/ahpd/packages/server/src/config.ts - the config folder, `~/.config/ahpd`, at lines 282-299
  - file:///github/opendoop/packages/tauri/src-tauri/src/pood_process.rs - how doop finds Node and spawns its daemon
  - file:///github/ahp-desk/mockups/02-hosts.md - Add host, kind "This computer"
---

## Goal

The host list has "This computer".
The shell uses an ahpd on `PATH`, or installs one with the Node that it ships, with the backends that the user picks.
It starts, finds and stops that ahpd, with the two settings for the start and the quit.

## Reconnaissance

The files read and the patterns to reuse are the `refs` above, each with its note.

### Searches performed

- `rg "bin" ahpd/packages/server/package.json` - the bin `ahpd`, Node 22 or later.
- `rg "426" ahpd/packages/server/src/http.ts` - a plain GET on the port answers 426, so it is up.

### Runtime path

```
Add host (This computer) -> find ahpd on PATH or the set path
  -> or: bundled node + npm -> npm install @ahpd/server + backends into <app data>/ahpd
  -> ahpd status --json -> ahpd start -> daemon.json -> Host { kind: local, url } -> hosts-changed
quit -> stop setting on -> ahpd stop
```

### Gaps

- `ahpd stop` is not graceful on Windows: SIGTERM there is a hard kill, and the AHP `shutdown` method signals itself the same way (task 01).
- A bare plugin name resolves from the config folder, then from `AHPD_PLUGIN_ROOT` (task 01).
- The Node runtime for each OS, and how the build fetches it.

## Decisions locked in

| # | Decision | Rationale / source |
| --- | --- | --- |
| 1 | [A local ahpd is found, or installed with the bundled Node](../../../decisions/a-local-ahpd-is-found-or-installed-with-the-bundled-node.md) | Softov, 2026-10-11 |
| 2 | [The backends are picked in Add host](../../../decisions/the-backends-are-picked-in-add-host.md) | Softov, 2026-10-11 |
| 3 | [Stopping the local ahpd on quit is a host setting](../../../decisions/stopping-the-local-ahpd-on-quit-is-a-host-setting.md) | Softov, 2026-10-11 |
| 4 | [The local ahpd runs through its own start and stop](../../../decisions/the-local-ahpd-runs-through-its-own-start-and-stop.md) | Softov, 2026-10-11 |
| 5 | [The local ahpd uses a token file](../../../decisions/the-local-ahpd-uses-a-token-file.md) | Softov, 2026-10-11 |
| 6 | [ahpd updates itself when the first install says so](../../../decisions/ahpd-updates-itself-when-the-first-install-says-so.md) | Softov, 2026-10-11 |
| 7 | [An ahpd the desk installs keeps the usual config folder, with its backends under AHPD_PLUGIN_ROOT](../../../decisions/the-installed-ahpd-keeps-the-usual-config-folder.md) | Softov, 2026-10-13 |

| What | Source | Task |
| --- | --- | --- |
| The install folder is `<app local data>/ahpd`, an npm prefix. | Decision 1 | 03 |
| "Start when the desktop app starts" is on by default; "Stop when the desktop app quits" is off. | (defaulted) | 05 |

## Proposed architecture

- **Data flow** - `local_ahpd.rs` finds or installs ahpd, runs `status`, `start` and `stop`, and reads `daemon.json`.
- **Event flow** - The install sends its steps over a Tauri `Channel`, as doop's `remote_install` does.
- **State flow** - The local host in `hosts.json` holds `ahpdPath`, `startOnLaunch`, `stopOnQuit`, `autoUpdate` and `startedByDesk`.
- **Layer responsibilities** - `src-tauri`: find, install, start, stop · `packages/shell`: Add host "This computer" and the install steps.
- **Source-of-truth files** - `src-tauri/src/local_ahpd.rs`.

## Tasks

| Task | Status | Depends on |
| --- | --- | --- |
| [01 - check ahpd start, stop and status against the needs of the shell](task-01-check-ahpd-commands.md) | done | - |
| [02 - find ahpd on PATH or at a set path](task-02-find-ahpd.md) | todo | 01 |
| [03 - ship Node and install ahpd with the chosen backends](task-03-install-ahpd.md) | todo | 01 |
| [04 - start, find and stop through ahpd's own commands](task-04-start-find-stop.md) | todo | 02 |
| [05 - the start and quit settings](task-05-start-quit-settings.md) | todo | 04 |
| [06 - Add host, This computer, with the install steps](task-06-add-this-computer.md) | todo | 03, 04 |
| [07 - automatic update of ahpd, and its checkbox](task-07-auto-update.md) | todo | 03, 04 |
| [08 - end to end on a clean Windows user](task-08-end-to-end.md) | todo | 05, 06, 07 |

## Risks and tradeoffs

- `ahpd stop` is a hard kill on Windows until ahpd changes, so the stop on quit waits for that change.
- An npm install can fail behind a proxy. The install steps show npm's own output.

## Resume state

- **Done so far:** the task files, and task 01, 2026-10-13, on `@ahpd/server` 0.11.0.
- **Next action:** task 02.
- **Open questions:** none for the desk; a graceful `ahpd stop` on Windows is an ahpd change, taken up in ahpd's own session.
- **Watch out for:** an ahpd that the shell found and did not start is never stopped by the shell.

## Final verification checklist

- [ ] On a Windows user with no Node, Add host "This computer" installs ahpd and connects.
- [ ] With ahpd already on `PATH`, the shell uses it and installs nothing.
- [ ] Quit with "Stop when the desktop app quits" on stops ahpd; with it off, ahpd stays on.
- [ ] `plans/index.md` updated.
