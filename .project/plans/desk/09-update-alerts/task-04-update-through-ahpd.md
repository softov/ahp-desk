---
title: Update on a host's row runs ahpd's own update command
status: done
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
- 2026-10-20: built on ahpd 0.13.0, whose `x-ahpd/update` has the method and params step 2 names.
- `ahpd-update.ts` has `UPDATE_METHOD`, `UpdateParams`, `mayUpdate`, `requestUpdate`, and `checkUpdate(client, version)`, which sends `{ version, check: true }` and is false on any error, so a refusal (`-32009`) or an ahpd without the command (`-32601`) hides Update.
  It takes a `Requester` (`request(method, params)`) instead of `AhpClient`, since `MultiHostClient`'s handle is what `status.ts` holds; the cast to it is in `status.ts`, once.
- The check names the row's version: ahpd 0.13.0 asks npm for `latest` only when no `version` is given, and that ask sends the scoped name with a bare `/`, which a registry such as verdaccio answers 404 (task 03's Resume).
- `status.ts`: `updateWay(host, version)` takes the row's version; it keeps each host's answer per connection (keyed on `rawClient()`) and per version, so a redraw asks nothing, and forgets it on removal.
  Step 4's `updateThroughAhpd` is `updateHost`'s branch for a way of `ahpd`: it sends `{ version, force }`, shows an RPC error before the drop as `hosts.updateFailed` (already there from task 05), and then waits up to 3 minutes for the host to come back connected on that version (`backOn`), so the row's "Updating ahpd…" lasts through the install and the restart.
- `HostList.tsx`: while an update runs, the row keeps "Updating ahpd…" and shows no Retry through the drop, for this way and task 05's.
- Plan 03 task 07 uses the same way for the local ahpd, only when the desk installed it.
- Verified: `pnpm typecheck`, `pnpm test` (53, with `ahpd-update.test.ts`), `cargo test` (137), `cargo clippy --all-targets -- -D warnings` through `pnpm lint`.
- By hand on a Windows 10 VM, on a release NSIS build of this branch, with a local npm registry holding repacks of 0.13.0 as 0.13.1 to 0.13.3 and `npm_config_registry` set: the local host the desk installed, with automatic update off, showed "ahpd 0.13.3 available" with Update; a click made the row say "Updating ahpd…" with no Retry while ahpd was down, and the row came back green on 0.13.3 with no notice, about 70 seconds later.
- Not seen by hand: a URL host with the deployment token, a URL host with a person's token, an ahpd before 0.13.0, and the question over a running turn, since no backend was signed in.
  A URL host goes through the same `checkUpdate` and `requestUpdate` as the local host above; the refusals are in `ahpd-update.test.ts`.

