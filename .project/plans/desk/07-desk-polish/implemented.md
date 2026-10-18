---
title: Desk polish before the first release - implemented
date: 2026-10-13
refs:
  - git://82e1118
  - "[code://packages/shell/src/i18n.ts](../../../../packages/shell/src/i18n.ts)"
  - "[code://packages/shell/src/FirstRun.tsx](../../../../packages/shell/src/FirstRun.tsx)"
  - "[code://src-tauri/icons/icon.svg](../../../../src-tauri/icons/icon.svg)"
---

The host list window now looks like ahpw: frameless and rounded, in ahpw's palette, with its own title bar and ahpio's footer.
It speaks English, Portuguese (Brasil) and Spanish, and a desk with no host opens on a first run that ends in an ahpw window on the new host.
Opening a host hides the host list; ahpw's switcher brings it back to manage hosts or add one, and closing the last app window shows it again.
The desk, its windows and its installer carry ahpw's mark, and the desk builds on the published `@ahpd/web` 0.3.0.

## What was built

- [`code://src-tauri/src/lib.rs`](../../../../src-tauri/src/lib.rs) - the frameless, rounded host list window at 640 x 400; its close hides it while an app window is open; `AHP_DESK_CONFIG_DIR` runs the desk on another host list.
- [`code://src-tauri/src/app_window.rs`](../../../../src-tauri/src/app_window.rs) - Open hides the host list, the last app window's close shows it, `show_host_list { add }`, and web links open in the system browser.
- [`code://packages/shell/src/app.css`](../../../../packages/shell/src/app.css) - ahpw's palette, light and dark, its buttons, rows, dots and keyboard focus.
- [`code://packages/shell/src/TitleBar.tsx`](../../../../packages/shell/src/TitleBar.tsx) - the title bar with ahpw's mark, the language picker and the window buttons.
- [`code://packages/shell/src/Footer.tsx`](../../../../packages/shell/src/Footer.tsx) - ahpio's logo and "© ahpio.com", opened in the browser.
- [`code://packages/shell/src/i18n.ts`](../../../../packages/shell/src/i18n.ts) - the three dictionaries, `t()`, the OS language until one is picked, the pick in `localStorage`.
- [`code://packages/shell/src/FirstRun.tsx`](../../../../packages/shell/src/FirstRun.tsx) - welcome, where the host is ("This computer" coming soon), the host form with Test, Save and open.
- [`code://src-tauri/icons/icon.svg`](../../../../src-tauri/icons/icon.svg) - ahpw's mark, the source of every icon; `installerIcon` in `tauri.conf.json`.
- ahpw, released as `@ahpd/web` 0.3.0: "Manage hosts" and "Add host" in the host switcher over `Embedder.manageHosts`.

## Verified

- `pnpm typecheck`, `pnpm test` (shell 24, embedder 8, cargo 15) and `pnpm lint` (clippy `-D warnings`) pass in ahp-desk; `pnpm typecheck` and `pnpm test` (276) pass in ahpw.
- A release build on `@ahpd/web` 0.3.0, driven over WebView2's debugging port against a throwaway ahpd 0.7.0 with a connection token, on host lists in the session scratchpad:
  - the first run in English, Portuguese and Spanish, from an empty host list to a connected ahpw window, with the host list hidden (Win32 `IsWindowVisible`);
  - the list, the edit form, Test and the remove question in each language;
  - the host list beside ahpw's Default theme, dark and light, by emulated `prefers-color-scheme`;
  - "Manage hosts" and "Add host" show the host list, on its Add form for the second; its close hides it while ahpw is open;
  - closing the last ahpw window shows the host list, and closing the host list then quits the desk;
  - minimize (`IsIconic`), maximize to the screen and restore to 640 x 400.
- The exe and the installer carry ahpw's mark, read from the files with `ExtractAssociatedIcon`.

## Departures from the plan

- Task 03 - `placeholder.svg` is renamed `icon.svg`, and the installer needed `installerIcon` to stop showing NSIS's own icon.
- Task 06 - the host list's empty state is gone: with no host the desk is always on the first run, which also starts again when the last host is removed.
- Task 06 - if the new host fails to open, the first run stays on it with Retry and "Host list" instead of losing the error.

## Left for later

- The errors Rust returns, such as a failed save, are in English only.
- Not checked: a drag with a real mouse, 150% display scale, and the taskbar itself (the exe's icon was read from the file instead).
