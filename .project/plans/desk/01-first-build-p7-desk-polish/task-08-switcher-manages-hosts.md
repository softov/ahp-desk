---
title: ahpw's host switcher manages hosts
status: todo
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

- Not started.
