---
title: The tray menu gets "Show pill" with the pill, in plan p6
status: accepted
date: 2026-10-14
refs:
  - "[code://.project/plans/desk/01-first-build-p4-tray-and-releases/task-01-tray-icon-and-menu.md](../../.project/plans/desk/01-first-build-p4-tray-and-releases/task-01-tray-icon-and-menu.md) - the menu p4 builds, Hosts and Quit"
  - "[code://.project/plans/desk/01-first-build-p4-tray-and-releases/task-02-close-to-tray-and-setting.md](../../.project/plans/desk/01-first-build-p4-tray-and-releases/task-02-close-to-tray-and-setting.md) - adds Settings between them"
  - "[code://.project/plans/desk/01-first-build-p6-bot-pill/plan.md](../../.project/plans/desk/01-first-build-p6-bot-pill/plan.md) - the pill, and with it the item"
---

## Context

Plan p4's runtime path listed the tray menu as Hosts, Settings, Show pill and Quit.
The pill does not exist until plan p6.

## Decision

Plan p4's tray menu is Hosts, Settings and Quit, with no "Show pill".
Plan p6 adds "Show pill" when it builds the pill.
Source: Softov, asked "p4 q3: does the tray menu have \"Show pill\" before the pill exists?": "No, p6 adds it".

## Consequences

The menu never shows an item that does nothing.
Plan p6 adds the item's id to `tray.rs`, its field to `TrayWords`, and its word in the three languages.

## Options

- **Yes, disabled until p6.** Lost: a disabled item with no pill behind it tells the user nothing, and p6 would still have to wire it.
