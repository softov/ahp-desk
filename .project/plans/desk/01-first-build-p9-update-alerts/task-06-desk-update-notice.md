---
title: The desk checks for its own update again every 6 hours, into p4 task 03's notice
status: todo
depends: [task-01-npm-latest-ahpd.md]
layer: "src-tauri"
refs:
  - file:///github/ahp-desk/.project/plans/desk/01-first-build-p4-tray-and-releases/task-03-updater-with-signing-key.md - p4 task 03: the check at start, `DeskUpdate` and its `Ready`, `DESK_UPDATE_READY`, the host list's notice, the tray's first item and the install, all reused here
  - https://v2.tauri.app/plugin/updater/ - `UpdaterExt`, `check()` and `download()`
  - "[code://src-tauri/src/lib.rs#L20-L95](../../../../src-tauri/src/lib.rs#L20-L95) - the setup, where p4 task 03 starts its check and task 01 starts `updates::watch`"
---

## Objective

p4 task 03 checks for a newer signed desk release at start, downloads it, and shows "ahp-desk X is ready. Restart to update." in the host list and as the tray menu's first item, installing only on a click (decisions 1 and 9).
This task adds the check every 6 hours after that start check, so a desk left running for days still finds a new release.
A newer release found here goes into p4 task 03's `DeskUpdate` and sends its `DESK_UPDATE_READY`, so the same notice, the same tray item and the same install show it; this task adds no notice, no command and no words.
It waits for p4 task 03 and for task 01's `watch` loop.

## Files

- `UPDATE: src-tauri/src/updates.rs` - `check_desk`, the body of p4's start check made callable, and its call in task 01's `watch` loop.

## Steps

1. Read p4 task 03 as built: `DeskUpdate`, `Ready`, `check_on_start` and `DESK_UPDATE_READY`; the names below follow what it has.
2. Move the body of `check_on_start` into `async fn check_desk(app: &AppHandle)`, and have `check_on_start` spawn it as before:
   - `app.updater()?.check().await`; on `Some(update)` whose version is not the one already in `Ready`, `update.download(|_, _| {}, || {}).await`, keep it in `Ready`, and emit `DESK_UPDATE_READY` with the version;
   - with the same version already ready, nothing is downloaded and nothing is sent;
   - a failed check or download is written with `eprintln!` and shows nothing, as at start.
3. In task 01's `watch` loop, after each `MAX_AGE` sleep, run `check_desk(&app)` beside the npm ask; never on the loop's first round, since `check_on_start` has just asked.
   A debug build never runs it, as p4's start check does not (`cfg!(debug_assertions)`).
4. Nothing on disk: the downloaded release stays in memory, as p4 keeps it.

## Validation

- `cargo test --manifest-path src-tauri/Cargo.toml updates` passes; `pnpm typecheck`, `pnpm test` and `pnpm lint` pass.
- A debug desk left running makes no request to github.com.
- By hand, on Windows with an installed older signed release (p4 task 06's pair): start the desk before the newer release is published, publish it, and leave the desk running; within 6 hours the host list's notice and the tray's first item appear with no restart, and nothing installs until one is clicked.
- With the host list window closed when the re-check finds the release, the tray's first item still appears; p4 task 03 has the host list send the tray its words, so if the item waits for the host list to open, that is p4 task 03's to fix, written in its Resume.

## Resume
