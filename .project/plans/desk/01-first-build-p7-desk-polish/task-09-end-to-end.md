---
title: The polish end to end
status: done
depends: [task-01-host-list-frame.md, task-02-host-list-palette.md, task-03-icon.md, task-04-footer.md, task-05-languages.md, task-06-first-run.md, task-07-open-hides.md, task-08-switcher-manages-hosts.md]
layer: "ahp-desk, ahpw"
---

## Objective

A release build shows every item of this plan working.

## Files

- None.

## Steps

1. Run a release build with an empty `hosts.json` copy: the first run, in each language.
2. With hosts: the palette beside ahpw, dark and light; Open, Manage hosts, + Add host.

## Validation

- Every item of the plan's checklist.

## Resume

- Done 2026-10-13, on a release build of `@ahpd/web` 0.3.0 against a throwaway ahpd, with `AHP_DESK_CONFIG_DIR` on host lists in the session scratchpad. Every checklist item passes; the runs are in [implemented.md](implemented.md).
- Found and fixed: the first run's autofocus drew the browser's dark focus ring, now an accent ring on keyboard focus only; the installer showed NSIS's icon.
- Found, not fixed here: on the desk every host shares ahpw's origin, so ahpw's kept tabs, layout and added folders cross from one host to another (a tab from another host reads "This session is gone").
