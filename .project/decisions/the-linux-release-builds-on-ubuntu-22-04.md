---
title: The Linux release builds on ubuntu-22.04
status: accepted
date: 2026-10-14
refs:
  - "[code://.project/plans/desk/01-first-build-p4-tray-and-releases/task-04-release-workflow.md](../../.project/plans/desk/01-first-build-p4-tray-and-releases/task-04-release-workflow.md) - the build matrix"
  - https://v2.tauri.app/distribute/appimage/ - an AppImage runs only where the glibc is as new as the one it was built with
---

## Context

An AppImage links against the glibc of the machine that built it and does not start on a system with an older one.
`ubuntu-latest` is Ubuntu 24.04, whose glibc is newer than that of many distributions still in use.
A pinned older runner looks like a version left behind, and a routine runner bump would move it to `ubuntu-latest`.

## Decision

The release workflow builds the Linux release on the runner `ubuntu-22.04`, not on `ubuntu-latest`.
Source: Softov, asked "p4 q7: which Linux runner builds the AppImage and deb?": "ubuntu-22.04".

## Consequences

The AppImage runs on distributions as old as Ubuntu 22.04.
The runner moves to a newer one only by a deliberate change, when GitHub retires 22.04.

## Options

- **ubuntu-latest (24.04).** Lost: its newer glibc would keep the AppImage off older distributions.
