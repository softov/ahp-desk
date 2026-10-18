---
title: The bot pill
domain: desk
status: draft
priority: medium
created: 2026-10-11
revalidated: 2026-10-11
requires:
  - plans/desk/01-shell-and-url-hosts/plan.md
  - plans/desk/04-tray-and-releases/plan.md
decisions:
  - decisions/the-bood-data-lives-in-cofold.md
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

## Answers

| # | What | Source |
| --- | --- | --- |
| 1 | The bot is the pill: when a bot is available, the floating window is only the bot's creature, transparent, with no frame and no name label. Bots dropped on each other form a cluster: on top of one another, side by side, or packed as balloons in a circle or a cloud. | Softov, 2026-10-11, asked to choose in `mockups/04-bot-on-pill.md`: "on bot I prefer it Option B - the bot is the pill" |
| 2 | A hover on a bot, or a click, shows an input beside it. Its placeholder carries the bot's name, as "Say something to Nimbo...", and a `⋯` in the input opens the other options. | Softov, 2026-10-11: "instead put Say {Bot name} Something as placeholder on a input that show on mouse over or on click with more ... options" |
| 3 | The pill is plain React DOM in a transparent webview, with its own `<svg>` renderer over the bood data, which is shared, not copied. | Softov, 2026-10-11, asked "How does the pill draw the bood creature?": "DOM renderer over bood.ts data (Recommended)" |
| 4 | One picker lists the bots of every connected host, grouped by host. Each bot has an "on the pill" box, any number can be on, and bots float apart until one is dropped on another. | Softov, 2026-10-11, asked "Mockup 07 proposes a picker over the bots of every connected host, each with an 'on the pill' box. Lock that?": "Yes, across all hosts (Recommended)" |
| 5 | A setting chooses what shows when no bot is on the pill: the ahp-desk logo, the default, or nothing. | Softov, 2026-10-11, asked "When no bot is on the pill, what shows?": "A setting: logo or hidden (Recommended)" |
| 6 | The bot shows its session state with the moods, actions and badge of mockup 06, says its last line in a speech bubble for a few seconds, and plays a transit when the pill shows, hides or changes bot. | Softov, 2026-10-11, asked "Mockup 06 maps the bot's session state to bood moods. Lock that mapping as the first pill behaviour?": "Yes, all of 06 (Recommended)" |
| 7 | [The bood data lives in cofold](../../../decisions/the-bood-data-lives-in-cofold.md) | Softov, 2026-10-11 |
| 8 | A drag on any bot of a cluster moves the whole cluster; a drag with Alt held takes that bot out, as the menu item "Take <name> out" does. | Softov, 2026-10-11, asked "In a cluster, how does a drag tell 'move the group' from 'take this bot out'?": "Drag moves group, Alt+drag takes out (Recommended)" |
| 9 | A cluster draws at most four bots; a fifth and later show as a `+N` bubble that opens the picker. | Softov, 2026-10-11, asked "How many bots can one cluster hold?": "Four, rest behind +N (Recommended)" |
| 10 | The `⋯` menu of the bot's input has Hosts and Settings, as mockup 05 shows, and the tray has them too. | Softov, 2026-10-11, asked "Is the pill the way into the host list and settings, as doop's pill is?": "Yes, from the ⋯ menu (Recommended)" |

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
