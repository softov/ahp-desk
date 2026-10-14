---
title: A ready desk update is the first item of the tray menu, and a click installs it and restarts
status: accepted
date: 2026-10-14
refs:
  - file:///github/ahp-desk/.project/plans/desk/01-first-build-p4-tray-and-releases/task-03-updater-with-signing-key.md - p4 task 03, which builds the notice, the install command and this tray item
  - file:///github/ahp-desk/.project/plans/desk/01-first-build-p4-tray-and-releases/task-01-tray-icon-and-menu.md - p4 task 01, which makes the tray and its menu
---

## Context

Plan p9's decision 1 says a downloaded desk update is announced in the host list and in the tray, and installed only on a click.
The host list may be closed while the desk runs in the tray, so the tray has to say it too.
A tray can say it as a menu item, an OS notification, or its icon's tooltip or badge.

## Decision

While a desk update is ready, the first item of the tray menu says so with its version ("Restart to update to X"), and a click installs it and restarts the desk.
p4 task 03 builds it, with the host list's notice and the install command ([decision](the-desk-update-notice-is-built-in-p4-task-03.md)); p9 task 06 only checks again every 6 hours and fills the same notice.
Source: Softov, asked "p9 q5: how does the tray show that a desk update is ready?": "First menu item".

## Consequences

The tray item and the host list's Restart call the same install.
The item is seen only when the menu is opened, so a desk left in the tray tells nobody until then, which suits an update that never installs by itself.
The item's words are the host list notice's, in the desk's three languages.

## Options

- **An OS notification.** Lost: it is shown once and gone, and a click on it is not a place the person can come back to.
- **The tray icon's tooltip or a badge.** Lost: a tooltip cannot be clicked, and a badge is not drawn the same on Windows, macOS and Linux.
