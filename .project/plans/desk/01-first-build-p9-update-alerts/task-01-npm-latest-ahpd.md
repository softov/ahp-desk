---
title: The desk knows npm's latest @ahpd/server, asked at most every 6 hours and kept
status: todo
depends: []
layer: "src-tauri, packages/shell"
refs:
  - file:///github/ahpd/packages/server/src/update.ts - the rules this check follows: `MAX_AGE_MS` (6 h) at line 23, `registry()` at lines 65-68, `askRegistry()` at lines 82-93, `stale()` at lines 106-110, `refreshUpdate()` at lines 122-131
  - file:///github/ahpd/packages/server/src/commands/run.ts - ahpd asks when its answer is stale at start, then every 6 h, at lines 1114-1118
  - "[code://src-tauri/src/lib.rs#L1-L87](../../../../src-tauri/src/lib.rs#L1-L87) - the modules, the managed state, the setup and the command list this task adds to"
  - "[code://src-tauri/src/statuses.rs](../../../../src-tauri/src/statuses.rs) - the pattern for state kept in Rust and sent to every window with an event"
  - "[code://src-tauri/build.rs#L237-L255](../../../../src-tauri/build.rs#L237-L255) - each command is declared, which makes it a permission"
  - "[code://src-tauri/capabilities/shell.json](../../../../src-tauri/capabilities/shell.json) - the host list window's permissions"
  - "[code://src-tauri/Cargo.toml](../../../../src-tauri/Cargo.toml) - the dependencies; the desk names no HTTP client yet"
  - "[code://src-tauri/Cargo.lock#L2810-L2811](../../../../src-tauri/Cargo.lock#L2810-L2811) - the `reqwest` the build already holds through tauri"
  - "[code://packages/shell/src/hosts/ipc.ts](../../../../packages/shell/src/hosts/ipc.ts) - the desk's commands and events as the host list calls them"
---

## Objective

`src-tauri/src/updates.rs` asks npm for the latest `@ahpd/server` when its last answer is older than 6 hours, at start and then every 6 hours, through `npm_config_registry` when set, as ahpd's `update.ts` does.
It asks with `reqwest` in Rust, over rustls (decision 5).
It keeps the answer in `updates.json` beside `hosts.json`, and sends `updates-changed` to every window when it changes; the host list reads it with `updates()`.

## Files

- `UPDATE: src-tauri/src/updates.rs` (made by p4 task 03, which this plan requires) - the check, the file and the command (below), beside p4's `DeskUpdate`.
- `UPDATE: src-tauri/src/lib.rs:1-7` - `mod updates;`, unless p4 task 03 has already added it.
- `UPDATE: src-tauri/src/lib.rs:33-41` - `app.manage(Updates::load(config.join("updates.json")));`, with the same `config` folder as `hosts.json`, so `AHP_DESK_CONFIG_DIR` covers it.
- `UPDATE: src-tauri/src/lib.rs:67-68` - `updates::watch(app.handle());` after `local_ahpd::start_on_launch`.
- `UPDATE: src-tauri/src/lib.rs:71-87` - `updates::updates` in `generate_handler!`.
- `UPDATE: src-tauri/build.rs:238-254` - `"updates"` in the command list.
- `UPDATE: src-tauri/capabilities/shell.json` - `"allow-updates"`.
- `UPDATE: src-tauri/Cargo.toml` - `reqwest` with `default-features = false` and its rustls and `json` features, in the version `Cargo.lock` already holds through tauri (0.13 today, whose rustls feature is named for that version) (decision 5).
- `UPDATE: packages/shell/src/hosts/ipc.ts` - `NpmLatest`, `Updates`, `updates()` and `onUpdatesChanged()`.

## Steps

1. In `updates.rs`, the shapes, in the words a UI reads:

   ```rust
   /// The event each change sends, with the whole `UpdatesView`.
   pub const UPDATES_CHANGED: &str = "updates-changed";
   /// The package asked about.
   pub const AHPD_PACKAGE: &str = "@ahpd/server";
   /// How long an answer is trusted, as ahpd's `MAX_AGE_MS`.
   pub const MAX_AGE: Duration = Duration::from_secs(6 * 60 * 60);

   /// What npm said, and when, in seconds since the Unix epoch.
   #[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
   #[serde(rename_all = "camelCase")]
   pub struct NpmLatest { pub name: String, pub latest: String, pub checked_at: u64 }

   /// What the desk knows can be updated on its hosts. The desk's own update stays in p4's `DeskUpdate`.
   #[derive(Clone, Debug, Default, PartialEq, Serialize)]
   #[serde(rename_all = "camelCase")]
   pub struct UpdatesView { pub ahpd: Option<NpmLatest> }

   pub struct Updates { file: PathBuf, view: Mutex<UpdatesView> }
   ```

2. `Updates::load(file: PathBuf) -> Updates` reads `updates.json`; a missing, broken or misshapen file is an empty view, as ahpd's `readUpdate()`.
   `Updates::view(&self) -> UpdatesView` clones it.
   `Updates::set_ahpd(&self, found: NpmLatest) -> Result<bool, String>` writes the file and answers whether the view changed.
3. `pub fn registry(named: Option<String>) -> String`: `npm_config_registry` without its trailing slashes, else `https://registry.npmjs.org`, as ahpd's `registry()`.
4. `pub fn stale(found: Option<&NpmLatest>, now: u64, max_age: Duration) -> bool`: no answer, or one older than `max_age`, as ahpd's `stale()`.
5. `pub async fn ask_npm(registry: &str, name: &str) -> Option<String>`: one `reqwest` GET of `<registry>/-/package/<name>/dist-tags` with `accept: application/json` and a 5 second timeout, answering its `latest` when it is a string.
   Every failure is `None`: offline, a timeout, an answer that is not `ok`, a body that is not JSON, as ahpd's `askRegistry()`.
6. `pub fn watch(app: &AppHandle)` spawns one loop on `tauri::async_runtime`: when `stale`, ask; on an answer, `set_ahpd`, and on a change emit `UPDATES_CHANGED` with the view; then sleep `MAX_AGE` and go round.
   A failed ask leaves the file as it was and is asked again on the next round, as ahpd's `setInterval`.
7. `#[tauri::command] pub fn updates(state: State<'_, Updates>) -> UpdatesView`.
8. In `ipc.ts`:

   ```ts
   /** npm's latest of a package, and when it was asked, in seconds since the Unix epoch. */
   export interface NpmLatest { name: string; latest: string; checkedAt: number }
   /** What the desk knows can be updated. */
   export interface Updates { ahpd: NpmLatest | null }
   export function updates(): Promise<Updates> { return invoke<Updates>('updates'); }
   /** Calls `handler` with the whole view each time it changes. */
   export function onUpdatesChanged(handler: (updates: Updates) => void): Promise<UnlistenFn> {
     return listen<Updates>('updates-changed', (event) => handler(event.payload));
   }
   ```

## Validation

- Unit tests in `updates.rs`, run by `cargo test --manifest-path src-tauri/Cargo.toml updates`:
  - `registry` with nothing set, with `https://mirror.example/npm//`;
  - `stale` with no answer, a fresh one, one 6 hours and a second old;
  - `Updates::load` of a missing, a broken and a good file, and `set_ahpd` writing what `load` reads back;
  - `ask_npm` against a `std::net::TcpListener` on `127.0.0.1:0` answering `{"latest":"0.12.0"}`, a `404`, HTML with `200`, and no answer for 6 seconds.
- `pnpm typecheck`, `pnpm test` and `pnpm lint` pass.
- By hand: `pnpm tauri dev` with `AHP_DESK_CONFIG_DIR` on a scratch folder writes `updates.json` with npm's latest; a second start within 6 hours asks nothing (no request in the network log); with `npm_config_registry` on an address that does not answer, the desk starts as before and the file stays as it was.

## Resume
