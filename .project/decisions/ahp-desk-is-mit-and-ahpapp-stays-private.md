---
title: ahp-desk is MIT, and ahpapp stays private
status: accepted
date: 2026-10-11
refs:
  - file:///github/ahp-desk/LICENSE - the MIT license
  - file:///github/ahpapp/LICENSE - the Softov Private License
  - file:///github/ahp-desk/.project/decisions/each-desk-build-ships-one-ui.md - the public build ships ahpw, a private build ships ahpapp
---

## Context

ahp-desk can ship ahpw, which is MIT, or ahpapp, which is private.
Its updater reads the releases from GitHub, and a private repository needs a token for them.

## Decision

ahp-desk is a public repository under the MIT license.
ahpapp stays private, and no file from it goes into ahp-desk.
The public build's releases are on ahp-desk's GitHub releases.
Source: Softov: "ahp-desk as MIT, ahpapp as PRIVATE mine.", and then "done, public.".

## Consequences

The updater of the public build reads `latest.json` with no token.
A private build reads ahpapp from a local checkout, so a clone of ahp-desk alone cannot build it.
A private build ships the MIT notice of the shell beside ahpapp.
The private build's releases need a place of their own, which plan 05 chooses.

## Options

- **ahp-desk private, releases in a separate public repository.** Lost: two repositories for one app, and no public source for the public build.
