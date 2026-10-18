---
title: Host list
status: current
covers:
  - src-tauri/src/hosts.rs
  - src-tauri/src/tokens.rs
  - src-tauri/src/statuses.rs
  - packages/shell/src/**
decisions:
  - decisions/tokens-are-in-the-os-keyring-with-hosts-json-as-the-fallback.md
  - decisions/the-host-list-window-publishes-host-status.md
  - decisions/the-host-list-is-plain-react-in-ahpws-colours.md
refs:
  - "[code://src-tauri/src/hosts.rs#L28](../../src-tauri/src/hosts.rs#L28) - `Host`, a host as `hosts.json` keeps it"
  - "[code://src-tauri/src/hosts.rs#L153](../../src-tauri/src/hosts.rs#L153) - `HostStore::load`, which moves a token left in the file into the keyring"
  - "[code://src-tauri/src/hosts.rs#L313](../../src-tauri/src/hosts.rs#L313) - `write_json`, the write to a temporary file and rename"
  - "[code://src-tauri/src/tokens.rs#L47](../../src-tauri/src/tokens.rs#L47) - `set` and `get`, the keyring first and the file as the fallback"
  - "[code://src-tauri/src/hosts.rs#L356](../../src-tauri/src/hosts.rs#L356) - `host_token`, which a window may call only for its own host"
  - "[code://src-tauri/src/statuses.rs#L52](../../src-tauri/src/statuses.rs#L52) - `publish_host_statuses`, the map every window reads"
  - "[code://packages/shell/src/hosts/status.ts#L122](../../packages/shell/src/hosts/status.ts#L122) - `publish`, the host list's side of it"
  - "[code://packages/shell/src/FirstRun.tsx#L21](../../packages/shell/src/FirstRun.tsx#L21) - `FirstRun`"
  - "[code://packages/shell/src/i18n.ts#L296](../../packages/shell/src/i18n.ts#L296) - `fromOs`, the language before one is picked"
  - "[code://src-tauri/src/app_window.rs#L218](../../src-tauri/src/app_window.rs#L218) - `hide_if_host_list`"
  - git://4c89294 - the deleted decisions this spec was written from: a-ui-takes-its-hosts-from-a-command-and-an-event, open-hides-the-host-list, the-desk-speaks-english-portuguese-and-spanish, the-first-run-adds-a-url-host, tokens-are-kept-in-the-os-keyring
---

The host list is the desk's own window: it keeps the AHP hosts, shows each one's status, and opens a host in an app window.
The Rust side keeps the list in `hosts.json` and each token in the OS keyring.
The React side, in `packages/shell/`, draws the list, Add host and the first run.
The host list window holds one AHP connection per host for the status dots, and publishes the statuses to every window.
The code is in [`code://src-tauri/src/hosts.rs`](../../src-tauri/src/hosts.rs) and [`code://packages/shell/src`](../../packages/shell/src).

## Scenarios

### The list is kept in hosts.json

WHEN a host is added, changed or removed
THEN the whole list is written to `hosts.json` in the app config folder, through a temporary file and a rename
AND every window gets the event `hosts-changed` with the new list
AND `AHP_DESK_CONFIG_DIR` puts the file in another folder, as a test does.

### A token is kept in the keyring, else in the file

WHEN a host is saved with a token
THEN the token goes in the OS keyring, under the service `ahp-desk` and the host's id
AND with no keyring, the token goes in the host's `token` field in `hosts.json`, and the host's row says so
AND a read asks the keyring first, then the file
AND a token left in the file moves into the keyring at the next start where the keyring works.

### A window reads only its own host's token

WHEN a window asks for a host's token
THEN the host list window may read any host's token
AND an app window may read only the token of the host it shows.

### The first run

WHEN the desk starts with no host
THEN the host list shows a welcome with "Start with the OS"
AND then where the host is: on the internet or my network, or this computer
AND then the host form, with Test, or This computer's install and start
AND the first run ends when the new host's window opens
AND removing the last host starts the first run again.

### Opening a host hides the host list

WHEN the host list opens a host
THEN the host list hides and keeps running, so the status dots stay live
AND it comes back when the last app window closes, and on a second start of the desk
AND from ahpw's "Manage hosts" and "+ Add host", and from the tray.

### Each host's status reaches every window

WHEN a host's connection changes
THEN the host list window sends every host's status, `connected`, `connecting` or `down`, as the event `hosts-status`
AND the shell keeps the last map, so a window that opens later reads it with `host_statuses`.

### The desk speaks three languages

WHEN the desk shows a word
THEN it is in English, Portuguese (Brasil) or Spanish
AND the language follows the OS until one is picked in the title bar, and the pick is kept.

## Known gaps

- The statuses are worked out in the host list's webview, so with no window open nothing watches the hosts.
  Rust owning them is the way forward (decision `the-host-list-window-publishes-host-status`).
