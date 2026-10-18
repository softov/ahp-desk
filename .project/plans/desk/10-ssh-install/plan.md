---
title: Install ahpd on a Linux server over SSH
domain: desk
status: active
priority: medium
created: 2026-10-13
revalidated: 2026-10-14
requires:
  - plans/desk/03-local-ahpd/plan.md
decisions:
  - decisions/a-new-server-key-refuses-the-connection-and-trust-connects-again.md
  - decisions/an-ssh-password-without-a-keyring-is-kept-in-hosts-json-when-asked.md
  - decisions/the-remote-node-download-is-the-newest-22.md
  - decisions/the-installed-ahpd-keeps-the-usual-config-folder.md
  - decisions/the-desk-adds-backends-through-ahpd-plugin-install.md
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
  -> ahpd found: its version (too old shown, not replaced), the backends it lacks through its own plugin install, its own ahpd start when it is off (no unit) -> its port and token from status --json and daemon.json -> connect
  -> none found: Node 22+ there, or latest-v22.x downloaded into ~/.local/share/ahpd/node
     -> npm install @ahpd/server into ~/.local/share/ahpd; backends through ahpd plugin install with AHPD_PLUGIN_ROOT=~/.local/share/ahpd; config in ~/.config/ahpd; the folder as paths
     -> ~/.config/systemd/user/ahpd.service on the ahpd port; systemctl --user enable, restart
  -> reach: tunnel (127.0.0.1, a local port kept on the host) | open port (0.0.0.0 + token)
  -> Host { kind: ssh | url, ssh details } -> hosts-changed -> connect
Remove on a host the desk installed -> offer to uninstall: disable --now, the unit, ~/.local/share/ahpd (~/.config/ahpd left) -> forget the host
  | server unreachable -> why, with "Remove from the desk only" and Cancel
Remove on a host the desk did not install -> nothing on the server, and the question says so -> forget the host
```

### Gaps

- `Not found: an SSH client in the desk - doop's is in another repository, to be brought in.`
- A user service stops at logout without lingering, which needs an administrator to turn on.
- `ahpd paths add <folder>` sets the served folders with no terminal; it is in ahpd 0.12.0.

## Answers

| What | Source | Task |
| --- | --- | --- |
| The SSH installer puts ahpd on Linux only, under the signed-in user, as a `systemctl --user` service, with no sudo; macOS, other Unixes and Windows come later, each as its own installer. | Softov, 2026-10-13, asked "Which machines should the SSH installer handle first?": "Linux, user systemd (Recommended)" | 03, 05 |
| The first connection shows the server key's fingerprint to confirm; the desk keeps the confirmed key and refuses a different one with a warning that names both fingerprints. A key already in `~/.ssh/known_hosts` for that host counts as confirmed. | Softov, 2026-10-13, asked "How does the desk trust a server's SSH key the first time?": "Show the fingerprint, then remember (Recommended)" | 02 |
| SSH signs in with the agent (Pageant or the OpenSSH agent) first, then a key file the person picks, asking its passphrase, then a password. A password the person asks to keep goes in the OS keyring; a passphrase is never kept. | Softov, 2026-10-13, asked "How does the desk sign in over SSH?": "Also a password" | 01 |
| The installer uses a Node 22 or later already on the server; without one, the server downloads the official Node build for its CPU into the install folder, and nothing large goes from the desk. | Softov, 2026-10-13, asked "Where does Node come from on the remote machine?": "Its own Node 22+, else download (Recommended)" | 03, 04 |
| The installer offers two ways to reach the ahpd, the tunnel first: through the SSH connection, with ahpd on 127.0.0.1 and a forwarded local port, reachable while the desk holds the session; or an open port with its token, ahpd on 0.0.0.0, added as a URL host, the token in cleartext without TLS, which the form says. | Softov, 2026-10-13, asked "How does the desk reach an ahpd it installed over SSH?": "Ask per install" | 06, 07 |
| [The backends are picked in Add host](../../../specs/local-ahpd.md) | Softov, 2026-10-11 | 05, 07 |
| [The local ahpd uses a token file](../../../specs/local-ahpd.md) | Softov, 2026-10-11 | 05 |
| [An ahpd the desk installs keeps the usual config folder, with its backends under AHPD_PLUGIN_ROOT](../../../decisions/the-installed-ahpd-keeps-the-usual-config-folder.md) | Softov, 2026-10-13, plan 03; held for the SSH install below | 03, 05 |
| [The desk adds backends through ahpd plugin install](../../../decisions/the-desk-adds-backends-through-ahpd-plugin-install.md) | Softov, 2026-10-13, plan 03; held for the SSH install below | 05 |
| A typed SSH passphrase, and a typed password not saved, are held in memory per host until the desk quits, never on disk; after a restart, a tunnelled host that needs one shows "Sign in" on its row. | Softov, 2026-10-14: "In memory until quit" | 01, 06, 07 |
| [A server key never seen before refuses the first connection, and Trust connects again](../../../decisions/a-new-server-key-refuses-the-connection-and-trust-connects-again.md) | Softov, 2026-10-14: "Refuse, reconnect" | 02, 07 |
| A changed server key is refused with both fingerprints and a "Forget the old key" button, which removes the desk's key so the next connection asks to trust the new one; a changed key from `~/.ssh/known_hosts` names `ssh-keygen -R <host>` instead. | Softov, 2026-10-14: "On the refusal" | 02, 07 |
| [With no keyring, an SSH password is kept in hosts.json only when the person ticks a box that says so](../../../decisions/an-ssh-password-without-a-keyring-is-kept-in-hosts-json-when-asked.md) | Softov, 2026-10-14: "Opt-in, says plain file" | 01, 07 |
| Add host over SSH has a Folder field, filled with the server user's home, and the remote ahpd serves that folder, written as `paths` by the same ahpd command as for "This computer". | Softov, 2026-10-14: "A form field" | 05, 07 |
| Add host over SSH has an "ahpd port" field, 9187 by default, apart from the SSH Port; the host keeps it and Update leaves it. | Softov, 2026-10-14: "A field, default 9187" | 05, 06, 07 |
| An ahpd found on the server is used: its version is shown (one too old is not replaced), the backends it lacks are offered and added with its own `plugin install`, and the desk installs ahpd only where none is found. | Softov, 2026-10-14: "Use it, offer backends" | 03, 05, 07 |
| [The Node downloaded on a server is the newest 22, not the version the desk pins](../../../decisions/the-remote-node-download-is-the-newest-22.md) | Softov, 2026-10-14: "latest-v22.x" | 04 |
| A tunnelled host's local port is picked once, when the host is added, and kept on the host. | Softov, 2026-10-14: "Chosen once, kept" | 06, 07 |
| Remove, on a host the desk installed over SSH, offers to stop and uninstall its ahpd on the server: the user unit, the desk's install and its token file. | Softov, 2026-10-14: "Offer to uninstall" | 10 |
| A found ahpd that is not running is started with its own `ahpd start`, with no unit written, and the form says it will not come back after the server restarts. | Softov, 2026-10-14: "Start it, say it won't survive reboot" | 05, 07 |
| A found ahpd is reached on its own port, read from its `ahpd status --json`, with its token from its `daemon.json`; the tunnel is always offered, and the open port only when it already listens beyond 127.0.0.1. | Softov, 2026-10-14: "Its own port, via tunnel" | 05, 06, 07 |
| Remove on an SSH host whose ahpd the desk did not install does nothing on the server, and the Remove question says so. | Softov, 2026-10-14: "Nothing, and say so" | 10 |
| The uninstall leaves `~/.config/ahpd` on the server as it is. | Softov, 2026-10-14: "Leave it" | 10 |
| An uninstall that cannot reach the server, or fails there, keeps the host and shows why, with "Remove from the desk only" and Cancel. | Softov, 2026-10-14: "Error + remove-only button" | 10 |
| The SSH client is `russh`, as doop's. | doop's `ssh/mod.rs` | 01 |
| The install streams its steps and can be cancelled, as doop's. | doop's `installer/mod.rs` | 05 |
| The two plan 03 decisions above hold for the SSH install: the server's ahpd uses `~/.config/ahpd`, the desk's install puts `@ahpd/server` and the backends under `AHPD_PLUGIN_ROOT=~/.local/share/ahpd`, and the backends go in through the installed ahpd's `plugin install`, so a server's own ahpd and the desk's install look the same. | Softov, 2026-10-14, asked "p10 q5: do p3's two rules (~/.config/ahpd, backends with ahpd plugin install) apply to the remote install, so a server's own ahpd and the desk's install look the same?": "Yes, same layout"; rejected: plan 10's own layout under ~/.local/share/ahpd; Softov's reason, asked first: "Think like a remote server with ahpd already installed on it per example... it will also work?" | 05 |

## Proposed architecture

- **Data flow** - `src-tauri/src/ssh/` holds the session, sign-in, the secrets held in memory, the confirmed keys and the tunnels; `src-tauri/src/ssh_install.rs` probes, installs, writes the unit and uninstalls.
- **Event flow** - the install's and the uninstall's steps over a Tauri `Channel`; `tunnels-changed` per tunnel; `hosts-changed` when the host is saved or removed.
- **State flow** - `hosts.json` keeps the host's SSH address, user, sign-in kind, reach, ports, folder and whether the desk installed its ahpd; a kept password goes in the keyring, or in `hosts.json` when there is none and the person ticked so; a typed passphrase or password not kept lives in memory until quit; confirmed keys in `known_keys.json` beside it.
- **Layer responsibilities** - `src-tauri`: SSH, the install, the tunnel, the uninstall · `packages/shell`: the form, the fingerprint question and refusal, the steps, Remove's question.
- **Source-of-truth files** - `src-tauri/src/ssh/mod.rs`, `src-tauri/src/ssh_install.rs`.

## Tasks

| Task | Status | Depends on |
| --- | --- | --- |
| [01 - an SSH session: agent, key file with passphrase, password in the keyring or the plain file, secrets held until quit](task-01-ssh-session.md) | done | - |
| [02 - the server key: known_hosts, refuse a new one until Trust, refuse a changed one with Forget](task-02-server-key.md) | done | 01 |
| [03 - the probe: Linux, CPU, systemctl --user, lingering, Node, an ahpd already there](task-03-probe.md) | done | 01 |
| [04 - Node: the server's own, or the newest 22 downloaded there](task-04-node.md) | todo | 03 |
| [05 - the install, or the found ahpd: @ahpd/server and the backends, the folder, the token file, the user unit, the health wait, with steps and cancel](task-05-install.md) | todo | 04 |
| [06 - reach it: the tunnel on a kept local port (host kind `ssh`) or the open port (a URL host)](task-06-reach.md) | todo | 05 |
| [07 - Add host and the first run: "A server over SSH"](task-07-add-host-ssh.md) | todo | 02, 05, 06 |
| [08 - update over SSH, for plan 09](task-08-update-over-ssh.md) | todo | 05 |
| [10 - Remove offers to uninstall the desk's ahpd on the server](task-10-uninstall-over-ssh.md) | todo | 05, 06, 07 |
| [09 - end to end on dev02](task-09-end-to-end.md) | todo | 01 to 08, 10 |

## Risks and tradeoffs

- The end to end runs on dev02, which Softov offered for it on 2026-10-13; never on dev01 or another host in use.
- A tunnelled host is down whenever the desk is closed; the form says so.
- A password kept in `hosts.json` on a machine with no keyring is readable by anything that reads the desk's files; the checkbox and the host's row say so.
- Using an ahpd found on the server means the desk changes the plugins of an ahpd it did not install, only when the person ticks a backend.

## Resume state

- **Done so far:** the plan, its decisions and Softov's answers of 2026-10-14; tasks 01 to 03, 2026-10-18, with their by-hand checks.
- **Next action:** task 04.
- **Open questions:** none.
- **Watch out for:** doop's `AcceptAllHandler` must not be copied; the Answers row on the server key replaces it.
  The end to end and every real SSH test run against dev02 only.
  Task 10 runs before task 09, the end to end, which covers it.

## Final verification checklist

- [ ] On dev02, each sign-in installs ahpd and connects, by tunnel and by open port.
- [ ] A new server key is refused until Trust; a changed one is refused with both fingerprints and forgotten from the refusal.
- [ ] An ahpd already on dev02 is used, with the backends it lacks offered, and nothing installed beside it.
- [ ] Update over SSH installs the newer ahpd and the host comes back.
- [ ] Remove with uninstall leaves no unit, no install and no token of the desk's on dev02, and `~/.config/ahpd` as it was.
- [ ] A found ahpd that was off is started with no unit, and Remove leaves it running.
- [ ] `plans/index.md` updated.
