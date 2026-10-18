---
title: Desk - what exists today
domain: desk
revalidated: 2026-10-16
---

The desk domain is the ahp-desk desktop app: a Tauri 2 shell that keeps the hosts and loads a UI that talks to them.
The shell is in `src-tauri/` and `packages/shell/`; `packages/ahpw-embedder/` gives ahpw the desk's hosts in each app window.

## Glossary

| Term | Meaning |
| --- | --- |
| shell | The Rust side of ahp-desk, in `src-tauri/`, with its own React screens in `packages/shell/`. |
| host | An AHP server that the shell knows: an ahpd or any other server that speaks AHP. |
| host list | The hosts that the shell keeps, with their tokens. Mockup 02 sketches it. |
| UI | A web app that the shell loads in an app window: ahpw or ahpapp. |
| build | A distribution of the desk with one UI: the public build with ahpw, or a private build with ahpapp. |
| UI source | Where a UI's bundle comes from: `local` (a sibling repository) or `npm` (a published package). |
| app window | A normal window that shows a UI on one host. Mockup 03 sketches it. |
| embedder | What an app gives ahpw as `globalThis.ahpwEmbedder`: the hosts, the window's host and its token, and the window controls. ahpw owns the contract; the desk's is `packages/ahpw-embedder`, injected into each app window. |
| local ahpd | The ahpd on this computer, shown as the host "This computer". |
| backend | An ahpd agent plugin, such as `@ahpd/agent-claude`. ahpd does not start without one. |
| pill | The floating window. With a bot on it, it is only the bot's creature. |
| bot | A host's record at `bot://<slug>`, with a body, a colour, a name and a session. |
| cluster | Bots that you drop on each other, which move as one. |
| bood | The creature bodies, moods and actions that ahpapp draws. |

## Packages

- [`code://src-tauri`](../../../src-tauri) - the shell: `hosts.rs` (the host list), `tokens.rs`, `app_window.rs`, `ui_source.rs` (the UI schemes) and `local_ahpd.rs`.
- [`code://packages/shell`](../../../packages/shell) - the host list window: the host list, Add host, the first run and the title bar.
- [`code://packages/ahpw-embedder`](../../../packages/ahpw-embedder) - ahpw's embedder over the desk's Tauri commands, injected into each app window.

## Contracts

- ahpw's `src/shell/embed/index.ts` - the embedding contract; `packages/ahpw-embedder` implements it.
- [`code://src-tauri/src/hosts.rs`](../../../src-tauri/src/hosts.rs) - the host list and `hosts.json`.

## Runtime path

```
shell starts -> host list (hosts.json + OS keyring) -> app window -> ahpw-embedder (injected) -> UI bundle -> host over WebSocket
```

## Tests

- [`code://src-tauri/src`](../../../src-tauri/src) - `cargo test` in `hosts.rs`, `local_ahpd.rs`, `statuses.rs` and `ui_source.rs`; the tests that need a real install or a real ahpd are `#[ignore]`.
- [`code://packages/shell/src`](../../../packages/shell/src) - the dictionaries, host URLs and protocol versions.
- [`code://packages/ahpw-embedder/src/embedder.test.ts`](../../../packages/ahpw-embedder/src/embedder.test.ts) - the embedder over a fake of the desk's commands.

## Known gaps

- ahpapp keeps its own host list and does not know the shell (plan 05).
- No CI builds a Tauri installer yet (plan 04).

## The first release

ahp-desk becomes a desktop app that keeps a list of AHP hosts and opens ahpw on any of them.
It can find, install, start and stop an ahpd on this computer.
It lives in the tray, updates itself, and ships as an installer.
A private build of the same app ships ahpapp, with little work, because both UIs use one contract.
A bot from any host floats on the desktop as the pill, shows what its session does, and takes a message.

Plans 01 to 10 build it; [plans/index.md](../index.md) has each one's status.

### Answers

| What | Source | Plan |
| --- | --- | --- |
| A plan first, then code. | Softov, asked "What should I produce first in ahp-desk?": "Plan first (Recommended)" | all |
| The first build has app window and URL hosts, local ahpd, tray and updater, and the bot pill. | Softov, asked "What does the first milestone include?" | 01 to 06 |
| ahpd is installed on a Linux server over SSH. | Softov, 2026-10-13: "this computer and the ssh installer" | 10 |
| The desk says what can be updated (itself, ahpw with it, each host's ahpd) and updates it. | Softov, 2026-10-13: "how to make it alert updates" | 09 |
| ahpw comes first, and ahpapp after it. | Softov: "initially for working for ahpw and with easy to work with ahpapp" | 02 before 05 |

### Specs

What is built, as scenarios:

- [Host list](../../specs/host-list.md) - `hosts.json`, the tokens, the first run, the statuses, the languages.
- [The local ahpd](../../specs/local-ahpd.md) - find, install, backends, folder, start and stop.
- [App window](../../specs/app-window.md) - the UI's source and scheme, one host per window, the embedder, the frame.
- [Tray and settings](../../specs/tray-and-settings.md) - the tray, the close, `settings.json`, one desk at a time, start with the OS.

### Decisions

The choices a later reader might undo because they look wrong; each file says why:

- [A local ahpd is found, or installed with the bundled Node](../../decisions/a-local-ahpd-is-found-or-installed-with-the-bundled-node.md)
- [A server key never seen before refuses the first connection, and Trust connects again](../../decisions/a-new-server-key-refuses-the-connection-and-trust-connects-again.md)
- [ahp-desk is MIT, and ahpapp stays private](../../decisions/ahp-desk-is-mit-and-ahpapp-stays-private.md)
- [ahpw keeps a host's pages under the host's address](../../decisions/ahpw-keeps-a-hosts-pages-under-the-hosts-address.md)
- [ahpw owns its embedding contract, and the desk injects the embedder](../../decisions/ahpw-owns-its-embedding-contract.md)
- [A new ahpw reaches the desk with a desk release](../../decisions/ahpw-reaches-the-desk-with-a-desk-release.md)
- [An ahpd on PATH too old for the desk is shown, not replaced](../../decisions/an-ahpd-too-old-on-path-is-shown-not-replaced.md)
- [With no keyring, an SSH password is kept in hosts.json only when the person ticks a box that says so](../../decisions/an-ssh-password-without-a-keyring-is-kept-in-hosts-json-when-asked.md)
- [Each desk build ships one UI](../../decisions/each-desk-build-ships-one-ui.md)
- [Each UI is served from its own URI scheme](../../decisions/each-ui-is-served-from-its-own-uri-scheme.md)
- [The bood data lives in cofold](../../decisions/the-bood-data-lives-in-cofold.md)
- [The desk adds backends through ahpd plugin install](../../decisions/the-desk-adds-backends-through-ahpd-plugin-install.md)
- [The folder Add host asks for is written by an ahpd command](../../decisions/the-folder-is-written-by-an-ahpd-command.md)
- [The host list stays plain React, in ahpw's colours copied as tokens](../../decisions/the-host-list-is-plain-react-in-ahpws-colours.md)
- [The host list window publishes each host's status to every window](../../decisions/the-host-list-window-publishes-host-status.md)
- [An ahpd the desk installs keeps the usual config folder, with its backends under AHPD_PLUGIN_ROOT](../../decisions/the-installed-ahpd-keeps-the-usual-config-folder.md)
- [The Linux release builds on ubuntu-22.04](../../decisions/the-linux-release-builds-on-ubuntu-22-04.md)
- [The local ahpd runs through its own start and stop](../../decisions/the-local-ahpd-runs-through-its-own-start-and-stop.md)
- [The Node downloaded on a server is the newest 22, not the version the desk pins](../../decisions/the-remote-node-download-is-the-newest-22.md)
- [Tokens are in the OS keyring, with hosts.json as the fallback](../../decisions/tokens-are-in-the-os-keyring-with-hosts-json-as-the-fallback.md)
- [The updater artifacts are made only in the release build](../../decisions/updater-artifacts-are-made-only-in-the-release-build.md)

### Risks

- A transparent window on Windows can make other webviews white. Plan 06 starts the hidden host window first, as doop does.
- ahpapp's web build is not checked by its own team yet. Plan 05 starts with a run of that build in a browser.
- The Linux keyring needs a Secret Service. With none, the token is in `hosts.json`, and the host list shows a warning.

### Release check

- [ ] Plans 01 to 10 are `built`, or `dropped` with the reason in the index.
- [ ] An installer from CI runs on a clean Windows 11 and opens ahpw on a URL host.
