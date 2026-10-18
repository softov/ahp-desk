---
title: ahpw owns its embedding contract, and the desk injects the embedder
status: accepted
date: 2026-10-11
supersedes: decisions/the-shell-contract-is-a-shared-npm-package.md
refs:
  - git://4c89294 - the last commit that holds the superseded decision the-shell-contract-is-a-shared-npm-package
  - file:///github/ahpw/src/shell/embed/index.ts - the contract, `Embedder`, and `embedder` read from `globalThis.ahpwEmbedder`
  - file:///github/ahpw/plugin/index.ts - ahpw is also the ahpd plugin `ahpd-web`, served to browsers at `/plugins/ahpd-web/`
  - "[code://packages/ahpw-embedder/src/embedder.ts](../../packages/ahpw-embedder/src/embedder.ts) - the desk's embedder over its Tauri commands"
  - "[code://src-tauri/src/app_window.rs](../../src-tauri/src/app_window.rs) - `initialization_script(EMBEDDER)` on each app window"
---

## Context

ahpw imported `@ahpd/desk-client` and `@tauri-apps/api` to reach the desk.
The same build of ahpw is the ahpd plugin `ahpd-web`, so every browser that opens a host's plugin page loaded desk code it never runs.
ahpw's build needed the ahp-desk checkout beside it through a `link:`, and the two repositories had to release in step.

## Decision

ahpw defines what it needs from an app that embeds it, `Embedder` in `src/shell/embed/index.ts`, and reads it from `globalThis.ahpwEmbedder`; ahpw imports nothing from the desk or from Tauri.
`@ahpd/web` exports the contract's types as `@ahpd/web/embed`.
The desk implements it in `packages/ahpw-embedder`, a private package built into one script, and injects that script into each app window with `initialization_script`, so it runs before ahpw.
Source: Softov, asked "Should I switch ahpw to the injected `ahpwEmbedder` object and stop publishing `@ahpd/desk-client`?": "switch ahpw to the injected ahpwEmbedder".
Then, asked "Where does the TypeScript definition of the `ahpwEmbedder` contract live?": "ahpw owns it (Recommended)"; and "What happens to ahp-desk's `packages/desk-client`?": "Becomes the adapter (Recommended)".

## Consequences

ahpw in a browser, from `ahpw serve` or a host's plugin, carries no desk code.
Any app can embed ahpw by setting the same object; ahpw does not change for it.
The desk publishes no npm package.
The desk's adapter is type-checked against ahpw's contract, so it takes `@ahpd/web` from the ahpw checkout through a `link:` until a version with `./embed` is on npm.
A change to the contract starts in ahpw, and the desk's adapter follows.
The host list window is the desk's own page and calls the Tauri commands directly.
ahpapp, in plan 05, gets its own adapter for its own contract.

## Options

- **A `postMessage` or `MessageChannel` protocol.** Lost: everything becomes asynchronous messages with more code on both sides, and the desk has no cross-origin frame that needs it; an injected object can later be backed by a channel without ahpw changing.
- **Each side writes the contract.** Lost: the desk's copy of the types could drift from ahpw's.
- **Keep the shared npm package.** Lost: ahpw ships desk and Tauri code to every browser and depends on the desk's releases.
