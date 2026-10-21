---
title: The SSH probe finds a running ahpd through its daemon.json
status: accepted
date: 2026-10-20
refs:
  - "[code://src-tauri/src/ssh_install.rs](../../src-tauri/src/ssh_install.rs) - `PROBE_SCRIPT`, `parse_probe` and `ahpd_command`"
---

## Context

The probe looks for an ahpd on `PATH`, then through the login shell (`"${SHELL:-sh}" -lc`).
A global npm install often puts its `PATH` line in `.bashrc`, and Debian's `.bashrc` returns at once for a shell that is not interactive, so neither try sees it.
On 2026-10-20 the probe answered "No ahpd" on the test machine while `~/.npm-global/bin/ahpd` was running there, with its `~/.config/ahpd/daemon.json`; the desk then installed a second ahpd over the same config folder.

## Decision

When neither try finds an ahpd, the probe reads the pid in `${XDG_CONFIG_HOME:-$HOME/.config}/ahpd/daemon.json`.
When that pid runs a program named `ahpd`, or one under `@ahpd/server`, the probe takes the Node and the script from `/proc/<pid>/cmdline`, asks them `--version`, and the desk uses that ahpd as it uses one found on `PATH`, running its commands with that Node and script.
Source: Softov, asked "How should the probe find it?": "Read the running daemon (Recommended)".

## Consequences

A running ahpd is found whatever the shell files say.
A stopped ahpd that only an interactive shell finds is still missed, and the desk installs its own.

## Options

- **Try an interactive shell.** Lost: `bash -ic` can print a banner or wait on a prompt, so its output needs fencing, and it runs the user's whole `.bashrc`.
- **Both.** Lost: the interactive shell's cost, for the stopped case only.
