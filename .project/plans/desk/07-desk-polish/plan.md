---
title: Desk polish before the first release
domain: desk
status: built
priority: high
created: 2026-10-11
revalidated: 2026-10-11
requires:
  - plans/desk/02-ahpw-on-the-desk/plan.md
decisions:
  - decisions/the-host-list-is-plain-react-in-ahpws-colours.md
  - decisions/ahpw-owns-its-embedding-contract.md
refs:
  - "[code://packages/shell/src/app.css](../../../../packages/shell/src/app.css) - the host list's own colours, to replace"
  - "[code://packages/shell/src/App.tsx](../../../../packages/shell/src/App.tsx) - the list and form screens"
  - "[code://src-tauri/src/lib.rs](../../../../src-tauri/src/lib.rs) - the shell window, still decorated"
  - "[code://src-tauri/src/frame.rs](../../../../src-tauri/src/frame.rs) - `round_corners`, reused for the shell window"
  - "[code://src-tauri/src/app_window.rs](../../../../src-tauri/src/app_window.rs) - `open_app_window`, `show_host_list`"
  - "[code://src-tauri/icons/icon.svg](../../../../src-tauri/icons/icon.svg) - ahpw's mark, the source of every icon"
  - file:///github/ahpw/node_modules/@softov/scena/dist/styles/themes/tokens.css - ahpw's light palette
  - file:///github/ahpw/node_modules/@softov/scena/dist/styles/themes/dark.css - ahpw's dark palette
  - file:///github/ahpw/public/ahpw.svg - ahpw's mark, for the footer
  - file:///github/ahpw/src/embed/embed.css - ahpw's frame and controls, which the host list's match
  - file:///github/ahpw/src/embed/HostSwitcher.tsx - the switcher that gains "Manage hosts" and "+ Add host"
---

## Goal

The host list window looks like ahpw: its palette, its hovers, a frameless rounded window with its own title bar, and ahpw's mark with "© ahpio.com" at the bottom.
It speaks English, Portuguese (Brasil) and Spanish.
With no host it opens on a first run that ends in an open host.
Opening a host hides it, and ahpw's host switcher brings it back, to manage hosts or to add one.

## Reconnaissance

The files read and the patterns to reuse are the `refs` above.

### Runtime path

```
lib.rs setup -> shell window (decorated, 720x520) -> packages/shell App -> HostList | HostForm
HostList Open -> open_app_window -> app window + injected embedder -> ahpw
ahpw HostSwitcher -> embedder.switchHost | (new) embedder.manageHosts({ add }) -> show_host_list -> event to the shell
```

### Gaps

- The shell window has the OS frame and border; the host list has its own colours, its own hovers and green-orange dots.
- No footer, one language, and an empty table when there is no host.
- Open leaves the host list on screen; nothing in an app window brings it back.

## Answers

| # | What | Source |
| --- | --- | --- |
| 1 | [The host list stays plain React, in ahpw's colours](../../../decisions/the-host-list-is-plain-react-in-ahpws-colours.md) | Softov, 2026-10-11 |
| 2 | [The desk speaks English, Brazilian Portuguese and Spanish](../../../specs/host-list.md) | Softov, 2026-10-11 |
| 3 | [Opening a host hides the host list](../../../specs/host-list.md) | Softov, 2026-10-11 |
| 4 | [The first run adds a URL host](../../../specs/host-list.md) | Softov, 2026-10-11 |
| 5 | [The app window draws its own rounded frame](../../../specs/app-window.md) | Softov, 2026-10-11 |
| 6 | [ahpw owns its embedding contract](../../../decisions/ahpw-owns-its-embedding-contract.md) | Softov, 2026-10-11 |

| What | Source | Task |
| --- | --- | --- |
| The host list window has no OS frame, round corners and its own title bar, as the app window. | Softov: "no border, rounded"; decision 5 | 01 |
| The host list's close hides it while an app window is open, and quits the desk when none is. | (defaulted: the host list holds the statuses; decision 3) | 01 |
| The host list follows the OS's light or dark. | (defaulted: ahpw's default theme mode is "system") | 02 |
| The desk's icon takes ahpw's mark and colours. | Softov: "Bot default icon color ... does not match" | 03 |
| The footer is ahpio's logo, for now ahpapp's mark at 26 px, and "© ahpio.com", linking to https://ahpio.com in the browser; the title bar keeps ahpw's mark. | Softov: "I'm thinking on the ahpio to use for now the ahpapp logo, give it move visibility. just on the hosts list. on bottom." | 04 |
| The host list window opens at 640 x 400, at least 480 x 300; it opens at 640 x 480 since [the host list window opens at 640 by 480](../../../decisions/the-host-list-window-opens-at-640-by-480.md). | Softov: "Also make it less height." | 07 |
| ahpw's switcher lists "Manage hosts" and "+ Add host" under the hosts. | Decision 3, decision 6 | 08 |

## Proposed architecture

- **Data flow** - The host list reads its words from `i18n.ts` by key; the language is the OS's until one is picked, kept in `localStorage`.
- **Event flow** - `show_host_list { add }` shows the host list and emits `host-list-add` when `add`; the host list opens its Add form on it. `open_app_window` from the shell hides the shell. The last app window's `Destroyed` shows the shell.
- **State flow** - The first run is the host list with no host; it ends when the first host is saved and opened.
- **Layer responsibilities** - `src-tauri`: the shell's frame, hide and show · `packages/shell`: palette, title bar, footer, words, first run · ahpw: the switcher's two items, over `Embedder.manageHosts`.
- **Source-of-truth files** - `packages/shell/src/app.css`, `packages/shell/src/i18n.ts`, ahpw's `src/embed/index.ts`.

## Tasks

| Task | Status | Depends on |
| --- | --- | --- |
| [01 - the host list window has no OS frame](task-01-host-list-frame.md) | done | - |
| [02 - the host list in ahpw's palette](task-02-host-list-palette.md) | done | 01 |
| [03 - the desk's icon in ahpw's mark](task-03-icon.md) | done | - |
| [04 - the footer](task-04-footer.md) | done | 02 |
| [05 - three languages](task-05-languages.md) | done | 02 |
| [06 - the first run](task-06-first-run.md) | done | 05 |
| [07 - Open hides the host list](task-07-open-hides.md) | done | - |
| [08 - ahpw's switcher manages hosts](task-08-switcher-manages-hosts.md) | done | 07 |
| [09 - end to end](task-09-end-to-end.md) | done | 01 to 08 |

## Risks and tradeoffs

- The palette is a copy; when ahpw's theme changes, the host list does not follow until it is copied again.
- A hidden host list with no app window and no tray yet: the last app window's close shows it, so the desk never runs with no window.

## Resume state

- **Done so far:** every task; built 2026-10-13, see [implemented.md](implemented.md).
- **Next action:** none.
- **Open questions:** none.
- **Watch out for:** ahpw's changes go on its branch `desk`, merged with `main` first.

## Final verification checklist

- [x] The host list and an ahpw window side by side share background, text, borders, buttons, hovers and dots, dark and light.
- [x] The host list window has no OS border, round corners, and working minimize, maximize and close.
- [x] The first run, in each language, ends in an open host.
- [x] Open hides the host list; "Manage hosts" and "+ Add host" bring it back.
- [x] `pnpm typecheck`, `pnpm test`, `pnpm lint` pass in ahp-desk; `pnpm typecheck` and `pnpm test` in ahpw.
- [x] `plans/index.md` updated.
