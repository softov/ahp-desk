---
title: Update on an SSH-installed host's row installs the newer ahpd over SSH
status: done
depends: [task-02-host-row-notice-and-update.md]
layer: "src-tauri, packages/shell"
refs:
  - file:///github/ahp-desk/.project/plans/desk/10-ssh-install/plan.md - plan 10: the SSH session, the install into `~/.local/share/ahpd`, the user unit `ahpd.service`, and its task 08, the update over SSH this task calls
  - "[code://src-tauri/src/lib.rs#L71-L87](../../../../src-tauri/src/lib.rs#L71-L87) - the command list"
  - "[code://src-tauri/build.rs#L237-L255](../../../../src-tauri/build.rs#L237-L255) - each command is declared"
  - "[code://src-tauri/capabilities/shell.json](../../../../src-tauri/capabilities/shell.json) - the host list window's permissions"
  - "[code://src-tauri/src/local_ahpd.rs#L766-L795](../../../../src-tauri/src/local_ahpd.rs#L766-L795) - `InstallEvent` and the install streamed over a Tauri `Channel`, the pattern for the update's lines"
  - "[code://packages/shell/src/hosts/ipc.ts](../../../../packages/shell/src/hosts/ipc.ts) - `InstallEvent` and the install calls"
  - "[code://packages/shell/src/hosts/status.ts](../../../../packages/shell/src/hosts/status.ts) - `updateWay` and `updateHost`, from task 02"
---

## Objective

For a host the desk installed over SSH (plan 10), the row's Update signs in over SSH, installs the newer `@ahpd/server` into that host's install, restarts its user service and waits for it to answer (Answers: the host's ahpd update).
It waits for plan 10 task 08, which writes the update over SSH as a function; this task makes it a command and puts it behind the row's Update.

## Files

- `UPDATE: src-tauri/src/updates.rs` - `#[tauri::command] update_ssh_ahpd`.
- `UPDATE: src-tauri/src/lib.rs:71-87` - `updates::update_ssh_ahpd` in `generate_handler!`.
- `UPDATE: src-tauri/build.rs:238-254` - `"update_ssh_ahpd"`.
- `UPDATE: src-tauri/capabilities/shell.json` - `"allow-update-ssh-ahpd"`.
- `UPDATE: packages/shell/src/hosts/ipc.ts` - `updateSshAhpd(id, version, onEvent)`.
- `UPDATE: packages/shell/src/hosts/status.ts` - `updateWay(id)` answers `{ kind: 'ssh' }` for a host with SSH details, and `updateHost` calls `updateSshAhpd` for it.

## Steps

1. Wait for plan 10 task 08 and read the update function it adds to `src-tauri/src/ssh_install.rs`, and the SSH details plan 10 keeps on a host in `hosts.json`; the names below follow them.
2. `update_ssh_ahpd(app: AppHandle, id: String, version: String, events: Channel<InstallEvent>) -> Result<(), String>`, async:
   - reads the host from `HostStore` and refuses one with no SSH details;
   - signs in as plan 10 does (the agent, a key file, or the password in the keyring; the confirmed server key);
   - runs plan 10 task 08's update with `@ahpd/server@<version>`, the version the row named (Answers: the version the row named): `npm install` into the host's `~/.local/share/ahpd` with its own Node, `systemctl --user restart ahpd`, and the health wait;
   - sends each line npm and systemctl write as `InstallEvent::Line`, as `install_local_ahpd` does;
   - keeps running when the host list window goes away, as an install does.
3. `ipc.ts`:

   ```ts
   /** Updates an SSH-installed host's ahpd to `version`, calling `onEvent` with each line it writes. */
   export function updateSshAhpd(id: string, version: string, onEvent: (event: InstallEvent) => void): Promise<void> {
     const events = new Channel<InstallEvent>();
     events.onmessage = onEvent;
     return invoke('update_ssh_ahpd', { id, version, events });
   }
   ```

4. `status.ts`: `updateWay(id)` answers `{ kind: 'ssh' }` for a host with SSH details before it looks at ahpd's command, since the Answers row on the host's ahpd update updates an SSH-installed host over SSH; `updateHost` keeps the last lines and, on a failure, shows them with `hosts.updateFailed` (task 04).
5. A tunnelled host (plan 10, kind `ssh`) loses its tunnel for the restart; the row reconnects as after any restart.

## Validation

- A unit test in `updates.rs`: `update_ssh_ahpd` on a host with no SSH details is refused with why, before any connection.
- `pnpm typecheck`, `pnpm test` and `pnpm lint` pass.
- By hand on the test machine (never a host in use), with an older `@ahpd/server` installed there by the desk over SSH: the row shows Update; a click installs the newer one, the service restarts, the row comes back connected on the newer version with no notice, by tunnel and by open port.

## Resume

- 2026-10-18: `updates.rs` has the command `update_ssh_ahpd(id, version, events)`, which runs plan 10 task 08's `ssh_install::update_host` under `InstallControl` (one at a time with the installs over SSH, whose Cancel stops it), registered in `lib.rs`, `build.rs` and `capabilities/shell.json`; `InstallControl::cancelled()` gives it the cancel flag.
- `update_host` refuses a host with no `ssh` details, or whose ahpd the desk did not install, before any connection; its tests from plan 10 task 08 are the refusal this task's Validation names, since the command only adds the window's channel.
- `updateWay(host)` answers `{ kind: 'ssh' }` for a host whose `ssh.byDesk` is true, not every host with SSH details: plan 10's update only works on the desk's own install, and an ahpd found on the server updates through ahpd (task 04).
- `updateHost` calls `updateSshAhpd(id, version, onEvent)` (`ipc.ts`), keeps the last five lines, and on a failure throws `hosts.updateFailed` with them; then it reconnects the host, so the row takes the new ahpd at once; the list's error line now shows a failure with `ErrorText`, its first line and the rest in a box.
- The row's notice moved from under the name to a line under the whole row (`.host .row-note`, from the name's column to the end), since in the name's column "ahpd X available" with Update was cut even at 640 px; the words stay those of the Answers row.
- Verified: `cargo test` (135), `cargo clippy --all-targets -- -D warnings`, `pnpm typecheck`, `pnpm test` (42).
- By hand on a release build with a scratch `AHP_DESK_CONFIG_DIR`, against the test machine Softov named: a host set up through the desk's own commands from the page (trust, probe, `install_ssh_ahpd`, `save_host`, `open_ssh_tunnel`), with the server's `@ahpd/server` then put back to 0.11.0 by hand: the row said "ahpd 0.12.0 available" with Update; a click said "Updating ahpd…", and the row came back connected on 0.12.0 with no notice, with the same token, port and local port; by tunnel, and by open port on 9287.
  The line fit at 640x400 and 480x300.
- Not seen by hand: a failed update's message; the confirmation over running chats, since no chat was running.
- The test machine was left as before, with its installs removed and the `~/.config/ahpd` they made; no entry of the run is left in the Credential Manager.
