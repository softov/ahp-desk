---
title: About and the desk's update check in windows of their own
domain: desk
status: planned
priority: medium
created: 2026-10-27
revalidated: 2026-10-27
requires:
  - plans/desk/13-about-and-update-gaps/plan.md
changes:
  - specs/tray-and-settings.md
decisions:
  - decisions/about-is-a-window-of-its-own.md
  - decisions/the-desks-update-check-is-a-window-of-its-own.md
  - decisions/the-host-list-publishes-each-hosts-running-turns.md
refs:
  - "[code://packages/shell/src/about/AboutScreen.tsx](../../../../packages/shell/src/about/AboutScreen.tsx) - the About screen, whose versions and links move to the About window and whose Check now moves to the Updates window"
  - "[code://packages/shell/src/App.tsx#L32](../../../../packages/shell/src/App.tsx#L32) - `Screen`, with `about`; the listener at line 72 and the screen at line 161"
  - "[code://packages/shell/src/main.tsx](../../../../packages/shell/src/main.tsx) - the one entry of the shell's bundle, which picks a page by the URL's hash"
  - "[code://packages/shell/src/TitleBar.tsx#L52-L105](../../../../packages/shell/src/TitleBar.tsx#L52-L105) - `MenuWays` and the logo menu's items"
  - "[code://packages/shell/src/settings/SettingsScreen.tsx#L82-L84](../../../../packages/shell/src/settings/SettingsScreen.tsx#L82-L84) - the \"About ahp-desk\" link"
  - "[code://packages/shell/src/hosts/DeskUpdateBar.tsx](../../../../packages/shell/src/hosts/DeskUpdateBar.tsx) - the ready update's Restart, which reads running turns from `HostStatuses`"
  - "[code://packages/shell/src/hosts/status.ts#L162-L172](../../../../packages/shell/src/hosts/status.ts#L162-L172) - `publish`, the pattern the running turns follow"
  - "[code://packages/shell/src/i18n.ts#L636-L670](../../../../packages/shell/src/i18n.ts#L636-L670) - the language, kept in `localStorage` and followed only in its own window"
  - "[code://src-tauri/src/lib.rs#L84-L94](../../../../src-tauri/src/lib.rs#L84-L94) - the host list window's builder, frameless and transparent"
  - "[code://src-tauri/src/frame.rs](../../../../src-tauri/src/frame.rs) - `round_corners`, for every frameless window"
  - "[code://src-tauri/src/statuses.rs](../../../../src-tauri/src/statuses.rs) - the status relay the running turns mirror"
  - "[code://src-tauri/src/tray.rs#L32-L42](../../../../src-tauri/src/tray.rs#L32-L42) - `entries`, the menu's order; `on_menu_event` at line 105"
  - "[code://src-tauri/src/app_window.rs#L325-L329](../../../../src-tauri/src/app_window.rs#L325-L329) - `HOST_LIST_ABOUT` and `HOST_LIST_DESK_UPDATE`, which go"
  - "[code://src-tauri/src/updates.rs#L131-L150](../../../../src-tauri/src/updates.rs#L131-L150) - `desk_update`, `install_desk_update`, `check_desk_update`"
  - "[code://src-tauri/src/about.rs#L20](../../../../src-tauri/src/about.rs#L20) - `about`, the versions"
  - "[code://src-tauri/capabilities/shell.json](../../../../src-tauri/capabilities/shell.json) - the host list's capability, the shape of the two new ones"
---

## Goal

About and the desk's update check stop being screens of the host list and open as small windows of their own.
A person can keep the host list on the screen with a problem, open About beside it, and take one screenshot.
"Check for updates…" opens a window just for the check, from the title bar's logo menu, the tray and Settings, and its Restart asks about running turns in that window.

## Reconnaissance

The files read and the patterns to reuse are the `refs` above, each with its note.

### Searches performed

- `rg "HOST_LIST_ABOUT|onAboutAsked|AboutScreen" src-tauri/src packages/shell/src` - the tray's About, the host list's listener and screen; nothing else opens About.
- `rg "hosts-status|host_statuses" packages/ahpw-embedder/src` - ahpw reads `hosts-status` as a map of dots, so its shape cannot carry the turns.
- `rg "statuses.forget" src-tauri/src` - `remove_host` at `hosts.rs:578` forgets a removed host's status; its running turns go the same way.
- `rg "WebviewWindowBuilder::new" src-tauri/src` - the host list in `lib.rs` and the app windows in `app_window.rs`; no window opens a second page of the shell.

### Runtime path

```
tray About / Settings link / logo menu -> open_about_window -> side window "about" -> index.html#about -> AboutWindow -> about()
tray Check for updates / Settings / logo menu -> open_updates_window -> side window "updates" -> index.html#updates -> UpdatesWindow
UpdatesWindow -> desk_update() + desk-update-ready -> Check now -> check_desk_update() -> Restart -> hosts() + host_running() + local_ahpd_status() -> question or install_desk_update()
host list status.ts publish -> publish_host_running -> hosts-running -> every window; host_running() for a late one
```

### Gaps

- No window but the host list and the app windows exists; nothing opens a page of the shell by hash.
- No window but the host list knows a host's running turns.
- A language picked in one window does not reach another; `useLanguage` follows only its own window's picks.
- `specs/tray-and-settings.md` still names the Settings gear, "Start with the OS" and the About screen, which PRs #13 and #15 changed.

## Decisions locked in

| # | Decision | Rationale / source |
| --- | --- | --- |
| 1 | [About is a window of its own, not a screen of the host list](../../../decisions/about-is-a-window-of-its-own.md) | Softov, 2026-10-27: "it can be another window? ... I can open the page with problem.. says host form.. and the about and take a shot on the same screen" |
| 2 | [The desk's update check is a window of its own](../../../decisions/the-desks-update-check-is-a-window-of-its-own.md) | Softov, 2026-10-27: "the check for update then instead in about could be another window to open, just for that?" |
| 3 | [The host list publishes each host's running turns to every window](../../../decisions/the-host-list-publishes-each-hosts-running-turns.md) | Softov, 2026-10-27: "In the Updates window (Recommended)" |

| What | Source | Task |
| --- | --- | --- |
| Both windows have a fixed size: minimize and close, no maximize, no resize. | Softov, 2026-10-27, asked "Can the two windows be resized?": "Fixed size (Recommended)" | 02, 03 |
| "Check for updates…" is in the logo menu, the tray menu and Settings. | Softov, 2026-10-27, asked "Which menus get 'Check for updates…'?": "Logo menu, Tray menu, Settings" | 03 |
| About is 360 by 320 and Updates 400 by 260, centred on the screen. | (defaulted: room for their content at the host list's type size) | 02, 03 |
| Each window's title bar shows ahpio's logo, its title ("About ahp-desk", "Check for updates"), minimize and close. | (defaulted: the host list's look, with a title since no menu says what the window is) | 02 |
| The tray's "Restart to update" item opens the Updates window, whose Restart asks there; `host-list-desk-update` goes. | (defaulted: one place asks Restart's question, the Updates window) | 03 |
| The logo menu is Hosts, Add host, Settings, a line, Check for updates…, About ahp-desk, a line, Quit; the tray is the update item while one is ready, Hosts, Settings, Check for updates…, About ahp-desk, Quit. | (defaulted: the check beside About, as most apps) | 03 |
| The Updates window shows the running desk's version over Check now. | (defaulted: a check reads better against the version it checks) | 03 |

## Proposed architecture

- **Data flow** - `about()` answers the versions to the About window; the Updates window reads `about()` for the desk's version, `desk_update()` for a ready one, `hosts()`, `host_running()` and `local_ahpd_status()` for Restart's question.
- **Event flow** - The host list sends `hosts-running` beside `hosts-status`; `desk-update-ready` and `hosts-changed` reach the Updates window as they reach the host list; the tray opens the windows itself instead of emitting to the host list.
- **State flow** - `src-tauri` keeps the last running map (`HostRunning`) as it keeps the statuses; each window holds its own view state; the language follows `localStorage`'s `storage` event across windows.
- **Layer responsibilities** - `src-tauri`: `side_windows.rs` (open or focus a side window), the running relay, the tray's items, the two capabilities · `packages/shell`: the hash entry, `WindowBar`, `AboutWindow`, `UpdatesWindow`, the menus' entries.
- **Source-of-truth files** - [`code://src-tauri/src/statuses.rs`](../../../../src-tauri/src/statuses.rs), [`code://src-tauri/src/tray.rs`](../../../../src-tauri/src/tray.rs), [`code://packages/shell/src/main.tsx`](../../../../packages/shell/src/main.tsx), [`code://packages/shell/src/hosts/status.ts`](../../../../packages/shell/src/hosts/status.ts).

## Tasks

| Task | Status | Depends on |
| --- | --- | --- |
| [01 - every window reads each host's running turns](task-01-every-window-reads-running-turns.md) | todo | - |
| [02 - the About window](task-02-the-about-window.md) | todo | - |
| [03 - the Updates window](task-03-the-updates-window.md) | todo | 01, 02 |

## Risks and tradeoffs

- A transparent frameless window that cannot resize still needs `round_corners` on Windows 11 and the page's rounded root - both side windows use the host list's frame code, and Windows 10 keeps square corners as the host list does.
- The side windows load the same bundle as the host list, so `App`'s start work (statuses, tray words, tunnels) must not run in them - `main.tsx` picks the page before rendering, and only `App` starts that work.
- A side window opened while the host list is hidden still reads the last published maps; they are as fresh as the host list, which runs hidden.

## Resume state

- **Done so far:** nothing.
- **Next action:** [task-01-every-window-reads-running-turns.md](task-01-every-window-reads-running-turns.md) and [task-02-the-about-window.md](task-02-the-about-window.md), in either order.
- **Open questions:** none.
- **Watch out for:** `hosts-status` keeps its shape (ahpw's embedder reads it); a new command needs its name in `build.rs`, its permission in the window's capability, and its handler in `lib.rs`; the permission files under `permissions/autogenerated/` are written by the build.

## Final verification checklist

- [ ] `pnpm typecheck`, `pnpm test` and `cargo clippy --all-targets -- -D warnings` pass.
- [ ] By hand on Windows: About opens from the tray, Settings and the logo menu while the host list shows a host form, and a second open focuses it.
- [ ] By hand: Check for updates opens from the three places; Check now answers; with an update ready and a turn running on a local ahpd the desk started, Restart names the turn in the Updates window.
- [ ] By hand: a language picked in the host list changes an open About or Updates window.
- [ ] `specs/tray-and-settings.md` corrected.
- [ ] `plans/index.md` updated.
