---
title: ahpw skips its sign-in wall on the desk
status: done
depends: [task-02-ahpw-depends-on-desk-client.md]
layer: "ahpw"
refs:
  - file:///github/ahpw/src/token-provider.ts - `restoreSession()` at lines 82-93
  - file:///github/ahpw/src/shell.tsx - the `chrome:sign-out` mount
---

## Objective

On the desk, ahpw opens signed in, keeps no token in localStorage, and shows no Sign out.

## Files

- `UPDATE: ahpw/src/token-provider.ts:82-93` - a desk branch at the top of `restoreSession()`.
- `UPDATE: ahpw/src/shell.tsx` - mount `chrome:sign-out` only when not `onDesk`.

## Steps

1. When `onDesk`, `restoreSession()` returns `{ userId: 'desk', displayName: <the window host's name>, permissions: [SIGNED_IN] }` without reading `ahpw.json` or localStorage.
2. When `onDesk`, leave out the Sign out button; the desk holds the token.

## Validation

- Open a host in the desk; no sign-in wall shows.
- In a browser through `ahpw serve`, the wall shows as before.

## Resume

- Done 2026-10-11. No wall and no Sign out on the desk; the wall shows in a browser.
