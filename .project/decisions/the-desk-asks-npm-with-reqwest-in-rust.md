---
title: The desk asks npm for @ahpd/server's latest with reqwest in Rust
status: accepted
date: 2026-10-14
refs:
  - file:///github/ahpd/packages/server/src/update.ts - `registry()` and `askRegistry()`, the rules the desk's ask follows: `npm_config_registry` first, one GET of the dist-tags, every failure the same nothing
  - "[code://src-tauri/Cargo.toml](../../src-tauri/Cargo.toml) - the desk's dependencies, with no HTTP client named yet"
  - "[code://src-tauri/Cargo.lock#L2810-L2811](../../src-tauri/Cargo.lock#L2810-L2811) - the `reqwest` the desk's build already holds"
---

## Context

Plan p9 shows "ahpd X available" on a host's row when npm has a newer `@ahpd/server` than the host runs.
The desk has to ask npm for that latest version, at most every 6 hours, and keep the answer in `updates.json` beside `hosts.json`.
A person behind a mirror has named it in `npm_config_registry`, and ahpd's own check reads it first.
The desk ships a Node for the local ahpd, and the host list's webview can make requests too, so the ask could be made from three places.

## Decision

`src-tauri/src/updates.rs` asks npm with `reqwest` in Rust, over rustls, with no native TLS.
It reads `npm_config_registry` before it assumes `https://registry.npmjs.org`, as ahpd's `registry()` does.
Source: Softov, asked "p9 q1: how does the desk ask npm for @ahpd/server's latest version?": "reqwest in Rust".

## Consequences

The check runs in the Rust process that keeps `updates.json`, whether or not a window is open.
`reqwest` is already in the build's lockfile through tauri, so naming it adds no second copy.
`src-tauri/Cargo.toml` names `reqwest` with its default features off and its rustls and `json` features on, in the version the lockfile holds.
The ask is tested in Rust against a local listener, with no Node and no webview.

## Options

- **`npm view @ahpd/server version` with the bundled Node.** Lost: a process per ask, npm's start time, and a check that depends on the local ahpd's Node being installed.
- **`fetch` from the host list's webview.** Lost: no check while the host list is closed, and the answer would have to be sent back to Rust to be kept and shared.
