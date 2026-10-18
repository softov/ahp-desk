---
title: What plan 01 built
plan: plans/desk/01-shell-and-url-hosts/plan.md
date: 2026-10-11
---

## What exists

- A pnpm workspace with the Tauri 2 app in `src-tauri/`, the host list in `packages/shell/`, and the contract `@ahpd/desk-client` in `packages/desk-client/`.
- `hosts.rs` and `tokens.rs`: the host list in `hosts.json`, each token in the OS keyring, or in the host's `token` field when there is no keyring, moved to the keyring when it works again.
- `ui_source.rs`: the build's one UI from `AHP_DESK_UI`, served on its own scheme from npm's `@ahpd/web`, ahpw's dev server, or the bundled copy.
- `app_window.rs`: one frameless window per host, `window_host`, `host_token` only for the shell and the host's own window, and Remove closes the host's windows.
- The host list screen: status dots from one `MultiHostClient`, Add and Edit with Test, Open, Retry, and Remove with a question.
- The release build makes `ahp-desk_0.1.0_x64-setup.exe`.

## Verified

- On Windows, in `pnpm tauri dev`, driven over WebView2's debugging port: every command, the permission refusals, the Credential Manager entries, a restart, and the host list against an ahpd 0.10.0 with a connection token.
- `cargo clippy -- -D warnings`, `cargo test`, `pnpm -r typecheck`, `pnpm -r test`.

## Departures from the plan

- The token tests use a `Vault` trait with an in-memory and a failing vault: the `keyring` mock keeps nothing between two entries.
- `build.rs` declares the commands in an app manifest, so each window's capability grants only the commands it lists.
- The status dots are drawn in CSS.
- Not checked: 150% display scale, and the plain-file warning on screen.
