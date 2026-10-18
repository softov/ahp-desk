---
title: App window
status: current
covers:
  - src-tauri/src/app_window.rs
  - src-tauri/src/ui_source.rs
  - src-tauri/src/frame.rs
  - packages/ahpw-embedder/src/**
  - scripts/stage-ui.mjs
decisions:
  - decisions/ahpw-owns-its-embedding-contract.md
  - decisions/ahpw-keeps-a-hosts-pages-under-the-hosts-address.md
  - decisions/each-ui-is-served-from-its-own-uri-scheme.md
  - decisions/each-desk-build-ships-one-ui.md
  - decisions/ahpw-reaches-the-desk-with-a-desk-release.md
refs:
  - "[code://src-tauri/src/app_window.rs#L118](../../src-tauri/src/app_window.rs#L118) - `open_app_window`: a new window, the open one focused, or a switch"
  - "[code://src-tauri/src/app_window.rs#L26](../../src-tauri/src/app_window.rs#L26) - `page_url`, the UI with `?host=<address>`"
  - "[code://src-tauri/src/app_window.rs#L191](../../src-tauri/src/app_window.rs#L191) - the last app window closed shows the host list"
  - "[code://src-tauri/src/app_window.rs#L208](../../src-tauri/src/app_window.rs#L208) - `open_outside`, links in the OS's own app"
  - "[code://src-tauri/src/ui_source.rs#L29](../../src-tauri/src/ui_source.rs#L29) - `resolve`, where the UI's files come from"
  - "[code://src-tauri/src/ui_source.rs#L102](../../src-tauri/src/ui_source.rs#L102) - `file_for`, the scheme's fallback to `index.html`"
  - "[code://packages/ahpw-embedder/src/embedder.ts#L34](../../packages/ahpw-embedder/src/embedder.ts#L34) - `createEmbedder`, ahpw's contract over the desk's commands"
  - git://4c89294 - the deleted decisions this spec was written from: a-ui-bundle-comes-from-the-local-repository-or-from-npm, ahpw-shows-one-host-at-a-time-on-the-desk, the-app-window-draws-its-own-rounded-frame, the-shell-contract-is-a-shared-npm-package
---

An app window shows the build's UI, ahpw in the public build, on one host.
With the setting "One window with every host", one app window shows every host instead.
The UI is served from its own URI scheme.
The desk injects ahpw's embedder into the window before the page runs.
ahpw reads the window's host, its token and the host list from it.
The window has no OS frame; ahpw draws its title bar and a rounded edge.
The code is in [`code://src-tauri/src/app_window.rs`](../../src-tauri/src/app_window.rs) and [`code://packages/ahpw-embedder`](../../packages/ahpw-embedder).

## Scenarios

### Where the UI comes from

WHEN the desk is built
THEN `AHP_DESK_UI` names the one UI it ships, `ahpw` by default, and that name is the URI scheme too
AND a debug build serves the published `@ahpd/web` from `node_modules`, or with `AHP_DESK_AHPW=local` loads ahpw's dev server
AND a release build serves the copy bundled in its resources
AND the scheme falls back to `index.html` for a path the page handles itself
AND a new ahpw reaches the desk only inside a desk release.

### A host opens in a window

WHEN a host is opened
THEN the desk opens a window titled with the host's name, on the UI with `?host=<the host's address>`
AND a host that already has a window gets that window shown and focused instead
AND the host list hides when it opened the host.

### Every host in one window

WHEN a host is opened while Settings says "One window with every host"
THEN the desk opens one window, `app-:every`, on the UI with `?hosts=every&focus=<the host's id>`
AND with that window open, it is shown and focused, and sent `focus-host` with the host's id
AND ahpw shows only that host in its lists, and the host list hides when it opened the host
AND removing a host closes no window there; the host leaves its list through `hosts-changed`
AND a change of the setting applies to the next Open, and leaves open windows as they are.

### Who reads a token

WHEN a window asks for a host's token
THEN the host list and the window with every host may read any host's token
AND a window of one host reads only its own host's token, and is refused another's.

### The window moves to another host

WHEN ahpw's host switcher picks another host
THEN the same window navigates to that host's page and takes its name as the title.

### A host's pages stay with its address

WHEN ahpw keeps its tabs, layout and added folders
THEN it keeps them under a key that carries the window's host address, because every host shares the UI's origin
AND the theme, the language and the groupings stay shared.

### The embedder

WHEN ahpw runs in an app window
THEN `globalThis.ahpwEmbedder` is there before ahpw's own scripts run
AND it gives the window's host, with its address and token
AND the host list and its changes, and every host's status
AND a switch to another host, the host list on demand, on its Add form or on a host's form, and the window controls
AND in the window with every host, it says `mode: 'multi'`, gives `*` as the address, gives any host's token, and passes on `focus-host`
AND the host that window opened on is given once, and dropped from its address, so a reload keeps the person's filter.

### The frame

WHEN an app window is built
THEN it has no OS frame and no OS shadow, it is transparent, and on Windows it asks DWM for round corners
AND ahpw rounds its own root while the window is not maximized, and draws minimize, maximize or restore, and close.

### Links leave the desk

WHEN a page opens a link to another site, or a new window
THEN an `http`, `https` or `mailto` link opens in the OS's own app, and anything else is dropped
AND the window never leaves the UI's origin.

### The last window closes

WHEN the last app window closes
THEN the host list shows again, so the desk always has a window to reach it.

## Known gaps

- The window with every host needs an `@ahpd/web` with ahpw's hosts plan 02 contract; the pin moves when ahpw releases it.
