/**
 * @ahpd/desk-client: the contract between ahp-desk and a UI it hosts.
 *
 * A UI in an app window asks the shell which host it shows, reads that host's
 * token when it connects, follows the host list, and drives its own window,
 * which has no OS frame. Every call is a Tauri command or event; nothing here
 * holds state.
 */

import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { getCurrentWindow } from '@tauri-apps/api/window';

/** One host in the desk's host list. A token is never part of it; ask `hostToken`. */
export interface DeskHost {
  id: string;
  name: string;
  kind: 'url' | 'local';
  /** The host's WebSocket address, as `ws://127.0.0.1:9187`. */
  url: string;
}

/** A host's state, as the desk's host list sees it. */
export type DeskHostStatus = 'connected' | 'connecting' | 'down';

/** True inside ahp-desk, false in a browser tab. */
export function isDesk(): boolean {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
}

/** Every host in the desk's list. */
export function hosts(): Promise<DeskHost[]> {
  return invoke<DeskHost[]>('hosts');
}

/** The host this window shows. */
export function windowHost(): Promise<DeskHost> {
  return invoke<DeskHost>('window_host');
}

/** The host's token, or null for a host with none. A window may only read its own host's. */
export function hostToken(id: string): Promise<string | null> {
  return invoke<string | null>('host_token', { id });
}

/** Calls `handler` with the whole list each time a host is added, changed or removed. */
export function onHostsChanged(
  handler: (hosts: DeskHost[]) => void,
): Promise<UnlistenFn> {
  return listen<DeskHost[]>('hosts-changed', (event) => handler(event.payload));
}

/** Every host's status by id, as last published; a host not yet seen is missing. */
export function hostStatuses(): Promise<Record<string, DeskHostStatus>> {
  return invoke<Record<string, DeskHostStatus>>('host_statuses');
}

/** Calls `handler` with every host's status each time one changes. */
export function onHostStatus(
  handler: (statuses: Record<string, DeskHostStatus>) => void,
): Promise<UnlistenFn> {
  return listen<Record<string, DeskHostStatus>>('hosts-status', (event) =>
    handler(event.payload),
  );
}

/**
 * Opens a host in its own window, or focuses the window that shows it.
 * With `replace`, this window switches to the host instead.
 */
export function openHost(
  id: string,
  options: { replace?: boolean } = {},
): Promise<void> {
  return invoke('open_app_window', {
    hostId: id,
    replace: options.replace ?? false,
  });
}

/** Shows and focuses the window with the host list. */
export function showHostList(): Promise<void> {
  return invoke('show_host_list');
}

/** The controls a UI's own title bar needs, for the window it is in. */
export const windowControls = {
  minimize: (): Promise<void> => getCurrentWindow().minimize(),
  toggleMaximize: (): Promise<void> => getCurrentWindow().toggleMaximize(),
  close: (): Promise<void> => getCurrentWindow().close(),
  /** Call on mousedown in the title bar, so a drag moves the window. */
  startDragging: (): Promise<void> => getCurrentWindow().startDragging(),
  /** Whether the window fills the screen, so the UI squares its corners. */
  isMaximized: (): Promise<boolean> => getCurrentWindow().isMaximized(),
  /** Calls `handler` on each resize, a maximize and a restore among them. */
  onResized: (handler: () => void): Promise<UnlistenFn> =>
    getCurrentWindow().onResized(() => handler()),
};
