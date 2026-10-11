---
title: ahpw draws the window's frame and title bar on the desk
status: done
depends: [task-02-ahpw-depends-on-desk-client.md, task-04-shell-publishes-statuses.md, task-05-app-window-frame.md]
layer: "ahpw"
refs:
  - file:///github/ahpw/src/shell.tsx - the titlebar mounts at lines 111-149
  - file:///github/ahpw/src/app.css - ahpw's own styles
  - file:///github/opendoop/packages/web/src/app/globals.css - pood's rounded root at lines 35-48
  - file:///github/opendoop/packages/web/src/components/shell/TitleBar.tsx - pood's window controls at lines 468-540
  - file:///github/ahp-desk/mockups/03-app-window.md - the host switcher in the title bar
---

## Objective

On the desk, ahpw's root is rounded when the window is restored and square when maximized.
The title bar drags the window, maximizes it on a double click, and ends in minimize, maximize or restore, and close.
A host switcher after the title lists every host with its status dot, and moves the window to another.

## Files

- `CREATE: ahpw/src/desk/frame.ts` - `startFrame()`: the `is-desk` class on `<html>`, and `is-maximized` kept in step with `isMaximized()` and `onResized()`; the drag and double click on `[data-surface="titlebar"]`.
- `CREATE: ahpw/src/desk/WindowControls.tsx` - the three buttons over `windowControls`.
- `CREATE: ahpw/src/desk/HostSwitcher.tsx` - the window's host name with a menu of `hosts()`, each with its dot from `hostStatuses()` and `onHostStatus()`; a pick calls `openHost(id, { replace: true })`.
- `CREATE: ahpw/src/desk/desk.css` - `html.is-desk` transparent; the root rounded by 10 px with a 1 px border in the theme's line colour; square under `is-maximized`; the controls' look.
- `UPDATE: ahpw/src/shell.tsx` - when `onDesk`, mount `HostSwitcher` in the left slot after the title, and `WindowControls` last in the right slot.
- `UPDATE: ahpw/src/main.tsx` - when `onDesk`, import `desk/desk.css` and call `startFrame()`.

## Steps

1. In `startFrame()`, add `is-desk` to `<html>`, set `is-maximized` from `isMaximized()`, and update it on each `onResized`.
2. On `pointerdown` with the primary button on the title bar, when the target is not inside a button, input, select or link, call `windowControls.startDragging()`.
3. On `dblclick` on the same area, call `windowControls.toggleMaximize()`.
4. Draw `WindowControls` as three buttons, 46 px wide and the bar's height, with the close button red on hover, as pood's.
5. Draw `HostSwitcher`: the host's name and a chevron; its menu lists each host with a dot, the current one marked; a pick calls `openHost(id, { replace: true })`.
6. Give the rounded root `overflow: hidden`, so no child paints past the corner.

## Validation

- Restored, the window shows round corners and no green edge; maximized, square corners and no gap.
- A drag on the empty title bar moves the window; on a button it does not.
- Minimize, maximize, restore and close work.
- The switcher lists every host with the right dot, and a pick shows the other host in the same window.
- Check at 100% and 150% display scale.

## Resume

- Done 2026-10-11. Rounded 10 px root with the theme's border, square when maximized; a double click on the empty bar maximizes, Restore restores; the switcher lists both hosts with their dots and moves the window to the other host and back. `src/desk.ts` became `src/desk/index.ts` beside the new files.
- Not checked: a drag with a real mouse, and 150% display scale.
