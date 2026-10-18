---
title: The remote ahpd has a Node, the server's own 22 or later, or the newest official 22 downloaded there
status: done
depends: [task-03-probe.md]
layer: "src-tauri"
refs:
  - file:///github/opendoop/packages/tauri/src-tauri/src/installer/systemd_node.rs - doop pushes its staged Node, about 80 MB, which the Answers row on the remote Node replaces with a download on the server
  - "[code://scripts/stage-node.mjs](../../../../scripts/stage-node.mjs) - the desk's own Node fetch: the official build checked against its SHASUMS256, the check this task runs on the server; its pinned version is not the server's (Answers: the newest Node 22)"
  - https://nodejs.org/dist/latest-v22.x/ - the newest 22 release and its `SHASUMS256.txt`, which names the archive for each CPU
---

## Objective

`ssh_install::node()` answers the Node the remote ahpd runs on: the server's own Node 22 or later, or the newest official Node 22 for the server's CPU, from `latest-v22.x`, downloaded by the server into `~/.local/share/ahpd/node` and checked against that release's `SHASUMS256.txt` (Answers: the remote Node, and the newest Node 22).
Nothing large goes from the desk.

## Files

- `UPDATE: src-tauri/src/ssh_install.rs` - `RemoteNode { node, npm, bin_dir, version }`, `node()`, `download_script()`, `NODE_DIST`, `https://nodejs.org/dist/latest-v22.x/`.

## Steps

1. With `NodeOnServer::Own` from the probe, `node()` answers its path and the `npm` beside it, and downloads nothing.
2. With `NodeOnServer::Download { cpu }`, `node()` runs `download_script()` through task 01's `exec_lines`, so its lines reach the install steps: in a temporary folder under `~/.local/share/ahpd`, fetch `<NODE_DIST>SHASUMS256.txt` with the `curl` or `wget` the probe found; take from it the one line whose name matches `node-v22.*-linux-<cpu>.tar.gz`, which names the version; fetch that archive from the same folder; check it with `sha256sum -c` against that line; unpack it, and move it to `~/.local/share/ahpd/node`, replacing an older one only after the new one is whole (Answers: the newest Node 22).
   A `SHASUMS256.txt` with no line, or more than one, for that CPU is refused with what it held.
3. A failed fetch, as on a server with no internet, is refused with that reason (Answers: the remote Node); a checksum that does not match is refused and the archive deleted.
4. `node()` ends by running `<node> -v`, refusing a version older than 22, and answers the version, which the install's steps show, since two installs can get different 22 releases (Answers: the newest Node 22).

## Validation

- Cargo tests: `download_script()` for `x64`, `arm64` and `armv7l`, with `curl` and with `wget`, fetches only from `latest-v22.x`, takes the archive's name from `SHASUMS256.txt`, quotes every path with `quote()` and checks the sum before it unpacks.
- Run in `sh` on this machine's Git Bash against a scratch folder with a canned `SHASUMS256.txt`: the line for the CPU is picked, a missing one is refused, and a wrong sum deletes the archive.
- Cargo test against task 01's `test_server.rs`: `Own` sends no download; a script that exits non-zero fails `node()` with its last lines.
- `pnpm test` and `pnpm lint` pass.
- By hand on the test machine only: the ignored test `ssh_a_real_node_download`, with the download forced into a scratch folder, ends with `node -v` printing a 22 version; a second run replaces it whole.

## Resume

- 2026-10-18: `ssh_install.rs` has `NODE_DIST`, `RemoteNode { node, npm, binDir, version }`, `download_script()` and `node()`; the probe keeps `fetch` (`curl` or `wget`), which the script downloads with.
- The script works in a `mktemp -d` folder under the install folder, removed on exit; it exits 10 to 14 by what stopped it (no `SHASUMS256.txt`, not one line for the CPU, no archive, a wrong sum, the move), and a failed `node()` carries its last 5 lines.
- An older `<install>/node` is moved aside, the new one moved in, and the old one removed; a failed move puts the old one back.
- `node()` with the server's own Node runs nothing on the server.
- `ssh_install.rs` is allowed unused outside tests until the install calls `node()` (task 05).
- Verified: `cargo test` (99: the script's text for each CPU and tool; the script run in `sh` against a `file://` dist for the CPU's line, a whole replacement, a missing line and a wrong sum; `node()` against the test server), `cargo clippy --all-targets -- -D warnings`, `pnpm typecheck`, `pnpm test`.
- By hand, `ssh_a_real_node_download` on the test server Softov named: twice into a scratch folder in the user's home, each ending in `v22.23.3` for x64, and the folder removed after.
