---
title: Install over SSH works end to end on dev02
status: todo
depends: [task-01-ssh-session.md, task-02-server-key.md, task-03-probe.md, task-04-node.md, task-05-install.md, task-06-reach.md, task-07-add-host-ssh.md, task-08-update-over-ssh.md]
layer: "ahp-desk"
refs:
  - "[code://src-tauri/src/lib.rs#L31-L37](../../../../src-tauri/src/lib.rs#L31-L37) - `AHP_DESK_CONFIG_DIR`, so the run uses a scratch host list and never the user's"
---

## Objective

Every item of the plan's final checklist holds on a release build of the desk on this Windows machine, against dev02.
dev02 is the only server used; never dev01 or any other host (the plan's Risks).

## Files

- None.

## Steps

1. Before: ask Softov for what dev02 needs and the run cannot set itself: a user for the run, whether password sign-in is on in its `sshd`, whether lingering may be turned on for that user, and whether a port may be opened for the open-port case.
   Record dev02's host key fingerprint from `ssh-keygen -lf /etc/ssh/ssh_host_ed25519_key.pub`.
2. Start from a clean server user: no `~/.local/share/ahpd`, no `~/.config/systemd/user/ahpd.service`, no `~/.config/ahpd`; and a clean desk: an empty `AHP_DESK_CONFIG_DIR`, no `known_keys.json`, no `ssh-password:` entry in the Windows Credential Manager.
3. Sign-in, each from Add host "A server over SSH", by tunnel: the OpenSSH agent; a key file with a passphrase, no agent running; a password with Remember on, then the desk restarted and the tunnel open again with no question.
   Between runs, the server's install is removed as in step 2.
4. Reach: one install by open port connects from the desk; ahpw in a browser on this machine reaches `ws://dev02:<port>` with the token.
5. The server key: the first connection shows the same fingerprint as step 1, and Trust remembers it; with `known_keys.json` edited to another key, the next connection is refused and names both fingerprints.
6. Node: on dev02 as it is (its own Node, or none), the install uses it or downloads Node 22 into `~/.local/share/ahpd/node`; the other case is covered by task 04's ignored test `ssh_a_real_node_download`, run again on the release build's code.
7. Lingering: with lingering off, the form warns and gives the command; with it on, ahpd is still active after the SSH session ends and the desk quits (`systemctl --user is-active ahpd` from a new login).
8. Update: with `@ahpd/server` pinned one release back, `update_ssh_ahpd` installs the newer one and the host comes back connected, with the same token.
9. The first run on an empty host list reaches an open app window on dev02.
10. The window in en, pt-BR and es at its smallest size, with nothing cut or overlapping.
11. After: remove what the run installed on dev02 (step 2's list), the scratch host list, and the run's keyring entries.

## Validation

- The plan's final verification checklist, each item with what was seen, in this file's Resume.

## Resume
