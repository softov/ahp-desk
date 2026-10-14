---
title: The remote ahpd has a Node, the server's own 22 or later, or the newest official 22 downloaded there
status: todo
depends: [task-03-probe.md]
layer: "src-tauri"
refs:
  - file:///github/opendoop/packages/tauri/src-tauri/src/installer/systemd_node.rs - doop pushes its staged Node, about 80 MB, which decision 4 replaces with a download on the server
  - "[code://scripts/stage-node.mjs](../../../../scripts/stage-node.mjs) - the desk's own Node fetch: the official build checked against its SHASUMS256, the check this task runs on the server; its pinned version is not the server's (decision 17)"
  - https://nodejs.org/dist/latest-v22.x/ - the newest 22 release and its `SHASUMS256.txt`, which names the archive for each CPU
---

## Objective

`ssh_install::node()` answers the Node the remote ahpd runs on: the server's own Node 22 or later, or the newest official Node 22 for the server's CPU, from `latest-v22.x`, downloaded by the server into `~/.local/share/ahpd/node` and checked against that release's `SHASUMS256.txt` (decisions 4 and 17).
Nothing large goes from the desk.

## Files

- `UPDATE: src-tauri/src/ssh_install.rs` - `RemoteNode { node, npm, bin_dir, version }`, `node()`, `download_script()`, `NODE_DIST`, `https://nodejs.org/dist/latest-v22.x/`.

## Steps

1. With `NodeOnServer::Own` from the probe, `node()` answers its path and the `npm` beside it, and downloads nothing.
2. With `NodeOnServer::Download { cpu }`, `node()` runs `download_script()` through task 01's `exec_lines`, so its lines reach the install steps: in a temporary folder under `~/.local/share/ahpd`, fetch `<NODE_DIST>SHASUMS256.txt` with the `curl` or `wget` the probe found; take from it the one line whose name matches `node-v22.*-linux-<cpu>.tar.gz`, which names the version; fetch that archive from the same folder; check it with `sha256sum -c` against that line; unpack it, and move it to `~/.local/share/ahpd/node`, replacing an older one only after the new one is whole (decision 17).
   A `SHASUMS256.txt` with no line, or more than one, for that CPU is refused with what it held.
3. A failed fetch, as on a server with no internet, is refused with that reason (decision 4's consequence); a checksum that does not match is refused and the archive deleted.
4. `node()` ends by running `<node> -v`, refusing a version older than 22, and answers the version, which the install's steps show, since two installs can get different 22 releases (decision 17's consequence).

## Validation

- Cargo tests: `download_script()` for `x64`, `arm64` and `armv7l`, with `curl` and with `wget`, fetches only from `latest-v22.x`, takes the archive's name from `SHASUMS256.txt`, quotes every path with `quote()` and checks the sum before it unpacks.
- Run in `sh` on this machine's Git Bash against a scratch folder with a canned `SHASUMS256.txt`: the line for the CPU is picked, a missing one is refused, and a wrong sum deletes the archive.
- Cargo test against task 01's `test_server.rs`: `Own` sends no download; a script that exits non-zero fails `node()` with its last lines.
- `pnpm test` and `pnpm lint` pass.
- By hand on dev02 only: the ignored test `ssh_a_real_node_download`, with the download forced into a scratch folder, ends with `node -v` printing a 22 version; a second run replaces it whole.

## Resume
