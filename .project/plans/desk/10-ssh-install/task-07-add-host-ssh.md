---
title: Add host and the first run offer "A server over SSH", which signs in, trusts the key, uses the ahpd found or installs one, and connects
status: done
depends: [task-02-server-key.md, task-05-install.md, task-06-reach.md]
layer: "packages/shell, src-tauri"
refs:
  - "[code://packages/shell/src/hosts/KindField.tsx](../../../../packages/shell/src/hosts/KindField.tsx) - Add host's Kind, which gains a third choice"
  - "[code://packages/shell/src/hosts/LocalForm.tsx#L18-L24](../../../../packages/shell/src/hosts/LocalForm.tsx#L18-L24) - `BACKENDS` and `BOT`, the backends list both forms share"
  - "[code://packages/shell/src/hosts/LocalForm.tsx#L112-L130](../../../../packages/shell/src/hosts/LocalForm.tsx#L112-L130) - a found ahpd's ready plugins ticked and fixed, and at least one backend, the rule the SSH form keeps for an ahpd found on the server"
  - "[code://packages/shell/src/hosts/LocalForm.tsx#L176-L220](../../../../packages/shell/src/hosts/LocalForm.tsx#L176-L220) - the found ahpd's line, the too-old note, and the backends checkboxes, moved into shared fields"
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
Its form takes the server, the user, the sign-in, the folder, the ahpd port, the backends and the reach.
It refuses a new server key until the person trusts it, and a changed one with both fingerprints and a way to forget the old key.
It shows what the probe found: an ahpd already on the server, which it uses, or what the install will do; then it installs or adds the backends, with its steps and Cancel, saves the host, and connects.

## Files

- `CREATE: packages/shell/src/hosts/SshForm.tsx` - the form and its phases: `form`, `connecting`, `key`, `probed`, `installing`, `saving`.
- `CREATE: packages/shell/src/hosts/ServerKey.tsx` - the new key to trust, and the refusal of a changed key with both fingerprints and "Forget the old key".
- `CREATE: packages/shell/src/hosts/BackendsField.tsx` - `BACKENDS`, `BOT` and the checkboxes, with the ready plugins of a found ahpd ticked and fixed, moved from `LocalForm.tsx`.
- `CREATE: packages/shell/src/hosts/FoundAhpd.tsx` - the found ahpd's line, and the too-old note with Check again, moved from `LocalForm.tsx` and given the server's words.
- `UPDATE: packages/shell/src/hosts/LocalForm.tsx` - uses `BackendsField` and `FoundAhpd`; nothing else changes.
- `UPDATE: packages/shell/src/hosts/KindField.tsx` - `AddKind` gains `'ssh'`, a third segment.
- `UPDATE: packages/shell/src/hosts/InstallSteps.tsx` - takes `steps` too: each step's words by its id, with running, done, failed or skipped, above the lines box.
- `UPDATE: packages/shell/src/App.tsx` - Add host with kind `ssh` shows `SshForm`.
- `UPDATE: packages/shell/src/FirstRun.tsx` - a third choice and a step `ssh`.
- `UPDATE: packages/shell/src/hosts/HostForm.tsx` - Edit an SSH host: its name; its host, SSH port, user, ahpd port, folder and reach shown; its password and Remember, worded as in Add host.
- `UPDATE: packages/shell/src/hosts/url.ts` - `sshTarget(host, port, user)`: the three fields checked and trimmed, or the field that is wrong; `ahpdPort(text)`: a port from 1 to 65535, or the error.
- `UPDATE: packages/shell/src/hosts/url.test.ts` - their cases.
- `UPDATE: src-tauri/src/hosts.rs` - `save_host` takes `sshPassword`: `null` keeps what is kept, `''` forgets it, a value keeps it through task 01's `password::keep` (Answers: the sign-in, and a password with no keyring).
- `UPDATE: packages/shell/src/hosts/ipc.ts` - `saveHost(host, token, sshPassword)`.
- `UPDATE: packages/shell/src/i18n.ts`, `packages/shell/src/app.css` - the words in en, pt-BR and es, and the styles.

## Steps

1. The fields, as mockup 02 draws them: Name (the host's name until typed); Host, Port (22) and User on one line; Key file, optional, with Change (the dialog plugin, as `LocalForm`'s Change); Password, optional, with its Remember checkbox; Folder, the server user's home by default, filled from the probe's `home` once the server is checked (Answers: the folder); "ahpd port", 9187, labelled apart from the SSH Port (Answers: the ahpd port); the backends, `@ahpd/agent-claude` on and `@ahpd/bot` offered (Answers: the backends); Reach, "Through the SSH connection" first and chosen, with its note that the host is reachable while the desk is open, and "An open port", with its note that the token travels unencrypted (Answers: the reach).
2. The Remember checkbox's words come from `sshKeyringWorks()` (task 01), asked when the form opens: with a keyring, "Remember the password"; with none, "Remember password (kept unencrypted in the desk's file, no keyring here)" (Answers: a password with no keyring).
3. Check the server: connects and probes (tasks 01 to 03).
   An encrypted key file shows a Passphrase field; what is typed there, and a password not remembered, is held by the desk until it quits, never on disk, and the field says so (Answers: secrets held until quit).
   A new key refuses the connection and shows `ServerKey`: the fingerprint, its type and the server, with Trust and Cancel; Trust calls `confirmSshKey` and checks the server again, which now passes the key (Answers: a new server key).
   A changed key shows both fingerprints, the warning that a changed key can mean someone in the middle, and "Forget the old key", which calls `forgetSshKey` and checks again, so the new key is asked as a first one (Answers: a changed server key); a changed key from `~/.ssh/known_hosts` shows `ssh-keygen -R <host>` instead of the button.
4. The probe's answer, when it found an ahpd (Answers: a found ahpd): its version and where it was found, as `FoundAhpd` shows one on this computer; one too old shows the version needed and Check again, and nothing more; a supported one shows the backends with its ready ones ticked and fixed, and the button reads "Save and connect".
   The Folder and the ahpd port then follow that ahpd: folders its config already names stay (Answers: the folder); the ahpd port shows its own port, fixed, and Reach offers "An open port" only when it already listens beyond 127.0.0.1 (Answers: how a found ahpd is reached).
   One that is not running shows that the desk will start it with its own `ahpd start` and that it will not come back after the server restarts (Answers: a found ahpd that is off).
5. The probe's answer, when it found none: the OS, the CPU, the Node to use (the server's, or "Node 22 will be downloaded on the server"), and lingering; lingering off shows the warning of the Answers row on the user systemd service with `sudo loginctl enable-linger <user>`, and does not stop the install; the button reads "Install and connect".
   A refusal from the probe shows its reason, and the button is off.
6. Save at least one backend, as `LocalForm` asks: one ticked, or one the found ahpd has ready.
7. Install, or add the backends: `installSshAhpd` with the steps and the lines in `InstallSteps`, and Cancel calling `cancelSshInstall`; a failure keeps the form, with the lines, for a retry.
8. Once done: for a tunnel, `pickSshLocalPort()` gives the host its local port, once (Answers: the local port); the host is saved as kind `ssh` (a tunnel) or `url` (an open port), with its `ssh` details (`remotePort`, `folder`, `byDesk` from the install's answer, and `localPort` for a tunnel) and the token the install read; the password is kept when Remember is on, in the keyring or, with none, in `hosts.json` (Answers: a password with no keyring).
   A tunnel is opened with `openSshTunnel`; then, as `LocalForm` does, Add host returns to the list and the first run opens the host.
9. The first run's where step has "A server over SSH", with a hint, after "This computer".
10. `sshTarget` checks the Host, Port and User fields, and `ahpdPort` the ahpd port; the id is `hostId(name, taken)`, as the other forms.

## Validation

- `sshTarget` tests: a name, an IPv4 and an IPv6 address, spaces trimmed; an empty host or user, a port of 0 or above 65535, and a host with a space refused.
- `ahpdPort` tests: 9187, 1 and 65535 pass; 0, 65536, empty and text refused.
- `i18n.test.ts` passes: every new word in en, pt-BR and es, with its placeholders.
- `pnpm typecheck`, `pnpm test` and `pnpm lint` pass.
- By hand on the debug desk over CDP, with a host list in `AHP_DESK_CONFIG_DIR`, against the test machine only: the form in en, pt-BR and es at the window's smallest size; the new key's question shows the same fingerprint as `ssh-keygen -lf /etc/ssh/ssh_host_ed25519_key.pub` on the test machine, and Trust connects; with the desk's entry edited to another key, the refusal shows both fingerprints and "Forget the old key" leads back to the trust question; an install by tunnel on another ahpd port and one by open port each end with the host connected and its app window open; a second Add host on the same server finds that ahpd and offers only the backends it lacks; the first run reaches the same end.
- By hand over CDP, with the page's `ssh_keyring_works` answer patched to false in `window.__TAURI_INTERNALS__.invoke`: the checkbox names the plain file, in en, pt-BR and es, and is not cut at the window's smallest size; the save with no keyring itself is task 01's Cargo tests.
- By hand, task 06's window checks, on a host the form added, with a host list in `AHP_DESK_CONFIG_DIR`: a tunnelled host connects and its app window opens; with the desk closed, nothing on the server listens beyond 127.0.0.1 (`ss -ltn`); restarting the desk opens the tunnel again on the same local port and the window keeps its tabs; a host signed in with a key with a passphrase shows Sign in after the restart, and opens once it is typed; an open-port host connects from the desk.

## Resume

- 2026-10-18: `packages/shell/src/hosts/SshForm.tsx` has the form and its phases; `ServerKey.tsx` the new key's question and the changed key's refusal; `BackendsField.tsx` (`BACKENDS`, `BOT`, `hasBackend`, `startingBackends`) and `FoundAhpd.tsx` (`FoundAhpdField`) are shared with `LocalForm.tsx`.
- `InstallSteps` takes `steps` too, and `withStep()` keeps each step's last status; `KindField`, `App` and `FirstRun` offer "A server over SSH"; `saveHost(host, token, sshPassword)` and `save_host` keep, forget or leave the password; `url.ts` has `sshTarget()` and `ahpdPort()`.
- The probe answers `signIn`, which the host keeps, and `ahpdConfig` in place of its English `warnings`: the form says lingering, a found ahpd that is off, and an old config folder in its own words, in three languages.
- A found ahpd the desk installed shows no "it is off" note: its unit is restarted on every save, and `ahpd status` from a shell does not see the ahpd systemd runs (the plan's pending check).
- Edit host shows an SSH host's server, port, reach and folder as text, keeps `ssh` on save, and takes a password: typed with Remember, it is kept; typed without, the kept one is forgotten and the typed one is held until the desk quits; empty, nothing changes (defaulted: the form had no other way to replace a kept password).
- An SSH agent that does not answer counts as none: Pageant's handle opens whether or not Pageant runs, which showed "agent: early eof".
- `App` reconnects a host as soon as its tunnel comes up, so a new or reopened tunnel never shows down until the client's next retry.
- The host list ellipsizes a long name, and the row's Sign in keeps its buttons on one line at the window's smallest size.
- Verified: `cargo test` (117), `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check`, `pnpm typecheck`, `pnpm test` (27, with `url.test.ts` and `i18n.test.ts`).
- By hand on a release build over CDP, with a scratch `AHP_DESK_CONFIG_DIR` and home, against the test machine Softov named: the new key's question showed the fingerprint Softov matched, and Trust connected; with the desk's entry changed to another key, the refusal showed both fingerprints and "Forget the old key" led back to the question.
- Installed by tunnel on ahpd port 9287 and, through the first run, by open port on 9187 with a key with a passphrase: each ended with the host connected and its app window open; a second Add host found that ahpd, offered only the backends it lacked, and connected with nothing installed.
- Task 06's window checks: with the desk closed, ahpd listened on 127.0.0.1 only; a restart opened the tunnel again on the same local port; the host with a passphrase showed Sign in after the restart and connected once it was typed; the open-port host connected.
- The form in en, pt-BR and es at 480x300 and 640x400; the key file was picked through the real dialog.
- Not run: the no-keyring checkbox by patching `ssh_keyring_works`, since `window.__TAURI_INTERNALS__.invoke` is read-only; its words are in `i18n.test.ts` and the save is task 01's Cargo test.
- Not shown: that the app window keeps its tabs across a restart, as no tab was open before it.
- The test machine was left as before the task: the desk's ahpd, unit and config removed, and the key added for the passphrase check taken out of `authorized_keys`.
- Open: a host whose server key changed after it was added shows "tunnel down" with Retry, which cannot fix it; neither the row nor Edit host offers the key's question.
