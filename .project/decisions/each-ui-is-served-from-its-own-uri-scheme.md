---
title: Each UI is served from its own URI scheme
status: accepted
date: 2026-10-11
refs:
  - file:///github/ahpapp/app.json - no base URL, so Expo emits paths from the root, such as /_expo/static/
  - file:///github/ahpw/vite.config.ts - base './', so ahpw works under any path
  - https://v2.tauri.app/reference/javascript/api/namespacecore/#convertfilesrc - Tauri's custom protocols
---

## Context

The shell's own screens are the app's `frontendDist`.
ahpw and ahpapp are other bundles, chosen from a UI source.
ahpapp's web build needs to be at the root of its origin, with a fallback to `index.html` for its routes.

## Decision

The shell registers one URI scheme for each UI: `ahpw` and `ahpapp`.
Each scheme serves that UI's bundle from the root, with a fallback to `index.html`.
A `local` source under `pnpm tauri dev` loads the UI's dev server URL instead.
Source: Softov, asked "Each UI gets its own URI scheme, served from the root with an index.html fallback. Keep it?": "Keep (Recommended)".

## Consequences

ahpw and ahpapp have separate origins, so their storage is apart.
The Tauri capability for the contract lists both origins and the dev server URLs.
On Windows the origin is `http://ahpw.localhost`, so a page can open a `ws://` socket.

## Options

- **A localhost HTTP server in the shell.** Lost: a port to choose and to guard on the machine.
- **ahpw as the `frontendDist`.** Lost: the shell's screens and ahpapp need another way in.
