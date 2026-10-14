---
title: Every update the desk offers works end to end, as a person meets it
status: todo
depends: [task-01-npm-latest-ahpd.md, task-02-host-row-notice-and-update.md, task-03-ahpd-update-command.md, task-04-update-through-ahpd.md, task-05-update-over-ssh.md, task-06-desk-update-notice.md, task-07-release-steps.md]
layer: "src-tauri, packages/shell"
refs:
  - "[code://packages/shell/src/hosts/HostList.tsx](../../../../packages/shell/src/hosts/HostList.tsx) - the row and the list, where every notice is seen"
  - "[code://DEVELOPER.md](../../../../DEVELOPER.md) - `AHP_DESK_CONFIG_DIR`, testing on Windows, and the release steps from task 07"
---

## Objective

On a release build, a person sees each update the desk knows of and can act on each one: a URL host's ahpd through its command, an SSH-installed host's over SSH, a host the desk cannot update with the notice only, and the desk itself on a click.
This checks the plan's final verification checklist.

## Files

None, unless a run finds a defect; each fix goes in the task that owns the code, with a line in that task's Resume.

## Steps

1. Set up, with the desk on a scratch host list (`AHP_DESK_CONFIG_DIR`):
   - a local npm registry holding two ahpd releases that have task 03's command, the older one installed; `npm_config_registry` set for the desk, for the URL host's ahpd and on dev02;
   - a URL host on the older release, added with its deployment token, and the same address added a second time with a person's token;
   - an SSH-installed host on dev02 on the older release (p10), never dev01 or a host in use;
   - the local host from p3, whose updates stay with p3 task 07.
2. Host rows:
   - each older ahpd says "ahpd X available", in English, Portuguese (Brasil) and Spanish;
   - Update on the deployment-token host and on the dev02 host; the notice without Update on the person's-token host;
   - with a session running a turn on a host, Update names it and waits; Cancel leaves it running;
   - each Update ends with the row connected on the newer version and no notice;
   - the local host row shows only p3's own notice.
3. The desk: an installed older signed release, with a newer one published (p4 task 06's pair):
   - the notice appears in the host list and as the tray menu's first item once downloaded, and nothing installs while it is left;
   - a release published while the desk runs is found by task 06's check within 6 hours, with no restart;
   - a click restarts the desk on the newer version, with the new `@ahpd/web` in its app windows when the pin moved (task 07).
4. Look at every row and notice in both the light and the dark theme, at the list's smallest size (480 by 300): nothing overlaps, nothing is cut, and the Update button lines up with the row's other buttons.
5. Tick the plan's final verification checklist.

## Validation

- Each step above passes on Windows (the desk) with dev02 (the SSH host); a step that fails is fixed in its task and run again from that step.
- `pnpm test` and `pnpm lint` pass on the build that ran.

## Resume
