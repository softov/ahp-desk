---
title: Two hosts on the desk keep their own pages
status: done
depends: [task-03-release.md]
layer: "ahp-desk, ahpw"
---

## Objective

A release build on 0.3.1 shows each host its own tabs and layout, with one theme.

## Files

- None.

## Steps

1. Remove the unscoped `ahpd-web.session.v1`, `ahpd-web.layout.v1` and `ahpd-web.folders-added` from the desk profile.
2. With two throwaway hosts: open a tab on one, open the other, and see none of the first's tabs; switch back and see it again.

## Validation

- Every item of the plan's checklist.

## Resume

- Done 2026-10-13 on a release build of `@ahpd/web` 0.3.1 from npm; see [implemented.md](implemented.md).
