---
title: The SSH installer puts ahpd on Linux as a user systemd service
status: accepted
date: 2026-10-13
refs:
  - file:///github/opendoop/packages/tauri/src-tauri/src/installer/systemd_node.rs - doop's user-level systemd install, the pattern
  - file:///github/opendoop/packages/tauri/src-tauri/src/ssh/mod.rs - doop's SSH session over `russh`
---

## Context

The first milestone left SSH hosts out; a person with a server, as dev01, installs ahpd there by hand.
doop installs its daemon over SSH as a Linux user service, with no sudo.

## Decision

The desk installs ahpd over SSH on Linux only, under the signed-in user, as a `systemctl --user` service, with no sudo.
macOS, other Unixes and Windows come later, each as its own installer.
Source: Softov, asked "Which machines should the SSH installer handle first?": "Linux, user systemd (Recommended)".

## Consequences

A server without `systemctl --user` is refused with that reason.
A user service stops at logout unless lingering is on; the installer checks and says how to turn it on, since that needs an administrator.

## Options

- **Linux and macOS.** Lost: launchd is a second installer to build and test now.
- **Linux, macOS and Windows.** Lost: a Windows service over OpenSSH is the most work, for the fewest servers.
