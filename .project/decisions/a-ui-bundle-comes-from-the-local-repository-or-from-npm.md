---
title: A UI bundle comes from the local repository or from npm
status: accepted
date: 2026-10-11
refs:
  - file:///github/ahpw/vite.config.ts - ahpw builds to dist/app with base './', and its dev server listens on 127.0.0.1:5180
  - npm://@ahpd/web@^0.2.0 - the published ahpw, with dist/app inside
---

## Context

ahp-desk loads ahpw and, later, ahpapp.
A developer wants to test a change in ahpw inside the desk before it is published.
A release must load a known, published version.

## Decision

Each UI has a source, `local` or `npm`, that a developer chooses when the shell runs or builds.
`local` uses the sibling repository: its dev server under `pnpm tauri dev`, or its built `dist` in a build.
`npm` uses the published package at the version that ahp-desk pins.
Source: Softov, asked "Where does ahp-desk get the ahpw UI bundle from?": "optional? so I can test with local and with npm?".

## Consequences

The shell reads the source from an environment variable for each UI.
ahpapp is not on npm, so its only source is `local`.
A release build uses `npm` for ahpw.

## Options

- **npm only.** Lost: no test of an unpublished ahpw inside the desk.
- **A git submodule.** Lost: ahpw is built from source in every build, and a version pin moves through git, not npm.
