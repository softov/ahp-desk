---
title: The workspace builds and opens an empty shell window
status: todo
depends: []
layer: "workspace, src-tauri, packages/shell"
refs:
  - file:///github/opendoop/packages/tauri/src-tauri/Cargo.toml - the crates to start from
  - file:///github/opendoop/packages/tauri/src-tauri/tauri.conf.json - build, bundle and `windows: []`
  - file:///github/opendoop/packages/tauri-shell/vite.config.ts - Vite on a fixed port with `strictPort`
  - file:///github/ahpw/package.json - `packageManager: pnpm@11.21.0`
---

## Objective

A pnpm workspace with a Tauri 2 app in `src-tauri/` and a React app in `packages/shell/`.
`pnpm tauri dev` opens one normal window, label `shell`, that shows the text "Hosts".

## Files

- `CREATE: package.json` - the workspace root, `packageManager: pnpm@11.21.0`, scripts `tauri`, `typecheck`, `test`, `lint`, and devDependency `@tauri-apps/cli@^2`.
- `CREATE: pnpm-workspace.yaml` - the packages `packages/*`.
- `CREATE: packages/shell/package.json` - `@ahp-desk/shell`, private, React 19, Vite, `@tauri-apps/api@^2`, scripts `dev`, `build`, `typecheck`, `test`.
- `CREATE: packages/shell/vite.config.ts` - port 5175 on 127.0.0.1 with `strictPort`, `outDir: dist`.
- `CREATE: packages/shell/index.html` - the root element and the script.
- `CREATE: packages/shell/src/main.tsx` - mounts `App`.
- `CREATE: packages/shell/src/App.tsx` - shows the heading "Hosts".
- `CREATE: packages/shell/tsconfig.json` - strict TypeScript for the shell.
- `CREATE: src-tauri/Cargo.toml` - crate `ahp-desk`, lib `ahp_desk_lib`, `tauri = "2"`, `serde`, `serde_json`, `tokio`.
- `CREATE: src-tauri/build.rs` - calls `tauri_build::build()`.
- `CREATE: src-tauri/src/main.rs` - calls `ahp_desk_lib::run()`.
- `CREATE: src-tauri/src/lib.rs` - `run()` with the builder, the setup that builds the `shell` window, and an empty command list.
- `CREATE: src-tauri/tauri.conf.json` - productName `ahp-desk`, identifier `dev.softov.ahpdesk`, devUrl `http://127.0.0.1:5175`, frontendDist `../packages/shell/dist`, `windows: []`.
- `CREATE: src-tauri/capabilities/shell.json` - windows `["shell"]`, `core:default`.
- `CREATE: src-tauri/icons/` - the icons from `pnpm tauri icon` on a placeholder logo.
- `UPDATE: .gitignore` - add `node_modules/`, `dist/`, `src-tauri/target/` and `src-tauri/gen/`.
- `UPDATE: DEVELOPER.md` - remove the note "There is no code yet".

## Steps

1. Create the root `package.json` and `pnpm-workspace.yaml` from the Files list.
2. Create `packages/shell` with Vite, React and the heading "Hosts".
3. Create `src-tauri` with the files in the Files list.
4. Build the `shell` window in the setup of `lib.rs` with `WebviewWindowBuilder` and `WebviewUrl::App("/")`.
5. Set `beforeDevCommand` to `pnpm --filter @ahp-desk/shell dev`.
6. Set `beforeBuildCommand` to `pnpm --filter @ahp-desk/shell build`.
7. Generate the icons with `pnpm tauri icon`.
8. Run `pnpm install`.

## Validation

- Run `pnpm tauri dev`; a window with the heading "Hosts" opens.
- Run `cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings`; it prints no warning.
- Run `pnpm -r typecheck`; it passes.

## Resume
