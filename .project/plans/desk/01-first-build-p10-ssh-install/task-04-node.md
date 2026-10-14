---
title: The remote ahpd has a Node, the server's own 22 or later, or the official build downloaded there
status: todo
depends: [task-03-probe.md]
layer: "src-tauri"
refs:
  - file:///github/opendoop/packages/tauri/src-tauri/src/installer/systemd_node.rs - doop pushes its staged Node, about 80 MB, which decision 4 replaces with a download on the server
  - "[code://scripts/stage-node.mjs](../../../../scripts/stage-node.mjs) - the desk's own Node fetch: the official build checked against its SHASUMS256, the check this task runs on the server"
---

## Objective

`ssh_install::node()` answers the Node the remote ahpd runs on: the server's own Node 22 or later, or the official build for the server's CPU, downloaded by the server into `~/.local/share/ahpd/node` and checked against nodejs.org's `SHASUMS256.txt` (decision 4).
Nothing large goes from the desk.

## Files

- `UPDATE: src-tauri/src/ssh_install.rs` - `RemoteNode { node, npm, bin_dir }`, `node()`, `download_script()`, `NODE_VERSION`.
- `UPDATE: src-tauri/build.rs` - `NODE_VERSION`, only if open question 9 is answered with the pinned version: read from `scripts/stage-node.mjs` and passed as `cargo:rustc-env`.

## Steps

1. With `NodeOnServer::Own` from the probe, `node()` answers its path and the `npm` beside it, and downloads nothing.
2. With `NodeOnServer::Download { cpu }`, `node()` runs `download_script()` through task 01's `exec_lines`, so its lines reach the install steps: in a temporary folder under `~/.local/share/ahpd`, fetch `SHASUMS256.txt` and `node-<version>-linux-<cpu>.tar.gz` from `https://nodejs.org/dist/<version>/` with the `curl` or `wget` the probe found, check the archive with `sha256sum -c`, unpack it, and move it to `~/.local/share/ahpd/node`, replacing an older one only after the new one is whole.
   Which version is open question 9 in the plan.
3. A failed fetch, as on a server with no internet, is refused with that reason (decision 4's consequence); a checksum that does not match is refused and the archive deleted.
4. `node()` ends by running `<node> -v` and refusing a version older than 22.

## Validation

- Cargo tests: `download_script()` for `x64`, `arm64` and `armv7l`, with `curl` and with `wget`, quotes every path with `quote()` and checks the sum before it unpacks.
- Cargo test against task 01's `test_server.rs`: `Own` sends no download; a script that exits non-zero fails `node()` with its last lines.
- `pnpm test` and `pnpm lint` pass.
- By hand on dev02 only: the ignored test `ssh_a_real_node_download`, with the download forced into a scratch folder, ends with `node -v` printing a 22 version; a second run replaces it whole.

## Resume
