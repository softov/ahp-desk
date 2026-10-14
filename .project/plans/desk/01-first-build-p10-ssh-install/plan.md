---
title: Install ahpd on a Linux server over SSH
domain: desk
status: planned
priority: medium
created: 2026-10-13
revalidated: 2026-10-14
requires:
  - plans/desk/01-first-build-p3-local-ahpd/plan.md
decisions:
  - decisions/the-ssh-installer-targets-linux-user-systemd.md
  - decisions/an-ssh-server-key-is-confirmed-once-and-remembered.md
  - decisions/ssh-signs-in-with-the-agent-a-key-file-or-a-password.md
  - decisions/the-remote-node-is-its-own-or-downloaded.md
  - decisions/an-ssh-installed-ahpd-is-reached-by-tunnel-or-open-port.md
  - decisions/the-backends-are-picked-in-add-host.md
  - decisions/the-local-ahpd-uses-a-token-file.md
  - decisions/a-typed-ssh-passphrase-or-password-is-held-in-memory-until-quit.md
  - decisions/a-new-server-key-refuses-the-connection-and-trust-connects-again.md
  - decisions/a-changed-server-key-is-forgotten-from-its-refusal.md
  - decisions/an-ssh-password-without-a-keyring-is-kept-in-hosts-json-when-asked.md
  - decisions/the-installed-ahpd-keeps-the-usual-config-folder.md
  - decisions/the-desk-adds-backends-through-ahpd-plugin-install.md
  - decisions/the-remote-ahpd-serves-the-folder-asked-in-add-host-else-home.md
  - decisions/the-remote-ahpd-port-is-a-field-with-9187.md
  - decisions/an-ahpd-found-on-the-server-is-used-and-offered-the-backends.md
  - decisions/the-remote-node-download-is-the-newest-22.md
  - decisions/a-tunnels-local-port-is-chosen-once-and-kept.md
  - decisions/removing-an-ssh-host-offers-to-uninstall-its-ahpd.md
refs:
  - file:///github/opendoop/packages/tauri/src-tauri/src/ssh/mod.rs - doop's `russh` session, agent then key-file sign-in, `exec_capture`
  - file:///github/opendoop/packages/tauri/src-tauri/src/ssh/push.rs - doop's file push over SSH
  - file:///github/opendoop/packages/tauri/src-tauri/src/ssh_probe.rs - doop's probe of the remote OS and runtime
  - file:///github/opendoop/packages/tauri/src-tauri/src/installer/systemd_node.rs - doop's user systemd unit, enable, and health wait
  - file:///github/opendoop/packages/tauri/src-tauri/src/installer/mod.rs - the install steps streamed over a Tauri `Channel`, and cancel
  - file:///github/ahpd/packages/server/src/commands/options.ts - `--host`, `--port`, `--connection-token-file`
  - "[code://packages/shell/src/FirstRun.tsx](../../../../packages/shell/src/FirstRun.tsx) - where the host is: a third choice, \"A server over SSH\""
  - "[code://packages/shell/src/hosts/LocalForm.tsx](../../../../packages/shell/src/hosts/LocalForm.tsx) - the found ahpd on this computer, the pattern a found ahpd on the server follows"
  - "[code://src-tauri/src/hosts.rs](../../../../src-tauri/src/hosts.rs) - the host kinds, which gain `ssh`"
---

## Goal

From Add host or the first run, a person names a Linux server, signs in over SSH, picks the backends, and the desk uses the ahpd it finds there, or installs ahpd there as a user service, and connects to it.
The desk reaches it through the SSH connection or an open port, as the person chooses.
The desk can later update that ahpd over the same SSH access, and Remove offers to uninstall it.

## Reconnaissance

The files read and the pattern reused are the `refs` above: doop installs its daemon the same way, with the gaps its comments name (any server key accepted, a full bundle pushed).

### Runtime path

```
Add host "A server over SSH" -> address, SSH port, user, sign-in (agent | key file | password), folder, ahpd port
  -> server key: known_hosts or confirmed passes; new or changed refuses with its fingerprints -> Trust (or Forget the old key) -> connect again
  -> probe: Linux, CPU, systemctl --user, lingering, node -v, an ahpd already there
  -> ahpd found: its version (too old shown, not replaced), the backends it lacks through its own plugin install -> connect
  -> none found: Node 22+ there, or latest-v22.x downloaded into ~/.local/share/ahpd/node
     -> npm install @ahpd/server into ~/.local/share/ahpd; backends through ahpd plugin install with AHPD_PLUGIN_ROOT=~/.local/share/ahpd; config in ~/.config/ahpd; the folder as paths
     -> ~/.config/systemd/user/ahpd.service on the ahpd port; systemctl --user enable, restart
  -> reach: tunnel (127.0.0.1, a local port kept on the host) | open port (0.0.0.0 + token)
  -> Host { kind: ssh | url, ssh details } -> hosts-changed -> connect
Remove on a host the desk installed -> offer to uninstall: disable --now, the unit, ~/.local/share/ahpd -> forget the host
```

### Gaps

- `Not found: an SSH client in the desk - doop's is in another repository, to be brought in.`
- A user service stops at logout without lingering, which needs an administrator to turn on.
- `Not found: the ahpd command that sets paths with no terminal - the SSH install's folder waits for it, as p3's does.`

## Decisions locked in

| # | Decision | Rationale / source |
| --- | --- | --- |
| 1 | [The SSH installer puts ahpd on Linux as a user systemd service](../../../decisions/the-ssh-installer-targets-linux-user-systemd.md) | Softov, 2026-10-13 |
| 2 | [An SSH server's key is confirmed once and remembered](../../../decisions/an-ssh-server-key-is-confirmed-once-and-remembered.md) | Softov, 2026-10-13 |
| 3 | [SSH signs in with the agent, a key file, or a password](../../../decisions/ssh-signs-in-with-the-agent-a-key-file-or-a-password.md) | Softov, 2026-10-13 |
| 4 | [The remote ahpd runs on the server's own Node 22, or one it downloads](../../../decisions/the-remote-node-is-its-own-or-downloaded.md) | Softov, 2026-10-13 |
| 5 | [An SSH-installed ahpd is reached through the SSH connection or an open port, chosen per install](../../../decisions/an-ssh-installed-ahpd-is-reached-by-tunnel-or-open-port.md) | Softov, 2026-10-13 |
| 6 | [The backends are picked in Add host](../../../decisions/the-backends-are-picked-in-add-host.md) | Softov, 2026-10-11 |
| 7 | [The local ahpd uses a token file](../../../decisions/the-local-ahpd-uses-a-token-file.md) | Softov, 2026-10-11 |
| 8 | [A typed SSH passphrase, or a password not saved, is held in memory until the desk quits](../../../decisions/a-typed-ssh-passphrase-or-password-is-held-in-memory-until-quit.md) | Softov, 2026-10-14 |
| 9 | [A server key never seen before refuses the first connection, and Trust connects again](../../../decisions/a-new-server-key-refuses-the-connection-and-trust-connects-again.md) | Softov, 2026-10-14 |
| 10 | [A changed server key is forgotten from its refusal](../../../decisions/a-changed-server-key-is-forgotten-from-its-refusal.md) | Softov, 2026-10-14 |
| 11 | [With no keyring, an SSH password is kept in hosts.json only when the person ticks a box that says so](../../../decisions/an-ssh-password-without-a-keyring-is-kept-in-hosts-json-when-asked.md) | Softov, 2026-10-14 |
| 12 | [An ahpd the desk installs keeps the usual config folder, with its backends under AHPD_PLUGIN_ROOT](../../../decisions/the-installed-ahpd-keeps-the-usual-config-folder.md) | Softov, 2026-10-13, p3; held for the SSH install below |
| 13 | [The desk adds backends through ahpd plugin install](../../../decisions/the-desk-adds-backends-through-ahpd-plugin-install.md) | Softov, 2026-10-13, p3; held for the SSH install below |
| 14 | [The remote ahpd serves the folder asked in Add host, the server user's home by default](../../../decisions/the-remote-ahpd-serves-the-folder-asked-in-add-host-else-home.md) | Softov, 2026-10-14 |
| 15 | [The port the remote ahpd listens on is a field, 9187 by default](../../../decisions/the-remote-ahpd-port-is-a-field-with-9187.md) | Softov, 2026-10-14 |
| 16 | [An ahpd found on the server is used and offered the backends, and the desk installs only where none is found](../../../decisions/an-ahpd-found-on-the-server-is-used-and-offered-the-backends.md) | Softov, 2026-10-14 |
| 17 | [The Node downloaded on a server is the newest 22, from latest-v22.x, checked against its SHASUMS256.txt](../../../decisions/the-remote-node-download-is-the-newest-22.md) | Softov, 2026-10-14 |
| 18 | [A tunnelled host's local port is chosen once and kept on the host](../../../decisions/a-tunnels-local-port-is-chosen-once-and-kept.md) | Softov, 2026-10-14 |
| 19 | [Removing an SSH host offers to stop and uninstall its ahpd on the server](../../../decisions/removing-an-ssh-host-offers-to-uninstall-its-ahpd.md) | Softov, 2026-10-14 |

| What | Source | Task |
| --- | --- | --- |
| The SSH client is `russh`, as doop's. | doop's `ssh/mod.rs` | 01 |
| The install streams its steps and can be cancelled, as doop's. | doop's `installer/mod.rs` | 05 |
| Decisions 12 and 13 hold for the SSH install: the server's ahpd uses `~/.config/ahpd`, the desk's install puts `@ahpd/server` and the backends under `AHPD_PLUGIN_ROOT=~/.local/share/ahpd`, and the backends go in through the installed ahpd's `plugin install`, so a server's own ahpd and the desk's install look the same. | Softov, 2026-10-14, asked "p10 q5: do p3's two rules (~/.config/ahpd, backends with ahpd plugin install) apply to the remote install, so a server's own ahpd and the desk's install look the same?": "Yes, same layout"; rejected: p10's own layout under ~/.local/share/ahpd; Softov's reason, asked first: "Think like a remote server with ahpd already installed on it per example... it will also work?" | 05 |

## Proposed architecture

- **Data flow** - `src-tauri/src/ssh/` holds the session, sign-in, the secrets held in memory, the confirmed keys and the tunnels; `src-tauri/src/ssh_install.rs` probes, installs, writes the unit and uninstalls.
- **Event flow** - the install's and the uninstall's steps over a Tauri `Channel`; `tunnels-changed` per tunnel; `hosts-changed` when the host is saved or removed.
- **State flow** - `hosts.json` keeps the host's SSH address, user, sign-in kind, reach, ports, folder and whether the desk installed its ahpd; a kept password goes in the keyring, or in `hosts.json` when there is none and the person ticked so; a typed passphrase or password not kept lives in memory until quit; confirmed keys in `known_keys.json` beside it.
- **Layer responsibilities** - `src-tauri`: SSH, the install, the tunnel, the uninstall · `packages/shell`: the form, the fingerprint question and refusal, the steps, Remove's question.
- **Source-of-truth files** - `src-tauri/src/ssh/mod.rs`, `src-tauri/src/ssh_install.rs`.

## Tasks

| Task | Status | Depends on |
| --- | --- | --- |
| [01 - an SSH session: agent, key file with passphrase, password in the keyring or the plain file, secrets held until quit](task-01-ssh-session.md) | todo | - |
| [02 - the server key: known_hosts, refuse a new one until Trust, refuse a changed one with Forget](task-02-server-key.md) | todo | 01 |
| [03 - the probe: Linux, CPU, systemctl --user, lingering, Node, an ahpd already there](task-03-probe.md) | todo | 01 |
| [04 - Node: the server's own, or the newest 22 downloaded there](task-04-node.md) | todo | 03 |
| [05 - the install, or the found ahpd: @ahpd/server and the backends, the folder, the token file, the user unit, the health wait, with steps and cancel](task-05-install.md) | todo | 04 |
| [06 - reach it: the tunnel on a kept local port (host kind `ssh`) or the open port (a URL host)](task-06-reach.md) | todo | 05 |
| [07 - Add host and the first run: "A server over SSH"](task-07-add-host-ssh.md) | todo | 02, 05, 06 |
| [08 - update over SSH, for p9](task-08-update-over-ssh.md) | todo | 05 |
| [10 - Remove offers to uninstall the desk's ahpd on the server](task-10-uninstall-over-ssh.md) | todo | 05, 06, 07 |
| [09 - end to end on dev02](task-09-end-to-end.md) | todo | 01 to 08, 10 |

## Risks and tradeoffs

- The end to end runs on dev02, which Softov offered for it on 2026-10-13; never on dev01 or another host in use.
- A tunnelled host is down whenever the desk is closed; the form says so.
- A password kept in `hosts.json` on a machine with no keyring is readable by anything that reads the desk's files; the checkbox and the host's row say so.
- Using an ahpd found on the server means the desk changes the plugins of an ahpd it did not install, only when the person ticks a backend.

## Resume state

- **Done so far:** the plan and decisions 1 to 7, 2026-10-13; the task files and decisions 8 to 19, 2026-10-14.
- **Next action:** task 01.
- **Open questions:** each has a proposed answer.
  None is decided until Softov answers it and it is written as a decision file.
  1. A found ahpd on the server (decision 16) that is not running: does the desk start it with that ahpd's own `ahpd start`, or write its user unit to run it?
     Proposed: its own `ahpd start --port <ahpd port> --connection-token-file`, as p3 starts a found ahpd on this computer; the desk writes no unit for an ahpd it did not install, and the probe's answer says it will not come back after the server restarts.
     Tasks 05, 07.
  2. How a found ahpd on the server is reached: the reach and the ahpd port when it already runs.
     Proposed: its address and token from its `ahpd status --json` and `daemon.json`, as p3 reads a found ahpd's; the ahpd port field shows its port, fixed; the tunnel is always offered, and the open port only when it already listens beyond 127.0.0.1.
     Tasks 05, 06, 07.
  3. Remove on an SSH host whose ahpd the desk found and did not install (decision 16): offer only a stop, or nothing on the server?
     Proposed: nothing; Remove says the server's ahpd stays as it is because the desk did not install it, and forgets the host.
     Task 10.
  4. The server's `~/.config/ahpd` on uninstall: left, or removed with the desk's install?
     Proposed: left, since it is ahpd's usual config folder (decision 12) and may serve an ahpd the person installs later; Remove's question says it stays.
     Task 10.
  5. Uninstall when the server cannot be reached: the host stays, or is removed from the desk anyway?
     Proposed: the uninstall's error is shown with "Remove from the desk only", which forgets the host and leaves the server as it is.
     Task 10.
- **Watch out for:** doop's `AcceptAllHandler` must not be copied; decision 2 replaces it.
  The end to end and every real SSH test run against dev02 only.
  Task 10 runs before task 09, the end to end, which covers it.

## Final verification checklist

- [ ] On dev02, each sign-in installs ahpd and connects, by tunnel and by open port.
- [ ] A new server key is refused until Trust; a changed one is refused with both fingerprints and forgotten from the refusal.
- [ ] An ahpd already on dev02 is used, with the backends it lacks offered, and nothing installed beside it.
- [ ] Update over SSH installs the newer ahpd and the host comes back.
- [ ] Remove with uninstall leaves no unit, no install and no token of the desk's on dev02.
- [ ] `plans/index.md` updated.
