---
title: The desk downloads its own update, says so in the host list and the tray, and installs it on a click
status: todo
depends: [task-01-npm-latest-ahpd.md]
layer: "src-tauri, packages/shell"
refs:
  - file:///github/ahp-desk/.project/plans/desk/01-first-build-p4-tray-and-releases/plan.md - p4: task 01 makes the tray and its menu, task 03 adds the updater with its endpoint and key
  - "[code://src-tauri/tauri.conf.json](../../../../src-tauri/tauri.conf.json) - the updater's endpoint and public key, from p4 task 03"
  - "[code://src-tauri/src/lib.rs#L20-L95](../../../../src-tauri/src/lib.rs#L20-L95) - setup, the command list, and `RunEvent::Exit`, which stops the local ahpd on quit and so on a restart too"
  - "[code://src-tauri/build.rs#L237-L255](../../../../src-tauri/build.rs#L237-L255) - each command is declared"
  - "[code://src-tauri/capabilities/shell.json](../../../../src-tauri/capabilities/shell.json) - the host list window's permissions"
  - "[code://packages/shell/src/hosts/HostList.tsx#L74-L86](../../../../packages/shell/src/hosts/HostList.tsx#L74-L86) - the list's head and its error line, where the notice goes"
  - "[code://packages/shell/src/app.css#L605-L617](../../../../packages/shell/src/app.css#L605-L617) - `.result`, the list's message line"
  - "[code://packages/shell/src/i18n.ts](../../../../packages/shell/src/i18n.ts) - the words in the three languages"
---

## Objective

At start and every 6 hours the desk asks its updater for a newer signed release; when there is one it downloads it in the background, then says "ahp-desk X is ready. Restart to update." in the host list and in the tray (decision 1).
It installs and restarts only when that notice is clicked; until then nothing changes, however long it waits.
It waits for p4 task 03 (the updater) and p4 task 01 (the tray).

## Files

- `UPDATE: src-tauri/src/updates.rs` - the desk's own update: the check, the download kept in memory, `DeskUpdate` in `UpdatesView`, and `install_desk_update`.
- `UPDATE: src-tauri/src/lib.rs:71-87` - `updates::install_desk_update` in `generate_handler!`.
- `UPDATE: src-tauri/build.rs:238-254` - `"install_desk_update"`.
- `UPDATE: src-tauri/capabilities/shell.json` - `"allow-install-desk-update"`.
- `UPDATE: src-tauri/src/tray.rs` (made by p4 task 01) - the menu item, shown while an update is ready.
- `UPDATE: packages/shell/src/hosts/ipc.ts` - `DeskUpdate` in `Updates`, and `installDeskUpdate()`.
- `UPDATE: packages/shell/src/hosts/HostList.tsx:74-86` - the notice line under the head, with Restart.
- `UPDATE: packages/shell/src/App.tsx` - pass `desk` from `updates()` to `HostList`, beside task 02's `latestAhpd`.
- `UPDATE: packages/shell/src/i18n.ts` - `desk.updateReady` and `desk.restart`, in `en`, `ptBR` and `es`.

## Steps

1. If p4 task 03 already checks at start, this task takes that check over: the updater is only ever run from `updates.rs`.
2. `updates.rs`:

   ```rust
   /// A desk release downloaded and ready to install.
   #[derive(Clone, Debug, PartialEq, Serialize)]
   #[serde(rename_all = "camelCase")]
   pub struct DeskUpdate { pub version: String }
   // UpdatesView gains: pub desk: Option<DeskUpdate>
   ```

   - `Updates` keeps `pending: Mutex<Option<(tauri_plugin_updater::Update, Vec<u8>)>>`, in memory only (plan: nothing on disk but the npm answer).
   - The loop of task 01 also runs `app.updater()?.check().await`; on `Some(update)` that is not the one already pending, `update.download(..).await`, keeps it, sets `desk`, and emits `UPDATES_CHANGED`.
     A failed check or download is logged and asked again on the next round.
   - `#[tauri::command] pub async fn install_desk_update(app: AppHandle, state: State<'_, Updates>) -> Result<(), String>` takes the pending update, runs `update.install(bytes)`, then `app.restart()`; with none pending it is refused with why.
3. The tray (p4 task 01): while `desk` is set, its menu has the item "ahp-desk X is ready. Restart to update." first, and a click calls the same install as the command; the item's words come in the desk's language the same way as the tray's other items.
   The form of the tray notice is plan Resume state, open question 5.
4. `HostList.tsx`: while `desk !== null`, a line under the head, `<p className="result ok">`, says `t('desk.updateReady', { version })` with a `t('desk.restart')` button calling `installDeskUpdate()`; a failure shows in the error line.
5. Words:
   - `desk.updateReady`: "ahp-desk {version} is ready." / "O ahp-desk {version} está pronto." / "ahp-desk {version} está listo.";
   - `desk.restart`: "Restart to update" / "Reiniciar para atualizar" / "Reiniciar para actualizar".

## Validation

- A unit test in `updates.rs`: `install_desk_update` with nothing pending is refused; `UpdatesView` serializes `desk` as `{ "version": "0.2.0" }` and `null`.
- `i18n.test.ts` passes with the new keys; `pnpm typecheck`, `pnpm test` and `pnpm lint` pass.
- By hand, on Windows with an installed older signed release and a newer one published (the pair p4 task 06 makes): the notice appears in the host list and in the tray after the download; the desk keeps running and installs nothing over a day left alone; a click on either installs, the desk restarts on the newer version, and the notice is gone.

## Resume
