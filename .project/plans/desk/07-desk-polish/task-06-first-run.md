---
title: The first run, for a desk with no host
status: done
depends: [task-05-languages.md]
layer: "packages/shell"
---

## Objective

With no host, the host list opens on a welcome, the choice of where the host is, the host form with Test, and Open.

## Files

- `CREATE: packages/shell/src/FirstRun.tsx` - the steps.
- `UPDATE: packages/shell/src/App.tsx` - the first run when the list is empty.
- `UPDATE: packages/shell/src/hosts/HostForm.tsx` - reused in the first run, with Back and "Save and open".
- `UPDATE: packages/shell/src/hosts/HostList.tsx` - the empty state goes; with no host the desk is on the first run.

## Steps

1. Welcome, with ahpw's mark and one button.
2. Where is the host: "On the internet or my network", and "This computer", disabled, marked coming soon.
3. The form with Test; Save and open.

## Validation

- With an empty `hosts.json`, the first run ends in an ahpw window on the new host.

## Resume

- Done 2026-10-13. The first run holds until the new host opens, so the list arriving after the save does not cut it short; a failed open stays on it with Retry and "Host list". Removing the last host starts it again.
- Checked in task 09 in each language.
