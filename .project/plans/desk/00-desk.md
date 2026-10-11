---
title: Desk - what exists today
domain: desk
revalidated: 2026-10-11
---

The desk domain is the ahp-desk desktop app: a Tauri 2 shell that keeps the hosts and loads a UI that talks to them.
No code exists yet.
The repository holds the mockups in `mockups/`, the developer guide, and the records in `.project/`.

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

None yet.
The first plan creates `src-tauri/`, `packages/shell/` and `packages/ahpw-embedder/`.

## Contracts

None yet.
`packages/ahpw-embedder` implements ahpw's embedding contract over the desk's Tauri commands.

## Runtime path

```
shell starts -> host list (hosts.json + OS keyring) -> app window -> ahpw-embedder (injected) -> UI bundle -> host over WebSocket
```

## Tests

None yet.

## Known gaps

- There is no code.
- ahpw connects only to the origin that served it.
- ahpapp keeps its own host list and does not know the shell.
