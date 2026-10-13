/**
 * ahpw's embedder on the desk: ahpw's `Embedder` contract over the desk's
 * Tauri commands and events. Nothing here holds state; every call asks Rust.
 */

import type { EmbedHost, EmbedHostStatus, Embedder, Unsubscribe } from '@ahpd/web/embed';
import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { getCurrentWindow } from '@tauri-apps/api/window';

/** A Tauri listener as ahpw's synchronous unsubscribe: a stop before the listener is set still stops it. */
export function unsubscribe(listening: Promise<UnlistenFn>): Unsubscribe {
  let stopped = false;
  let unlisten: UnlistenFn | undefined;
  void listening.then((fn) => {
    if (stopped) fn();
    else unlisten = fn;
  });
  return () => {
    if (stopped) return;
    stopped = true;
    unlisten?.();
  };
}

/** The window's host address, which the desk puts in the page's address as `?host=`. */
export function hostUrlOf(location: { search: string }): string {
  const url = new URLSearchParams(location.search).get('host');
  if (url === null || url === '') throw new Error('ahp-desk: this window was opened with no ?host= address');
  return url;
}

/** The embedder for the page at `location`. */
export function createEmbedder(location: { search: string }): Embedder {
  const host = () => invoke<EmbedHost>('window_host');
  return {
    host,
    hostUrl: hostUrlOf(location),
    // The desk lets a window read only its own host's token.
    token: async () => invoke<string | null>('host_token', { id: (await host()).id }),
    hosts: () => invoke<EmbedHost[]>('hosts'),
    onHosts: (handler) =>
      unsubscribe(listen<EmbedHost[]>('hosts-changed', (event) => handler(event.payload))),
    statuses: () => invoke<Record<string, EmbedHostStatus>>('host_statuses'),
    onStatuses: (handler) =>
      unsubscribe(
        listen<Record<string, EmbedHostStatus>>('hosts-status', (event) => handler(event.payload)),
      ),
    switchHost: (id) => invoke('open_app_window', { hostId: id, replace: true }),
    manageHosts: (options) => invoke<void>('show_host_list', { add: options?.add === true }),
    window: {
      minimize: () => getCurrentWindow().minimize(),
      toggleMaximize: () => getCurrentWindow().toggleMaximize(),
      close: () => getCurrentWindow().close(),
      startDragging: () => getCurrentWindow().startDragging(),
      isMaximized: () => getCurrentWindow().isMaximized(),
      onResized: (handler) => unsubscribe(getCurrentWindow().onResized(() => handler())),
    },
  };
}
