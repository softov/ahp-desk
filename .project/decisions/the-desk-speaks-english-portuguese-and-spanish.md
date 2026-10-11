---
title: The desk speaks English, Brazilian Portuguese and Spanish
status: accepted
date: 2026-10-11
refs:
  - "[code://packages/shell/src](../../packages/shell/src) - the host list, whose words are translated"
---

## Context

The host list, its forms and the first run were written in English only.

## Decision

The desk's own windows are in English, Portuguese (Brasil) and Spanish.
The language follows the OS until one is picked; the pick is kept.
Source: Softov, asked which languages the desk ships with: "English, Português (Brasil), Español".

## Consequences

Every word the desk shows is a key in three dictionaries, checked by a test to have the same keys.
ahpw's own words are ahpw's to translate; the desk does not reach into them.

## Options

- **English only for the first build.** Lost: the first users read Portuguese.
