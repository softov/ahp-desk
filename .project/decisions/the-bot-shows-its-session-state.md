---
title: The bot shows its session state
status: accepted
date: 2026-10-11
refs:
  - file:///github/ahp-desk/mockups/06-bot-states.md - the table of states, the speech bubble, and the transits
  - file:///github/ahpapp/src/components/BotCluster.tsx - ahpapp's mood is for the UI only, at line 216
---

## Context

A bot's record names its session.
ahpapp sets the bot's mood from the UI only: idle, observing on hover, happy when open.
On the desk the bot is the whole pill, so it is the place to see what the session does.

## Decision

The bot shows its session state with the moods, actions and badge in mockup 06.
It shows its last line in a speech bubble for a few seconds.
It plays a transit when the pill shows, hides, or changes bot.
Source: Softov, asked "Mockup 06 maps the bot's session state to bood moods. Lock that mapping as the first pill behaviour?": "Yes, all of 06 (Recommended)".

## Consequences

The pill follows the summary status of each bot's session on its host.
"Needs you" asks for a click, which opens the input on the question.

## Options

- **Moods only.** Lost: the bot's words have no place, because the bot has no frame for text.
