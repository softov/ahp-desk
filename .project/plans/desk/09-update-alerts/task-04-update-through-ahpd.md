---
title: Update on a host's row runs ahpd's own update command
status: blocked
depends: [task-02-host-row-notice-and-update.md, task-03-ahpd-update-command.md]
layer: "packages/shell"
refs:
  - "[code://packages/shell/src/hosts/status.ts#L75-L161](../../../../packages/shell/src/hosts/status.ts#L75-L161) - the one AHP connection per host, through `MultiHostClient`, which the command is sent on"
  - "[code://packages/shell/src/hosts/HostList.tsx](../../../../packages/shell/src/hosts/HostList.tsx) - the row's Update, from task 02"
  - file:///github/ahpd/packages/server/src/commands/scopes.ts - the refusal a person's token gets, which hides Update
---

## Objective

For a host the desk did not install over SSH, the row's Update runs ahpd's `x-ahpd/update` (task 03) over the host's open AHP connection (Answers: x-ahpd/update), and ahpd installs the version the row named and restarts itself (Answers: the host's ahpd update, and the version the row named).
Update shows only when ahpd says, with `check`, that this token may update it; a host whose ahpd has no such command, or refuses the token, keeps the notice without the button (Answers: the check).

## Files

- `UPDATE: packages/shell/src/hosts/status.ts` - `updateWay(id)` answers `{ kind: 'ahpd' }` from ahpd's check, and `updateThroughAhpd(id, version, force)`.
- `CREATE: packages/shell/src/hosts/ahpd-update.ts` - the request and the reading of its answer, apart from the client so they can be tested.
- `CREATE: packages/shell/src/hosts/ahpd-update.test.ts` - its cases.
- `UPDATE: packages/shell/src/i18n.ts` - `hosts.updateFailed` in `en`, `ptBR` and `es`.

## Steps

1. Wait for the ahpd release with task 03's command, and write its version in this file's Resume; the method name and params here follow what that release has.
2. `ahpd-update.ts`:

   ```ts
   /** The AHP method of ahpd's update command (task 03). */
   export const UPDATE_METHOD = 'x-ahpd/update';
   export interface UpdateParams { version?: string; check?: boolean; force?: boolean }
   /** Whether ahpd's answer to a check lets this token update it. A refusal or an unknown method is false. */
   export function mayUpdate(answer: unknown): boolean;
   /** Sends the command on `client`; the typed `request` knows only the protocol's own methods, so the name is cast here, once. */
   export function requestUpdate(client: AhpClient, params: UpdateParams): Promise<unknown>;
   ```

3. In `status.ts`, `updateWay(id)` for a host with no SSH details (task 05 answers for the others):
   - asks `requestUpdate(client, { check: true })` once per connection, through `multi.client(id)?.rawClient()`, only when task 02 shows the notice for that host;
   - keeps the answer until the host reconnects, so a row redraw asks nothing;
   - answers `{ kind: 'ahpd' }` when `mayUpdate(answer)`, else `null`.
4. `updateThroughAhpd(id, version, force)` sends `requestUpdate(client, { version, force })` with the version the row named (Answers: the version the row named).
   The connection dropping after the answer is the restart, not a failure; an RPC error before it is shown with `t('hosts.updateFailed', { error })`.
5. `hosts.updateFailed`: "ahpd could not update: {error}" / "O ahpd não conseguiu atualizar: {error}" / "ahpd no pudo actualizarse: {error}".

## Validation

- `ahpd-update.test.ts`: `mayUpdate` on `{ may: true }`, `{ may: false }`, a 403 refusal, a `-32601` method not found, and a malformed answer; `requestUpdate` sends `x-ahpd/update` with the params given, through a fake `AhpClient`.
- `pnpm typecheck` and `pnpm test` pass.
- By hand, with a local npm registry holding two ahpd releases with the command (`npm_config_registry` set for the desk and for ahpd), on `pnpm tauri dev` with `AHP_DESK_CONFIG_DIR` on a scratch folder:
  - a URL host on the older release with the deployment token shows Update; a click restarts it, the row goes "Updating…", then back connected on the newer version with no notice;
  - with a session running a turn, Update asks first and names it;
  - the same host with a person's token shows the notice without Update;
  - an ahpd release without the command shows the notice without Update.

## Resume

- Blocked 2026-10-16: it waits for ahpd's own update command (task 03), which Softov looks at in the ahpd repository: "I will see on ahpd a self update command".
