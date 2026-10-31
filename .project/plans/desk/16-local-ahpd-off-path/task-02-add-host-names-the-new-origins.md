---
title: Add host names an ahpd found running or in npm's global folder
status: todo
depends: [task-01-find-through-the-record-and-npms-prefix.md]
layer: "packages/shell"
refs:
  - "[code://packages/shell/src/hosts/ipc.ts#L248-L257](../../../../packages/shell/src/hosts/ipc.ts#L248-L257) - `FoundAhpd.origin`"
  - "[code://packages/shell/src/hosts/LocalForm.tsx#L40-L44](../../../../packages/shell/src/hosts/LocalForm.tsx#L40-L44) - `foundLine`"
  - "[code://packages/shell/src/i18n.ts#L181-L185](../../../../packages/shell/src/i18n.ts#L181-L185) - `local.found*` in `en`; `ptBR` at 386 and `es` at 589"
---

## Objective

Add host shows an ahpd found through its record as running, and one found through npm's prefix as in npm's global folder, and treats both as one on `PATH` (decision 2).

## Files

- `UPDATE: packages/shell/src/hosts/ipc.ts:248-251` - `origin: 'set' | 'installed' | 'path' | 'running' | 'npm'`, and its doc comment.
- `UPDATE: packages/shell/src/hosts/LocalForm.tsx:40-44` - `foundLine`'s `words` gains `running: 'local.foundRunning'` and `npm: 'local.foundNpm'`.
- `UPDATE: packages/shell/src/i18n.ts` - the two keys in `en`, `ptBR` and `es`.

## Steps

1. `en`: `'local.foundRunning': 'ahpd {version}, running'` and `'local.foundNpm': 'ahpd {version}, in npm\'s global folder'`.
2. `ptBR`: `'ahpd {version}, em execução'` and `'ahpd {version}, na pasta global do npm'`.
3. `es`: `'ahpd {version}, en ejecución'` and `'ahpd {version}, en la carpeta global de npm'`.
4. Check that every other place reading `origin` (`rg "origin" packages/shell/src/hosts`) treats `running` and `npm` as it treats `path`; in particular nothing offers an install or the desk's update for them.

## Validation

- `pnpm typecheck` and `pnpm test` pass; the i18n test that every locale has every key passes with the two new keys.

## Resume

