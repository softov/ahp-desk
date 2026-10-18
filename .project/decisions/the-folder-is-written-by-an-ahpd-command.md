---
title: The folder Add host asks for is written by an ahpd command
status: accepted
date: 2026-10-14
refs:
  - file:///github/ahpd/packages/server/src/commands/configure.ts - `configure` asks at a terminal and refuses without one
  - file:///github/ahpd/packages/server/src/commands/options.ts - `paths` in the config, or `--path`, at lines 991-1001
---

## Context

Decision 10 has Add host write the folder it asks for into ahpd's config as `paths`.
ahpd sets `paths` only through `configure`, which asks at a terminal and refuses without one.
The desk already leaves `config.json` to ahpd for the backends ([the desk adds backends through ahpd plugin install](the-desk-adds-backends-through-ahpd-plugin-install.md)).

## Decision

ahpd gains a command that sets `paths` with no terminal, and the desk runs it.
The desk does not write ahpd's `config.json`.
Source: Softov, asked "How should Add host put the chosen folder into ahpd's config (decision 10)?": "An ahpd command (Recommended)".

## Consequences

ahpd stays the only writer of its config.
Add host's folder step waits for the ahpd release with that command.

## Options

- **The desk edits `config.json`.** Lost: a second writer of ahpd's file.
- **`--path` on each start.** Lost: a terminal `ahpd start` would serve another folder.
  It also changes plan 03's answer: the folder asked in Add host, else the home folder.
