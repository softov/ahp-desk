---
title: ahpw's host switcher manages hosts
status: done
depends: [task-07-open-hides.md]
layer: "ahpw"
---

## Objective

ahpw's switcher menu ends in "Manage hosts" and "+ Add host", which show the desk's host list.

## Files

- `UPDATE: ahpw/src/embed/index.ts` - `manageHosts(options?: { add?: boolean })`.
- `UPDATE: ahpw/src/embed/HostSwitcher.tsx` - a line, then the two items.

## Steps

1. Merge ahpw's `main` into `desk`.
2. Add the two items under a separator.

## Validation

- Each item shows the host list; "+ Add host" opens its Add form.

## Resume

- Done 2026-10-11. ahpw's `desk` was level with `origin/main`, so step 1 had nothing to merge. The menu ends in a line, "Manage hosts" and "Add host"; the embedder sends `show_host_list` with `add`. Embedder tests cover both calls; not clicked in the release build yet (task 09).
