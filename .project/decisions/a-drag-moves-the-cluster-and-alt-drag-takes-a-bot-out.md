---
title: A drag moves the cluster, and Alt with a drag takes a bot out
status: accepted
date: 2026-10-11
refs:
  - file:///github/ahp-desk/mockups/04-bot-on-pill.md - "Open: how a drag tells move the group from take this bot out"
---

## Context

A cluster moves as one, and a bot can leave it.
Both start with a drag on a bot.

## Decision

A drag on any bot of a cluster moves the whole cluster.
A drag with Alt held takes that one bot out.
The menu item "Take <name> out" also takes a bot out.
Source: Softov, asked "In a cluster, how does a drag tell 'move the group' from 'take this bot out'?": "Drag moves group, Alt+drag takes out (Recommended)".

## Consequences

The common move needs no key.
Taking a bot out needs a key or the menu, so it does not happen by accident.

## Options

- **A long press takes a bot out.** Lost: a slow drag would take a bot out by accident.
- **The menu only.** Lost: no direct way with the mouse.
