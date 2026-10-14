---
title: The Node downloaded on a server is the newest 22, from latest-v22.x, checked against its SHASUMS256.txt
status: accepted
date: 2026-10-14
refs:
  - "[code://scripts/stage-node.mjs](../../scripts/stage-node.mjs) - the desk's own Node, pinned to v22.23.3 and checked against its SHASUMS256"
  - https://nodejs.org/dist/latest-v22.x/ - the newest 22 release, with its SHASUMS256.txt
---

## Context

[The remote ahpd runs on the server's own Node 22, or one it downloads](the-remote-node-is-its-own-or-downloaded.md) has the server download the official Node build when it has no Node 22 or later.
The desk pins its own Node in `scripts/stage-node.mjs`.

## Decision

The server downloads the newest Node 22 from `https://nodejs.org/dist/latest-v22.x/`.
The archive's name and its sum come from that folder's `SHASUMS256.txt`, and the archive is checked against it before it is unpacked.
Source: Softov, asked "p10 q9: when the server has no Node 22, which Node does the desk download there?": "latest-v22.x".

## Consequences

A server gets the fixes of the 22 line without a desk release.
Two installs on different days can run different Node 22 releases, so the install reports the version it got.
The desk's build carries no Node version for the server.

## Options

- **The desk's pinned v22.23.3.** Lost: a server keeps an older 22 until the desk is released again.
