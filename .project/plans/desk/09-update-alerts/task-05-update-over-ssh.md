---
title: Update on an SSH-installed host's row installs the newer ahpd over SSH
status: todo
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
- By hand on dev02 (never dev01 or a host in use), with an older `@ahpd/server` installed there by the desk over SSH: the row shows Update; a click installs the newer one, the service restarts, the row comes back connected on the newer version with no notice, by tunnel and by open port.

## Resume
