---
title: A local ahpd is found through its record and npm's prefix
status: todo
depends: []
layer: "src-tauri"
refs:
  - "[code://src-tauri/src/local_ahpd.rs#L58-L67](../../../../src-tauri/src/local_ahpd.rs#L58-L67) - `Origin`"
  - "[code://src-tauri/src/local_ahpd.rs#L403-L448](../../../../src-tauri/src/local_ahpd.rs#L403-L448) - `on_path`, `find` and `plain`"
  - "[code://src-tauri/src/local_ahpd.rs#L654-L674](../../../../src-tauri/src/local_ahpd.rs#L654-L674) - `find_local_ahpd`"
  - "[code://src-tauri/src/local_ahpd.rs#L726-L745](../../../../src-tauri/src/local_ahpd.rs#L726-L745) - `launcher_for`"
  - "[code://src-tauri/src/ssh_install.rs#L91-L100](../../../../src-tauri/src/ssh_install.rs#L91-L100) - the SSH probe's record try"
---

## Objective

On Linux and macOS, when the set path, the desk's install and `PATH` find no ahpd, `find` answers the ahpd that `daemon.json`'s pid runs, else `<npm prefix>/bin/ahpd` from `~/.npmrc` (decisions 1 and 2).

## Files

- `UPDATE: src-tauri/src/local_ahpd.rs:58-67` - `Origin` gains `Running` (found through its record) and `Npm` (found through npm's prefix), serialized `running` and `npm`.
- `UPDATE: src-tauri/src/local_ahpd.rs:403-448` - two new functions and `find`'s two new tries.
- `UPDATE: src-tauri/src/local_ahpd.rs:654-674, 726-745` - `find_local_ahpd` and `launcher_for` pass the config and home folders.

## Steps

1. Add `fn running_ahpd(config: &Path, args_of: impl Fn(u32) -> Option<String>) -> Option<Launcher>`.
   It reads `pid` from `config.join("daemon.json")`, asks `args_of(pid)`, splits on whitespace, and takes the first word as the Node and the second as the script when the script's file name is `ahpd` or its path holds `/@ahpd/server/`.
   The launcher is `{ program: <node>, before: [<script>], env: [("PATH", <node's folder> + ":" + $PATH)] }`.
2. Add `fn args_of_pid(pid: u32) -> Option<String>`, which runs `ps -o args= -p <pid>` and answers its trimmed stdout, or `None` on a failure or an empty answer.
3. Add `fn npm_prefix_ahpd(home: &Path) -> Option<PathBuf>`.
   It reads `home.join(".npmrc")`, takes the last line whose key, trimmed, is `prefix`, expands a leading `~` and `${HOME}` to `home`, and answers `<prefix>/bin/ahpd` when it is a file.
4. `find` takes `off_path: Option<OffPath>`, a struct of `config: PathBuf`, `home: PathBuf` and `args_of: fn(u32) -> Option<String>`.
   After `on_path` gives nothing, and only under `cfg!(unix)`, it tries `running_ahpd` (`Origin::Running`), then `npm_prefix_ahpd` with `plain` (`Origin::Npm`).
   Windows passes `None`.
5. `find_local_ahpd` and `launcher_for` build `OffPath` from `this_config_dir()`, the home folder and `args_of_pid`.

## Validation

- Tests in `local_ahpd.rs`, on a scratch folder as the existing `find` tests do:
  - a record whose `args_of` answers `/n/node /p/bin/ahpd` gives `Running` with that Node and script, and `PATH` starting with `/n`;
  - a record whose pid runs `/usr/bin/python3 x.py` is ignored, and so is a record with no `pid`;
  - `~/.npmrc` with `prefix=~/.npm-global` and a file at `.npm-global/bin/ahpd` gives `Npm`; with no such file, `None`;
  - `PATH` still wins over the record, and the record over npm's prefix.
- `pnpm test` (with `CARGO_TARGET_DIR=F:/github/ahp-desk/src-tauri/target`) and `cargo clippy --all-targets -- -D warnings` pass.

## Resume

