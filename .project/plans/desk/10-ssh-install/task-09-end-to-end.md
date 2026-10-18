---
title: Install over SSH works end to end on the test machine
status: done
depends: [task-01-ssh-session.md, task-02-server-key.md, task-03-probe.md, task-04-node.md, task-05-install.md, task-06-reach.md, task-07-add-host-ssh.md, task-08-update-over-ssh.md, task-10-uninstall-over-ssh.md, task-11-refused-key-on-a-saved-host.md]
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

- 2026-10-18: run on release builds of `d2121b3` and the two fixes below, on this Windows machine, against the test machine Softov named, with a scratch `AHP_DESK_CONFIG_DIR` and home; Softov allowed every sign-in case, a port opened briefly, an ahpd installed by hand and lingering on, and picked each key file in the dialog.
- Step 1: the host key's fingerprint was recorded from `ssh-keygen -lf`; `sshd -T` showed password sign-in on; lingering off; passwordless `sudo`.
- Step 2: the server user had no desk install, unit, `~/.config/ahpd` or `ahpd` on `PATH` (an older config moved aside before the plan stayed untouched); no `ssh-password:` entry in the Credential Manager.
- Step 3: key file, the OpenSSH agent, a password with Remember, and a key file with a passphrase each installed by tunnel and connected; each was removed with Remove and uninstall before the next.
  After a restart, the agent and password hosts reopened on the same local port with no question, and the password host's app window kept its editor tab; the passphrase host showed Sign in and opened once the passphrase was typed.
  The password was kept as `ssh-password:<id>` in the Credential Manager, and removing the host deleted it.
- Step 4: an install by open port on 9287 connected from the desk, and ahpw, from its dev server proxying to `ws://<test machine>:9287`, in a headless browser with the token, showed it connected.
- Step 5: the folder typed as `~/e2e-folder` was saved as the absolute path, `paths` in the server's ahpd config named it, and the app window's explorer opened on it.
- Step 6: the first connection was refused with the recorded fingerprint and Trust connected; with `known_keys.json` changed to another key, the row read "key changed", Edit host named both fingerprints, and "Forget the old key" led back to Trust.
- Step 7: with an install left in place, a second Add host by the agent found it, offered only the backends it lacked with Claude fixed, and connected with nothing installed.
  An ahpd installed by hand with its own Node and `@ahpd/agent-pi` was found on the login `PATH`, shown with its version, given Claude and, on a later run, cofold through its own `plugin install`, and reached by tunnel on its own port 9187; stopped first, Add host said it would not come back after a restart, started it with its own `ahpd start`, and wrote no unit.
- Step 8: the server has no Node, so each install downloaded the newest 22 (v22.23.3) and showed it; `ssh_a_real_node_download` passed again.
- Step 9: with lingering off, the form warned with `sudo loginctl enable-linger <user>`; turned on, the warning was gone and ahpd stayed active.
  The difference after the last session ends was not shown: the user has long-lived sessions of their own on the server, which keep the user manager running either way, and they were left alone.
- Step 10: the row's Update is plan 09's task 05; `ssh_a_real_update` passed on this code: newest, 0.11.0, newest, each active with the token file unchanged.
- Step 11: four Remove and uninstall runs each left no unit, no install and nothing on the ahpd port, with `~/.config/ahpd` as the install left it; "Remove from the desk only" left the open-port ahpd active and forgot the host's token; Remove on the hand-installed ahpd offered no uninstall, said it stays, and left it running.
  With the SSH port closed (the host pointed at a port nothing listens on), Remove and uninstall showed the failed Sign in and why, with Cancel, which kept the host, and "Remove from the desk only".
- Step 12: the first run on an empty host list ended with the app window open on the test machine, every time.
- Step 13: the form, the trust question, the changed-key refusal, the row's key state and Check key, and the Remove questions in en, pt-BR and es at 480x300, none cut or overlapping; the row's state words end in an ellipsis at that width, as the other states do.
  Not run: the Remember checkbox's no-keyring wording, since `ssh_keyring_works` cannot be patched in the page; its words are checked by `i18n.test.ts`.
- Step 14: the run's installs, the hand ahpd and its `~/.profile` line, `~/e2e-folder`, the passphrase key in `authorized_keys` and lingering were put back as before; the scratch host list, profiles and throwaway keys were removed; no entry of the run is left in the Credential Manager.
- Found and fixed in the run: the Folder field takes `~` and `~/` paths, turned into the server's home, and refuses a relative folder (`serverFolder()` in `url.ts`, Softov's ask); a long step on an ahpd the desk did not install kept its pid file in `~/.local/share/ahpd`, which created the desk's folder beside that ahpd, and now uses `~/.cache/ahp-desk`.
- Still open: a terminal `ahpd status` does not see the ahpd systemd runs (it answered "None running" during the run), the plan's pending check, waiting on ahpd.
