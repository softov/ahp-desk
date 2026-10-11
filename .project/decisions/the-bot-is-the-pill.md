---
title: The bot is the pill
status: accepted
date: 2026-10-11
refs:
  - file:///github/ahp-desk/mockups/04-bot-on-pill.md - the sketch of all three options
  - file:///github/doop/packages/tauri/src-tauri/src/windows.rs - doop's pill, a framed window with a label and a status dot
---

## Context

doop's desktop shell has a floating pill: a frame with the bood logo, a label and a status dot.
ahp-desk shows a bot from a host in that place when a bot is available.
The bot can sit inside a frame, or it can be the whole floating window.

## Decision

When a bot is available, the floating window is only the bot's creature, with a transparent background and no frame.
No name label is drawn.
Bots dropped on each other form a cluster.
A cluster is on top of one another, side by side, or packed as balloons in a circle or a cloud.
Source: Softov, asked to choose between options A, B and C in `mockups/04-bot-on-pill.md`: "on bot I prefer it Option B - the bot is the pill".

## Consequences

The creature reads as a companion and not as a widget.
The status needs a badge on the creature, because there is no frame to hold a dot.
Text has no fixed place, so the bot's name and input show only on hover or click.
The window shape follows the creature or the cluster, so its size changes with the cluster's layout.

## Options

- **A - the bot inside the pill frame.** Closest to doop, with room for the name and a dot. Lost: the frame makes the bot a widget.
- **C - a fixed row of up to four bots.** Lost: dropping one bot on another makes a cluster, in a layout you choose.
