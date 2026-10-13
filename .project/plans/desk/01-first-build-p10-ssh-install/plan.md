---
title: Install ahpd on a Linux server over SSH
domain: desk
status: draft
priority: medium
created: 2026-10-13
revalidated: 2026-10-13
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

These task files are written when this plan moves to `planned`.

| Task | Status | Depends on |
| --- | --- | --- |
| 01 - an SSH session: agent, key file with passphrase, password in the keyring | todo | - |
| 02 - the server key: known_hosts, confirm the fingerprint, remember, refuse a change | todo | 01 |
| 03 - the probe: Linux, CPU, systemctl --user, lingering, Node | todo | 01 |
| 04 - Node: the server's own, or the official build downloaded there | todo | 03 |
| 05 - the install: @ahpd/server and the backends, the token file, the user unit, the health wait, with steps and cancel | todo | 04 |
| 06 - reach it: the tunnel (host kind `ssh`) or the open port (a URL host) | todo | 05 |
| 07 - Add host and the first run: "A server over SSH" | todo | 02, 05, 06 |
| 08 - update over SSH, for p9 | todo | 05 |
| 09 - end to end on a throwaway Linux server | todo | 01 to 08 |

## Risks and tradeoffs

- The end to end needs a Linux server that can be thrown away; a container with sshd and systemd, or a VM, never a real host.
- A tunnelled host is down whenever the desk is closed; the form says so.

## Resume state

- **Done so far:** the plan and its decisions, 2026-10-13.
- **Next action:** write the task files, then task 01.
- **Open questions:** none.
- **Watch out for:** doop's `AcceptAllHandler` must not be copied; decision 2 replaces it.

## Final verification checklist

- [ ] On a throwaway Linux server, each sign-in installs ahpd and connects, by tunnel and by open port.
- [ ] A changed server key is refused with both fingerprints.
- [ ] Update over SSH installs the newer ahpd and the host comes back.
- [ ] `plans/index.md` updated.
