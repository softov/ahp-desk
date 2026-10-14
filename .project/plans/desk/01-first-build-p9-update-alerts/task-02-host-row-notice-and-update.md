---
title: A host row says when its ahpd is behind npm, and offers Update
status: todo
depends: [task-01-npm-latest-ahpd.md]
layer: "packages/shell"
refs:
  - file:///github/ahpd/packages/server/src/update.ts - `newer()` at lines 48-56, the comparison the row uses, and its cases in ahpd's `test/update.test.ts` at lines 21-39
  - file:///github/ahpd/packages/sdk/src/host/handshake.ts - ahpd answers `initialize` with `serverInfo: { name: 'ahpd', version }`, at line 209
  - "[code://packages/shell/src/hosts/status.ts#L31-L61](../../../../packages/shell/src/hosts/status.ts#L31-L61) - `serverInfo` is read from the `initialize` answer and kept as one string"
  - "[code://packages/shell/src/hosts/status.ts#L138-L161](../../../../packages/shell/src/hosts/status.ts#L138-L161) - each host's status, which gains the ahpd version and the running sessions"
  - "[code://packages/shell/src/hosts/HostList.tsx#L87-L140](../../../../packages/shell/src/hosts/HostList.tsx#L87-L140) - the row, and the remove confirmation the update confirmation follows"
  - "[code://packages/shell/src/app.css#L328-L400](../../../../packages/shell/src/app.css#L328-L400) - the row grid and `.name .warn`, the second line under the name"
  - "[code://packages/shell/src/i18n.ts](../../../../packages/shell/src/i18n.ts) - the words in English, Portuguese (Brasil) and Spanish"
  - "[code://packages/shell/src/App.tsx#L35-L52](../../../../packages/shell/src/App.tsx#L35-L52) - where the host list reads its events"
---

## Objective

A connected host whose server is ahpd, and whose version is behind npm's latest from task 01, says "ahpd X available" on its row, in each of the desk's three languages (Answers: the host's ahpd update).
The row has Update when a way to update that host exists (tasks 04 and 05 add the ways); with none, it has the notice only.
Update on a host with running sessions names them and waits for a click; with none, it starts at once.
The `local` host is left to its own rule (Answers: the local ahpd's own update, p3 task 07) and never shows this notice or this Update.

## Files

- `CREATE: packages/shell/src/hosts/versions.ts` - `newer(latest, current)`.
- `CREATE: packages/shell/src/hosts/versions.test.ts` - ahpd's cases for `newer`.
- `UPDATE: packages/shell/src/hosts/status.ts:31-35` - keep the parsed `ServerInfo`, not only its text.
- `UPDATE: packages/shell/src/hosts/status.ts:80` - `servers: Map<string, ServerInfo>`.
- `UPDATE: packages/shell/src/hosts/status.ts:138-161` - `connected` carries `ahpd`; `running(id)` and `updateWay(id)` on `HostStatuses`.
- `UPDATE: packages/shell/src/hosts/HostList.tsx:87-140` - the notice, Update, the confirmation and "Updating…".
- `UPDATE: packages/shell/src/App.tsx:35-52` - read `updates()` and `onUpdatesChanged()`, and pass `latestAhpd` to `HostList`.
- `UPDATE: packages/shell/src/app.css` - `.name .note`, as `.name .warn` in the accent colour, and the inline Update in it.
- `UPDATE: packages/shell/src/i18n.ts` - the keys below, in `en`, `ptBR` and `es`.

## Steps

1. `versions.ts`: `export function newer(latest: string, current: string): boolean`, a port of ahpd's `newer()`, with its `X.Y.Z[-pre]` reader; anything that does not parse is not newer.
2. `status.ts`:
   - `servers` keeps `ServerInfo` per host; `serverName(info)` still makes the text the row shows.
   - `HostStatus` `connected` becomes `{ kind: 'connected'; server: string | null; ahpd: string | null }`, `ahpd` being `serverInfo.version` when `serverInfo.name === 'ahpd'`.
   - `running(id: string): string[]` - the titles of that host's sessions with a turn on, from `multi.aggregatedSessions()` filtered by `hostId`, where `(summary.status & IN_PROGRESS) !== 0`.
     `IN_PROGRESS = 1 << 3` is `SessionStatus.InProgress` of `@microsoft/agent-host-protocol`, written out because `isolatedModules` cannot read an ambient const enum; `InputNeeded` carries the same bit.
   - `updateWay(id: string): Promise<UpdateWay | null>` with `type UpdateWay = { kind: 'ssh' } | { kind: 'ahpd' }`, answering `null` until tasks 04 and 05 fill it.
3. `App.tsx`: `const [latestAhpd, setLatestAhpd] = useState<string | null>(null)` from `updates()` and `onUpdatesChanged()` (task 01), passed as `HostList`'s `latestAhpd`.
4. `HostList.tsx`, for a host with `kind !== 'local'`, `status.kind === 'connected'`, `status.ahpd !== null` and `latestAhpd !== null && newer(latestAhpd, status.ahpd)`:
   - a second line under the name, `<span className="note">`, with `t('hosts.ahpdAvailable', { version: latestAhpd })`;
   - when `updateWay(host.id)` answered a way, an `Update` button in that line;
   - a click with `statuses.running(host.id)` empty starts the update; otherwise the row becomes a confirmation, as the remove one (`.host.confirm`): `t('hosts.updateQuestion', { name: host.name, sessions: running.join(', ') })`, with Cancel and Update;
   - while the update runs the line says `t('hosts.updating')` and the button is gone; a failure shows in the list's error line, as a failed remove does.
5. The update itself is `updateHost(host, latestAhpd, force)` in `status.ts`, which calls the way's function (task 04 or 05); `force` is true when the person confirmed over running sessions.
   `latestAhpd` is the version the row named, and the way installs that one (Answers: the version the row named).
   The row goes back to normal when the host reconnects with the new version, since the notice then has nothing newer to say.
6. Words, in `en`, `ptBR` and `es`, beside the `hosts.*` keys:
   - `hosts.ahpdAvailable`: "ahpd {version} available" / "ahpd {version} disponível" / "ahpd {version} disponible";
   - `hosts.update`: "Update" / "Atualizar" / "Actualizar";
   - `hosts.updateQuestion`: "Update ahpd on {name}? It restarts, and these sessions are running: {sessions}" / "Atualizar o ahpd em {name}? Ele reinicia, e estas sessões estão rodando: {sessions}" / "¿Actualizar ahpd en {name}? Se reinicia, y estas sesiones están en curso: {sessions}";
   - `hosts.updating`: "Updating ahpd…" / "Atualizando o ahpd…" / "Actualizando ahpd…".

## Validation

- `versions.test.ts`: every case of ahpd's `newer` table (`0.6.0` over `0.5.0`, `0.10.0` over `0.9.0`, a release over its prerelease, `unknown`, `1.2`, the empty string), run by `pnpm --filter @ahp-desk/shell test`.
- `i18n.test.ts` passes as it is: every language has the new keys with the same placeholders.
- `pnpm typecheck` and `pnpm test` pass.
- By hand, on `pnpm tauri dev` with `AHP_DESK_CONFIG_DIR` on a scratch folder: a URL host on an ahpd older than npm's latest says "ahpd X available" with no button (no way yet); in Portuguese and Spanish too; an up-to-date ahpd and a non-ahpd server say nothing; the local host says nothing from this task.

## Resume
