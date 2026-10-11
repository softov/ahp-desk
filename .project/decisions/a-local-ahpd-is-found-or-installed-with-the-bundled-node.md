---
title: A local ahpd is found, or installed with the bundled Node
status: accepted
date: 2026-10-11
refs:
  - file:///github/ahpd/packages/server/package.json - @ahpd/server, bin `ahpd`, Node 22 or later
  - file:///github/opendoop/packages/tauri/src-tauri/src/pood_process.rs - doop ships Node and pood as resources
  - file:///github/ahp-desk/mockups/02-hosts.md - "ahpd found at ~/.local/bin/ahpd [Change]"
---

## Context

ahpd needs Node 22 or later, and its plugins are npm packages.
A machine may already have ahpd, may have Node with no ahpd, or may have neither.
doop ships Node and its daemon in the installer, so each daemon release needs a desk release.

## Decision

The shell first looks for an `ahpd` on `PATH`, or at the path set in Add host, and uses it.
If there is none, the shell installs `@ahpd/server` and the chosen backends with npm into a folder that the app owns.
That npm comes from a Node 22 runtime that the installer ships.
The installer does not ship ahpd or its plugins.
Source: Softov, asked "How does ahp-desk get a local ahpd onto a machine?": "Use an existing ahpd, else install with bundled Node (Recommended)".

## Consequences

The desk works on a machine with no Node.
ahpd updates through npm, with no desk release.
The first setup of "This computer" needs the network.
The installer is about 30 MB larger for the Node runtime.

## Options

- **System Node only.** Lost: a fresh machine needs a manual Node install first.
- **Ship everything.** Lost: a large installer, and each ahpd release needs a desk release.
