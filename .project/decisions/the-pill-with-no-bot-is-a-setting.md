---
title: The pill with no bot is a setting
status: accepted
date: 2026-10-11
refs:
  - file:///github/ahp-desk/mockups/07-bot-picker.md - "When no bot is on the pill: show the ahp-desk logo, or hide the pill"
---

## Context

The pill is the way to reach the host list and the settings.
With no bot on it, there is no creature to show.

## Decision

A setting chooses what shows when no bot is on the pill: the ahp-desk logo, or nothing.
The default is the logo.
Source: Softov, asked "When no bot is on the pill, what shows?": "A setting: logo or hidden (Recommended)".

## Consequences

By default, the menu stays one click away.
With the pill hidden, the tray is the only way in.

## Options

- **Always the logo.** Lost: no way to have a clean desktop with no bot.
- **Always hidden.** Lost: a first run with no bot shows nothing on the desktop.
