---
title: Plans index
---

# Plans index

One row per plan; a plan is a folder with `plan.md` and one file per task.
Status: `draft` · `planned` · `active` · `built` · `dropped` (a task: `todo` · `doing` · `done` · `blocked` · `dropped`).
Format and rules: the `do-spec` skill in `.agents/skills/do-spec/`.
What the shell reuses from doop is in [research/doop-tauri-shell.md](../research/doop-tauri-shell.md).

## desk

What exists today, and the first release that plans 01 to 10 build: [00-desk.md](desk/00-desk.md).

| Plan | Status | Next |
| --- | --- | --- |
| [01 - The shell, the host list and the app window](desk/01-shell-and-url-hosts/plan.md) | built | - |
| [02 - ahpw on the desk](desk/02-ahpw-on-the-desk/plan.md) | built | - |
| [03 - The local ahpd](desk/03-local-ahpd/plan.md) | built | - |
| [04 - The tray, the updater and the releases](desk/04-tray-and-releases/plan.md) | built | a release with the Linux fixes waits (deferred.md) |
| [05 - ahpapp in a private build](desk/05-ahpapp-on-the-desk/plan.md) | draft | name and release channel |
| [06 - The bot pill](desk/06-bot-pill/plan.md) | draft | write the tasks |
| [07 - Desk polish before the first release](desk/07-desk-polish/plan.md) | built | - |
| [08 - Each host keeps its own pages on the desk](desk/08-host-scoped-storage/plan.md) | built | - |
| [09 - The desk says what can be updated, and updates it](desk/09-update-alerts/plan.md) | active | task 06's by-hand check on a 0.2.0 or 0.2.1 desk left running while the next release is published; then task 08 |
| [10 - Install ahpd on a Linux server over SSH](desk/10-ssh-install/plan.md) | active | task 13 (a running ahpd found, a free port) by hand; then the pending check |
| [11 - VS Code as a host](desk/11-vscode-hosts/plan.md) | draft | write the task files |
| [12 - A window with every host](desk/12-a-window-with-every-host/plan.md) | built | - |
| [13 - The About screen, the tray's Hosts, and the update of a local ahpd the desk did not install](desk/13-about-and-update-gaps/plan.md) | active | the 0.13.0 Update check once a newer ahpd is on npm |
| [14 - About and the desk's update check in windows of their own](desk/14-about-and-updates-windows/plan.md) | built | Restart check by hand with the release after 0.2.3 |
