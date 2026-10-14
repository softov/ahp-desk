---
title: The local ahpd serves the folder asked in Add host, else the home folder
status: accepted
date: 2026-10-14
refs:
  - file:///github/ahpd/packages/server/src/commands/options.ts - with no `--path` and no `paths`, ahpd serves `process.cwd()`, at lines 991-1001
---

## Context

ahpd serves the folders its config names in `paths`, and with none, the folder it was started in.
The desk starts ahpd from its own process, whose folder is the app's install folder.
ahpd's config folder, `~/.config/ahpd`, is not the folder it serves.

## Decision

Add host, "This computer", asks for the folder to serve, with the home folder as the default, and writes it into ahpd's config as `paths`.
When the desk starts an ahpd whose config names no folder, it starts it in the home folder.
Source: Softov, asked "When the desk starts the local ahpd and its config names no folder, which folder should it serve?": "both. but I think ahpd uses user home, does not? ~/.config/ahpd?".

## Consequences

A local ahpd never serves the desk's install folder.
Folders already in ahpd's config win, so a user who ran `ahpd configure` keeps theirs.

## Options

- **Home folder only.** Lost: no way to name the folder in Add host.
- **Asked in Add host only.** Lost: an ahpd whose config names no folder, found or set up outside the desk, would serve the install folder.
