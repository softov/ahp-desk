---
title: The host list publishes each host's running turns to every window
status: accepted
date: 2026-10-27
refs:
  - "[code://packages/shell/src/hosts/status.ts#L162-L172](../../packages/shell/src/hosts/status.ts#L162-L172) - `publish`, which sends only each host's dot"
  - "[code://packages/shell/src/hosts/status.ts#L246-L253](../../packages/shell/src/hosts/status.ts#L246-L253) - `running`, the titles of a host's chats with a turn on"
  - "[code://src-tauri/src/statuses.rs](../../src-tauri/src/statuses.rs) - the relay a second map follows"
  - "[code://packages/ahpw-embedder/src/embedder.ts#L54-L57](../../packages/ahpw-embedder/src/embedder.ts#L54-L57) - ahpw reads `hosts-status` as a map of dots, so its shape stays"
---

## Context

Restart of a desk update stops each local ahpd the desk started, and first names the turns running there.
Only the host list window knows the turns: it holds the one connection per host ([the host list window publishes each host's status](the-host-list-window-publishes-host-status.md)), and what it publishes is only connected, connecting or down.
The Updates window ([the desk's update check is a window of its own](the-desks-update-check-is-a-window-of-its-own.md)) needs the turns to ask the question where the person clicked.

## Decision

The host list publishes each host's running turns, the titles of its chats with a turn on, as the event `hosts-running`, and the shell keeps the last map for a window that opens later, as it does for the statuses.
`hosts-status` keeps its shape, since ahpw's embedder reads it.
The Updates window asks Restart's question itself, from that map.
Source: Softov, 2026-10-27, asked "Where does Restart's 'turns are running' question show, from the Updates window?": "In the Updates window (Recommended)".

## Consequences

A second map rides beside the statuses, sent only when it changes.
The titles of running chats reach every window the desk opens; they already reach ahpw's windows through each host's own connection.
Like the statuses, the map exists only while the host list window runs, which is never closed, only hidden.

## Options

- **Hand off to the host list.** Restart in the Updates window shows the host list and asks there, as the tray's item did. Lost: the question shows in another window than the click.
- **Add the turns to `hosts-status`.** Lost: it changes the shape ahpw's embedder reads.
