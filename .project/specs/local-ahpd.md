---
title: The local ahpd
status: current
covers:
  - src-tauri/src/local_ahpd.rs
  - packages/shell/src/hosts/LocalForm.tsx
  - scripts/stage-node.mjs
decisions:
  - decisions/a-local-ahpd-is-found-or-installed-with-the-bundled-node.md
  - decisions/the-installed-ahpd-keeps-the-usual-config-folder.md
  - decisions/the-desk-adds-backends-through-ahpd-plugin-install.md
  - decisions/an-ahpd-too-old-on-path-is-shown-not-replaced.md
  - decisions/the-folder-is-written-by-an-ahpd-command.md
  - decisions/the-local-ahpd-runs-through-its-own-start-and-stop.md
refs:
  - "[code://src-tauri/src/local_ahpd.rs#L370](../../src-tauri/src/local_ahpd.rs#L370) - `find`, the set path, the desk's install, then `PATH`"
  - "[code://src-tauri/src/local_ahpd.rs#L173](../../src-tauri/src/local_ahpd.rs#L173) - `LocalAhpd::install`, npm into the prefix, then the backends"
  - "[code://src-tauri/src/local_ahpd.rs#L213](../../src-tauri/src/local_ahpd.rs#L213) - `add_backends`, ahpd's own `plugin install`"
  - "[code://src-tauri/src/local_ahpd.rs#L233](../../src-tauri/src/local_ahpd.rs#L233) - `serve_folder`, ahpd's own `paths add`"
  - "[code://src-tauri/src/local_ahpd.rs#L465](../../src-tauri/src/local_ahpd.rs#L465) - `status`, `ahpd status --json` and the token from the record"
  - "[code://src-tauri/src/local_ahpd.rs#L506](../../src-tauri/src/local_ahpd.rs#L506) - `start`, with the desk's token file"
  - "[code://src-tauri/src/local_ahpd.rs#L741](../../src-tauri/src/local_ahpd.rs#L741) - `start_host` and `stop_host`, which stops only an ahpd the desk started"
  - "[code://src-tauri/src/local_ahpd.rs#L786](../../src-tauri/src/local_ahpd.rs#L786) - `start_on_launch` and `stop_on_quit`"
  - "[code://src-tauri/src/local_ahpd.rs#L678](../../src-tauri/src/local_ahpd.rs#L678) - `launcher_for`, which refuses an ahpd older than 0.8.0"
  - "[code://packages/shell/src/hosts/LocalForm.tsx#L147](../../packages/shell/src/hosts/LocalForm.tsx#L147) - Add host's save: install or add backends, write the folder, save, start"
  - git://4c89294 - the deleted decisions this spec was written from: a-found-ahpd-with-no-backend-is-offered-the-backends, stopping-the-local-ahpd-on-quit-is-a-host-setting, the-backends-are-picked-in-add-host, the-local-ahpd-serves-the-folder-asked-in-add-host-else-home, the-local-ahpd-uses-a-token-file
---

The local ahpd is the ahpd on this computer, shown as the host "This computer".
The desk uses an ahpd it finds, or installs one with the Node 22 its installer ships, with the backends the person picks.
It drives that ahpd only through ahpd's own commands: `status --json`, `start`, `stop`, `plugin install` and `paths add`.
A desk has at most one local host.
The code is in [`code://src-tauri/src/local_ahpd.rs`](../../src-tauri/src/local_ahpd.rs) and [`code://packages/shell/src/hosts/LocalForm.tsx`](../../packages/shell/src/hosts/LocalForm.tsx).

## Scenarios

### Which ahpd the desk uses

WHEN the desk looks for the local ahpd
THEN it takes the path set on the host, else the ahpd it installed, else the first `ahpd` on `PATH`
AND a set path that is not there is an error, never a reason to use another.

### An ahpd too old is shown, not replaced

WHEN the ahpd found is older than 0.8.0
THEN Add host shows its version, the version needed, `npm i -g @ahpd/server` and Check again
AND the desk installs nothing beside it.

### The desk installs one

WHEN no ahpd is found and Add host is saved
THEN the desk installs `@ahpd/server@latest` with the bundled npm into `<app local data>/ahpd`, an npm prefix
AND then the picked backends with that ahpd's `plugin install`, with `AHPD_PLUGIN_ROOT` set to the prefix
AND npm's lines show as the install runs
AND a build with no bundled Node says so and installs nothing.

### The backends are picked in Add host

WHEN Add host shows This computer
THEN it offers Claude, cofold and pi as backends, and the bot plugin, with Claude ticked
AND for a found ahpd, the plugins its `plugin list` has ready are ticked and fixed, and those ticked here are added with its own `plugin install`
AND Save and start needs at least one backend, because ahpd does not start with none.

### The folder to serve

WHEN Add host is saved
THEN the folder it shows, the home folder until Change picks another, is written with `ahpd paths add <folder>` before the first start
AND a folder already served, or under one that is, is left as it is
AND an ahpd whose config names no folder is started in the home folder.

### Start, find and stop

WHEN the desk starts the local ahpd
THEN it asks `ahpd status --json` first, and takes an ahpd already on, which it then did not start
AND otherwise it runs `ahpd start --connection-token-file <prefix>/connection-token`, and ahpd writes a token there when there is none
AND the address comes from `status --json`, and the token from ahpd's record when the record names the same process
AND the desk stops only an ahpd it started, with `ahpd stop`.

### The desk starts and quits

WHEN the desk starts
THEN it starts the local ahpd when "Start when the desktop app starts" is on, which it is by default
AND when the desk quits, it stops that ahpd only when the desk started it
AND only when "Stop when the desktop app quits" is on, which it is not by default.

## Known gaps

- The installed ahpd is not updated yet; plan 03 task 07 waits on ahpd's own update command (plan 09 task 03).
- Not checked end to end on a clean Windows user (plan 03 task 08).
