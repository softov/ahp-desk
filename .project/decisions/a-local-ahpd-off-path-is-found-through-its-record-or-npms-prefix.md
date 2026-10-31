---
title: A local ahpd that PATH misses is found through its record, else through npm's prefix, on Linux and macOS
status: accepted
date: 2026-10-31
refs:
  - "[code://src-tauri/src/local_ahpd.rs#L414-L440](../../src-tauri/src/local_ahpd.rs#L414-L440) - `find`: the set path, the desk's install, then `PATH`"
  - "[code://src-tauri/src/ssh_install.rs#L89-L91](../../src-tauri/src/ssh_install.rs#L89-L91) - the SSH probe's login shell try, then the record"
  - file:///github/ahpd/scripts/install/install.sh - moves npm to `~/.npm-global` and writes its `PATH` line into the shell's rc file
---

## Context

A desktop session does not read `~/.bashrc`, and a login shell reaches it only to hit Debian's early return for a shell that is not interactive.
An ahpd installed by ahpd's install script under `~/.npm-global`, or by any npm whose `PATH` line is in an rc file, is then not on the desk's `PATH`.
On 2026-10-30, on the Debian test box, the desk said "Not on this computer" while that ahpd 0.13.0 ran on 9187, and offered to install a second one over the same config folder.
Plan 10 task 13 solved the same case over SSH through the running ahpd's record ([decision](the-probe-finds-a-running-ahpd-through-its-record.md)).

## Decision

On Linux and macOS, after the set path, the desk's install and `PATH`, the desk tries two more places.
First the record: the pid in ahpd's `daemon.json`, in the config folder the desk already reads; when `ps -o args= -p <pid>` shows a Node running a script that is `ahpd` or under `@ahpd/server`, that Node and script are the ahpd.
Then npm's prefix: the `prefix` line of `~/.npmrc`, and `<prefix>/bin/ahpd` when it is a file; this finds one that is stopped too.
Windows is left as it is, because npm's global folder there is on the user `PATH` that every app gets.
Source: Softov, 2026-10-31, asked "When the desk's PATH has no ahpd, how should it find a local ahpd it did not install?": "Record + npm prefix (Recommended)"; and asked "Which platforms?": "Linux and macOS (Recommended)".

## Consequences

A running ahpd is found wherever it was installed; a stopped one is found when npm's prefix names it.
A stopped ahpd under nvm, or under a prefix set only in the environment, stays unseen.
`ps` joins the arguments with spaces, so a Node or script path with a space in it is not read.

## Options

- **The record only.** Lost: a global ahpd that is stopped stays unseen, and the desk offers a second install over its config.
- **An interactive login shell (`$SHELL -ilc`).** Lost: it runs the person's whole rc file, which can print, prompt or hang.
- **`/proc/<pid>/cmdline`, Linux only.** Lost: macOS has no `/proc`; `ps` serves both.
