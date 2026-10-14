---
title: The desk has a tray icon whose menu shows the host list and quits
status: todo
depends: []
layer: "src-tauri, packages/shell"
refs:
  - file:///github/opendoop/packages/tauri/src-tauri/src/tray.rs - doop's tray: menu ids, `on_menu_event`, the left click, at lines 8-62
  - https://v2.tauri.app/learn/system-tray/ - `TrayIconBuilder`, the `tray-icon` feature, `show_menu_on_left_click`
  - "[code://src-tauri/src/lib.rs#L20-L94](../../../../src-tauri/src/lib.rs#L20-L94) - the builder, the setup that makes the host list window, the handlers, and `RunEvent::Exit`"
  - "[code://src-tauri/src/app_window.rs#L228-L239](../../../../src-tauri/src/app_window.rs#L228-L239) - `show_host_list`, which the Hosts item calls"
  - "[code://src-tauri/build.rs#L51-L70](../../../../src-tauri/build.rs#L51-L70) - each command is declared as a permission"
  - "[code://src-tauri/capabilities/shell.json](../../../../src-tauri/capabilities/shell.json) - the host list window's permissions"
  - "[code://src-tauri/Cargo.toml#L18](../../../../src-tauri/Cargo.toml#L18) - `tauri` with no features yet"
  - "[code://packages/shell/src/i18n.ts](../../../../packages/shell/src/i18n.ts) - the three dictionaries the menu's words come from"
  - "[code://packages/shell/src/App.tsx#L15-L17](../../../../packages/shell/src/App.tsx#L15-L17) - the host list reads the language with `useLanguage()`"
---

## Objective

While the desk runs, a tray icon is there.
Its menu has Hosts, which shows and focuses the host list, and Quit, which quits the desk as closing it does today, so the local ahpd's stop on quit still runs.
The menu's words are in the language the host list shows, and change with a pick.

## Files

- `UPDATE: src-tauri/Cargo.toml:18` - `tauri = { version = "2", features = ["tray-icon"] }`; it has no features now.
- `CREATE: src-tauri/src/tray.rs` - `TRAY_ID`, the menu ids, `TrayWords`, `install()`, `set_tray_words` and the two event handlers.
- `UPDATE: src-tauri/src/lib.rs:1-7` - `mod tray;`.
- `UPDATE: src-tauri/src/lib.rs:30-70` - `tray::install(app.handle())?` after the host list window is built (line 51).
- `UPDATE: src-tauri/src/lib.rs:71-87` - `tray::set_tray_words` in `generate_handler!`.
- `UPDATE: src-tauri/build.rs:52-68` - `"set_tray_words"` in the command list.
- `UPDATE: src-tauri/capabilities/shell.json:6-30` - `"allow-set-tray-words"`.
- `UPDATE: packages/shell/src/hosts/ipc.ts` - `TrayWords` and `setTrayWords(words)`.
- `UPDATE: packages/shell/src/App.tsx:17` - keep the language from `useLanguage()` and send the words in an effect on it.
- `UPDATE: packages/shell/src/i18n.ts` - `tray.hosts` and `tray.quit` in `en` (line 20), `ptBR` (line 101) and `es` (line 180).

## Steps

1. In `tray.rs`, `pub const TRAY_ID: &str = "ahp-desk";` and the ids `tray.hosts` and `tray.quit`, as doop's `MENU_ID_*`.
2. `TrayWords { hosts: String, quit: String }`, `Deserialize` with `rename_all = "camelCase"`; tasks 02 and 03 add `settings` and `update` to it.
3. `pub fn install(app: &AppHandle) -> tauri::Result<()>` builds `TrayIconBuilder::with_id(TRAY_ID)` with the tooltip `ahp-desk`, `app.default_window_icon()` as its icon, `show_menu_on_left_click(false)`, `on_menu_event` and `on_tray_icon_event`, and no menu yet.
   The words live in the shell's dictionaries only (decision: the desk speaks English, Portuguese and Spanish, "every word the desk shows is a key in three dictionaries"), so the menu is set when the host list sends them.
4. `#[tauri::command] pub fn set_tray_words(app: AppHandle, words: TrayWords) -> Result<(), String>` builds `Menu::with_items` from `MenuItem::with_id` for each item, in the order Hosts, Quit, and sets it with `app.tray_by_id(TRAY_ID)` and `set_menu(Some(menu))`.
   Keep the order in one pure function, `fn entries(words: &TrayWords) -> Vec<(&'static str, String)>`, so a test reads it.
5. `on_menu_event`: Hosts calls `crate::app_window::show_host_list(app.clone(), None)`; Quit calls `app.exit(0)`, which reaches `RunEvent::Exit` and `local_ahpd::stop_on_quit` (lib.rs:90-94).
   An error from `show_host_list` is written with `eprintln!`, as `open_outside` does in app_window.rs.
6. `on_tray_icon_event`: a left click, `MouseButton::Left` with `MouseButtonState::Up`, shows the host list, and the right click opens the menu (decision 5 in plan.md; doop's left click toggles its pill).
   On Linux the tray is an appindicator, which sends no click events; there the menu is the only way in.
7. In `ipc.ts`: `export interface TrayWords { hosts: string; quit: string }` and `export function setTrayWords(words: TrayWords): Promise<void>` invoking `set_tray_words` with `{ words }`.
8. In `App.tsx`: `const lang = useLanguage();` and `useEffect(() => { setTrayWords({ hosts: t('tray.hosts'), quit: t('tray.quit') }).catch(...) }, [lang])`, logging a failure as the status sync does at line 51.
   The host list window is built at start and only ever hidden, so it is there to send the words.
9. English words: `tray.hosts` "Hosts", `tray.quit` "Quit ahp-desk"; Portuguese and Spanish in the same keys.
10. The menu has no "Show pill": plan p6 adds it with the pill (decision 6 in plan.md).

## Validation

- `cargo test --manifest-path src-tauri/Cargo.toml tray_`: `tray_entries_are_hosts_then_quit` checks the ids and words that `entries()` gives.
- `pnpm --filter @ahp-desk/shell test`: `i18n.test.ts` passes with the new keys in the three dictionaries.
- `pnpm lint` and `pnpm typecheck` pass.
- By hand on the debug desk on Windows, with `AHP_DESK_CONFIG_DIR` on a scratch host list: the icon is in the tray with the tooltip `ahp-desk`; the menu reads in English, then in Portuguese and Spanish after a pick in the title bar; open a host so the host list hides, then Hosts brings it back and focuses it; a left click does the same.
- Quit with a local host whose "Stop when the desktop app quits" is on: the desk exits and that ahpd stops, as a close does today.
- On Debian 13 with `libayatana-appindicator3` (DEVELOPER.md): the icon and its menu show, and both items work.

## Resume
