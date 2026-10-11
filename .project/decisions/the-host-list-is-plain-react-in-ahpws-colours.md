---
title: The host list stays plain React, in ahpw's colours copied as tokens
status: accepted
date: 2026-10-11
refs:
  - file:///github/ahpw/node_modules/@softov/scena/dist/styles/themes/tokens.css - the light values copied
  - file:///github/ahpw/node_modules/@softov/scena/dist/styles/themes/dark.css - the dark values copied
  - "[code://packages/shell/src/app.css](../../packages/shell/src/app.css) - the host list's styles"
---

## Context

The host list window had its own colours: a grey background, a cyan accent, an orange and a green of its own.
Beside an ahpw window it looked like another product: the background, the buttons, the dots and the hovers all differed.

## Decision

The host list stays plain React with its own CSS.
Its colours are ahpw's theme values copied into CSS custom properties with the same names (`--oo-color-*`), dark and light, and its buttons, hovers and dots use them as ahpw does.
Source: Softov, asked what the host list window is built with, now that it should match ahpw: "Plain React, copied colours".

## Consequences

The host list carries no UI library and stays small.
A change of ahpw's palette is copied by hand; the copied values name their source file.

## Options

- **Build the host list on scena, as ahpw is.** Lost: the host list would carry scena and its runtime for one list and one form.
