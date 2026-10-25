---
title: A local ahpd the desk did not install can be updated from its row, or its row says how
status: todo
depends: []
layer: "packages/shell"
refs:
  - "[code://packages/shell/src/hosts/status.ts#L242-L248](../../../../packages/shell/src/hosts/status.ts#L242-L248) - `updateWay`"
  - "[code://packages/shell/src/hosts/status.ts#L179](../../../../packages/shell/src/hosts/status.ts#L179) - `installedByDesk`"
  - "[code://packages/shell/src/hosts/HostList.tsx#L436-L455](../../../../packages/shell/src/hosts/HostList.tsx#L436-L455) - the notice and Update"
  - "[code://packages/shell/src/i18n.ts#L58](../../../../packages/shell/src/i18n.ts#L58) - `hosts.ahpdAvailable` in the three dictionaries"
---

## Objective

A local host on an ahpd the desk did not install shows Update when ahpd's check lets its token update it, and otherwise the words "Update it where it was installed: npm i -g @ahpd/server" (decision 2).

## Files

- `UPDATE: packages/shell/src/hosts/status.ts:242-248` - `updateWay` drops the line `if (host.kind === 'local' && !(await installedByDesk(host))) return null;`, so such a host goes through `checkOf` as a remote one does; the choice moves into a pure `export function wayFor(host: ShellHost, version: string | null, may: boolean | null): UpdateWay | null`, which `updateWay` calls with the awaited check.
- `UPDATE: packages/shell/src/hosts/status.ts:87` - `HostStatuses` gains `selfUpdate(host: ShellHost): Promise<boolean>`, true for a local host whose ahpd the desk did not install, from `installedByDesk`.
- `UPDATE: packages/shell/src/hosts/HostList.tsx:436-455` - when `ways[host.id]` is `null` and the host is such a local one, the note shows `t('hosts.ahpdSelfUpdate')` after the available words.
- `UPDATE: packages/shell/src/i18n.ts` - `hosts.ahpdSelfUpdate`: "Update it where it was installed: npm i -g @ahpd/server"; Portuguese "Atualize onde foi instalado: npm i -g @ahpd/server"; Spanish "Actualízalo donde se instaló: npm i -g @ahpd/server".
- `UPDATE: packages/shell/src/hosts/ahpd-update.test.ts` - a `describe('wayFor')` with the cases below.

## Steps

1. Keep the automatic update for the ahpd the desk installed only: `HostList.tsx`'s auto-update effect already requires `host.onDesk?.autoUpdate === true`, which a host on an ahpd the desk did not install never has; check it stays so.
2. `selfUpdate` is read once per host into a state map beside `ways`, in the same effect, so the row does not ask on each redraw.
3. The words sit in the note's `note-text` span, after "ahpd X available", and carry the whole line as their `title`, as the available words do.

## Validation

- `wayFor` tests: an SSH host the desk installed gets `{ kind: 'ssh' }`; a local host whose check answers true gets `{ kind: 'ahpd' }`, the desk's install or not; a check that answers false or none gets `null`; no version gets `null`.
- `pnpm --filter @ahp-desk/shell test` and `typecheck`; `i18n.test.ts` passes with the key in the three dictionaries.
- By hand: a "This computer" on an ahpd 0.12.0 installed with npm shows the words; after `npm i -g @ahpd/server@0.13.0` and a newer ahpd on npm, it shows Update.

## Resume
