---
title: The host list in English, Portuguese (Brasil) and Spanish
status: done
depends: [task-02-host-list-palette.md]
layer: "packages/shell"
---

## Objective

Every word of the host list comes from `i18n.ts` in the chosen language.

## Files

- `CREATE: packages/shell/src/i18n.ts` - `Lang`, the dictionaries `en`, `pt-BR`, `es`, `t(key, vars)`, `language()` from the OS or the pick, and `setLanguage()`.
- `CREATE: packages/shell/src/i18n.test.ts` - the three dictionaries have the same keys and placeholders.
- `CREATE: packages/shell/src/useOutsideClose.ts` - the close-on-outside-press the row menu and the picker share.
- `UPDATE: packages/shell/src/**/*.tsx` - words through `t()`; a language picker in the title bar.

## Steps

1. Move every word to the English dictionary.
2. Translate into Portuguese (Brasil) and Spanish.
3. Add the picker; keep the pick in `localStorage`.

## Validation

- Each language shows on every screen; the test passes.

## Resume

- Done 2026-10-13. `t()` reads the language on first use, so the dictionaries load in a test; `useLanguage()` in `App` redraws the window on a pick. The picker is `EN`, `PT` or `ES` beside the window buttons, each language named in itself. The host status errors are translated; the errors Rust returns stay English.
- Checked in a release build: the list, the edit form, Test and the remove question in each language.
