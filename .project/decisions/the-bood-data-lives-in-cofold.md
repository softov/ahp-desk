---
title: The bood data lives in cofold
status: accepted
date: 2026-10-11
refs:
  - file:///github/ahpapp/src/bood.ts - bodies, moods, actions and paths as data, with no imports
  - file:///github/cofold/README.md - one family of packages with no runtime dependencies, under MIT
  - file:///github/ahp-desk/.project/decisions/the-pill-draws-bood-in-the-dom-from-the-bood-data.md - the decision that needs one shared place for the data
---

## Context

The pill and ahpapp both draw bood from the same data.
That data lives only in ahpapp, which is private and not on npm.
cofold is a family of agent packages with no runtime dependencies, and `bood.ts` has no imports.

## Decision

The bood data moves to a cofold package, `@cofold/bood`.
ahpapp and the desk pill import it, and neither keeps a copy.
Source: Softov, asked "Where does bood.ts live, so ahpapp and the desk pill both import it?": "inside cofold bood? since its agent related?".

## Consequences

The bood data is published under cofold's MIT license.
A change to a body is a cofold release, and ahpapp and the desk move to it.
A renderer stays with its surface: React Native in ahpapp, the DOM in the desk.

## Options

- **A new repository, `@softov/bood`.** Lost: one more repository and release for a small data package.
- **A package inside ahp-desk.** Lost: ahpapp would depend on the desk's repository.
- **A package inside ahpapp.** Lost: ahpapp is private and not on npm.
