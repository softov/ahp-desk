---
title: A tag v* builds the Windows and Linux installers in CI and publishes them with latest.json
status: todo
depends: [task-03-updater-with-signing-key.md]
layer: ".github/workflows, src-tauri, docs"
refs:
  - file:///github/opendoop/packages/tauri/PUBLISH.md - doop's release by hand, the shape of `latest.json`, and the outline of a release workflow under "Automating with GitHub Actions"
  - https://github.com/tauri-apps/tauri-action - builds, uploads the bundles and their `.sig`, and writes `latest.json` into a release
  - https://v2.tauri.app/distribute/pipelines/github/ - Tauri's GitHub Actions guide, with the Linux packages
  - "[code://DEVELOPER.md#L14-L20](../../../../DEVELOPER.md#L14-L20) - the Linux packages a build needs"
  - "[code://DEVELOPER.md#L65-L78](../../../../DEVELOPER.md#L65-L78) - the bundles: NSIS on Windows, AppImage and deb on Linux"
  - "[code://DEVELOPER.md#L119-L134](../../../../DEVELOPER.md#L119-L134) - the signing variables"
  - "[code://src-tauri/tauri.conf.json#L1-L20](../../../../src-tauri/tauri.conf.json#L1-L20) - the version, `beforeBuildCommand` with `pnpm stage:all`, and the targets"
  - "[code://src-tauri/build.rs#L24-L29](../../../../src-tauri/build.rs#L24-L29) - `AHP_DESK_UI` and `AHP_DESK_AHPW`"
  - "[code://package.json#L9](../../../../package.json#L9) - the pinned pnpm"
---

## Objective

A tag `v<version>` pushed to GitHub makes one workflow build the public build on Windows and on Linux, sign the updater bundles with the CI secret, and publish one release with the NSIS installer, the AppImage, the deb, their `.sig` files and `latest.json`.
The release is published only when both builds pass, so `releases/latest/download/latest.json` never names a bundle that is missing.

## Before the first tag: the secrets (the user)

The user adds two repository secrets in GitHub, Settings, Secrets and variables, Actions:

- `TAURI_SIGNING_PRIVATE_KEY` - the content of the `.tauri/updater.key` made in task 03.
- `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` - its password.

Nothing in the workflow prints them, and no task file or log holds them.

## Files

- `CREATE: .github/workflows/release.yml` - the release workflow.
- `CREATE: src-tauri/tauri.release.conf.json` - `{ "bundle": { "createUpdaterArtifacts": true } }`, merged in by the workflow with `--config` (decision 11 in plan.md).
- `UPDATE: DEVELOPER.md:119-134` - "Signing keys" names the two CI secrets; a new section "Releases" after it says how to cut one.

## Steps

1. `on: push: tags: ['v*']`, and `permissions: contents: write`.
2. Job `version`, on `ubuntu-latest`: fails unless the tag without its `v` equals `version` in `src-tauri/tauri.conf.json`, `src-tauri/Cargo.toml` and `package.json`; a release is cut from the version the files say.
3. Job `draft`, needs `version`: `gh release create "$GITHUB_REF_NAME" --draft --title "ahp-desk $VERSION" --notes "ahp-desk $VERSION"`, with `GH_TOKEN: ${{ github.token }}`, and outputs the release id.
4. Job `build`, needs `draft`, with `strategy.matrix.os` the Windows runner `windows-latest` and the Linux runner `ubuntu-22.04`, so the AppImage runs on an older glibc (decision 10 in plan.md), and `fail-fast: false`.
   - `actions/checkout@v4`.
   - On Linux: `sudo apt-get update` and the packages of DEVELOPER.md line 19 but `build-essential curl wget file`, which the runner has.
   - `pnpm/action-setup@v4`, which reads `packageManager` (pnpm 11.21.0) from `package.json`.
   - `actions/setup-node@v4` with `node-version: 22` and `cache: pnpm`.
   - `dtolnay/rust-toolchain@stable` and `Swatinem/rust-cache@v2` with `workspaces: src-tauri -> target`.
   - `pnpm install --frozen-lockfile`.
   - `tauri-apps/tauri-action@v0` (decision 9 in plan.md) with `releaseId` from `draft`, `args: --config src-tauri/tauri.release.conf.json`, and `includeUpdaterJson: true`, so it uploads each bundle with its `.sig` and merges this platform into `latest.json`.
   - Its `env`: `GITHUB_TOKEN`, `TAURI_SIGNING_PRIVATE_KEY` and `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` from the secrets, `AHP_DESK_UI: ahpw` and `AHP_DESK_AHPW: npm` (decision 2: the release is the public build only).
5. `beforeBuildCommand` runs `pnpm stage:all`, which fetches the Node of the runner's own OS (DEVELOPER.md line 100), so each runner ships its own Node; nothing more is staged by hand.
6. Job `publish`, needs `build`: checks that the draft holds `latest.json` with both `windows-x86_64` and `linux-x86_64`, then `gh release edit "$GITHUB_REF_NAME" --draft=false --latest`.
7. Check in the updater's docs which Linux bundle `latest.json` points at, the AppImage; a desk installed from the deb is updated by installing the next deb, unless the plugin in use updates a deb too, and the "Releases" section says which.
8. DEVELOPER.md "Releases": bump `version` in `src-tauri/tauri.conf.json`, `src-tauri/Cargo.toml` and `package.json` together; commit; tag `v<version>`; push the tag; the workflow does the rest.
   It also says the Windows installer has no code signature, so SmartScreen warns (plan risk), and that a lost signing key strands every installed desk.

## Validation

- `actionlint .github/workflows/release.yml` passes, if actionlint is at hand; otherwise GitHub's own check on the first push.
- The user pushes the first tag, `v0.1.0` or the version the files say: the four jobs pass; the release is published with `ahp-desk_<version>_x64-setup.exe` and its `.sig`, the `.AppImage` and its `.sig`, the `.deb`, and `latest.json`.
- `curl -sL https://github.com/softov/ahp-desk/releases/latest/download/latest.json` answers the version, and a `signature` and a `url` for `windows-x86_64` and `linux-x86_64`.
- A tag whose version differs from the files fails in `version`, and no release is made.
- The workflow log shows no key: GitHub masks the secrets, and no step echoes them.

## Resume
