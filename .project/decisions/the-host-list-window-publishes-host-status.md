---
title: The host list window publishes each host's status to every window
status: accepted
date: 2026-10-11
refs:
  - file:///github/ahp-desk/packages/shell/src/hosts/status.ts - the one MultiHostClient that already watches every host
  - "[code://packages/ahpw-embedder/src/embedder.ts](../../packages/ahpw-embedder/src/embedder.ts) - the embedder ahpw reads the statuses through"
  - file:///github/ahp-desk/.project/decisions/ahpw-shows-one-host-at-a-time-on-the-desk.md - why ahpw holds no connection to the other hosts
---

## Context

ahpw's host switcher lists every host with its status dot, but ahpw connects only to its own host.
The host list window already holds one AHP connection per host and knows each one's state.
A second watcher in each app window would open one more connection per host per window.

## Decision

The host list window sends each host's status as the event `hosts-status`, and the shell keeps the last map so a window that opens late can read it.
ahpw's embedder gives it `statuses()` and `onStatuses(handler)` (decision: ahpw owns its embedding contract).
Source: Softov, asked "Where should ahpw get the other hosts' status from?": "shell publishes.. but consider moving to rust future case and rust managing it without the shell maybe."

## Consequences

One connection per host for the whole desk, and every window shows the same state.
The statuses exist only while the host list window runs; it is never destroyed, only hidden.
The contract does not say who computes a status, so the Rust side can take the watching over later without a change in any UI.

## Options

- **Rust owns the status.** An AHP client in Rust keeps one connection per host and works with no window open. Not now: it needs a WebSocket AHP client in Rust; it is the way forward when the desk runs with no window, as the tray and the bot pill of p4 and p6 may need.
- **No dots.** The switcher lists names only. Lost: the locked switcher decision of p2.
