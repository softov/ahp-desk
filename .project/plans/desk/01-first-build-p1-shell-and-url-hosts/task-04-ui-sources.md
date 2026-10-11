---
title: The ahpw scheme serves ahpw from the local or the npm source
status: todo
depends: [task-01-workspace-scaffold.md]
layer: "src-tauri, workspace"
refs:
  - file:///github/ahpw/vite.config.ts - `base: './'`, `outDir: 'dist/app'`, dev server on 127.0.0.1:5180
  - file:///github/ahpw/plugin/static.ts - how ahpd's plugin serves dist/app, with its content types
  - npm://@ahpd/web@0.2.0 - the published ahpw, with `dist/app`
---

## Objective

The shell resolves ahpw's source when it starts and serves it at `ahpw://localhost/`.
A path that is not a file returns `index.html`.
Under `pnpm tauri dev` with the `local` source, the app window uses ahpw's dev server instead.

## Files

- `CREATE: src-tauri/src/ui_source.rs` - `UiSource`, `resolve_ahpw()`, `ahpw_url()` and the scheme handler.
- `UPDATE: src-tauri/src/lib.rs` - call `register_asynchronous_uri_scheme_protocol("ahpw", ...)` on the builder.
- `CREATE: scripts/stage-ui.mjs` - copies the resolved `dist/app` to `src-tauri/resources/ui/ahpw/` before a build.
- `UPDATE: package.json` - add `@ahpd/web@0.2.0` as a devDependency, and the script `stage:ui`.
- `UPDATE: src-tauri/tauri.conf.json` - `bundle.resources` holds `resources/ui/**/*`, and `beforeBuildCommand` runs `stage:ui` first.
- `UPDATE: .gitignore` - add `src-tauri/resources/ui/`.
- `UPDATE: DEVELOPER.md` - a section "Testing ahpw from source" with the two variables.

## Steps

1. Read `AHP_DESK_AHPW`; accept `local` or `npm`, default `npm`, and fail on any other value.
2. Read `AHP_DESK_AHPW_DIR`, default `../ahpw` from the workspace root.
3. In a debug build with `local`, set `ahpw_url()` to `http://127.0.0.1:5180/`.
4. In a debug build with `npm`, serve `node_modules/@ahpd/web/dist/app`.
5. In a release build, serve `resource_dir()/ui/ahpw`.
6. Serve the file at the path; serve `index.html` when no file is there.
7. Set the content type from the file extension.
8. In `stage-ui.mjs`, copy `<AHP_DESK_AHPW_DIR>/dist/app` for `local`, and the npm `dist/app` for `npm`.
9. In `stage-ui.mjs`, fail with a clear message when the source folder has no `index.html`.
10. Write a unit test for the path rules: a file, a folder, an unknown route, and `..` in the path.

## Validation

- Run `cargo test --manifest-path src-tauri/Cargo.toml ui_source`; every case passes.
- Run `AHP_DESK_AHPW=local pnpm stage:ui` with a built ahpw; `src-tauri/resources/ui/ahpw/index.html` exists.
- Run `AHP_DESK_AHPW=npm pnpm stage:ui`; the same file exists, from the npm package.

## Resume
