---
title: A new ahpw reaches the desk with a desk release
status: accepted
date: 2026-10-13
refs:
  - "[code://scripts/stage-ui.mjs](../../scripts/stage-ui.mjs) - the build copies `@ahpd/web`'s `dist/app` into the bundle"
  - "[code://package.json](../../package.json) - the `@ahpd/web` pin"
---

## Context

The desk serves ahpw from a copy made at build time, from the `@ahpd/web` version `package.json` pins.
A new ahpw on npm reaches nobody's desk until something brings it.

## Decision

A new `@ahpd/web` reaches the desk only inside a desk release: the pin moves, the desk is built and released, and the desk's own updater delivers it.
The desk never downloads a UI bundle at run time.
Source: Softov, asked "How should a new ahpw reach the desk?": "With a desk release (Recommended)".

## Consequences

What a person runs is what CI built and signed, with one version to test.
Every ahpw release that matters to the desk is followed by a desk release; the release steps say so.
An ahpw fix waits for that desk release.

## Options

- **The desk fetches `@ahpd/web` from npm at run time.** Lost: an unsigned UI bundle the desk was never tested with, and two versions to track per install.
