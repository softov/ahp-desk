---
title: The remote ahpd serves the folder asked in Add host, the server user's home by default
status: accepted
date: 2026-10-14
refs:
  - file:///github/ahpd/packages/server/src/commands/options.ts - with no `--path` and no `paths`, ahpd serves `process.cwd()`, at lines 991-1001
---

## Context

ahpd serves the folders its config names in `paths`, and with none, the folder it was started in.
A server's ahpd is often meant for one project folder, not the whole home.
For "This computer", [the local ahpd serves the folder asked in Add host, else the home folder](the-local-ahpd-serves-the-folder-asked-in-add-host-else-home.md), and [an ahpd command writes it](the-folder-is-written-by-an-ahpd-command.md).

## Decision

Add host, "A server over SSH", has a Folder field, filled with the server user's home, and the remote ahpd serves that folder.
The desk writes it into the server's ahpd config as `paths` with the same ahpd command as for "This computer", and the unit starts ahpd in the home folder.
Source: Softov, asked "p10 q6: which folder does the remote ahpd serve?": "A form field".

## Consequences

The SSH form and the "This computer" form ask the same thing the same way.
The SSH install waits, as "This computer" does, for the ahpd release with the command that sets `paths`.
Folders already in the server's ahpd config win, as they do on this computer, so an ahpd found on the server keeps its own.

## Options

- **The home folder, with no field.** Lost: a person who wants one project served edits ahpd's config on the server by hand.
