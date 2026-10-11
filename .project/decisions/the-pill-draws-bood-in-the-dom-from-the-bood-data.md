---
title: The pill draws bood in the DOM from the bood data
status: accepted
date: 2026-10-11
refs:
  - file:///github/ahpapp/src/bood.ts - bodies, moods, actions and SVG paths as data, with no imports
  - file:///github/ahpapp/src/components/Bood.tsx - ahpapp's renderer, on react-native-svg
  - file:///github/opendoop/packages/web/src/components/bood - doop's DOM renderer, with seven creatures
  - file:///github/ahp-desk/mockups/08-floating-renderer.md - React in a transparent webview first
---

## Context

ahpapp draws bood with `react-native-svg` and React Native `Animated`, so its renderer does not run in plain React DOM.
Its `bood.ts` is pure data.
doop's web has a DOM renderer, but it has only seven creatures.

## Decision

The pill is plain React DOM in a transparent webview.
It has its own `<svg>` renderer over the data in ahpapp's `bood.ts`.
`bood.ts` is shared, not copied.
Source: Softov, asked "How does the pill draw the bood creature?": "DOM renderer over bood.ts data (Recommended)".

## Consequences

The pill has every body that ahpapp has.
The pill does not need ahpapp's web build, so it works when the user opens ahpw.
`bood.ts` moves to a place that ahpapp and the pill both import; the pill plan chooses where.
The animations are written a second time, for the DOM.

## Options

- **The pill is ahpapp's web build.** Lost: the pill needs ahpapp, even for a user of ahpw.
- **doop's DOM bood.** Lost: the newer bodies are not there.
