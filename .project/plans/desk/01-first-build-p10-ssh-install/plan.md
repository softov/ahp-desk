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
refs:
  - file:///github/opendoop/packages/tauri/src-tauri/src/ssh/mod.rs - doop's `russh` session, agent then key-file sign-in, `exec_capture`
  - file:///github/opendoop/packages/tauri/src-tauri/src/ssh/push.rs - doop's file push over SSH
  - file:///github/opendoop/packages/tauri/src-tauri/src/ssh_probe.rs - doop's probe of the remote OS and runtime
  - file:///github/opendoop/packages/tauri/src-tauri/src/installer/systemd_node.rs - doop's user systemd unit, enable, and health wait
  - file:///github/opendoop/packages/tauri/src-tauri/src/installer/mod.rs - the install steps streamed over a Tauri `Channel`, and cancel
  - file:///github/ahpd/packages/server/src/commands/options.ts - `--host`, `--port`, `--connection-token-file`
  - "[code://packages/shell/src/FirstRun.tsx](../../../../packages/shell/src/FirstRun.tsx) - where the host is: a third choice, \"A server over SSH\""
  - "[code://src-tauri/src/hosts.rs](../../../../src-tauri/src/hosts.rs) - the host kinds, which gain `ssh`"
---

## Goal

From Add host or the first run, a person names a Linux server, signs in over SSH, picks the backends, and the desk installs ahpd there as a user service and connects to it.
The desk reaches it through the SSH connection or an open port, as the person chooses.
The desk can later update that ahpd over the same SSH access.

## Reconnaissance

The files read and the pattern reused are the `refs` above: doop installs its daemon the same way, with the gaps its comments name (any server key accepted, a full bundle pushed).

### Runtime path

```
Add host "A server over SSH" -> address, user, sign-in (agent | key file | password) -> server key: known_hosts or confirm fingerprint
  -> probe: Linux, CPU, systemctl --user, lingering, node -v
  -> Node 22+ there, or download the official build into ~/.local/share/ahpd/node
  -> npm install @ahpd/server + backends into ~/.local/share/ahpd; token file
  -> ~/.config/systemd/user/ahpd.service; systemctl --user enable --now
  -> reach: tunnel (127.0.0.1, local forward) | open port (0.0.0.0 + token)
  -> Host { kind: ssh | url, ssh details } -> hosts-changed -> connect
```

### Gaps

- `Not found: an SSH client in the desk - doop's is in another repository, to be brought in.`
- A user service stops at logout without lingering, which needs an administrator to turn on.

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

| What | Source | Task |
| --- | --- | --- |
| The SSH client is `russh`, as doop's. | doop's `ssh/mod.rs` | 01 |
| The install streams its steps and can be cancelled, as doop's. | doop's `installer/mod.rs` | 05 |

## Proposed architecture

- **Data flow** - `src-tauri/src/ssh/` holds the session, sign-in and confirmed keys; `src-tauri/src/ssh_install.rs` probes, installs and writes the unit.
- **Event flow** - the install's steps over a Tauri `Channel`; `hosts-changed` when the host is saved.
- **State flow** - `hosts.json` keeps the host's SSH address, user, sign-in kind and reach; a kept password goes in the keyring; confirmed keys in `known_keys.json` beside it.
- **Layer responsibilities** - `src-tauri`: SSH, the install, the tunnel · `packages/shell`: the form, the fingerprint question, the steps.
- **Source-of-truth files** - `src-tauri/src/ssh/mod.rs`, `src-tauri/src/ssh_install.rs`.

## Tasks

| Task | Status | Depends on |
| --- | --- | --- |
| [01 - an SSH session: agent, key file with passphrase, password in the keyring](task-01-ssh-session.md) | todo | - |
| [02 - the server key: known_hosts, confirm the fingerprint, remember, refuse a change](task-02-server-key.md) | todo | 01 |
| [03 - the probe: Linux, CPU, systemctl --user, lingering, Node](task-03-probe.md) | todo | 01 |
| [04 - Node: the server's own, or the official build downloaded there](task-04-node.md) | todo | 03 |
| [05 - the install: @ahpd/server and the backends, the token file, the user unit, the health wait, with steps and cancel](task-05-install.md) | todo | 04 |
| [06 - reach it: the tunnel (host kind `ssh`) or the open port (a URL host)](task-06-reach.md) | todo | 05 |
| [07 - Add host and the first run: "A server over SSH"](task-07-add-host-ssh.md) | todo | 02, 05, 06 |
| [08 - update over SSH, for p9](task-08-update-over-ssh.md) | todo | 05 |
| [09 - end to end on dev02](task-09-end-to-end.md) | todo | 01 to 08 |

## Risks and tradeoffs

- The end to end runs on dev02, which Softov offered for it on 2026-10-13; never on dev01 or another host in use.
- A tunnelled host is down whenever the desk is closed; the form says so.

## Resume state

- **Done so far:** the plan and its decisions, 2026-10-13; the task files, 2026-10-14.
- **Next action:** task 01.
  Its steps 4 and 5 wait for open questions 1 and 4.
- **Open questions:** each has a proposed answer.
  None is decided until Softov answers it and it is written as a decision file.
  1. A key file with a passphrase, and a password not kept: does the desk hold the decrypted key and the typed password in memory until it quits, or ask at every connection, including each time a tunnel opens again?
     Proposed: hold them in memory until the desk quits, never on disk; a tunnelled host whose sign-in needs the person shows down on its row with "Sign in".
     Tasks 01, 06.
  2. How a confirmed fingerprint reaches the connection: the first connection is refused and a second one made once the person trusts the key, or the handshake waits for the person's answer.
     Proposed: refuse, then connect again, which then passes the check like any known key.
     Tasks 02, 07.
  3. Where the desk offers to forget a server's old key (decision 2's consequence): a Forget button on the refusal that names both fingerprints, or only in the host's Edit.
     Proposed: on the refusal; the next connection then asks to trust the new key as a first one.
     Tasks 02, 07.
  4. A password to remember when the OS keyring cannot keep it: write it in `hosts.json`, as a host token falls back to, or keep it nowhere and say so.
     Proposed: keep it nowhere; the form says the keyring is unavailable and the password is asked at each connection.
     Task 01.
  5. Do p3's decisions [An ahpd the desk installs keeps the usual config folder, with its backends under AHPD_PLUGIN_ROOT](../../../decisions/the-installed-ahpd-keeps-the-usual-config-folder.md) and [The desk adds backends through ahpd plugin install](../../../decisions/the-desk-adds-backends-through-ahpd-plugin-install.md) hold for the SSH install?
     Proposed: yes, linked in this plan's Decisions table: `~/.config/ahpd` on the server, `AHPD_PLUGIN_ROOT=~/.local/share/ahpd`, and the backends through the installed ahpd's `plugin install`.
     Task 05.
  6. The folder the remote ahpd serves: the server user's home, or a folder asked in the form.
     Proposed: the home folder (`WorkingDirectory=%h`), with no field, as the local ahpd does when nothing is asked.
     Task 05.
  7. The port ahpd listens on, on the server: ahpd's default 9187 always, or an "ahpd port" field with 9187 filled in, beside the SSH Port.
     Proposed: the field with 9187, since a server may already use that number.
     Tasks 05, 06, 07.
  8. A server that already has an ahpd: the desk's own earlier install, or one installed by hand (another unit, or the port in use).
     Proposed: the desk's own is installed over, keeping its token file; one installed by hand is refused with what the probe found, and connecting to it without installing is left to a later plan.
     Tasks 03, 05.
  9. The Node version downloaded on the server: the one `scripts/stage-node.mjs` pins for the desk (v22.23.3), or the newest 22 from `latest-v22.x`.
     Proposed: the pinned one, so the desk runs one tested Node everywhere it installs.
     Task 04.
  10. A tunnel's local port: chosen once at install and kept on the host, or chosen free each time the tunnel opens.
      Proposed: kept, because ahpw keeps a host's pages under its address (p8's decision), and a taken port shows as the tunnel's error.
      Task 06.
  11. Remove host on an SSH host: forget the host, its kept password and its tunnel and leave the server's ahpd running, or also offer to uninstall it.
      Proposed: leave it running; uninstall is a later plan.
      Tasks 06, 07.
- **Watch out for:** doop's `AcceptAllHandler` must not be copied; decision 2 replaces it.
  The end to end and every real SSH test run against dev02 only.

## Final verification checklist

- [ ] On dev02, each sign-in installs ahpd and connects, by tunnel and by open port.
- [ ] A changed server key is refused with both fingerprints.
- [ ] Update over SSH installs the newer ahpd and the host comes back.
- [ ] `plans/index.md` updated.
