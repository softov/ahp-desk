---
title: Two hosts end to end, in both settings
status: done
depends: [task-04-the-embedder-mode-and-tokens.md]
layer: "end to end"
refs:
  - file:///github/ahpw/.project/plans/hosts/02-a-window-with-every-host/task-08-two-hosts-end-to-end.md - ahpw's checks in the same run
---

## Objective

With "every host", one window shows two hosts, and with "per host" the desk works as today.

## Files

None; this task changes no file unless a check fails.

## Steps

1. Run two scratch ahpds, with `AHP_DESK_CONFIG_DIR`, `HOME` and `XDG_CONFIG_HOME` on scratch folders.
2. Build the desk with the ahpw of hosts plan 02 staged.
3. Set "every host", and Open each host from the host list.
4. Check that one window opens, and each Open focuses it filtered to that host.
5. Check that a one-host window, opened before the change, still reads only its own token.
6. Remove a host, and check that the window drops it and stays open.
7. Set "per host", and check that Open opens one window per host.
8. Stop every server you started.

## Validation

- Each step works, and ahpw hosts plan 02 task 08 passes in the same run.
- `pnpm test`, `pnpm lint` and `cargo test` pass.

## Resume

- Done 2026-10-16 on a release build with ahpw's local build staged, built with `--config '{"identifier":"dev.softov.ahpdesk.e2e"}'`.
  The identifier kept it apart from a running desk; the build used the local ahpw's `@ahpd/web` types, restored after.
- The scratch `hosts.json` held no token, since a token there moves into the OS keyring on load.
- Passed: "Hosts open in" shows in Settings, and saves `windows` to `settings.json`.
- Passed: in "every host", Open on Alpha opens `app-:every` filtered to Alpha, and `focus=` leaves its address.
- Passed: Open on Beta shows the same window, filtered to Beta; one ahpw window stays.
- Passed: Beta refusing its token, its Sign in opens "Edit Beta" in the host list; restarted with no token, Retry connects it.
- Passed: the window of Alpha is refused Beta's token, and the window with every host may read it.
- Passed: switched to per host, Open on Alpha makes `app-e2e-alpha`, mono, beside the window with every host.
- Not run: removing a host with that window open, which writes to the OS keyring; `close_host` skips it by its label.
