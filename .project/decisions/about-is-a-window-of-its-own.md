---
title: About is a window of its own, not a screen of the host list
status: accepted
date: 2026-10-27
supersedes: decisions/the-desk-shows-its-versions-on-an-about-screen.md
refs:
  - "[code://packages/shell/src/about/AboutScreen.tsx](../../packages/shell/src/about/AboutScreen.tsx) - the About screen this replaces"
  - "[code://packages/shell/src/App.tsx#L32](../../packages/shell/src/App.tsx#L32) - `Screen`, where `about` is one of the host list's screens"
  - "[code://src-tauri/src/lib.rs#L84-L94](../../src-tauri/src/lib.rs#L84-L94) - the host list window's builder, the frameless pattern the new window follows"
---

## Context

About is one of the host list window's screens, so opening it replaces whatever the host list showed.
To report a problem, a person wants the screen with the problem, such as a host form with its error, and the desk's versions in one screenshot.

## Decision

About opens in a small window of its own, with the desk's, ahpw's and Node's versions, the MIT license, and the links to ahpio.com and the releases.
Opening it again while it is open brings that window to the front.
The host list keeps whatever screen it was on.
The tray's About, Settings' "About ahp-desk" and the title bar's logo menu open it.
The desk's update check is not in it ([the desk's update check is a window of its own](the-desks-update-check-is-a-window-of-its-own.md)).
Source: Softov, 2026-10-27: "the about page.. instead a page.. it can be another window? so for like a problem... I can open the page with problem.. says host form.. and the about and take a shot on the same screen.."

## Consequences

The host list loses its About screen and the `host-list-about` event; the tray opens the window itself.
The window is a second page of the shell's bundle, with a capability of its own that grants only `about`.
A language picked in the host list must reach an open About window.

## Options

- **Keep the About screen in the host list.** Lost: About and the screen with the problem cannot show at once.
