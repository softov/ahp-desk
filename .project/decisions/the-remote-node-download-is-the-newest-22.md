---
title: The Node downloaded on a server is the newest 22, not the version the desk pins
status: accepted
date: 2026-10-14
refs:
  - "[code://scripts/stage-node.mjs](../../scripts/stage-node.mjs) - the desk's own Node, pinned to v22.23.3"
  - https://nodejs.org/dist/latest-v22.x/ - the newest 22 release
---

## Context

A server with no Node 22 or later downloads the official build, as [plan 10's Answers](../plans/desk/10-ssh-install/plan.md) say.
The desk pins its own Node in `scripts/stage-node.mjs`.
So an unpinned download on the server looks like a gap, to close by pinning it the same way.

## Decision

The server downloads the newest Node 22 release, from `latest-v22.x`, and not the version the desk pins for itself.
Source: Softov, asked "p10 q9: when the server has no Node 22, which Node does the desk download there?": "latest-v22.x".

## Consequences

A server gets the fixes of the 22 line without a desk release.
Two installs on different days can run different Node 22 releases.

## Options

- **The desk's pinned v22.23.3.** Lost: a server keeps an older 22 until the desk is released again.
