---
title: Every window reads each host's running turns
status: done
depends: []
layer: "src-tauri and packages/shell"
refs:
  - "[code://src-tauri/src/statuses.rs](../../../../src-tauri/src/statuses.rs) - `HostStatuses`, `publish_host_statuses` and `host_statuses`, which the running relay mirrors"
  - "[code://src-tauri/src/hosts.rs#L578](../../../../src-tauri/src/hosts.rs#L578) - `remove_host` forgets a removed host's status"
  - "[code://packages/shell/src/hosts/status.ts#L162-L172](../../../../packages/shell/src/hosts/status.ts#L162-L172) - `publish`, sent on each change"
  - "[code://packages/shell/src/hosts/status.ts#L246-L253](../../../../packages/shell/src/hosts/status.ts#L246-L253) - `running`, the titles of a host's chats with a turn on"
  - "[code://packages/shell/src/hosts/DeskUpdateBar.tsx#L11-L35](../../../../packages/shell/src/hosts/DeskUpdateBar.tsx#L11-L35) - Restart's question, from `statuses.running`"
  - "[code://packages/shell/src/hosts/HostList.tsx#L291](../../../../packages/shell/src/hosts/HostList.tsx#L291) - the host list's `DeskUpdateBar`"
---

## Objective

The host list publishes each host's running turns as `hosts-running` (decision 3), the shell keeps the last map, and any window reads it with `host_running()`.
`DeskUpdateBar` takes the running turns as a function, so a window without `HostStatuses` can show it.

## Files

- `UPDATE: src-tauri/src/statuses.rs` - `HOSTS_RUNNING`, `HostRunning`, `publish_host_running`, `host_running`, and their tests.
- `UPDATE: src-tauri/src/hosts.rs:578` - `remove_host` also forgets the host's running turns.
- `UPDATE: src-tauri/src/lib.rs` - `app.manage(HostRunning::default())` beside `HostStatuses`; both commands in `generate_handler!`.
- `UPDATE: src-tauri/build.rs` - `"publish_host_running"` and `"host_running"` in the command list.
- `UPDATE: src-tauri/capabilities/shell.json` - `"allow-publish-host-running"` and `"allow-host-running"`.
- `UPDATE: packages/shell/src/hosts/status.ts:162-172` - `publish` also sends the running map when it changes.
- `UPDATE: packages/shell/src/hosts/ipc.ts` - `hostRunning()` and `onHostRunning(handler)`.
- `UPDATE: packages/shell/src/hosts/DeskUpdateBar.tsx` - the prop `running: (id: string) => string[]` in place of `statuses: HostStatuses`.
- `UPDATE: packages/shell/src/hosts/HostList.tsx:291` and `packages/shell/src/about/AboutScreen.tsx` - pass `running={(id) => statuses.running(id)}`.

## Steps

1. In `statuses.rs`, add `pub const HOSTS_RUNNING: &str = "hosts-running";` and `pub struct HostRunning(Mutex<HashMap<String, Vec<String>>>)` with `replace`, `all` and `forget`, as `HostStatuses` has them.
2. Add `#[tauri::command] pub fn publish_host_running(app: AppHandle, running: State<'_, HostRunning>, map: HashMap<String, Vec<String>>) -> Result<(), String>`, which keeps the map and emits `HOSTS_RUNNING` with it, and `#[tauri::command] pub fn host_running(running: State<'_, HostRunning>) -> HashMap<String, Vec<String>>`.
3. In `hosts.rs` `remove_host`, after `statuses.forget(&id)`, take `running: State<'_, HostRunning>` and call `running.forget(&id)`.
4. Register both commands in `lib.rs` and `build.rs`, and grant both to the host list in `shell.json`; task 03 grants `host_running` to the Updates window.
5. In `status.ts`, keep `let publishedRunning = ''` beside `published`; in `publish`, build `const running: Record<string, string[]> = {}` with `statuses.running(known.id)` for each host that has any, and invoke `publish_host_running` with `{ map: running }` only when its JSON differs from `publishedRunning`, logging a failure as `publish` does.
6. In `ipc.ts`, add `export function hostRunning(): Promise<Record<string, string[]>>` invoking `host_running`, and `export function onHostRunning(handler: (map: Record<string, string[]>) => void): Promise<UnlistenFn>` listening to `hosts-running`.
7. In `DeskUpdateBar.tsx`, replace `statuses: HostStatuses` with `running: (id: string) => string[]`, and `statuses.running(host.id)` with `running(host.id)`; drop the `HostStatuses` import; pass it from `HostList.tsx:291` and from `AboutScreen.tsx` until task 02 removes that screen.

## Validation

- `statuses.rs` tests: a published running map is answered by `host_running`; `forget` drops one host; a host with no running turns is absent.
- `pnpm typecheck`, `pnpm test`, and `cargo clippy --all-targets -- -D warnings` in `src-tauri`.
- By hand on the debug desk: with a turn running on a host, `host_running` answers its title (from the devtools console, `__TAURI__.core.invoke('host_running')`).

## Resume

- Built: `HOSTS_RUNNING`, `HostRunning`, `publish_host_running` and `host_running` in `statuses.rs`; `remove_host` forgets the host's turns; `hostRunning` and `onHostRunning` in `hosts/ipc.ts`; `DeskUpdateBar` takes `running`.
- Beyond the steps: `status.ts` also publishes on each of the client's session events (`multi.events()`), since `hostEvents()` carries only connection changes and a turn starts or ends in a host's session list. The client updates its session cache before it sends the event, so the map read then is current. `publishRunning` is its own function beside `publish`, called from both.
- Checked: `pnpm typecheck`, `pnpm test` (with the new `statuses.rs` test), `cargo clippy --all-targets -- -D warnings` pass.
- The by-hand check runs with task 03's Restart check: until then no window reads the map.

