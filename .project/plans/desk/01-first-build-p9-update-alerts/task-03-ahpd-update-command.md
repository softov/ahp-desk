---
title: ahpd has an update command for an administrator, over CLI, HTTP and AHP
status: todo
depends: []
layer: "ahpd (the ahpd repository)"
refs:
  - file:///github/ahpd/packages/server/src/commands/plugin.ts - `plugin update`, at lines 177-216, the shape followed: a `registry.action` with a CLI pattern, an HTTP route, `scopes: ['config:write']`, `deploymentTokenOnly`, `oneAtATime`, and npm run through `install.ts`
  - file:///github/ahpd/packages/server/src/commands/restart.ts - `daemon.restart`, at lines 537-573, the restart an update ends with; `checkedRestart()` at lines 344-372 and `successorTakes()` at lines 303-326, which ask the newly installed binary whether it takes the recorded line
  - file:///github/ahpd/packages/server/src/commands/scopes.ts - `deploymentTokenOnly` refuses any token but the deployment's, at lines 37-47
  - file:///github/ahpd/packages/server/src/commands/registry.ts - `cliRegistry()`, where a new `declareUpdate` is registered, at lines 52-68
  - file:///github/ahpd/packages/server/src/update.ts - `newer()`, `askRegistry()` and `registry()`, which the command reuses for "latest"
  - file:///github/ahpd/packages/server/src/install.ts - `run` and `oneAtATime`, how ahpd runs npm and keeps one change at a time
  - file:///github/ahpd/packages/sdk/src/host/gate.ts - the AHP methods and the grant each needs; `shutdown` needs `config:change`, at line 119
  - file:///github/ahpd/packages/sdk/src/recordschannel.ts - the `x-ahpd/<domain>/<verb>` naming of ahpd's own AHP methods, at lines 240-247
---

## Objective

ahpd has an `update` command that installs a newer `@ahpd/server` where it is installed and restarts itself, for the deployment's token only, reached the same way over the CLI, HTTP and AHP (decision 3).
The desk calls it in task 04 as the AHP method `x-ahpd/update`, on the connection it already holds (decision 7): first with `check`, to know whether to show Update (decision 6), then with the `version` the row named (decision 8).
This task is ahpd work: it is handed to the ahpd repository, which writes it as its own plan; nothing in this repository changes for it, and no file in the ahpd repository is written from here.
It is done when an ahpd release has the command; the release version is then written in this file's Resume and in task 04.

## Files

None in this repository.
What the ahpd plan is asked for is below; the files it touches are its own to name (most likely a new `packages/server/src/commands/update.ts` with `declareUpdate`, registered in `commands/registry.ts`, and its tests in `packages/server/test/`).

## Steps

1. Hand this file to the ahpd repository as a plan request, with the shape below, and link the ahpd plan here once written.
2. The shape the desk needs, following `plugin update`:

   | | |
   | --- | --- |
   | id | `daemon.update` |
   | CLI | `ahpd update [--version <X.Y.Z>] [--check] [--force]` |
   | HTTP | `POST /api/update` with `{ "version"?: string, "check"?: boolean, "force"?: boolean }` |
   | AHP | the method `x-ahpd/update` with the same params, answering the same JSON as HTTP; this is the surface the desk calls (decision 7) |
   | scopes | `['config:write']` |
   | meta | `deploymentTokenOnly: 'update the daemon'`; over AHP the same rule holds: only the deployment's token, a person's token is refused whatever its grants |
   | effect | `change` |

3. What it does:
   - `version` is the version installed; the desk always passes it, the version its row named (decision 8).
     `version` absent, as from the CLI by hand, means npm's `latest` of `@ahpd/server`, asked through `registry()`.
     A `version` not newer than the running one is refused with why.
   - While a turn runs it is refused, naming the sessions, as `ahpd restart` is, unless `force`.
   - It installs `@ahpd/server@<version>` with npm where this ahpd is installed, then restarts as `daemon.restart` does, so the new binary is asked whether it takes the recorded line before anything goes down.
   - It answers before the restart: `{ "from": "0.11.0", "to": "0.12.0", "restarting": true }`.
     A client sees its connection drop, and the host come back on the new version.
   - It runs one at a time with the plugin commands (`oneAtATime`).
4. What the desk needs to know before it shows Update, answered without changing anything: with `check`, the command answers `{ "current": "0.11.0", "latest": "0.12.0", "may": true }`, or the refusal a real update would get (a person's token, an install it cannot update, a daemon it cannot restart) (decision 6).
5. Left to the ahpd plan, which decides them in its own records:
   - how ahpd finds where it is installed: a `--prefix` install as the desk's `<app local data>/ahpd` and p10's `~/.local/share/ahpd`, a global `npm i -g`, or a checkout or link, which it refuses;
   - what happens to installed plugins whose `@ahpd/sdk` range does not admit the new version, as `plugin update` asks for each;
   - a daemon not started by `ahpd start` (in the foreground, or under a service manager), which `daemon.restart` refuses today;
   - the update on Windows, where the running files are in use.

## Validation

- In the ahpd repository, by its plan: tests over CLI, HTTP and AHP for a person's token refused, the deployment's token accepted, a running turn refused without `force`, `check` changing nothing, and an install from a local registry followed by a restart on the new version.
- Here, once released: `ahpd update --check --json` on a host started with `ahpd start` answers `may: true`; `POST /api/update` with a person's token answers 403; `x-ahpd/update` with `check` over AHP answers as the CLI does; the desk's task 04 runs against that release.

## Resume
