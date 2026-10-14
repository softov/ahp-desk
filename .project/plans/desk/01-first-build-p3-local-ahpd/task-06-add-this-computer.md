---
title: Add host has "This computer", with the install steps
status: doing
depends: [task-03-install-ahpd.md, task-04-start-find-stop.md]
layer: "packages/shell"
refs:
  - "[code://packages/shell/src/FirstRun.tsx](../../../../packages/shell/src/FirstRun.tsx) - the first run's where step"
  - "[code://packages/shell/src/hosts/HostForm.tsx](../../../../packages/shell/src/hosts/HostForm.tsx) - Add host"
  - file:///github/ahp-desk/mockups/02-hosts.md - Add host, kind This computer
---

## Objective

Add host and the first run offer "This computer": the ahpd found or the install, the backends as checkboxes (decision 2), and the install's steps.

## Files

- `UPDATE: packages/shell/src/hosts/HostForm.tsx` - the Kind field first in Add host.
- `CREATE: packages/shell/src/hosts/LocalForm.tsx` - Add host "This computer": the ahpd found with Change, the too-old note, the backends, the two settings, the install and the start.
- `CREATE: packages/shell/src/hosts/KindField.tsx` - This computer or URL; This computer once.
- `UPDATE: packages/shell/src/App.tsx`, `packages/shell/src/app.css`, `src-tauri/src/lib.rs`, `src-tauri/capabilities/shell.json` - the local form, its styles, and the dialog plugin for Change.
- `CREATE: packages/shell/src/hosts/InstallSteps.tsx` - npm's lines, and the result.
- `UPDATE: packages/shell/src/FirstRun.tsx` - "This computer" in the where step.
- `UPDATE: packages/shell/src/i18n.ts` - the words, in en, pt-BR and es.

## Steps

1. `@ahpd/agent-claude` is on by default, and `@ahpd/bot` is offered.
2. The host is saved as `kind: local` once ahpd is on.
3. An ahpd found older than 0.8.0 (`supported: false`) is shown with its version, the version needed, `npm i -g @ahpd/server` and Check again, and nothing is installed (decision 9).
4. The folder to serve, the home folder by default, written into ahpd's config as `paths` by an ahpd command before the first start (decisions 10, 11).

## Validation

- Shell tests for the form; a CDP check of the form and the steps in the three languages.

## Resume

Started 2026-10-14.

- Done: Add host opens on Kind "This computer" while the list has none, else on URL; This computer is one host.
- Done: the local form shows the ahpd found (set here, installed by the desk, or on `PATH`) with Change, a file picker from Tauri's dialog plugin; an ahpd older than 0.8.0 is shown with `npm i -g @ahpd/server` and Check again, and Save is off (decision 9).
- Done: with none found, the backends (Claude on, cofold, pi, Bots) and Install and start, which streams npm's and ahpd's lines into a box; a failed install keeps the form in install mode, so a retry installs the backends again.
- Done: Save and start saves the host as `local`, starts its ahpd, and in the first run opens it; the first run's This computer is on.
- Done: the form fits the window at its smallest; the shell's scrollbars are thin, with no arrows, on the window's own background, as ahpw's.
- Found and fixed: Tauri gives the resource folder as a `\?\` path, through which Node cannot load a script (EISDIR on the drive); the desk strips the prefix from a drive path.
- Verified on the debug desk over CDP: found on `PATH` (0.11.0), Save and start, ahpw connected on the local host; a fake 0.7.0 first on `PATH`, the note in en, pt-BR and es; none found, the install of `@ahpd/server` with the desk's Node into its prefix, then `ahpd plugin install` failing on 0.11.0 as task 03 records.
- Done (decision 12): a found ahpd shows the backends too, those it has ready ticked and fixed; those ticked here are added with its own `plugin install` before the start. Checked on ahpd main 69ed7790+daemon/24 (6768d19f) first on `PATH` with a config naming no backend: Claude was added, ahpd started, and ahpw connected.
- Done: a failed start shows only the lines it added to `daemon.log`, in a box under the error; "Back to the list" goes to the list, in the first run too.
- Left: the folder to serve (step 4), which waits for the ahpd command of decision 11; a whole install, which waits for the ahpd release that fixes `plugin install` on Windows.
