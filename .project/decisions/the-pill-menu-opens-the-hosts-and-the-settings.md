---
title: The pill's menu opens the hosts and the settings
status: accepted
date: 2026-10-11
refs:
  - file:///github/ahp-desk/mockups/05-bot-input.md - the `⋯` menu with Hosts and Settings
  - file:///github/opendoop/packages/tauri-shell/src/App.tsx - doop's pill opens its panels
---

## Context

doop's pill is the way into its panels.
On the desk the pill is a bot, and the tray also has a menu.

## Decision

The `⋯` menu of the bot's input has Hosts and Settings, as mockup 05 shows.
The tray has them too.
Source: Softov, asked "Is the pill the way into the host list and settings, as doop's pill is?": "Yes, from the ⋯ menu (Recommended)".

## Consequences

The host list is one click from the bot.
The logo on the pill with no bot opens the same menu.

## Options

- **The tray only.** Lost: the pill is closer to the mouse than the tray.
