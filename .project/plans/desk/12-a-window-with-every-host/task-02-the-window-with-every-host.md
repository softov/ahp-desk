---
title: The window with every host opens, focuses and shows the host picked
status: done
depends: [task-01-the-setting-hosts-open-in.md]
layer: "src-tauri"
refs:
  - "[code://src-tauri/src/app_window.rs#L34-L75](../../../../src-tauri/src/app_window.rs#L34-L75) - `AppWindows`: `host_of`, `close_host`, `any_open`"
  - "[code://src-tauri/src/app_window.rs#L118-L190](../../../../src-tauri/src/app_window.rs#L118-L190) - `open_app_window`"
  - "[code://src-tauri/src/app_window.rs#L227](../../../../src-tauri/src/app_window.rs#L227) - `window_host`"
---

## Objective

With "every host", Open on any host shows and focuses one window with every host, and tells it to show that host.
Removing a host closes nothing in that window.

## Files

- `UPDATE: src-tauri/src/app_window.rs:34-75` - `AppWindows` holds `every: Mutex<Option<String>>`, the label of the window with every host, beside the map of one-host windows; `close_host` leaves that window open.
- `UPDATE: src-tauri/src/app_window.rs:118-190` - in `EveryHost`, `open_app_window` opens `app-every` on the UI with `?hosts=every`, or shows and focuses it, then emits `focus-host` with `host_id` to it.
- `UPDATE: src-tauri/src/app_window.rs:227` - `window_host` answers the host of a one-host window, and an error naming the window with every host otherwise.
- `UPDATE: src-tauri/src/app_window.rs` tests - the label of the window with every host, and `close_host` leaving it.

## Steps

1. Add `every` to `AppWindows`, with `is_every(label)` and `set_every(label)`.
2. Read `Settings.windows` in `open_app_window`, and open or focus `app-every` in `EveryHost`.
3. Emit `focus-host` to that window after it shows.
4. Keep the last app window closed showing the host list, with the window with every host counted.

## Validation

- `cargo test` in `src-tauri` passes, with the new cases.
- By hand, in task 05: Open on each host focuses the one window.

## Resume

- Done 2026-10-16: the window with every host is `EVERY_WINDOW`, `app-:every`, listed in `AppWindows` with the host it was opened on.
  `app-` keeps the UI's capability on it, and `label_for` never writes `:`, so no host's window takes the label.
- `AppWindows` gained no `every` field: the label is fixed, so the map says whether that window is open.
- In "every host", `open_app_window` shows that window and emits `FOCUS_HOST`, or builds it on `?hosts=every&focus=<id>`.
  The page reads `focus=` since an event sent while it loads would be lost.
- A switch from that window shows the host there; `close_host` and `window_of` skip it, and it counts as an open window.
- The window builder moved into `build_app_window`, shared by both kinds.
- Verified: `cargo test` with the new cases, and `cargo clippy --all-targets -- -D warnings`.
