---
title: ahpw shows one host at a time on the desk
status: accepted
date: 2026-10-11
refs:
  - file:///github/ahpw/src/connection/data.ts - one host id, `HOST = 'daemon'` at line 59, and store paths with no host key
  - file:///github/ahp-desk/mockups/03-app-window.md - the host switcher in the title bar
---

## Context

ahpw uses the SDK's `MultiHostClient`, but with one fixed host id.
Its store paths have no host key, so two hosts would write over each other.
Real support for many hosts means a host id through every call that follows, requests or dispatches.

## Decision

On the desk, an app window shows ahpw on one host.
The host switcher in the title bar moves the window to another host.
ahpw with many hosts is a later plan in ahpw.
Source: Softov, asked "In ahp-desk, what does ahpw do with the shell's host list in this plan?": "One host at a time (Recommended)".

## Consequences

The change in ahpw is small: the `connect()` seam, the sign-in, and the title bar.
A switch of host restarts ahpw's connection.
Mockup 03's "many hosts" window is ahpapp's, until ahpw has its own plan for it.

## Options

- **Many hosts now.** ahpw keys its store by host. Lost: a large change inside ahpw before the desk exists.
