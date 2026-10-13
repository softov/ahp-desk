---
title: ahpw works end to end from the desk, from both sources
status: done
depends: [task-01-ahpw-builds-on-windows.md, task-06-ahpw-connects-to-its-host.md, task-07-ahpw-skips-the-wall.md, task-08-ahpw-frame-and-title-bar.md]
layer: "ahp-desk, ahpw"
refs:
  - file:///github/ahp-desk/DEVELOPER.md - "Testing ahpw from source"
---

## Objective

The desk opens ahpw on a URL host from the `local` source and, after a new `@ahpd/web` is published, from `npm`; ahpw in a browser is unchanged.

## Files

- `UPDATE: package.json` in ahp-desk - the `@ahpd/web` pin, once the new version is published.

## Steps

1. Run ahpw's dev server and `AHP_DESK_AHPW=local pnpm tauri dev`; open a host with a token.
2. Build ahpw and `AHP_DESK_AHPW=local pnpm stage:ui`; run a release build and open the same host.
3. Run `ahpw serve` against the same ahpd in a browser; sign in with the token.
4. After `@ahpd/web` is published from the merged branch, move the pin and repeat step 1 with `npm`.

## Validation

- Each run shows the host's sessions with no sign-in wall on the desk, and with the wall in the browser.
- `pnpm typecheck` and `pnpm test` pass in ahpw and in ahp-desk; `cargo clippy` and `cargo test` pass.

## Resume

- 2026-10-11: steps 1 to 3 pass. `AHP_DESK_AHPW=local pnpm tauri dev` and a release build staged from ahpw's `desk` branch both open Desk test (ahpd 0.10.0) with no wall; a browser on ahpw's dev server shows the wall.
- Found and fixed in ahpw: the host's default folder read `/C:/Users/...` on Windows (`folderLabel`, commit `7fd0c0a`).
- Found: a second desk process cannot start while one runs with other WebView2 arguments, as they share one data folder. Single-instance belongs in p4.
- Left: step 4, after `@ahpd/web` is published from `desk`.
- 2026-10-13: the pin is `@ahpd/web` 0.3.0, and a release build on it opens a throwaway ahpd with a token and no wall (p7 task 09). Step 4 as written, `pnpm tauri dev` with `npm`, is not run yet.
- 2026-10-13: step 4 passes. `pnpm tauri dev` with the `npm` source serves `node_modules/@ahpd/web/dist/app` (the page's script is the package's `index-DBKNxflN.js`) and opens the throwaway ahpd 0.7.0 with its token, connected, with no wall.
- Found: every host shares ahpw's origin on the desk, so its kept tabs and layout cross hosts; fixed in [p8](../01-first-build-p8-host-scoped-storage/plan.md).
