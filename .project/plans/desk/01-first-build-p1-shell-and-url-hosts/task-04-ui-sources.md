---
title: The build's UI comes from AHP_DESK_UI, and the ahpw scheme serves ahpw from the local or the npm source
status: done
depends: [task-01-workspace-scaffold.md]
layer: "src-tauri, workspace"
refs:
  - file:///github/ahpw/vite.config.ts - `base: './'`, `outDir: 'dist/app'`, dev server on 127.0.0.1:5180
  - file:///github/ahpw/plugin/static.ts - how ahpd's plugin serves dist/app, with its content types
  - npm://@ahpd/web@0.2.0 - the published ahpw, with `dist/app`
---

## Objective

The build reads its one UI from `AHP_DESK_UI`, `ahpw` or `ahpapp`, with `ahpw` as the default.
The shell resolves ahpw's source when it starts and serves it at `ahpw://localhost/`.
A path that is not a file returns `index.html`.
Under `pnpm tauri dev` with the `local` source, the app window uses ahpw's dev server instead.

## Files

- `CREATE: src-tauri/src/ui_source.rs` - `DeskUi`, `UiSource`, `resolve_ui()`, `ui_url()` and the scheme handler.
- `UPDATE: src-tauri/src/lib.rs` - register the scheme of the build's UI with `register_asynchronous_uri_scheme_protocol`.
- `UPDATE: src-tauri/build.rs` - pass `AHP_DESK_UI` to the crate with `cargo:rustc-env`, and rerun when it changes.
- `CREATE: scripts/stage-ui.mjs` - copies the resolved `dist/app` to `src-tauri/resources/ui/ahpw/` before a build.
- `UPDATE: package.json` - add `@ahpd/web@0.2.0` as a devDependency, and the script `stage:ui`.
- `UPDATE: src-tauri/tauri.conf.json` - `bundle.resources` holds `resources/ui/**/*`, and `beforeBuildCommand` runs `stage:ui` first.
- `UPDATE: .gitignore` - add `src-tauri/resources/ui/`.
- `UPDATE: DEVELOPER.md` - a section "Testing ahpw from source" with the two variables.

## Steps

1. Read `AHP_DESK_UI` at build time in `build.rs`; accept `ahpw` or `ahpapp`, default `ahpw`, and fail on any other value.
2. In this plan, fail the build for `ahpapp` with the text "ahpapp support comes in plan p5".
3. Read `AHP_DESK_AHPW`; accept `local` or `npm`, default `npm`, and fail on any other value.
4. Read `AHP_DESK_AHPW_DIR`, default `../ahpw` from the workspace root.
5. In a debug build with `local`, set `ui_url()` to `http://127.0.0.1:5180/`.
6. In a debug build with `npm`, serve `node_modules/@ahpd/web/dist/app`.
7. In a release build, serve `resource_dir()/ui/ahpw`.
8. Serve the file at the path; serve `index.html` when no file is there.
9. Set the content type from the file extension.
10. In `stage-ui.mjs`, read `AHP_DESK_UI` and copy `<AHP_DESK_AHPW_DIR>/dist/app` for `local`, and the npm `dist/app` for `npm`.
11. In `stage-ui.mjs`, fail with a clear message when the source folder has no `index.html`.
12. Write a unit test for the path rules: a file, a folder, an unknown route, and `..` in the path.

## Validation

- Run `cargo test --manifest-path src-tauri/Cargo.toml ui_source`; every case passes.
- Run `AHP_DESK_UI=ahpapp pnpm tauri build`; the build stops with the p5 message.
- Run `AHP_DESK_AHPW=local pnpm stage:ui` with a built ahpw; `src-tauri/resources/ui/ahpw/index.html` exists.
- Run `AHP_DESK_AHPW=npm pnpm stage:ui`; the same file exists, from the npm package.

## Resume

- Built 2026-10-11. 4 `ui_source` tests pass. `pnpm stage:ui` copies the npm `dist/app`; `AHP_DESK_UI=ahpapp` and an unknown `AHP_DESK_AHPW` stop with their messages.
- `bundle.resources` uses the map form `{ "resources/ui/": "ui/" }`, so the bundle lands at `resource_dir()/ui/ahpw`.
- On Windows the scheme is served as `http://ahpw.localhost/`; `ui_url()` returns that there and `ahpw://localhost/` elsewhere.
- **Blocked:** the `local` source is not verified on Windows. ahpw's `src/sessions/` has `Activity.tsx` and `activity.ts`, which collide on a case-insensitive file system, so `vite build` in ahpw fails with `MISSING_EXPORT "Activity"`. It needs a rename in ahpw.
- Unblocked 2026-10-11 by p2 task 01. `AHP_DESK_AHPW=local pnpm stage:ui` copies ahpw's built `dist/app`, and `AHP_DESK_AHPW=local pnpm tauri dev` loads ahpw's dev server in the app window.
