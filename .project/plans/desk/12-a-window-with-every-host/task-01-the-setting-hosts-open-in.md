---
title: The setting "Hosts open in", in Settings and settings.json
status: done
depends: []
layer: "src-tauri, packages/shell"
refs:
  - "[code://src-tauri/src/settings.rs#L28-L38](../../../../src-tauri/src/settings.rs#L28-L38) - `Settings`: `close_behavior` and `start_with_os_asked`"
  - "[code://packages/shell/src/settings/SettingsScreen.tsx#L5-L8](../../../../packages/shell/src/settings/SettingsScreen.tsx#L5-L8) - `CLOSE`, the pattern for a choice of two"
  - "[code://packages/shell/src/settings/ipc.ts](../../../../packages/shell/src/settings/ipc.ts) - the `Settings` type the screen reads"
---

## Objective

Settings offers "Hosts open in": one window per host, the default, or one window with every host.
The choice is kept in `settings.json` and applies to the next Open.

## Files

- `UPDATE: src-tauri/src/settings.rs:28-38` - `pub windows: Windows`, with `#[serde(default)]`; `enum Windows { PerHost, EveryHost }`, `PerHost` the default, as `per-host` and `every-host`.
- `UPDATE: packages/shell/src/settings/ipc.ts` - `windows: 'per-host' | 'every-host'` in `Settings`.
- `UPDATE: packages/shell/src/settings/SettingsScreen.tsx:5-8` - `WINDOWS`, a list like `CLOSE`, drawn as a second group of radio buttons.
- `UPDATE: packages/shell/src/i18n.ts` - `settings.windows`, `settings.windowsPerHost` and `settings.windowsEveryHost`, in every locale.

## Steps

1. Add `Windows` and the `windows` field, with a test that an old `settings.json` reads as `PerHost`.
2. Add the field to the shell's `Settings` type.
3. Draw the group in `SettingsScreen`, saved as it changes, like the close behavior.

## Validation

- `cargo test` in `src-tauri` passes, with the case for an old file.
- `pnpm test` and `pnpm lint` pass.
- By hand: the choice shows in Settings, survives a restart, and changes no open window.

## Resume

- Done 2026-10-16: `Windows` in `src-tauri/src/settings.rs`, `per-host` by default, as `windows` in `settings.json`.
- Settings draws "Hosts open in" as a second group of radio buttons, saved as it changes; the words are in every locale.
- Verified: `cargo test`, with a file from before `windows` and a round trip of `every-host`; the shell's `tsc` and tests pass.
