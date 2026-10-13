---
title: Two hosts on the desk keep their own pages
status: todo
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
