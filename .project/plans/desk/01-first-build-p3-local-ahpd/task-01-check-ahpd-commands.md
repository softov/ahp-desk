---
title: ahpd's start, stop and status are checked against the desk's needs
status: done
depends: []
layer: "ahpd (read only)"
refs:
  - file:///github/ahpd/packages/server/src/commands/start.ts - `ahpd start` forwards the typed line to a detached child, and asks nothing with no TTY
  - file:///github/ahpd/packages/server/src/commands/status.ts - `status --json`, with pid, url, paths, startedAt and automations, or `{"running":false}`
  - file:///github/ahpd/packages/server/src/daemon.ts - daemon.json, whose `connectUrl` holds the token; `stop()` is `process.kill(pid, 'SIGTERM')`
  - file:///github/ahpd/packages/server/src/commands/run.ts - `way.shutdown`, the graceful stop, wired only to SIGINT and SIGTERM
  - file:///github/ahpd/packages/sdk/src/host/vscodemethods.ts - the AHP `shutdown` method, which ends in the same signal to itself
  - file:///github/ahpd/packages/server/src/plugins.ts - plugins resolve from the config folder, then from `AHPD_PLUGIN_ROOT`
---

## Objective

What the desk can rely on in `ahpd start`, `stop` and `status` is known, and what it cannot is raised.

## Files

- None.

## Steps

1. Read the three commands and the daemon record in ahpd 61feca58.
2. Run `@ahpd/server` 0.11.0 from npm as the desk would: `ahpd start --port --connection-token-file --path --config-file` with no TTY, then `status --json`, then `stop`.
3. Check what Node does with SIGTERM on Windows.

## Validation

- The run in step 2, on a throwaway config folder; nothing of the user's was touched.

## Resume

Done 2026-10-13 on `@ahpd/server` 0.11.0.

- **Start works for the desk.**
  `ahpd start` passes `--port`, `--connection-token-file`, `--path` and `--config-file` to the daemon, asks nothing with no TTY, and prints `ahpd on ws://127.0.0.1:<port> (pid <n>)`.
  ahpd writes a missing token file itself.
  With no backend loaded it exits 1 and says why in `daemon.log`, so the desk shows that log's tail on a failed start.
- **Status works for the desk.**
  `status --json` gives `pid`, `url`, `paths`, `startedAt` and `automations`, never the token; with none on, `{"running":false}` and exit 1.
  The token is in `daemon.json`'s `connectUrl`, one record per config folder.
- **Stop is not graceful on Windows.**
  `ahpd stop` sends SIGTERM, which Node on Windows turns into TerminateProcess: no handler runs, so `way.shutdown` (`stopping`, the plugin connections, the listeners, `host.close()`) is skipped.
  The AHP `shutdown` method ends in `process.kill(process.pid, 'SIGTERM')`, the same hard exit.
  Checked: a Node process with a SIGTERM handler that signals itself exits 1 with the handler not run, and the stopped daemon wrote nothing to its log.
  The decision [Stopping the local ahpd on quit is a host setting](../../../decisions/stopping-the-local-ahpd-on-quit-is-a-host-setting.md) says a stop on quit must be graceful, so this is an ahpd change, needed before task 04's stop.
- **Plugins resolve from the config folder.**
  A bare plugin name is looked up in `<config folder>/node_modules`, then in `$AHPD_PLUGIN_ROOT/node_modules`, and `ahpd plugin install` writes to `AHPD_PLUGIN_ROOT` when it is set.
  So an ahpd the desk installs into its own folder finds its backends only with `AHPD_PLUGIN_ROOT` set to that folder, or with a config folder of its own.
