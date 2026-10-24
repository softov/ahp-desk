---
title: The tray, the updater and the releases - deferred
date: 2026-10-23
---

## A release with the Linux fixes

- **What waits:** a release after v0.2.1 that carries `is_ui`, `tray::shown` and `pointer-away` from the branch `desk/no-tray-quits`.
- **Why:** v0.2.1's AppImage and deb open no window for a host on Linux, and on stock GNOME they hide to a tray that is not there.
  The fixes were checked on a local deb build, not on a release.
- **Where it goes:** Softov bumps the version and tags it, as for v0.2.1 ([DEVELOPER.md](../../../../DEVELOPER.md)); the release is also the newer release that plan 09 task 06's 6-hour check waits for.
