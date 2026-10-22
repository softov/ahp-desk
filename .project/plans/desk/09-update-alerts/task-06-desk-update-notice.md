---
title: The desk checks for its own update again every 6 hours, into plan 04 task 03's notice
status: doing
depends: [task-01-npm-latest-ahpd.md]
layer: "src-tauri"
refs:
  - file:///github/ahp-desk/.project/plans/desk/04-tray-and-releases/task-03-updater-with-signing-key.md - plan 04 task 03: the check at start, `DeskUpdate` and its `Ready`, `DESK_UPDATE_READY`, the host list's notice, the tray's first item and the install, all reused here
  - https://v2.tauri.app/plugin/updater/ - `UpdaterExt`, `check()` and `download()`
  - "[code://src-tauri/src/lib.rs#L20-L95](../../../../src-tauri/src/lib.rs#L20-L95) - the setup, where plan 04 task 03 starts its check and task 01 starts `updates::watch`"
---

## Objective

plan 04 task 03 checks for a newer signed desk release at start, downloads it, and shows "ahp-desk X is ready. Restart to update." in the host list and as the tray menu's first item, installing only on a click (Answers: asking before the desk update, and the tray item).
This task adds the check every 6 hours after that start check, so a desk left running for days still finds a new release.
A newer release found here goes into plan 04 task 03's `DeskUpdate` and sends its `DESK_UPDATE_READY`, so the same notice, the same tray item and the same install show it; this task adds no notice, no command and no words.
It waits for plan 04 task 03 and for task 01's `watch` loop.

## Files

- `UPDATE: src-tauri/src/updates.rs` - `check_desk`, the body of plan 04's start check made callable, and its call in task 01's `watch` loop.

## Steps

1. Read plan 04 task 03 as built: `DeskUpdate`, `Ready`, `check_on_start` and `DESK_UPDATE_READY`; the names below follow what it has.
2. Move the body of `check_on_start` into `async fn check_desk(app: &AppHandle)`, and have `check_on_start` spawn it as before:
   - `app.updater()?.check().await`; on `Some(update)` whose version is not the one already in `Ready`, `update.download(|_, _| {}, || {}).await`, keep it in `Ready`, and emit `DESK_UPDATE_READY` with the version;
   - with the same version already ready, nothing is downloaded and nothing is sent;
   - a failed check or download is written with `eprintln!` and shows nothing, as at start.
3. In task 01's `watch` loop, after each `MAX_AGE` sleep, run `check_desk(&app)` beside the npm ask; never on the loop's first round, since `check_on_start` has just asked.
   A debug build never runs it, as plan 04's start check does not (`cfg!(debug_assertions)`).
4. Nothing on disk: the downloaded release stays in memory, as plan 04 keeps it.

## Validation

- `cargo test --manifest-path src-tauri/Cargo.toml updates` passes; `pnpm typecheck`, `pnpm test` and `pnpm lint` pass.
- A debug desk left running makes no request to github.com.
- By hand, on Windows with an installed older signed release (plan 04 task 06's pair): start the desk before the newer release is published, publish it, and leave the desk running; within 6 hours the host list's notice and the tray's first item appear with no restart, and nothing installs until one is clicked.
- With the host list window closed when the re-check finds the release, the tray's first item still appears; plan 04 task 03 has the host list send the tray its words, so if the item waits for the host list to open, that is plan 04 task 03's to fix, written in its Resume.

## Resume

- 2026-10-20: `updates.rs` has `async fn check_desk(app)`, the body of `check_on_start`, which now spawns it; it returns at once on `cfg!(debug_assertions)`, and a failure is the same `eprintln!` line as at start.
- `download_desk` asks `new_release(ready, found)` before downloading: a release already in `DeskUpdate`'s `Ready` is not downloaded or sent again, so the notice and the tray item stay as they are.
- `watch` calls `check_desk(&app)` after each `MAX_AGE` sleep, never on its first round, which `check_on_start` has just covered.
- Verified: `cargo test` (137, with `updates_a_desk_release_already_ready_is_not_new`), `cargo clippy --all-targets -- -D warnings` through `pnpm lint`, `pnpm typecheck`, `pnpm test`.
- Still `doing`: the by-hand check needs an installed older signed release and a newer one published on GitHub while the desk runs, which is plan 04 task 06's pair; v0.1.0 is the older one, and the next release Softov tags is the newer one.
  The check with the host list window closed is run in the same pass.
- Not run: a debug desk with the network watched; `check_desk` returns on `cfg!(debug_assertions)` before any updater is built, as the start check does.

- 2026-10-22: v0.1.0 has the check at start only, so its update to 0.2.0 (plan 04 task 06) showed the notice after a start, as plan 04 task 03 built it; the check every 6 hours is in 0.2.0 and later, so it is seen by hand on a 0.2.0 desk left running while 0.2.1 is published.
