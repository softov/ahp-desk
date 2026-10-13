---
title: The remote ahpd runs on the server's own Node 22, or one it downloads
status: accepted
date: 2026-10-13
refs:
  - file:///github/opendoop/packages/tauri/src-tauri/src/installer/systemd_node.rs - doop pushes its staged bundle, Node included, about 80 MB
---

## Context

ahpd needs Node 22 or later.
doop pushes the desk's own Node over SSH, which makes its installer large and wastes the copy for the wrong OS.

## Decision

The installer uses a Node 22 or later already on the server.
Without one, the server downloads the official Node build for its CPU into the install folder; nothing large goes from the desk.
Source: Softov, asked "Where does Node come from on the remote machine?": "Its own Node 22+, else download (Recommended)".

## Consequences

A server with no internet and no Node 22 is refused with that reason.
The desk's installer stays small.

## Options

- **Push the desk's Node.** Lost: the desk carries Linux Node builds it otherwise never uses.
- **Require Node installed.** Lost: a fresh server needs a manual step before the desk can help.
