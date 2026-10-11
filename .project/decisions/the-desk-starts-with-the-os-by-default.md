---
title: The desk starts with the OS by default
status: accepted
date: 2026-10-11
refs:
  - https://v2.tauri.app/plugin/autostart/ - the Tauri autostart plugin
---

## Context

The desk keeps hosts and shows bots, so it is useful only while it runs.
An app that starts with the OS without asking can surprise the user.

## Decision

The first run shows the checkbox "Start with the OS", on by default.
The same checkbox is in Settings.
Source: Softov, asked "Does ahp-desk start with the OS?": "same as before. settings on as default on ui form.".

## Consequences

By default the pill and the tray are there after a login.
The user sees the choice once, at the first run.

## Options

- **A setting, off by default.** Lost: the user must find the setting to keep the desk on.
- **Never.** Lost: the user starts the desk by hand after every login.
