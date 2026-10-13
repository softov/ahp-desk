---
title: The first build of ahp-desk
domain: desk
status: active
priority: high
created: 2026-10-11
revalidated: 2026-10-11
decisions:
  - decisions/a-ui-takes-its-hosts-from-a-command-and-an-event.md
  - decisions/ahpw-owns-its-embedding-contract.md
  - decisions/ahpw-shows-one-host-at-a-time-on-the-desk.md
  - decisions/a-ui-bundle-comes-from-the-local-repository-or-from-npm.md
  - decisions/a-local-ahpd-is-found-or-installed-with-the-bundled-node.md
  - decisions/the-backends-are-picked-in-add-host.md
  - decisions/stopping-the-local-ahpd-on-quit-is-a-host-setting.md
  - decisions/the-pill-draws-bood-in-the-dom-from-the-bood-data.md
  - decisions/the-picker-lists-the-bots-of-every-host.md
  - decisions/the-pill-with-no-bot-is-a-setting.md
  - decisions/the-bot-shows-its-session-state.md
  - decisions/tokens-are-in-the-os-keyring-with-hosts-json-as-the-fallback.md
  - decisions/each-ui-is-served-from-its-own-uri-scheme.md
  - decisions/the-local-ahpd-runs-through-its-own-start-and-stop.md
  - decisions/the-local-ahpd-uses-a-token-file.md
  - decisions/ahpd-updates-itself-when-the-first-install-says-so.md
  - decisions/the-desk-starts-with-the-os-by-default.md
  - decisions/the-bood-data-lives-in-cofold.md
  - decisions/a-drag-moves-the-cluster-and-alt-drag-takes-a-bot-out.md
  - decisions/a-cluster-holds-four-bots.md
  - decisions/the-pill-menu-opens-the-hosts-and-the-settings.md
  - decisions/each-desk-build-ships-one-ui.md
  - decisions/ahp-desk-is-mit-and-ahpapp-stays-private.md
  - decisions/the-bot-is-the-pill.md
  - decisions/a-bot-is-talked-to-from-an-input-on-hover.md
refs:
  - file:///github/ahp-desk/mockups/README.md - the sketches that this plan builds
  - file:///github/ahp-desk/.project/research/doop-tauri-shell.md - what the shell reuses from doop
  - file:///github/opendoop/packages/tauri/src-tauri - doop's Tauri shell, the pattern for windows, tray, updater and installer
  - file:///github/ahpw/src/connection/data.ts - ahpw's one connection seam, at lines 325-335
  - file:///github/ahpapp/src/host-context.tsx - ahpapp's host list and its MultiHostClient
  - file:///github/ahpapp/src/bood.ts - the bood data for the pill
  - file:///github/ahpd/packages/server/src/daemon.ts - ahpd start, stop and status
  - file:///github/ahpd/packages/bot - the bot plugin, its records and the x-ahpd-bots channel
---

## Goal

ahp-desk becomes a desktop app that keeps a list of AHP hosts and opens ahpw on any of them.
It can find, install, start and stop an ahpd on this computer.
It lives in the tray, updates itself, and ships as an installer.
A private build of the same app ships ahpapp, with little work, because both UIs use one contract.
A bot from any host floats on the desktop as the pill, shows what its session does, and takes a message.

## Reconnaissance

The files read and the patterns to reuse are the `refs` above, each with its note.
The child plans hold the file and line detail.

### Searches performed

- `rg "window.location|localStorage" ahpw/src` - only `connection/data.ts:329-330` and `api.ts` build a host from the page.
- `rg "HOST" ahpw/src/connection/data.ts` - one host id, `'daemon'`, in every call.
- `rg "AsyncStorage|SecureStore" ahpapp/src` - the host list is `ahpapp.hosts`, and each token is `ahpapp.token.<id>`.
- `rg "keyring|stop|kill" opendoop/packages/tauri/src-tauri/src` - doop has no keyring and never stops its daemon.
- `rg "ahpd on ws://" ahpd/packages/server/src` - the line that `ahpd start` parses for the address.
- `rg "x-ahpd-bots" ahpd ahpapp` - the bots channel, and ahpapp's reader in `useHostBots.ts`.

### Runtime path

```
shell starts -> hosts.json + OS keyring -> host list screen
  -> open_app_window(host) -> the build's UI: ahpw:// or ahpapp:// -> ahpwEmbedder.host() -> WebSocket to the host
  -> pill window -> the desk's hosts command -> one MultiHostClient -> bots + session status -> bood in the DOM
```

### Gaps

- No code in ahp-desk.
- ahpw has no way to take a host that did not serve the page.
- ahpapp has no way to take hosts from outside its storage.
- `bood.ts` lives in ahpapp only, and ahpapp is not on npm. It moves to `@cofold/bood`.
- No CI builds a Tauri installer anywhere in these repositories.

## Decisions locked in

| # | Decision | Rationale / source |
| --- | --- | --- |
| 1 | [A UI takes its hosts from a command and an event](../../../decisions/a-ui-takes-its-hosts-from-a-command-and-an-event.md) | Softov, 2026-10-11 |
| 2 | [ahpw owns its embedding contract, and the desk injects the embedder](../../../decisions/ahpw-owns-its-embedding-contract.md) | Softov, 2026-10-11 |
| 3 | [ahpw shows one host at a time on the desk](../../../decisions/ahpw-shows-one-host-at-a-time-on-the-desk.md) | Softov, 2026-10-11 |
| 4 | [A UI bundle comes from the local repository or from npm](../../../decisions/a-ui-bundle-comes-from-the-local-repository-or-from-npm.md) | Softov, 2026-10-11 |
| 5 | [A local ahpd is found, or installed with the bundled Node](../../../decisions/a-local-ahpd-is-found-or-installed-with-the-bundled-node.md) | Softov, 2026-10-11 |
| 6 | [The backends are picked in Add host](../../../decisions/the-backends-are-picked-in-add-host.md) | Softov, 2026-10-11 |
| 7 | [Stopping the local ahpd on quit is a host setting](../../../decisions/stopping-the-local-ahpd-on-quit-is-a-host-setting.md) | Softov, 2026-10-11 |
| 8 | [The pill draws bood in the DOM from the bood data](../../../decisions/the-pill-draws-bood-in-the-dom-from-the-bood-data.md) | Softov, 2026-10-11 |
| 9 | [The picker lists the bots of every host](../../../decisions/the-picker-lists-the-bots-of-every-host.md) | Softov, 2026-10-11 |
| 10 | [The pill with no bot is a setting](../../../decisions/the-pill-with-no-bot-is-a-setting.md) | Softov, 2026-10-11 |
| 11 | [The bot shows its session state](../../../decisions/the-bot-shows-its-session-state.md) | Softov, 2026-10-11 |
| 12 | [The bot is the pill](../../../decisions/the-bot-is-the-pill.md) | Softov, 2026-10-11 |
| 13 | [A bot is talked to from an input that shows on hover](../../../decisions/a-bot-is-talked-to-from-an-input-on-hover.md) | Softov, 2026-10-11 |
| 14 | [Tokens are in the OS keyring, with hosts.json as the fallback](../../../decisions/tokens-are-in-the-os-keyring-with-hosts-json-as-the-fallback.md) | Softov, 2026-10-11 |
| 15 | [Each UI is served from its own URI scheme](../../../decisions/each-ui-is-served-from-its-own-uri-scheme.md) | Softov, 2026-10-11 |
| 16 | [The local ahpd runs through its own start and stop](../../../decisions/the-local-ahpd-runs-through-its-own-start-and-stop.md) | Softov, 2026-10-11 |
| 17 | [The local ahpd uses a token file](../../../decisions/the-local-ahpd-uses-a-token-file.md) | Softov, 2026-10-11 |
| 18 | [ahpd updates itself when the first install says so](../../../decisions/ahpd-updates-itself-when-the-first-install-says-so.md) | Softov, 2026-10-11 |
| 19 | [The desk starts with the OS by default](../../../decisions/the-desk-starts-with-the-os-by-default.md) | Softov, 2026-10-11 |
| 20 | [The bood data lives in cofold](../../../decisions/the-bood-data-lives-in-cofold.md) | Softov, 2026-10-11 |
| 21 | [A drag moves the cluster, and Alt with a drag takes a bot out](../../../decisions/a-drag-moves-the-cluster-and-alt-drag-takes-a-bot-out.md) | Softov, 2026-10-11 |
| 22 | [A cluster holds four bots](../../../decisions/a-cluster-holds-four-bots.md) | Softov, 2026-10-11 |
| 23 | [The pill's menu opens the hosts and the settings](../../../decisions/the-pill-menu-opens-the-hosts-and-the-settings.md) | Softov, 2026-10-11 |
| 24 | [Each desk build ships one UI](../../../decisions/each-desk-build-ships-one-ui.md) | Softov, 2026-10-11 |
| 25 | [ahp-desk is MIT, and ahpapp stays private](../../../decisions/ahp-desk-is-mit-and-ahpapp-stays-private.md) | Softov, 2026-10-11 |

| What | Source | Task |
| --- | --- | --- |
| A plan first, then code. | Softov, asked "What should I produce first in ahp-desk?": "Plan first (Recommended)" | this plan |
| The first build has app window and URL hosts, local ahpd, tray and updater, and the bot pill. | Softov, asked "What does the first milestone include?" | p1 to p6 |
| ahpd is installed on a Linux server over SSH. | Softov, 2026-10-13: "this computer and the ssh installer" | p10 |
| The desk says what can be updated (itself, ahpw with it, each host's ahpd) and updates it. | Softov, 2026-10-13: "how to make it alert updates" | p9 |
| ahpw comes first, and ahpapp after it. | Softov: "initially for working for ahpw and with easy to work with ahpapp" | p2 before p5 |

## Proposed architecture

- **Data flow** - The shell owns the host list. A UI reads it through the embedder the desk injects and connects to its host itself, over AHP.
- **Event flow** - The shell emits `hosts-changed` after every edit of the host list. Each UI and the pill listen to it.
- **State flow** - `hosts.json` and `settings.json` in the app config folder, and the tokens in the OS keyring. A UI keeps its own view state.
- **Layer responsibilities** - `src-tauri`: hosts, keyring, windows, URI schemes, local ahpd, tray, updater · `packages/shell`: host list, Add host, settings, pill, picker · `packages/ahpw-embedder`: ahpw's embedder, injected into each app window · ahpw and ahpapp: a desktop host source beside their current one.
- **Source-of-truth files** - ahpw's `src/embed/index.ts` for the contract; `src-tauri/src/hosts.rs` for the host list.

## Tasks

| Task | Status | Depends on |
| --- | --- | --- |
| [p1 - The shell, the host list and the app window](../01-first-build-p1-shell-and-url-hosts/plan.md) | built | - |
| [p2 - ahpw on the desk](../01-first-build-p2-ahpw-on-the-desk/plan.md) | built | p1 |
| [p3 - The local ahpd](../01-first-build-p3-local-ahpd/plan.md) | active | p1 |
| [p4 - The tray, the updater and the releases](../01-first-build-p4-tray-and-releases/plan.md) | draft | p1 |
| [p5 - ahpapp in a private build](../01-first-build-p5-ahpapp-on-the-desk/plan.md) | draft | p1, p2 |
| [p6 - The bot pill](../01-first-build-p6-bot-pill/plan.md) | draft | p1, p4 |
| [p7 - Desk polish before the first release](../01-first-build-p7-desk-polish/plan.md) | built | p2 |
| [p8 - Each host keeps its own pages on the desk](../01-first-build-p8-host-scoped-storage/plan.md) | built | p2 |
| [p9 - The desk says what can be updated, and updates it](../01-first-build-p9-update-alerts/plan.md) | draft | p3, p4 |
| [p10 - Install ahpd on a Linux server over SSH](../01-first-build-p10-ssh-install/plan.md) | draft | p3 |

## Risks and tradeoffs

- A transparent window on Windows can make other webviews white. p6 starts the hidden host window first, as doop does.
- A custom URI scheme may not count as a local origin for Tauri's IPC. p1 task 05 checks this before the host list uses it.
- ahpapp's web build is not checked by its own team yet. p5 starts with a run of that build in a browser.
- The Linux keyring needs a Secret Service. With none, the token is in `hosts.json`, and the host list shows a warning.

## Resume state

- **Done so far:** decisions and plans written; p1 tasks 01, 02, 03, 05 and 06 done, 2026-10-11. A custom scheme does count as a local origin for IPC on Windows (p1 task 05).
- **Next action:** [p3 task 02](../01-first-build-p3-local-ahpd/task-02-find-ahpd.md); then the task files of p4 to p6.
- **Open questions:** the open questions of each child plan are in its own *Resume state*.
- **Watch out for:** the child plans p2 to p6 are `draft`; write their task files before their first task starts.

## Final verification checklist

- [ ] Every child plan is `built`.
- [ ] An installer from CI runs on a clean Windows 11 and opens ahpw on a URL host.
- [ ] `plans/index.md` updated.
