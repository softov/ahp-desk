---
title: A bot is talked to from an input that shows on hover
status: accepted
date: 2026-10-11
refs:
  - file:///github/ahp-desk/mockups/05-bot-input.md - the sketch of the input
  - file:///github/ahpapp/src/components/BotLayer.tsx - ahpapp's popover, with the name, the description and a message box
---

## Context

ahpapp opens a popover when you tap a bot.
The popover shows the bot's name and description, a message box, and a menu.
On the desktop, the bot is the pill and has no frame for its name.

## Decision

Hover on a bot, or click it, and an input shows beside it.
The input's placeholder carries the bot's name, as "Say something to Nimbo...".
A `⋯` in the input opens the other options.
Source: Softov: "instead put Say {Bot name} Something as placeholder on a input that show on mouse over or on click with more ... options".

## Consequences

The bot's name is never drawn on its own, only in the placeholder.
A hover that shows an input must not take the keyboard focus from the app under the bot.
In a cluster, each bot has its own input.

## Options

- **ahpapp's popover on click.** It shows the name and description above a message box. Lost: it is more than a bot with no frame needs, and the name shows only on a click.
- **The name as a tooltip on hover.** Lost: a second thing to show, when the placeholder can carry the name.
