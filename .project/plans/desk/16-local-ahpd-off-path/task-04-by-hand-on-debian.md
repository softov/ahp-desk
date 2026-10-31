---
title: The finding and the free port work by hand on the Debian test box
status: todo
depends:
  - task-01-find-through-the-record-and-npms-prefix.md
  - task-02-add-host-names-the-new-origins.md
  - task-03-a-local-install-starts-on-a-free-port.md
layer: "by hand"
refs:
  - "[code://DEVELOPER.md](../../../../DEVELOPER.md) - the Linux toolchain and `pnpm tauri build --debug`"
---

## Objective

On the Debian test box, with a debug deb of this plan's branch, the three cases of the plan behave as its decisions say, and plan 13's last check passes.

## Files

- none.

## Steps

1. Build the debug deb on the box (`pnpm tauri build --debug --bundles deb`) and install it.
2. With ahpd's install script's ahpd running from `~/.npm-global` and no desk data: This computer shows "ahpd <version>, running", offers no install, and saving it connects to that ahpd.
3. Stop that ahpd (`ahpd stop`) and check again: "ahpd <version>, in npm's global folder"; saving it starts it, and the row says the desk did not install it.
4. Plan 13's check: with that ahpd on 0.13.0, its row shows Update; running it updates it to the newest on npm.
5. Move `~/.config/ahpd` aside, stop every ahpd, hold 9187 with `python3 -m http.server 9187 --bind 127.0.0.1`, and add This computer: the form says ahpd will listen on 9188; the install starts it there; quit and start the desk, and it starts on 9188 again.
6. Put the box back: stop the test server, the desk's ahpd and the desk; restore the moved folders, the release deb and the global ahpd.

## Validation

- Each step's result written in this task's Resume, with the screenshots' paths.

## Resume

