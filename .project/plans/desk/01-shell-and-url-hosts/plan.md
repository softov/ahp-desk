---
title: The shell, the host list and the app window
domain: desk
status: built
priority: high
created: 2026-10-11
revalidated: 2026-10-11
requires: []
decisions:
  - decisions/ahpw-owns-its-embedding-contract.md
  - decisions/tokens-are-in-the-os-keyring-with-hosts-json-as-the-fallback.md
  - decisions/each-ui-is-served-from-its-own-uri-scheme.md
  - decisions/each-desk-build-ships-one-ui.md
refs:
  - file:///github/opendoop/packages/tauri/src-tauri/Cargo.toml - the crate set to start from
  - file:///github/opendoop/packages/tauri/src-tauri/tauri.conf.json - `windows: []`, every window built in Rust
  - file:///github/opendoop/packages/tauri/src-tauri/src/profile.rs - the atomic write of a JSON file in the app config folder, at lines 230-242
  - file:///github/opendoop/packages/tauri/src-tauri/src/lib.rs - `open_web_ui`, an async command that builds a window, at lines 50-129
  - file:///github/opendoop/packages/tauri/src-tauri/capabilities/web-ui.json - window commands for a page that draws its own title bar
  - file:///github/opendoop/packages/tauri-shell/vite.config.ts - the shell UI's Vite setup on a fixed port
  - file:///github/ahpw/vite.config.ts - ahpw's dev server on 127.0.0.1:5180, and dist/app
  - file:///github/ahp-desk/mockups/02-hosts.md - the host list and Add host
  - file:///github/ahp-desk/mockups/03-app-window.md - the app window and the contract arrows
  - npm://@ahpd/web@0.2.0 - the published ahpw
  - npm://@tauri-apps/api@^2 - invoke, listen and the window API for desk-client
  - https://crates.io/crates/keyring - the OS keyring from Rust
---

## Goal

ahp-desk runs as a Tauri 2 app with a host list of URL hosts.
Each token is in the OS keyring or, with no keyring, in `hosts.json`.
"Open" on a host opens an app window with ahpw, from the `local` or the `npm` source.
The `@ahpd/desk-client` package gives a UI the hosts, its window's host, a token, and the window controls.
After this plan, ahpw loads in the window but still connects to its own origin; plan 02 changes that.

## Reconnaissance

The files read and the patterns to reuse are the `refs` above, each with its note.

### Searches performed

- `rg "tauri::command" opendoop/packages/tauri/src-tauri/src` - doop's commands, all in `lib.rs` and one module each.
- `rg "base|outDir|port" ahpw/vite.config.ts` - `base: './'`, `outDir: 'dist/app'`, port 5180.
- `rg "keyring" opendoop` - `Not found: keyring - searched keyring in opendoop/packages/tauri.`

### Runtime path

```
pnpm tauri dev -> lib.rs setup -> shell window (packages/shell) -> host list
  -> invoke('save_host') -> hosts.rs -> tokens.rs -> keyring, or hosts.json with no keyring -> emit('hosts-changed')
  -> invoke('open_app_window', {hostId}) -> app_window.rs -> window app-<id> at ahpw://localhost/
  -> ui_source.rs serves dist/app from the local or npm source
```

### Gaps

- No workspace, no Tauri app, no packages.
- No contract package.

## Answers

| # | What | Source |
| --- | --- | --- |
| 1 | [A UI takes its hosts from a command and an event](../../../specs/host-list.md) | Softov, 2026-10-11 |
| 2 | [The shell's contract is a shared npm package](../../../decisions/ahpw-owns-its-embedding-contract.md) | Softov, 2026-10-11 |
| 3 | [A UI bundle comes from the local repository or from npm](../../../specs/app-window.md) | Softov, 2026-10-11 |
| 4 | [Tokens are in the OS keyring, with hosts.json as the fallback](../../../decisions/tokens-are-in-the-os-keyring-with-hosts-json-as-the-fallback.md) | Softov, 2026-10-11 |
| 5 | [Each UI is served from its own URI scheme](../../../decisions/each-ui-is-served-from-its-own-uri-scheme.md) | Softov, 2026-10-11 |
| 6 | [Each desk build ships one UI](../../../decisions/each-desk-build-ships-one-ui.md) | Softov, 2026-10-11 |

| What | Source | Task |
| --- | --- | --- |
| The package is `@ahpd/desk-client`. | (defaulted: the `@ahpd` scope of `@ahpd/web`) | 02 |
| `hosts()` returns no tokens; `hostToken(id)` returns one. | Decision 1: "Tokens stay in the OS keyring until a UI asks for them." | 02, 03 |
| The build setting `AHP_DESK_UI=ahpw|ahpapp` chooses the build's UI, default `ahpw`; `ahpapp` fails until plan 05. | Softov: "AHP_DESK_UI on p1, could be." | 04 |
| The source is set by `AHP_DESK_AHPW=local|npm`, default `npm`, and `AHP_DESK_AHPW_DIR`, default `../ahpw`. | (defaulted) | 04 |
| The app identifier is `dev.softov.ahpdesk`. | (defaulted) | 01 |
| The app window has no OS frame; the UI draws its title bar. | Mockup 03: "title bar drawn by the UI" | 05 |
| A window is one host; Open on a host with a window focuses it. | [ahpw shows one host at a time on the desk](../../../specs/app-window.md) | 05 |

## Proposed architecture

- **Data flow** - `packages/shell` calls the host commands; `hosts.rs` writes `hosts.json` and the keyring. A UI in an app window calls `window_host` and `host_token`.
- **Event flow** - `hosts.rs` emits `hosts-changed` with the host list, without tokens, after every save or remove.
- **State flow** - `hosts.json` in `app_config_dir`. The map of window label to host id is in memory, in `AppWindows`.
- **Layer responsibilities** - `src-tauri/src/hosts.rs`: the host list · `src-tauri/src/ui_source.rs`: the bundles · `src-tauri/src/app_window.rs`: the app windows · `packages/shell`: the screens · `packages/desk-client`: the contract.
- **Source-of-truth files** - `packages/desk-client/src/index.ts`, `src-tauri/src/hosts.rs`.

## Tasks

| Task | Status | Depends on |
| --- | --- | --- |
| [01 - workspace-scaffold](task-01-workspace-scaffold.md) | done | - |
| [02 - desk-client](task-02-desk-client.md) | done | 01 |
| [03 - host-store](task-03-host-store.md) | done | 01 |
| [04 - ui-sources](task-04-ui-sources.md) | done | 01 |
| [05 - app-window](task-05-app-window.md) | done | 03, 04 |
| [06 - host-list](task-06-host-list.md) | done | 02, 03, 05 |

## Risks and tradeoffs

- Tauri may treat a custom scheme page as a remote origin for IPC. Task 05 tests `invoke` from `ahpw://` before task 06 depends on it.
- `keyring` on Linux needs a Secret Service. With none, task 03 saves the token in `hosts.json`, and task 06 shows a warning.
- `open_app_window` must be async, or the window build can deadlock on Windows, as doop's note says.

## Resume state

- **Done so far:** every task; see [implemented.md](implemented.md).
- **Next action:** none; plan 02 builds on it.
- **Open questions:** none.
- **Watch out for:** nothing.

## Final verification checklist

- [x] `pnpm tauri dev` opens the host list on Windows.
- [x] A URL host saved in the host list survives a restart, and its token is in the Windows Credential Manager.
- [x] Open shows ahpw's sign-in wall with `AHP_DESK_AHPW=npm` and with `AHP_DESK_AHPW=local`.
- [x] `cargo clippy -- -D warnings`, `cargo test`, `pnpm -r typecheck` and `pnpm -r test` pass.
- [x] `plans/index.md` updated.
