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

/**
 * The embedder for the page at `location`. The window with every host is
 * opened with `?hosts=every`, and `focus=` names the host it opened on;
 * `forgetFocus` drops that from the address once it is given, so a reload
 * keeps the person's own filter.
 */
export function createEmbedder(location: { search: string }, forgetFocus: () => void = () => undefined): Embedder {
  const params = new URLSearchParams(location.search);
  const multi = params.get('hosts') === 'every';
  let focus = params.get('focus');
  const host = () => invoke<EmbedHost>('window_host');
  return {
    mode: multi ? 'multi' : 'mono',
    host,
    // The window with every host has no one address; ahpw keys its pages under `*` there.
    hostUrl: multi ? '*' : hostUrlOf(location),
    // A window of one host reads only its own host's token; the window with every host reads any.
    token: async (id) => invoke<string | null>('host_token', { id: id ?? (await host()).id }),
    hosts: () => invoke<EmbedHost[]>('hosts'),
    onHosts: (handler) =>
      unsubscribe(listen<EmbedHost[]>('hosts-changed', (event) => handler(event.payload))),
    statuses: () => invoke<Record<string, EmbedHostStatus>>('host_statuses'),
    onStatuses: (handler) =>
      unsubscribe(
        listen<Record<string, EmbedHostStatus>>('hosts-status', (event) => handler(event.payload)),
      ),
    switchHost: (id) => invoke('open_app_window', { hostId: id, replace: true }),
    manageHosts: (options) =>
      invoke<void>('show_host_list', {
        add: options?.add === true,
        ...(options?.edit === undefined ? {} : { edit: options.edit }),
      }),
    ...(multi
      ? {
          onFocusHost: (handler: (id: string) => void) => {
            // The host the window opened on comes first, once: the desk's event came before this listened.
            const first = focus;
            focus = null;
            if (first !== null && first !== '') {
              forgetFocus();
              queueMicrotask(() => handler(first));
            }
            return unsubscribe(listen<string>('focus-host', (event) => handler(event.payload)));
          },
        }
      : {}),
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
