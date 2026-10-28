---
title: About opens in a window of its own
status: doing
depends: []
layer: "src-tauri and packages/shell"
refs:
  - "[code://packages/shell/src/about/AboutScreen.tsx](../../../../packages/shell/src/about/AboutScreen.tsx) - the versions, the license and the links the window shows"
  - "[code://packages/shell/src/about/ipc.ts](../../../../packages/shell/src/about/ipc.ts) - `about`, `checkDeskUpdate`, `onAboutAsked`"
  - "[code://packages/shell/src/App.tsx#L32](../../../../packages/shell/src/App.tsx#L32) - `Screen`, with `about`; lines 72, 146, 159 and 161 open or show it"
  - "[code://packages/shell/src/main.tsx](../../../../packages/shell/src/main.tsx) - the bundle's entry"
  - "[code://packages/shell/src/TitleBar.tsx](../../../../packages/shell/src/TitleBar.tsx) - the host list's title bar, the look `WindowBar` follows"
  - "[code://packages/shell/src/window.ts](../../../../packages/shell/src/window.ts) - `controls` and `startFrame`, the drag and the frame for any frameless window"
  - "[code://packages/shell/src/i18n.ts#L636-L670](../../../../packages/shell/src/i18n.ts#L636-L670) - the language and its listeners"
  - "[code://src-tauri/src/lib.rs#L84-L94](../../../../src-tauri/src/lib.rs#L84-L94) - the frameless builder"
  - "[code://src-tauri/src/tray.rs#L126-L130](../../../../src-tauri/src/tray.rs#L126-L130) - the tray's About, which emits `host-list-about`"
  - "[code://src-tauri/src/app_window.rs#L325](../../../../src-tauri/src/app_window.rs#L325) - `HOST_LIST_ABOUT`"
---

## Objective

About is a fixed-size frameless window of 360 by 320, labelled `about`, that loads `index.html#about` and shows the desk's, ahpw's and Node's versions, the MIT license, and the links to ahpio.com and the releases (decision 1).
The tray's About, Settings' "About ahp-desk" and the logo menu open it, or bring it to the front; the host list keeps its screen.

## Files

- `CREATE: src-tauri/src/side_windows.rs` - `ABOUT_WINDOW`, `open_side`, `open_about`, and the command `open_about_window`.
- `CREATE: src-tauri/capabilities/about.json` - the `about` window's capability.
- `UPDATE: src-tauri/src/lib.rs` - `mod side_windows;` and `side_windows::open_about_window` in `generate_handler!`.
- `UPDATE: src-tauri/build.rs` - `"open_about_window"` in the command list.
- `UPDATE: src-tauri/capabilities/shell.json` - `"allow-open-about-window"`.
- `UPDATE: src-tauri/src/tray.rs:126-130` - About calls `side_windows::open_about(app)`; the `HOST_LIST_ABOUT` import goes.
- `UPDATE: src-tauri/src/app_window.rs:324-325` - `HOST_LIST_ABOUT` goes.
- `CREATE: packages/shell/src/WindowBar.tsx` - a side window's title bar.
- `CREATE: packages/shell/src/about/AboutWindow.tsx` - the About window's page.
- `DELETE: packages/shell/src/about/AboutScreen.tsx` - its versions and links move to `AboutWindow`, its Check now to task 03.
- `UPDATE: packages/shell/src/about/ipc.ts` - `openAboutWindow()`; `onAboutAsked` goes.
- `UPDATE: packages/shell/src/main.tsx` - the page by `location.hash`.
- `UPDATE: packages/shell/src/App.tsx` - the `about` screen, its listener and import go; Settings and the logo menu call `openAboutWindow()`.
- `UPDATE: packages/shell/src/i18n.ts` - the language follows another window's pick; `about.window` for the window's title if `about.title` does not read as one.
- `UPDATE: packages/shell/src/app.css` - `.window-bar` and the About window's layout.

## Steps

1. In `side_windows.rs`, write `fn open_side(app: &AppHandle, label: &str, view: &str, size: (f64, f64)) -> tauri::Result<()>`: when `app.get_webview_window(label)` is `Some`, `unminimize`, `show` and `set_focus` it; otherwise build `WebviewWindowBuilder::new(app, label, WebviewUrl::App(format!("index.html#{view}").into()))` with `.title("ahp-desk")`, `.decorations(false)`, `.transparent(true)`, `.shadow(false)`, `.inner_size(size.0, size.1)`, `.resizable(false)`, `.maximizable(false)` and `.center()`, then `frame::round_corners(&window)`.
2. Add `pub const ABOUT_WINDOW: &str = "about";`, `pub fn open_about(app: &AppHandle)` calling `open_side(app, ABOUT_WINDOW, "about", (360.0, 320.0))` and logging a failure as `ahp-desk: About did not open: {err}`, and `#[tauri::command] pub fn open_about_window(app: AppHandle) -> Result<(), String>` mapping the error to a string.
3. Write `capabilities/about.json`: `"identifier": "about"`, `"windows": ["about"]`, permissions `core:default`, `core:window:allow-minimize`, `core:window:allow-close`, `core:window:allow-start-dragging`, `core:window:allow-set-title`, `allow-about`.
4. Register the command in `lib.rs` and `build.rs`, grant it to the host list in `shell.json`, and make the tray's `MENU_ABOUT` call `side_windows::open_about(app)`; remove `HOST_LIST_ABOUT` from `app_window.rs` and `tray.rs`.
5. In `main.tsx`, render `AboutWindow` when `location.hash === '#about'`, and `App` otherwise; task 03 adds `#updates`.
6. Write `WindowBar({ title }: { title: string })`: a `header.titlebar` with `AhpioLogo` at 14px, the title, and minimize and close from `controls`, the close calling `controls.close()`; it sets the OS title with `getCurrentWindow().setTitle(title)` in an effect.
7. Write `AboutWindow`: `WindowBar` with `t('about.title')`, then the versions list, the license and the two links from `AboutScreen`, and the error line; no Back, no update parts.
8. In `App.tsx`, remove `{ kind: 'about' }` from `Screen`, the `onAboutAsked` listener, the `AboutScreen` branch and import; Settings' `onAbout` and the logo menu's `onAbout` call `openAboutWindow()`, logging a failure.
9. In `i18n.ts`, add a `storage` listener on `window` that, when `STORAGE_KEY` changes, sets the language from the new value and calls the listeners, so an open side window follows the host list's pick.
10. Delete `AboutScreen.tsx`; the `about.check`, `about.checking` and `about.upToDate` words stay for task 03.

## Validation

- `pnpm typecheck`, `pnpm test` (`i18n.test.ts` still passes), and `cargo clippy --all-targets -- -D warnings`.
- By hand on the debug desk: with the host list on a host form, the tray's About opens the window beside it, and the form stays; a second About brings the window to the front; Settings' link and the logo menu do the same; the window has round corners on Windows 11, cannot resize or maximize, and closes; a language picked in the host list changes the open window.

## Resume

- Built: `side_windows.rs` with `open_side`, `open_about` and the async command `open_about_window` (async, as a window built in a synchronous command deadlocks on Windows); `capabilities/about.json`; `WindowBar`; `AboutWindow`; the hash entry in `main.tsx`; the `storage` listener in `i18n.ts`. `AboutScreen.tsx`, `onAboutAsked` and `HOST_LIST_ABOUT` are gone.
- Beyond the steps: `about.json` also grants `opener:allow-open-url` for the two links, as `shell.json` does; `startFrame` takes `{ maximizes }`, so a double click on a side window's bar does not maximize it.
- Until task 03, Check now has no place: it left with `AboutScreen`; `checkDeskUpdate` and the `about.check*` words wait for it.
- Checked: `pnpm typecheck`, `pnpm test`, `cargo clippy --all-targets -- -D warnings` pass.
- Next: the by-hand checks in Validation, then `status: done`.
