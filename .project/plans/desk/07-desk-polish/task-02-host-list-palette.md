---
title: The host list in ahpw's palette
status: done
depends: [task-01-host-list-frame.md]
layer: "packages/shell"
---

## Objective

The host list uses ahpw's colours, dark and light, with ahpw's button, hover and dot styles.

## Files

- `UPDATE: packages/shell/src/app.css` - ahpw's `--oo-rgb-*` and `--oo-color-*` values for light and dark; every rule on them; buttons as ahpw's (plain: border and hover; primary: accent outline, filled on hover); dots as ahpw's `.web-connection__dot`; the rounded root as ahpw's `embed.css`.

## Steps

1. Copy the light values from scena's `tokens.css` and the dark from `dark.css`, naming the source.
2. Replace every colour in `app.css` with a token.
3. Draw the rows without borders, rounded on hover, as ahpw's lists.
4. Compare the host list and an ahpw window side by side at the pixel.

## Validation

- Same background, text, borders, buttons, hovers and dots as ahpw, dark and light.

## Resume

- Done 2026-10-11, dark checked in a release build: canvas, text, border, primary button, row hover, dots and menu as ahpw's. Light not checked yet; task 09 does.
