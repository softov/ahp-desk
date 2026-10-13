---
title: ahpw keeps a host's pages under the host's address
status: accepted
date: 2026-10-13
refs:
  - file:///github/ahpw/src/tabs.ts - `ahpd-web.session.v1`, the open tabs, each naming a session
  - file:///github/ahpw/src/App.tsx - `ahpd-web.layout.v1`, the layout, with the active tab and the window bounds keyed by session
  - file:///github/ahpw/src/folders/added.ts - `ahpd-web.folders-added`, the folders added to the explorer
  - "[code://src-tauri/src/app_window.rs](../../src-tauri/src/app_window.rs) - every app window loads ahpw from one origin, `http://ahpw.localhost`"
---

## Context

In a browser each ahpd serves ahpw from its own origin, so each host has its own `localStorage`.
On the desk every app window loads ahpw from `http://ahpw.localhost`, so every host shares one `localStorage`.
A host's open tabs, its layout and its added folders then show on every other host: a tab from one host reads "This session is gone" on another.

## Decision

On an embedder, ahpw keeps the tabs, the layout and the added folders under a key that carries the address of the page's host, as `ahpd-web.session.v1@ws://127.0.0.1:9187`.
The embedder gives that address synchronously, as `hostUrl`, because these keys are read when ahpw's modules load.
The theme, the main layout choice and the groupings stay shared: they are the person's, not the host's.
In a browser the keys do not change, since the origin is already the host.
Source: Softov, asked whether the key is the host's address (recommended) or the desk's host id: "ok with the recomendation"; and on the leak: "pages saved on one app, should not advertise to another."

## Consequences

Each host on the desk opens on its own tabs and layout.
A host whose address is edited to another server starts clean; a host removed and added again at the same address gets its tabs back, as they are that server's sessions.
The contract gains `hostUrl`, so the desk's embedder must set it, and the two release together.
The unscoped keys a desk profile already holds are removed once by hand: the desk was never released, so ahpw carries no migration for them.

## Options

- **The desk's host id.** Lost: the id is a slug of the name, so a host removed and added again under the same name at another address would get the old server's tabs.
- **One origin per host on the desk** (`ahpw-<id>.localhost`). Lost: the theme and the other personal choices would split per host too, and each host would need its own scheme registration.
