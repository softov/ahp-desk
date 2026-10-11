---
title: The backends are picked in Add host
status: accepted
date: 2026-10-11
refs:
  - file:///github/ahpd/packages/server/src/commands/run.ts - ahpd exits when no agent plugin is installed, at lines 848-852
  - file:///github/ahpd/packages - the backends agent-claude, agent-acp, agent-cofold and agent-pi, and the bot plugin
---

## Context

ahpd does not start without an agent backend.
ahpd has four backends today, and the bot plugin is a separate package.
The shell installs the local ahpd, so it must choose plugins.

## Decision

Add host, kind "This computer", lists the backends as checkboxes, with `@ahpd/agent-claude` on by default.
It also offers `@ahpd/bot`, so the bots work.
Source: Softov, asked "ahpd refuses to start with no agent backend plugin. Which backend(s) does the local setup install?": "Let the user pick in Add host (Recommended)".

## Consequences

The user sees the backends before the install.
A new backend in ahpd is a change to the list in the shell.

## Options

- **A fixed set, agent-claude and bot.** Lost: another backend needs ahpd's own commands after the setup.
