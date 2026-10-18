---
title: A host row says when its ahpd is behind npm, and offers Update
status: done
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
The `local` host is left to its own rule (Answers: the local ahpd's own update, plan 03 task 07) and never shows this notice or this Update.

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

- 2026-10-18: `packages/shell/src/hosts/versions.ts` has `newer(latest, current)`, a port of ahpd's, and `versions.test.ts` runs ahpd's table in its order.
- `status.ts` keeps each host's `ServerInfo`; `connected` carries `ahpd`, the version when the server is ahpd; `HostStatuses` has `running(id)`, `updateWay(host)` and `updateHost(host, version, force)`, with `UpdateWay` and `IN_PROGRESS` (`1 << 3`).
- `updateWay` takes the `ShellHost`, not its id, since task 05's way reads the host's `ssh` details; it answers `null`, and `updateHost` rejects with `hosts.noUpdateWay`, until tasks 04 and 05.
- `running(id)` reads the chats of the host's sessions: in `@microsoft/agent-host-protocol` 1.0.0 a `SessionSummary` has no `status` or `title` of its own, and each `SessionChatSummary` has both; a chat with the `IN_PROGRESS` bit is running, named by its title.
- `App.tsx` reads `updates()` and `onUpdatesChanged()` into `latestAhpd` for `HostList`; the row of a connected non-local host on an ahpd behind it shows `.note` under the name, "ahpd X available", ending in an ellipsis with the whole line as its title when the column is narrow, with an inline Update when `updateWay` answered a way; a click with running chats turns the row into the `.host.confirm` question, and while it runs the line says "Updating ahpd…"; a failure goes to the list's error line.
- Words `hosts.ahpdAvailable`, `hosts.update`, `hosts.updateQuestion`, `hosts.updating` and `hosts.noUpdateWay` in en, pt-BR and es.
- Verified: `pnpm typecheck`, `pnpm test` (42, with 13 for `newer` and `i18n.test.ts` over the new words).
- By hand on a release build with a scratch `AHP_DESK_CONFIG_DIR`, a URL host on ahpd 0.11.0 run from a scratch folder on this machine, and npm's latest 0.12.0 from task 01: the row said "ahpd 0.12.0 available", "disponível" and "disponible", with no Update since no way exists yet; with the kept latest at 0.11.0, the row said nothing.
  The ellipsis at 480x300 was seen with its rules put in the running page; the build after it was not run.
- Not seen by hand: a non-ahpd server and the local host, both left out by the row's condition (`ahpd` is `null` for a server not named ahpd, and `kind !== 'local'`); the confirmation over running chats, which needs a way, so tasks 04 and 05 see it.
