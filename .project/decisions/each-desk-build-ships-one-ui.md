---
title: Each desk build ships one UI
status: accepted
date: 2026-10-11
refs:
  - file:///github/ahp-desk/mockups/02-hosts.md - "Open with" is there only if both UIs ship; "If only one UI ships, this row goes."
  - file:///github/ahpapp/LICENSE - ahpapp is under a private license
  - npm://@ahpd/web@0.2.0 - ahpw is MIT and public
---

## Context

ahpw is public under MIT, and ahpapp is private.
The desk can load either one.
One build could ship both, with a choice of UI on each Open, or each build could ship one.

## Decision

A desk build ships one UI, chosen when it is built.
The public build ships ahpw.
A private build ships ahpapp.
No build has both, so the host list has no row "Open with".
Source: Softov, on the "Open with" question: "I was thinking in separated distro... one public another private. ahpapp private". Asked then what is wrong in p5's goal: "\"Open with\" row".

## Consequences

The public build holds no private code.
The contract has no choice of UI: a build opens every host in its one UI.
The build reads its UI from a build setting, and the private build needs its own name, identifier and release channel.

## Options

- **One build with both UIs and a row "Open with".** Lost: the private ahpapp would ship inside the public build.
