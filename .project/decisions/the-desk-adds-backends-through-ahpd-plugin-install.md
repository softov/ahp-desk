---
title: The desk adds backends through ahpd plugin install
status: accepted
date: 2026-10-13
refs:
  - file:///github/ahpd/packages/server/src/install.ts - `installPlugins`: npm into the plugin root, then the names into `config.json`'s `plugins`
---

## Context

An ahpd the desk installs needs the backends picked in Add host, installed where ahpd resolves them and named in `config.json`.
`ahpd plugin install` does both, pinning the `@ahpd/sdk` the daemon runs and keeping the rest of `config.json` as it was.
In `@ahpd/server` 0.11.0 it starts `npm` with no shell, which fails with ENOENT on Windows.

## Decision

The desk adds the backends by running the installed ahpd's `plugin install <backends>`, with `AHPD_PLUGIN_ROOT` set (decision 7).
The Windows failure is fixed in ahpd, and the desk's install on Windows waits for that release.
Source: Softov, asked "How does task 03 add the backends while ahpd plugin install fails on Windows?": "I will fix ahpd."

## Consequences

ahpd stays the one place that installs a backend and edits `config.json`.
The desk's backends step on Windows works only from the ahpd release with the fix.

## Options

- **The desk installs the backends in its own npm call and writes `plugins` itself.** Lost: it copies ahpd's install logic into the desk, where the two can drift.
