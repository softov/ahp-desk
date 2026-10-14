---
title: Add host and the first run offer "A server over SSH", which signs in, confirms the key, installs and connects
status: todo
depends: [task-02-server-key.md, task-05-install.md, task-06-reach.md]
layer: "packages/shell, src-tauri"
refs:
  - "[code://packages/shell/src/hosts/KindField.tsx](../../../../packages/shell/src/hosts/KindField.tsx) - Add host's Kind, which gains a third choice"
  - "[code://packages/shell/src/hosts/LocalForm.tsx#L18-L24](../../../../packages/shell/src/hosts/LocalForm.tsx#L18-L24) - `BACKENDS` and `BOT`, the backends list both forms share"
  - "[code://packages/shell/src/hosts/LocalForm.tsx#L202-L220](../../../../packages/shell/src/hosts/LocalForm.tsx#L202-L220) - the backends checkboxes, moved into a shared field"
  - "[code://packages/shell/src/hosts/LocalForm.tsx#L132-L163](../../../../packages/shell/src/hosts/LocalForm.tsx#L132-L163) - the phases of a save and the failure kept in the form, the pattern the SSH form follows"
  - "[code://packages/shell/src/hosts/InstallSteps.tsx](../../../../packages/shell/src/hosts/InstallSteps.tsx) - the lines box, which gains the steps"
  - "[code://packages/shell/src/FirstRun.tsx#L95-L117](../../../../packages/shell/src/FirstRun.tsx#L95-L117) - the where step, which gains \"A server over SSH\""
  - "[code://packages/shell/src/App.tsx#L72-L90](../../../../packages/shell/src/App.tsx#L72-L90) - where Add host picks its form by kind"
  - "[code://packages/shell/src/hosts/HostForm.tsx](../../../../packages/shell/src/hosts/HostForm.tsx) - Edit host, which shows an SSH host's details"
  - "[code://packages/shell/src/hosts/url.ts](../../../../packages/shell/src/hosts/url.ts) - `hostId`, and where the SSH address is read"
  - "[code://packages/shell/src/i18n.ts](../../../../packages/shell/src/i18n.ts) - the words, in en, pt-BR and es"
  - "[code://src-tauri/src/hosts.rs#L328-L337](../../../../src-tauri/src/hosts.rs#L328-L337) - `save_host`, which keeps the password the person asked to keep"
  - "[code://mockups/02-hosts.md](../../../../mockups/02-hosts.md) - Add host, kind SSH: Host, Port and User on one line, then Key with Browse"
---

## Objective

Add host and the first run have a third kind, "A server over SSH".
Its form takes the server, the user, the sign-in, the backends and the reach, confirms a new server key by its fingerprint, shows what the probe found, installs with its steps and Cancel, saves the host, and connects.

## Files

- `CREATE: packages/shell/src/hosts/SshForm.tsx` - the form and its phases: `form`, `connecting`, `key`, `probed`, `installing`, `saving`.
- `CREATE: packages/shell/src/hosts/ServerKey.tsx` - the fingerprint to confirm, and the refusal of a changed key with both fingerprints.
- `CREATE: packages/shell/src/hosts/BackendsField.tsx` - `BACKENDS`, `BOT` and the checkboxes, moved from `LocalForm.tsx`.
- `UPDATE: packages/shell/src/hosts/LocalForm.tsx` - uses `BackendsField`; nothing else changes.
- `UPDATE: packages/shell/src/hosts/KindField.tsx` - `AddKind` gains `'ssh'`, a third segment.
- `UPDATE: packages/shell/src/hosts/InstallSteps.tsx` - takes `steps` too: each step's words by its id, with running, done, failed or skipped, above the lines box.
- `UPDATE: packages/shell/src/App.tsx` - Add host with kind `ssh` shows `SshForm`.
- `UPDATE: packages/shell/src/FirstRun.tsx` - a third choice and a step `ssh`.
- `UPDATE: packages/shell/src/hosts/HostForm.tsx` - Edit an SSH host: its name; its host, port, user and reach shown; its password and Remember.
- `UPDATE: packages/shell/src/hosts/url.ts` - `sshTarget(host, port, user)`: the three fields checked and trimmed, or the field that is wrong.
- `UPDATE: packages/shell/src/hosts/url.test.ts` - its cases.
- `UPDATE: src-tauri/src/hosts.rs` - `save_host` takes `sshPassword`: `null` keeps what is kept, `''` forgets it, a value keeps it (decision 3).
- `UPDATE: packages/shell/src/hosts/ipc.ts` - `saveHost(host, token, sshPassword)`.
- `UPDATE: packages/shell/src/i18n.ts`, `packages/shell/src/app.css` - the words in en, pt-BR and es, and the styles.

## Steps

1. The fields, as mockup 02 draws them: Name (the host's name until typed); Host, Port (22) and User on one line; Key file, optional, with Change (the dialog plugin, as `LocalForm`'s Change); Password, optional, with "Remember the password" (decision 3); the backends, `@ahpd/agent-claude` on and `@ahpd/bot` offered (decision 6); Reach, "Through the SSH connection" first and chosen, with its note that the host is reachable while the desk is open, and "An open port", with its note that the token travels unencrypted (decision 5).
2. Check the server: connects and probes (tasks 01 to 03).
   An encrypted key file shows a Passphrase field, never kept (decision 3).
   A new key shows `ServerKey`: the fingerprint, its type and the server, with Trust and Cancel; Trust calls `confirmSshKey` and checks again (decision 2, open question 2).
   A changed key shows both fingerprints and refuses; Forget is offered where open question 3 settles.
3. The probe's answer: the OS, the CPU, the Node to use (the server's, or "Node 22 will be downloaded on the server"), and lingering; lingering off shows decision 1's warning with `sudo loginctl enable-linger <user>`, and does not stop the install.
   A refusal from the probe shows its reason, and Install is off.
4. Install and start: `installSshAhpd` with the steps and the lines in `InstallSteps`, and Cancel calling `cancelSshInstall`; a failure keeps the form, with the lines, for a retry.
5. Once installed: the host is saved as kind `ssh` (a tunnel) or `url` (an open port), with its `ssh` details and the token the install read; the password is kept when Remember is on.
   A tunnel is opened with `openSshTunnel`; then, as `LocalForm` does, Add host returns to the list and the first run opens the host.
6. The first run's where step has "A server over SSH", with a hint, after "This computer".
7. `sshTarget` checks the Host, Port and User fields; the id is `hostId(name, taken)`, as the other forms.

## Validation

- `sshTarget` tests: a name, an IPv4 and an IPv6 address, spaces trimmed; an empty host or user, a port of 0 or above 65535, and a host with a space refused.
- `i18n.test.ts` passes: every new word in en, pt-BR and es, with its placeholders.
- `pnpm typecheck`, `pnpm test` and `pnpm lint` pass.
- By hand on the debug desk over CDP, with a host list in `AHP_DESK_CONFIG_DIR`, against dev02 only: the form in en, pt-BR and es at the window's smallest size; the fingerprint question shows the same fingerprint as `ssh-keygen -lf /etc/ssh/ssh_host_ed25519_key.pub` on dev02; an install by tunnel and one by open port each end with the host connected and its app window open; the first run reaches the same end.

## Resume
