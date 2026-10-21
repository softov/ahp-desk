---
title: A desk update stops the local ahpd the desk started, and the new desk starts it again
status: accepted
date: 2026-10-20
refs:
  - "[code://src-tauri/src/updates.rs](../../src-tauri/src/updates.rs) - `download_desk`'s `on_before_exit`, which runs just before the Windows installer starts"
  - "[code://src-tauri/src/local_ahpd.rs](../../src-tauri/src/local_ahpd.rs) - `start_on_launch` and `stop_host`"
  - "[code://packages/shell/src/hosts/HostList.tsx](../../packages/shell/src/hosts/HostList.tsx) - the desk update's notice and Restart"
---

## Context

An ahpd the desk installed runs on the desk's bundled Node, `<install folder>\node\node.exe` ([a local ahpd is found, or installed with the bundled Node](a-local-ahpd-is-found-or-installed-with-the-bundled-node.md)).
"Stop when the desktop app quits" is off by default, so a desk update's quit leaves that ahpd running.
On Windows a running file cannot be replaced: the NSIS installer stops at "Error opening file for writing ...\ahp-desk\node\node.exe" until ahpd is stopped by hand.
Seen on a Windows 10 VM on 2026-10-20, installing a desk build over a running desk with This computer on.

## Decision

Restart, in the desk update's notice and in the tray, stops each local ahpd the desk started, whatever "Stop when the desktop app quits" says, just before the installer runs.
The new desk starts it again as at any launch, by "Start when the desktop app starts".
When turns run on such a host, Restart names them and waits for a click, as an ahpd update does; the tray's item opens the host list to ask there.
Source: Softov, asked "A desk update can't overwrite the desk's bundled node.exe while the local ahpd is running on it [...] How should the desk handle it?": "Stop ahpd for the install (Recommended)".

## Consequences

A desk update ends the turns running on the local ahpd, after the person confirms.
A local host with "Start when the desktop app starts" off stays stopped after the update, as after any quit that stops it.
An ahpd the desk did not start, such as one on `PATH`, runs on its own Node and is left on.

## Options

- **ahpd runs on its own copy of Node.** Lost: about 80 MB more on disk, and the desk would have to refresh the copy whenever its Node changes.
- **Record it, fix later.** Lost: every desk update on Windows with This computer running would stop at the installer's error.
