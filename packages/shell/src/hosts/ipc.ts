/** The desk's commands and events as the host list calls them; a UI in an app window reaches them through its embedder. */

import { Channel, invoke } from '@tauri-apps/api/core';
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
  /** A `local` host's two settings; a `url` host has none. */
  onDesk?: OnDesk;
  /** The host's server over SSH, on a host reached through one. */
  ssh?: SshHost;
}

/** A host's server over SSH. A password is never part of it. */
export interface SshHost {
  address: string;
  port: number;
  user: string;
  /** The way the last sign-in went through. */
  signIn?: 'agent' | 'key-file' | 'password';
  keyFile?: string;
}

/** What a `local` host's ahpd does when the desk starts and quits. */
export interface OnDesk {
  startOnLaunch: boolean;
  stopOnQuit: boolean;
}

/** A host as the host list sees it: with the flags for a token or an SSH password kept in the plain file. */
export interface ShellHost extends Host {
  tokenInFile?: boolean;
  sshPasswordInFile?: boolean;
}

/** Whether a kept SSH password would go in the OS keyring, so the form says where it goes. */
export function sshKeyringWorks(): Promise<boolean> {
  return invoke<boolean>('ssh_keyring_works');
}

/** An ahpd on this computer, and where it was found: the path set on the host, the desk's own install, or `PATH`. */
export interface FoundAhpd {
  path: string;
  origin: 'set' | 'installed' | 'path';
  version: string;
  /** False for an ahpd older than the oldest the desk drives. */
  supported: boolean;
  /** The plugins it has ready; none asked of one too old. */
  plugins: string[];
}

/** The ahpd the desk would use for a `local` host, or null when there is none. */
export function findLocalAhpd(ahpdPath: string | null): Promise<FoundAhpd | null> {
  return invoke<FoundAhpd | null>('find_local_ahpd', { ahpdPath });
}

/** Adds `backends` to the ahpd a local host would use, calling `onEvent` with each line ahpd writes. */
export function addLocalBackends(
  ahpdPath: string | null,
  backends: string[],
  onEvent: (event: InstallEvent) => void,
): Promise<void> {
  const events = new Channel<InstallEvent>();
  events.onmessage = onEvent;
  return invoke('add_local_backends', { ahpdPath, backends, events });
}

/** Writes `folder` into the config of the ahpd a local host would use, as the folder it serves. */
export function serveLocalFolder(ahpdPath: string | null, folder: string): Promise<void> {
  return invoke('serve_local_folder', { ahpdPath, folder });
}

/** The folder Add host offers to serve before one is picked: the home folder. */
export function localHomeFolder(): Promise<string> {
  return invoke<string>('local_home_folder');
}

/** A local host's ahpd: on or off, and whether the desk started it, which is the only one it stops. */
export interface LocalState {
  on: boolean;
  startedByDesk: boolean;
}

/** Whether the local host's ahpd is on; its address and token are saved on the host. */
export function localAhpdStatus(id: string): Promise<LocalState> {
  return invoke<LocalState>('local_ahpd_status', { id });
}

/** Starts the local host's ahpd, or takes the one already on. */
export function startLocalAhpd(id: string): Promise<LocalState> {
  return invoke<LocalState>('start_local_ahpd', { id });
}

/** Stops the local host's ahpd; refused for one the desk did not start. */
export function stopLocalAhpd(id: string): Promise<LocalState> {
  return invoke<LocalState>('stop_local_ahpd', { id });
}

/** One step of an install: a line npm or ahpd wrote. */
export type InstallEvent = { kind: 'line'; text: string };

/** Installs ahpd and the `backends` with the desk's Node, calling `onEvent` as it goes, and answers the ahpd now used. */
export function installLocalAhpd(backends: string[], onEvent: (event: InstallEvent) => void): Promise<FoundAhpd | null> {
  const events = new Channel<InstallEvent>();
  events.onmessage = onEvent;
  return invoke<FoundAhpd | null>('install_local_ahpd', { backends, events });
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

/** Calls `handler` with a host's id when an app window asks for that host's form, as ahpw's Sign in does. */
export function onEditAsked(handler: (id: string) => void): Promise<UnlistenFn> {
  return listen<string>('host-list-edit', (event) => handler(event.payload));
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

/** The tray menu's words, in the host list's language. */
export interface TrayWords {
  hosts: string;
  settings: string;
  quit: string;
}

/** Sets the tray menu in the host list's language. */
export function setTrayWords(words: TrayWords): Promise<void> {
  return invoke('set_tray_words', { words });
}

/** Opens a host in its own window, or focuses the window that shows it. */
export function openHost(id: string): Promise<void> {
  return invoke('open_app_window', { hostId: id, replace: false });
}
