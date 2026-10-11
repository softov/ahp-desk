---
title: The picker lists the bots of every host
status: accepted
date: 2026-10-11
refs:
  - file:///github/ahp-desk/mockups/07-bot-picker.md - the picker, grouped by host
---

## Context

A user can have bots on many hosts.
The pill can follow one host, or show bots from any host.

## Decision

One picker lists the bots of every connected host, grouped by host.
Each bot has an "on the pill" box, and any number can be on.
Bots float apart until you drop one on another.
Source: Softov, asked "Mockup 07 proposes a picker over the bots of every connected host, each with an 'on the pill' box. Lock that?": "Yes, across all hosts (Recommended)".

## Consequences

The pill connects to every host in the host list, not only to the host of an app window.
A cluster can hold bots from different hosts.

## Options

- **One host's bots only.** Lost: a bot on another host needs a switch of host.
