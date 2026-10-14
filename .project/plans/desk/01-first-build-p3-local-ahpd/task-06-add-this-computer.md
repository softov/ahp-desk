---
title: Add host has "This computer", with the install steps
status: todo
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

- `UPDATE: packages/shell/src/hosts/HostForm.tsx` - the kind, the found ahpd with Change, the backends.
- `CREATE: packages/shell/src/hosts/InstallSteps.tsx` - npm's lines, and the result.
- `UPDATE: packages/shell/src/FirstRun.tsx` - "This computer" in the where step.
- `UPDATE: packages/shell/src/i18n.ts` - the words, in en, pt-BR and es.

## Steps

1. `@ahpd/agent-claude` is on by default, and `@ahpd/bot` is offered.
2. The host is saved as `kind: local` once ahpd is on.
3. An ahpd found older than 0.8.0 (`supported: false`) is shown with its version, the version needed, `npm i -g @ahpd/server` and Check again, and nothing is installed (decision 9).

## Validation

- Shell tests for the form; a CDP check of the form and the steps in the three languages.

## Resume
