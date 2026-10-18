---
title: Install over SSH works end to end on the test machine
status: todo
depends: [task-01-ssh-session.md, task-02-server-key.md, task-03-probe.md, task-04-node.md, task-05-install.md, task-06-reach.md, task-07-add-host-ssh.md, task-08-update-over-ssh.md, task-10-uninstall-over-ssh.md]
layer: "ahp-desk"
refs:
  - "[code://src-tauri/src/lib.rs#L31-L37](../../../../src-tauri/src/lib.rs#L31-L37) - `AHP_DESK_CONFIG_DIR`, so the run uses a scratch host list and never the user's"
---

## Objective

Every item of the plan's final checklist holds on a release build of the desk on this Windows machine, against the test machine.
The test machine is the only server used; never another host (the plan's Risks).

## Files

- None.

## Steps

1. Before: ask Softov for what the test machine needs and the run cannot set itself: a user for the run, whether password sign-in is on in its `sshd`, whether lingering may be turned on for that user, whether a port may be opened for the open-port case, and whether an ahpd may be installed by hand there for step 7.
   Record the test machine's host key fingerprint from `ssh-keygen -lf /etc/ssh/ssh_host_ed25519_key.pub`.
2. Start from a clean server user: no `~/.local/share/ahpd`, no `~/.config/systemd/user/ahpd.service`, no `~/.config/ahpd`, no `ahpd` on `PATH`; and a clean desk: an empty `AHP_DESK_CONFIG_DIR`, no `known_keys.json`, no `ssh-password:` entry in the Windows Credential Manager.
3. Sign-in, each from Add host "A server over SSH", by tunnel: the OpenSSH agent; a key file with a passphrase, no agent running; a password with Remember on.
   With the desk restarted: the password host's tunnel opens again with no question, on the same local port, and its app window keeps its tabs (Answers: secrets held until quit, and the local port); the passphrase host shows "Sign in" on its row, and opens once the passphrase is typed (Answers: secrets held until quit).
   Between runs, the server's install is removed with task 10's Remove and uninstall.
4. Reach: one install by open port, on an ahpd port other than 9187, connects from the desk (Answers: the ahpd port); ahpw in a browser on this machine reaches `ws://<test machine>:<port>` with the token.
5. Folder: one install with a folder other than the home serves it: `ahpd status --json` on the test machine names it in `paths`, and the app window's explorer opens on it (Answers: the folder).
6. The server key: the first connection is refused and shows the same fingerprint as step 1, and Trust connects (Answers: a new server key); with `known_keys.json` edited to another key, the next connection is refused and names both fingerprints, and "Forget the old key" leads back to the trust question (Answers: a changed server key).
7. A found ahpd (Answers: a found ahpd): with an install from step 3 left in place, a second Add host on the test machine finds it, offers only the backends it lacks, and connects with nothing installed beside it.
   With an ahpd installed by hand on the test machine instead, if Softov allowed it in step 1, Add host finds it, shows its version, adds a backend ticked through its own `plugin install`, and connects through a tunnel on its own port (Answers: how a found ahpd is reached); with that ahpd stopped first, Add host says it will not survive a reboot, starts it with no unit written, and connects (Answers: a found ahpd that is off).
8. Node: on the test machine as it is (its own Node, or none), the install uses it or downloads the newest Node 22 from `latest-v22.x` into `~/.local/share/ahpd/node` and shows its version (Answers: the newest Node 22); the other case is covered by task 04's ignored test `ssh_a_real_node_download`, run again on the release build's code.
9. Lingering: with lingering off, the form warns and gives the command; with it on, ahpd is still active after the SSH session ends and the desk quits (`systemctl --user is-active ahpd` from a new login).
10. Update: with `@ahpd/server` pinned one release back, `update_ssh_ahpd` installs the newer one and the host comes back connected, with the same token, port and local port.
11. Remove (Answers: the uninstall): "Remove and uninstall" runs the steps, and the test machine has no unit, no `~/.local/share/ahpd` and nothing listening on the ahpd port; "Remove from the desk only" leaves the server's ahpd running, which is then removed by hand.
    Remove on a host whose ahpd the desk did not install offers no uninstall, says the server's ahpd stays, and leaves it running (Answers: a host the desk did not install).
    With the test machine's SSH port blocked from this machine, "Remove and uninstall" shows why, with "Remove from the desk only" and Cancel, and Cancel keeps the host (Answers: an unreachable server).
    After any uninstall, `~/.config/ahpd` on the test machine is as it was (Answers: the config folder on uninstall).
12. The first run on an empty host list reaches an open app window on the test machine.
13. The window in en, pt-BR and es at its smallest size, with nothing cut or overlapping, including the trust question, the changed-key refusal, the Remember checkbox's two wordings and the Remove question.
14. After: remove what the run installed on the test machine (step 2's list), the scratch host list, and the run's keyring entries.

## Validation

- The plan's final verification checklist, each item with what was seen, in this file's Resume.

## Resume
