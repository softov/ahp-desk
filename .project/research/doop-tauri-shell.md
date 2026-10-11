---
title: doop's Tauri shell, and what ahp-desk can reuse
---

# doop's Tauri shell, and what ahp-desk can reuse

Read on 2026-10-11 at doop commit `4437fe4`.

## What doop has

doop's desktop shell is two packages.
`packages/tauri` is the Rust app, about 2000 lines.
`packages/tauri-shell` is the React UI of the pill and its panels.

| Part | Where | What it does |
| --- | --- | --- |
| Profiles | `src-tauri/src/profile.rs` | Keeps local, remote HTTP and SSH profiles. |
| Health check | `src-tauri/src/pood_probe.rs` | Checks if a pood answers. |
| Local daemon | `src-tauri/src/pood_process.rs` | Starts a local pood. |
| Remote install | `src-tauri/src/installer/`, `src-tauri/src/ssh/` | Sends pood over SSH with `russh` and a tar archive. |
| Pill | `src-tauri/src/windows.rs` | A transparent, always-on-top window with no frame. |
| Web UI window | `open_web_ui` in `src-tauri/src/lib.rs` | Opens a normal window at the pood's own URL. |
| Tray | `src-tauri/src/tray.rs` | The tray icon and its menu. |
| Updater | `tauri.conf.json`, `tauri-plugin-updater` | Checks GitHub releases, signed with minisign. |
| Installer | `tauri.conf.json` | NSIS only, for the current user. |

## Points to keep

- `open_web_ui` must be an `async` command. A sync command that builds a webview can deadlock on Windows.
- A transparent pill on Windows needs a hidden, normal webview window that starts first. Without it, other windows can come up white and do not respond.
- The `web-ui` capability lets pages on `http://127.0.0.1:*` call a short list of window commands, so the web UI can draw its own title bar.

## How ahp-desk differs

doop opens the daemon's URL, because pood serves its own UI.
ahpw and ahpapp will both get modes with one host and with many hosts.
So the shell keeps the host list and loads the UI bundle, and the UI connects to the hosts.
The pill shows a bot from a host, not the shell's own logo.

## Refs

- file:///github/doop/packages/tauri - the Rust shell
- file:///github/doop/packages/tauri-shell - the pill UI
- file:///github/ahpapp/src/bots.ts - bot records, placements and the floating cluster
- file:///github/ahpapp/src/bood.ts - bood bodies, moods and actions as data
- file:///github/ahpapp/src/components/BotLayer.tsx - the bot popover and its message box
