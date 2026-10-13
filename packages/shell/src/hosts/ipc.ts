/** The desk's commands and events as the host list calls them; a UI in an app window reaches them through its embedder. */

import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';

/** One host in the desk's host list. A token is never part of it; ask `hostToken`. */
export interface Host {
  id: string;
  name: string;
  kind: 'url' | 'local';
  /** The host's WebSocket address, as `ws://127.0.0.1:9187`. */
  url: string;
  /** The ahpd a `local` host uses, when one is set instead of the one found. */
  ahpdPath?: string;
}

/** A host as the host list sees it: with the flag for a token kept in the plain file. */
export interface ShellHost extends Host {
  tokenInFile?: boolean;
}

/** An ahpd on this computer, and where it was found: the path set on the host, the desk's own install, or `PATH`. */
export interface FoundAhpd {
  path: string;
  origin: 'set' | 'installed' | 'path';
  version: string;
  /** False for an ahpd older than the oldest the desk drives. */
  supported: boolean;
}

/** The ahpd the desk would use for a `local` host, or null when there is none. */
export function findLocalAhpd(ahpdPath: string | null): Promise<FoundAhpd | null> {
  return invoke<FoundAhpd | null>('find_local_ahpd', { ahpdPath });
}

/** A host's state, as the host list publishes it to the app windows. */
export type PublishedStatus = 'connected' | 'connecting' | 'down';

export function listHosts(): Promise<ShellHost[]> {
  return invoke<ShellHost[]>('hosts');
}

/** Calls `handler` with the whole list each time a host is added, changed or removed. */
export function onHostsChanged(handler: (hosts: ShellHost[]) => void): Promise<UnlistenFn> {
  return listen<ShellHost[]>('hosts-changed', (event) => handler(event.payload));
}

/** Calls `handler` when an app window asks for the Add form, as ahpw's "+ Add host" does. */
export function onAddAsked(handler: () => void): Promise<UnlistenFn> {
  return listen('host-list-add', () => handler());
}

/** The host's token, or null for a host with none. */
export function hostToken(id: string): Promise<string | null> {
  return invoke<string | null>('host_token', { id });
}

/** Saves the host. `token`: null keeps the one it has, `''` deletes it. */
export function saveHost(host: Host, token: string | null): Promise<void> {
  return invoke('save_host', { host, token });
}

export function removeHost(id: string): Promise<void> {
  return invoke('remove_host', { id });
}

/** Opens a host in its own window, or focuses the window that shows it. */
export function openHost(id: string): Promise<void> {
  return invoke('open_app_window', { hostId: id, replace: false });
}
