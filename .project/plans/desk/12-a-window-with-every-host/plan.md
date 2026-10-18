---
title: A window with every host
domain: desk
status: active
priority: medium
created: 2026-10-16
revalidated: 2026-10-16
changes:
  - specs/app-window.md
  - specs/tray-and-settings.md
refs:
  - "[code://src-tauri/src/app_window.rs#L34-L80](../../../../src-tauri/src/app_window.rs#L34-L80) - `AppWindows`, a window label to one host id, and `may_read`: a window reads only its own host's token"
  - "[code://src-tauri/src/app_window.rs#L118-L190](../../../../src-tauri/src/app_window.rs#L118-L190) - `open_app_window`, one host per window and `?host=` in the page"
  - "[code://src-tauri/src/app_window.rs#L227](../../../../src-tauri/src/app_window.rs#L227) - `window_host`, which errors for a window with no host"
  - "[code://src-tauri/src/settings.rs#L28-L38](../../../../src-tauri/src/settings.rs#L28-L38) - `Settings`, where the choice is kept"
  - "[code://packages/shell/src/settings/SettingsScreen.tsx](../../../../packages/shell/src/settings/SettingsScreen.tsx) - the Settings screen, where the choice is made"
  - "[code://packages/ahpw-embedder/src/embedder.ts#L34-L52](../../../../packages/ahpw-embedder/src/embedder.ts#L34-L52) - `createEmbedder`, which hands the window's host and its token"
  - file:///github/ahpw/.project/plans/hosts/02-a-window-with-every-host/plan.md - ahpw's side: the contract, the lists, the filter and a host that is down
---

## Goal

A person chooses in Settings how hosts open: one window per host, as today and by default, or one window with every host.
With every host, the desk opens one app window, and its ahpw shows all the hosts in the host list.
That window may read each host's token.

## Reconnaissance

The files read and the patterns to reuse are the `refs` above, each with its note.

### Runtime path

```
Settings: hosts open in one window with every host
  -> Open on a host -> the window with every host (one), shown, focused, filtered to that host
  -> embedder mode 'multi', token(id) for any host -> ahpw connects to every host
```

### Gaps

- `AppWindows` maps a window to one host, and `window_host` errors for a window with none.
- The token gate lets a window read only its own host's token.
- The embedder has no mode, one `token()`, and no way to ask the window to show one host.

## Answers

| What | Source | Task |
| --- | --- | --- |
| One window per host stays the default; one window with every host is a choice. | Softov, 2026-10-16: "in desk user could configure to use multi or mono" | 01 |
| The window with every host is ahpw's multi mode, from its hosts plan 02. | Softov, 2026-10-16: "on multi is one window with all hosts, lists add filters for hosts. when is mono, its one window per host." | 02 |
| In "every host", Open on a host row shows and focuses the window with every host, filtered to that host. | Softov, 2026-10-16: "Focus it, filtered to the host" | 02, 04 |
| The host list and the window with every host may read every token; a window of one host reads only its own. | Softov, 2026-10-16: "Host list + all-hosts window" | 03 |
| A change of the setting applies to the next Open, and open windows stay as they are. | Softov, 2026-10-16: "Next Open only" | 01 |

## Proposed architecture

- **Data flow** - `Settings` gains `windows: per-host | every-host`; in `every-host`, `open_app_window` opens or focuses the one window with every host.
- **Event flow** - Open on a host emits `focus-host` with its id to that window.
  Removing a host drops it from that window through `hosts-changed`, and closes nothing.
- **State flow** - `AppWindows` knows the window with every host apart from the windows of one host.
- **Layer responsibilities** - `src-tauri` holds the window kind, the token rule and the setting.
  `packages/ahpw-embedder` holds the mode, `token(id)` and `onFocusHost`, and `packages/shell` the setting in Settings.
- **Source-of-truth files** - [`code://src-tauri/src/app_window.rs`](../../../../src-tauri/src/app_window.rs)

## Tasks

| Task | Status | Depends on |
| --- | --- | --- |
| [01 - the setting "Hosts open in", in Settings and settings.json](task-01-the-setting-hosts-open-in.md) | done | - |
| [02 - the window with every host opens, focuses and shows the host picked](task-02-the-window-with-every-host.md) | done | 01 |
| [03 - the window with every host may read every token](task-03-the-token-rule.md) | done | 02 |
| [04 - the embedder gives the mode, each host's token and the host to show](task-04-the-embedder-mode-and-tokens.md) | blocked | 03, ahpw hosts plan 02 task 01 |
| [05 - two hosts end to end, in both settings](task-05-two-hosts-end-to-end.md) | done | 04, ahpw hosts plan 02 |

## Risks and tradeoffs

- The window with every host can read every token, which widens the rule that a window reads only its own host's token.

## Resume state

- **Done so far:** tasks 01 to 03 and 05, and task 04's code, 2026-10-16.
- **Next action:** task 04 step 1, the `@ahpd/web` pins, once ahpw releases hosts plan 02; then `implemented.md` and `built`.
- **Open questions:** none.
- **Watch out for:** the app-window spec's scenarios change with this plan, and a pinned `@ahpd/web` without the new contract cannot run multi.

## Final verification checklist

- [x] With "every host", one window shows two hosts, and Open on either focuses it filtered to that host.
- [x] With "per host", the desk works as today.
- [x] `specs/app-window.md` and `specs/tray-and-settings.md` say what the desk now does.
- [ ] `plans/index.md` updated.
