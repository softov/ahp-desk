---
title: The bot pill
domain: desk
status: draft
priority: medium
created: 2026-10-11
revalidated: 2026-10-11
requires:
  - plans/desk/01-first-build-p1-shell-and-url-hosts/plan.md
  - plans/desk/01-first-build-p4-tray-and-releases/plan.md
decisions:
  - decisions/the-bot-is-the-pill.md
  - decisions/a-bot-is-talked-to-from-an-input-on-hover.md
  - decisions/the-pill-draws-bood-in-the-dom-from-the-bood-data.md
  - decisions/the-picker-lists-the-bots-of-every-host.md
  - decisions/the-pill-with-no-bot-is-a-setting.md
  - decisions/the-bot-shows-its-session-state.md
  - decisions/the-bood-data-lives-in-cofold.md
  - decisions/a-drag-moves-the-cluster-and-alt-drag-takes-a-bot-out.md
  - decisions/a-cluster-holds-four-bots.md
  - decisions/the-pill-menu-opens-the-hosts-and-the-settings.md
refs:
  - file:///github/opendoop/packages/tauri/src-tauri/src/windows.rs - the hidden host window at lines 15-34, and the transparent pill at lines 36-57
  - file:///github/opendoop/packages/tauri-shell/src/presence/BoodCircle.tsx - drag after a 4 px move
  - file:///github/ahpapp/src/bood.ts - bodies, moods, actions and paths as data
  - file:///github/ahpapp/src/bots.ts - `botOf`, the fold of the catalogue, `CLUSTER_MAX`, and the cluster geometry
  - file:///github/ahpapp/src/useHostBots.ts - the reader of `x-ahpd-bots://` and of `resourceList bot://`
  - file:///github/ahpapp/src/components/BotLayer.tsx - `BotComposer` sends with `say`, or `queue` when a turn runs, at lines 506-559
  - file:///github/ahpd/packages/bot/src/catalogue.ts - the bots channel, at lines 8-28
  - file:///github/ahp-desk/mockups/04-bot-on-pill.md - the bot, the cluster and its layouts
  - file:///github/ahp-desk/mockups/05-bot-input.md - the input and its menu
  - file:///github/ahp-desk/mockups/06-bot-states.md - the states, the speech bubble and the transits
  - file:///github/ahp-desk/mockups/07-bot-picker.md - the picker
---

## Goal

The bots that the user picks float on the desktop as creatures with no frame.
Each bot shows its session state, says its last line, and takes a message from an input on hover.
Bots dropped on each other form a cluster with a layout.
With no bot, the pill shows the ahp-desk logo or hides, as the setting says.

## Reconnaissance

The files read and the patterns to reuse are the `refs` above, each with its note.

### Searches performed

- `rg "mood" ahpapp/src/components/BotCluster.tsx` - the mood is for the UI only, at line 216.
- `rg "x-ahpd-bots" ahpd/packages/bot/src` - the snapshot and the `added`, `changed` and `removed` actions.

### Runtime path

```
pill window (transparent) -> hosts() -> MultiHostClient -> x-ahpd-bots:// or resourceList bot://
  -> bots on the pill (settings.json) -> session status from aggregatedSessions() -> mood, badge, bubble
  -> input -> session say or queue -> host
```

### Gaps

- `bood.ts` lives only in ahpapp, and it moves to `@cofold/bood`.
- No DOM renderer has every bood body.
- No code maps a session status to a mood.

## Decisions locked in

| # | Decision | Rationale / source |
| --- | --- | --- |
| 1 | [The bot is the pill](../../../decisions/the-bot-is-the-pill.md) | Softov, 2026-10-11 |
| 2 | [A bot is talked to from an input that shows on hover](../../../decisions/a-bot-is-talked-to-from-an-input-on-hover.md) | Softov, 2026-10-11 |
| 3 | [The pill draws bood in the DOM from the bood data](../../../decisions/the-pill-draws-bood-in-the-dom-from-the-bood-data.md) | Softov, 2026-10-11 |
| 4 | [The picker lists the bots of every host](../../../decisions/the-picker-lists-the-bots-of-every-host.md) | Softov, 2026-10-11 |
| 5 | [The pill with no bot is a setting](../../../decisions/the-pill-with-no-bot-is-a-setting.md) | Softov, 2026-10-11 |
| 6 | [The bot shows its session state](../../../decisions/the-bot-shows-its-session-state.md) | Softov, 2026-10-11 |
| 7 | [The bood data lives in cofold](../../../decisions/the-bood-data-lives-in-cofold.md) | Softov, 2026-10-11 |
| 8 | [A drag moves the cluster, and Alt with a drag takes a bot out](../../../decisions/a-drag-moves-the-cluster-and-alt-drag-takes-a-bot-out.md) | Softov, 2026-10-11 |
| 9 | [A cluster holds four bots](../../../decisions/a-cluster-holds-four-bots.md) | Softov, 2026-10-11 |
| 10 | [The pill's menu opens the hosts and the settings](../../../decisions/the-pill-menu-opens-the-hosts-and-the-settings.md) | Softov, 2026-10-11 |

| What | Source | Task |
| --- | --- | --- |
| The pill is React DOM in a transparent webview, with the hidden host window first. | Mockup 08, option 1 | 01 |

## Proposed architecture

- **Data flow** - The pill window has one `MultiHostClient` over the host list. It reads the bots and the status of their sessions.
- **Event flow** - `hosts-changed` adds and removes hosts. The bots channel adds, changes and removes bots.
- **State flow** - `settings.json` holds the bots on the pill, the clusters and their layouts. It also holds the positions, as shares of the screen, and the setting for no bot.
- **Layer responsibilities** - `src-tauri`: the pill window, its size and click-through · `packages/shell`: the pill screen, bood, the picker, the input.
- **Source-of-truth files** - `packages/shell/src/pill/`.

## Tasks

These task files are written when this plan moves to `planned`.

| Task | Status | Depends on |
| --- | --- | --- |
| 01 - hidden host window and the transparent pill window | todo | - |
| 02 - move the bood data to `@cofold/bood`, and ahpapp imports it | todo | - |
| 03 - DOM bood renderer with every body, mood and action | todo | 02 |
| 04 - read the bots and their session status from every host | todo | - |
| 05 - the picker | todo | 03, 04 |
| 06 - the input, send and queue, and the menu | todo | 04 |
| 07 - the states, the speech bubble and the transits | todo | 03, 04 |
| 08 - clusters and their layouts | todo | 03 |
| 09 - the setting for no bot | todo | 01 |
| 10 - measure memory, CPU and click-through on Windows and Linux | todo | 07, 08 |

## Risks and tradeoffs

- A transparent window costs memory, and it can make other webviews white on Windows. Task 01 follows doop's order, and task 10 measures the cost.
- Click-through on the transparent area differs between X11, Wayland and Windows. Task 10 checks each one.

## Resume state

- **Done so far:** nothing.
- **Next action:** write the task files, then task 01.
- **Open questions:** none.
- **Watch out for:** `@cofold/bood` is MIT, so the bood data becomes open source when it moves.

## Final verification checklist

- [ ] Two bots from two hosts float, form a cluster, and change layout.
- [ ] A bot shows thinking, working, needs you and error from a real session.
- [ ] A message from the input starts a turn, and a second one goes in the queue.
- [ ] `plans/index.md` updated.
