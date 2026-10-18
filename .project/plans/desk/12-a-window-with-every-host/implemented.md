---
title: A window with every host - implemented
date: 2026-10-18
refs:
  - git://2d0e852
  - npm://@ahpd/web@0.5.0 - the ahpw release with the multi contract
  - "[code://src-tauri/src/app_window.rs](../../../../src-tauri/src/app_window.rs)"
  - "[code://src-tauri/src/settings.rs](../../../../src-tauri/src/settings.rs)"
  - "[code://packages/ahpw-embedder/src/embedder.ts](../../../../packages/ahpw-embedder/src/embedder.ts)"
---

A person chooses in Settings whether hosts open one window each, as before and by default, or one window with every host.
With every host, Open on a host shows that one window filtered to the host, and its ahpw connects to all the hosts.

## What was built

- [`code://src-tauri/src/settings.rs`](../../../../src-tauri/src/settings.rs) - `Windows`, `per-host` by default, kept as `windows` in `settings.json`.
- [`code://packages/shell/src/settings/SettingsScreen.tsx`](../../../../packages/shell/src/settings/SettingsScreen.tsx) - "Hosts open in" as a group of radio buttons, in every locale.
- [`code://src-tauri/src/app_window.rs`](../../../../src-tauri/src/app_window.rs) - the window with every host, `app-:every`, opened on `?hosts=every&focus=<id>` or shown with `focus-host`; `may_read` lets it read every token; `show_host_list` takes `edit`.
- [`code://packages/ahpw-embedder/src/embedder.ts`](../../../../packages/ahpw-embedder/src/embedder.ts) - `mode`, `token(id)`, `onFocusHost` and `manageHosts({ edit })`, on `@ahpd/web` 0.5.0.

## Verified

- `cargo test` and `cargo clippy --all-targets -- -D warnings`, with the settings round trip, the window label and the token cases.
- `pnpm typecheck`, `pnpm test` (the embedder's 16, the shell's 24) and `pnpm lint` on `@ahpd/web` 0.5.0 from npm.
- Two hosts end to end on a release build (task 05): one window for both, filtered to the host opened, Sign in opening the host's form, a one-host window refused another host's token, and per host working beside it.

## Departures from the plan

- `AppWindows` gained no `every` field: the window's label is fixed, so the map says whether it is open (task 02).
- The page reads `focus=` from its address on first load, since an event sent while it loads would be lost (task 02).

## Left for later

- Removing a host while the window with every host is open was not run by hand, since it writes to the OS keyring; `close_host` skips that window by its label.
