---
title: A cluster holds four bots
status: accepted
date: 2026-10-11
refs:
  - file:///github/ahpapp/src/bots.ts - `CLUSTER_MAX = 4` at line 100
  - file:///github/ahp-desk/mockups/04-bot-on-pill.md - "Open: how many bots a cluster can hold"
---

## Context

ahpapp draws at most four bots in a cluster and puts the rest behind `+N`.
On the desk, the window grows with the cluster.

## Decision

A cluster draws at most four bots.
A fifth and later bots show as a `+N` bubble, which opens the picker.
Source: Softov, asked "How many bots can one cluster hold?": "Four, rest behind +N (Recommended)".

## Consequences

The desk and ahpapp behave the same.
The pill window has a known largest size.

## Options

- **No limit.** Lost: the window can cover the desktop.
- **Eight.** Lost: a second number that ahpapp does not share.
