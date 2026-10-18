---
title: Open on a host shows ahpw in an app window that knows its host
status: done
depends: [task-03-host-store.md, task-04-ui-sources.md]
layer: "src-tauri"
refs:
  - file:///github/opendoop/packages/tauri/src-tauri/src/lib.rs - `open_web_ui` at lines 50-129, async, with the deadlock note at lines 54-60
  - file:///github/opendoop/packages/tauri/src-tauri/capabilities/web-ui.json - window commands for a page with its own title bar
---

## Objective

`open_app_window` opens ahpw for a host in a window with no OS frame, labelled `app-<host id>`.
In that window, `window_host` returns the host and `host_token` returns its token.
`show_host_list` shows and focuses the `shell` window.

## Files

- `CREATE: src-tauri/src/app_window.rs` - `AppWindows`, `open_app_window`, `window_host` and `show_host_list`.
- `UPDATE: src-tauri/src/lib.rs` - manage `AppWindows`, register the three commands.
- `CREATE: src-tauri/capabilities/ui.json` - windows `["app-*"]`, the four host commands, the three window commands, and the window controls.
- `UPDATE: src-tauri/capabilities/shell.json` - allow `open_app_window`.

## Steps

1. Define `AppWindows` as a `Mutex<HashMap<String, String>>` from window label to host id.
2. Write `open_app_window(host_id, replace)` as an `async` command; the window loads the build's one UI.
3. If an app window calls with `replace` true, map its label to the new host and reload it.
4. When a window for the host exists, show and focus it.
5. Otherwise build `app-<host id>` at `ui_url()`, with no frame, 1280 by 860, and a minimum of 600 by 400.
6. Write `window_host`, which finds the caller's label in `AppWindows` and returns the host.
7. Restrict `host_token` to the `shell` window and to an app window that holds that host.
8. Remove the label from `AppWindows` when Tauri destroys its window.
9. In `ui.json`, allow `minimize`, `toggle-maximize`, `close`, `start-dragging`, `set-title` and `internal-toggle-devtools`.
10. Call `invoke('window_host')` from the devtools of an `ahpw://` page.
11. If that call fails, add the scheme origins to `remote.urls` in `ui.json`, and call it again.
12. Write the working origins in the Resume section below.

## Validation

- From the devtools of the `shell` window, call `open_app_window` for a saved host; ahpw shows its sign-in wall.
- In the app window's devtools, call `invoke('window_host')`; it returns that host.
- Call `invoke('host_token', { id: '<other host>' })` from the app window; it fails.
- Call `open_app_window` again for the same host; the window gets focus, and no second window opens.

## Resume

- Done 2026-10-11, checked over WebView2's debugging port.
- `invoke` works from `http://ahpw.localhost/` with no `remote.urls`; `ui.json` lists `http://127.0.0.1:5180/*` for the `local` dev server only.
- `build.rs` declares the seven commands in an app manifest, so each is a permission: an app window calling `save_host` is refused, and `host_token` for another host fails.
- A second Open of the same host focuses its window; no second window opens.
- Added: `remove_host` closes the windows that show the host, which otherwise stay open on a host that is gone.
