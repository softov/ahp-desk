---
title: The desk finds a local ahpd that its PATH misses, and starts its own on a free port
domain: desk
status: planned
priority: high
created: 2026-10-31
revalidated: 2026-10-31
requires:
  - plans/desk/03-local-ahpd/plan.md
  - plans/desk/13-about-and-update-gaps/plan.md
changes:
  - specs/local-ahpd.md
decisions:
  - decisions/a-local-ahpd-off-path-is-found-through-its-record-or-npms-prefix.md
  - decisions/a-local-ahpd-found-off-path-is-used-as-one-on-path.md
  - decisions/a-local-install-starts-on-the-first-free-port.md
  - decisions/the-probe-finds-a-running-ahpd-through-its-record.md
  - decisions/the-local-ahpd-runs-through-its-own-start-and-stop.md
refs:
  - "[code://src-tauri/src/local_ahpd.rs#L403-L440](../../../../src-tauri/src/local_ahpd.rs#L403-L440) - `on_path` and `find`: the set path, the desk's install, then `PATH`"
  - "[code://src-tauri/src/local_ahpd.rs#L491-L506](../../../../src-tauri/src/local_ahpd.rs#L491-L506) - `config_dir` and `this_config_dir`, where `daemon.json` is"
  - "[code://src-tauri/src/local_ahpd.rs#L555-L587](../../../../src-tauri/src/local_ahpd.rs#L555-L587) - `start`, which runs `ahpd start` with no port"
  - "[code://src-tauri/src/local_ahpd.rs#L654-L674](../../../../src-tauri/src/local_ahpd.rs#L654-L674) - `find_local_ahpd`, the command Add host asks"
  - "[code://src-tauri/src/local_ahpd.rs#L726-L745](../../../../src-tauri/src/local_ahpd.rs#L726-L745) - `launcher_for`, which every local command goes through"
  - "[code://src-tauri/src/local_ahpd.rs#L790-L803](../../../../src-tauri/src/local_ahpd.rs#L790-L803) - `start_host`"
  - "[code://src-tauri/src/hosts.rs#L494-L504](../../../../src-tauri/src/hosts.rs#L494-L504) - `HostStore::local`, which answers the set path and whether the desk started it, not the address"
  - "[code://src-tauri/src/ssh_install.rs#L89-L100](../../../../src-tauri/src/ssh_install.rs#L89-L100) - the SSH probe's record try, the pattern the local one mirrors"
  - "[code://src-tauri/src/ssh_install.rs#L208-L213](../../../../src-tauri/src/ssh_install.rs#L208-L213) - `first_free_port`, reused"
  - "[code://packages/shell/src/hosts/ipc.ts#L248-L262](../../../../packages/shell/src/hosts/ipc.ts#L248-L262) - `FoundAhpd.origin` and `findLocalAhpd`"
  - "[code://packages/shell/src/hosts/LocalForm.tsx#L40-L44](../../../../packages/shell/src/hosts/LocalForm.tsx#L40-L44) - `foundLine`, the words per origin"
  - "[code://packages/shell/src/hosts/LocalForm.tsx#L150-L153](../../../../packages/shell/src/hosts/LocalForm.tsx#L150-L153) - the host saved on `ws://127.0.0.1:9187`"
  - "[code://packages/shell/src/i18n.ts#L181-L185](../../../../packages/shell/src/i18n.ts#L181-L185) - `local.found*` in `en`; `ptBR` at 386 and `es` at 589"
  - file:///github/ahpd/packages/server/src/commands/options.ts - at ahpd acdcd7ae, line 496: `--port`, default 9187
---

## Goal

On Linux and macOS, Add host's This computer finds an ahpd that the desk's `PATH` does not show: a running one through its record, and a stopped one through npm's prefix.
It uses that ahpd as one on `PATH`, so it never installs a second ahpd over the same config folder.
When no ahpd is found and 9187 is taken, the desk's own ahpd is installed and started on the first free port.
Windows does not change.

## Reconnaissance

The files read and the patterns to reuse are the `refs` above, each with its note.

### Searches performed

- On the Debian test box, 2026-10-30: `~/.bashrc:123-124` holds ahpd's install script's `PATH` line for `~/.npm-global/bin`; the desk 0.2.3 said "Not on this computer" while that ahpd 0.13.0 ran on 9187.
- `rg "daemon.json|first_free_port|AHPD_PORT" src-tauri/src` - the SSH probe reads the record and lists ports; the local side reads the record only for the token (`token_in_record`).
- `git grep -- "--port" origin/main -- packages/server/src` in ahpd - `ahpd start` takes `--port`; `ahpd restart` replays the start line.

### Runtime path

```
Add host, This computer -> findLocalAhpd -> find_local_ahpd -> find: set, installed, PATH, (new) record, (new) npm prefix
  -> found: FoundAhpd { origin: 'running' | 'npm' } -> foundLine -> no install offered
  -> none: (new) local_free_port -> host saved on ws://127.0.0.1:<port> -> install_local_ahpd -> start_local_ahpd -> start_host -> ahpd start [--port <port>]
```

### Gaps

- `find` stops at `PATH`; `launcher_for` and `find_local_ahpd` read nothing else.
- `start_host` cannot pass a port: `HostStore::local` does not answer the host's address.
- No local command answers a free port.

## Decisions locked in

| # | Decision | Rationale / source |
| --- | --- | --- |
| 1 | [A local ahpd that PATH misses is found through its record, else through npm's prefix, on Linux and macOS](../../../decisions/a-local-ahpd-off-path-is-found-through-its-record-or-npms-prefix.md) | Softov, 2026-10-31: "Record + npm prefix (Recommended)" and "Linux and macOS (Recommended)" |
| 2 | [A local ahpd found through its record or npm's prefix is used as one on PATH](../../../decisions/a-local-ahpd-found-off-path-is-used-as-one-on-path.md) | Softov, 2026-10-31: "Use it as one on PATH (Recommended)" |
| 3 | [A local install starts ahpd on the first free port from 9187 up](../../../decisions/a-local-install-starts-on-the-first-free-port.md) | Softov, 2026-10-31: "Propose a free port (Recommended)" |

| What | Source | Task |
| --- | --- | --- |
| The order is the set path, the desk's install, `PATH`, the record, npm's prefix. | (defaulted: what already works keeps its place; the record before the prefix, since a running ahpd is the one in use) | 01 |
| npm's prefix is read from `~/.npmrc` only, not from `NPM_CONFIG_PREFIX`. | (defaulted: a desktop session lacks the shell's environment, so the variable would rarely be there) | 01 |
| An ahpd found through the record runs with its Node's folder first on `PATH`, as the SSH probe's does (task 13 step 4). | (defaulted: ahpd's own `npm` and `node` then come from that Node) | 01 |
| `start_host` passes `--port` only when the host's address is not on 9187. | (defaulted: an ahpd the desk did not install keeps its own config's port) | 03 |
| The free port is checked by binding `127.0.0.1:<port>` and letting go. | (defaulted: no `/proc` on macOS, and binding is what ahpd will do) | 03 |
| The origins are `running` and `npm`, with the words "running" and "in npm's global folder". | (defaulted) | 02 |

## Proposed architecture

- **Data flow** - `find` takes two more inputs, the config folder and the home folder, and answers `Origin::Running` with a launcher of the record's Node and script, or `Origin::Npm` with `<prefix>/bin/ahpd`; `local_free_port()` answers a port; `HostStore::local` also answers the host's port.
- **Event flow** - none new.
- **State flow** - the host's `url` holds the port, as today; `set_local` still writes what ahpd reports after a start.
- **Layer responsibilities** - `src-tauri`: the finding, the free port, `--port` on start · `packages/shell`: the origins' words, the port the host is saved on, the moved-port note.
- **Source-of-truth files** - [`code://src-tauri/src/local_ahpd.rs`](../../../../src-tauri/src/local_ahpd.rs), [`code://packages/shell/src/hosts/LocalForm.tsx`](../../../../packages/shell/src/hosts/LocalForm.tsx).

## Tasks

| Task | Status | Depends on |
| --- | --- | --- |
| [01 - find a local ahpd through its record and npm's prefix](task-01-find-through-the-record-and-npms-prefix.md) | todo | - |
| [02 - Add host names the new origins](task-02-add-host-names-the-new-origins.md) | todo | 01 |
| [03 - a local install starts on the first free port](task-03-a-local-install-starts-on-a-free-port.md) | todo | - |
| [04 - by hand on the Debian test box](task-04-by-hand-on-debian.md) | todo | 01, 02, 03 |

## Risks and tradeoffs

- `ps -o args=` joins arguments with spaces - a path with a space is not read; the record try then finds nothing and npm's prefix is tried.
- A `#!/usr/bin/env node` script found through npm's prefix needs a `node` on the desk's `PATH` - when there is none, `--version` fails and Add host shows that error, not "Not on this computer".
- The record names a pid that may have been reused - the command line must show `ahpd` or `@ahpd/server`, else the record is ignored.

## Resume state

- **Done so far:** the plan and its decisions, 2026-10-31.
- **Next action:** [task-01-find-through-the-record-and-npms-prefix.md](task-01-find-through-the-record-and-npms-prefix.md); task 03 can go beside it.
- **Open questions:** none.
- **Watch out for:** the desk's own ahpd also writes `daemon.json`, but `Origin::Installed` is tried first, so the record never names it as found; `stop_host` still leaves on an ahpd the desk did not start.

## Final verification checklist

- [ ] `pnpm typecheck`, `pnpm test` and `cargo clippy --all-targets -- -D warnings` pass.
- [ ] By hand on Debian: a running global ahpd is found as running; a stopped one through npm's prefix; with 9187 taken and no ahpd, the install starts on 9188 and starts there again after a desk restart.
- [ ] By hand on Debian: plan 13's check, a local ahpd 0.13.0 the desk did not install shows Update.
- [ ] `specs/local-ahpd.md` corrected: where an ahpd is found, and the port of an install.
- [ ] `plans/index.md` updated.
