/** The desk's commands and events as the host list calls them; a UI in an app window reaches them through its embedder. */

import { Channel, invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';

/** One host in the desk's host list. A token is never part of it; ask `hostToken`. */
export interface Host {
  id: string;
  name: string;
  /** `ssh`: reached through a tunnel the desk holds over SSH, its `url` a kept local port. */
  kind: 'url' | 'local' | 'ssh';
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
  /** Through the SSH connection, or on ahpd's open port. */
  reach: 'tunnel' | 'port';
  /** The port ahpd listens on, on the server. */
  remotePort: number;
  folder?: string;
  /** The desk installed this ahpd. */
  byDesk?: boolean;
  /** A tunnel's local port, chosen once when the host is added. */
  localPort?: number;
}

/** A tunnel's state: on, or why it is down, and what the person types to open it again. */
export interface TunnelState {
  on: boolean;
  error: string | null;
  needs: 'passphrase' | 'password' | null;
  /** The server key it refused, which Edit host shows with Trust or Forget. */
  key: RefusedKey | null;
}

export function sshTunnelStates(): Promise<Record<string, TunnelState>> {
  return invoke<Record<string, TunnelState>>('ssh_tunnel_states');
}

/** Calls `handler` with every tunnel's state each time one changes. */
export function onTunnelsChanged(handler: (all: Record<string, TunnelState>) => void): Promise<UnlistenFn> {
  return listen<Record<string, TunnelState>>('tunnels-changed', (event) => handler(event.payload));
}

/** Opens a down tunnel now, as Retry does. */
export function openSshTunnel(id: string): Promise<void> {
  return invoke('open_ssh_tunnel', { id });
}

/** What the person typed on a row's Sign in, held until the desk quits, then the tunnel opens. */
export function sshSignIn(id: string, secret: string): Promise<void> {
  return invoke('ssh_sign_in', { id, secret });
}

/** A free local port, which a new tunnelled host keeps from then on. */
export function pickSshLocalPort(): Promise<number> {
  return invoke<number>('pick_ssh_local_port');
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

/** A server key as OpenSSH shows it. */
export interface Fingerprint {
  keyType: string;
  /** `SHA256:<base64>`, as `ssh-keygen -l` prints it. */
  sha256: string;
}

/** Why an SSH connection did not open; a refused key is refused before any sign-in. */
export type ConnectError =
  | { reason: 'newKey'; offered: Fingerprint }
  /** `inKnownHosts`: the old key is in `~/.ssh/known_hosts`, removed with `ssh-keygen -R <host>`, not by Forget. */
  | { reason: 'changedKey'; known: Fingerprint; offered: Fingerprint; inKnownHosts: boolean }
  | { reason: 'failed'; why: string };

/** A server key a connection refused: one the desk has not seen, or one that changed. */
export type RefusedKey = Extract<ConnectError, { reason: 'newKey' | 'changedKey' }>;

/** How an SSH sign-in failed, as a desk command rejects with it. */
export type SshError =
  | ({ kind: 'connect' } & ConnectError)
  | { kind: 'passphrase' }
  | { kind: 'needsPerson'; what: 'passphrase' | 'password' }
  | { kind: 'refused'; tried: string[] };

/** The `SshError` a desk command rejected with, or null for any other failure. */
export function sshError(error: unknown): SshError | null {
  return typeof error === 'object' && error !== null && 'kind' in error ? (error as SshError) : null;
}

/** Trusts the key the server last offered, only when it is the `fingerprint` the person was shown; then connect again. */
export function confirmSshKey(address: string, port: number, fingerprint: string): Promise<void> {
  return invoke('confirm_ssh_key', { address, port, fingerprint });
}

/** A server as Add host names it; `id` is the host's, under which what was typed is held until the desk quits. */
export interface ProbeTarget {
  id: string;
  address: string;
  port: number;
  user: string;
  keyFile?: string;
}

/** Where the Node of an ahpd the desk installs comes from. */
export type NodeOnServer =
  | { kind: 'own'; path: string; version: string; npm: string | null }
  | { kind: 'download'; cpu: 'x64' | 'arm64' | 'armv7l' }
  | { kind: 'refused'; reason: string };

/** An ahpd already on the server, which the desk uses and never installs over. */
export interface FoundOnServer {
  path: string;
  node: string | null;
  loginShell: boolean;
  version: string;
  /** False for an ahpd older than the oldest the desk drives. */
  supported: boolean;
  plugins: string[];
  running: boolean;
  url: string | null;
  /** The desk's own install, with the desk's unit. */
  byDesk: boolean;
}

/** What the probe found on a server, and whether the desk can go on. */
export interface Probe {
  os: string;
  pretty: string | null;
  arch: string;
  home: string;
  user: string;
  systemd: boolean;
  runtimeDir: string | null;
  linger: boolean;
  /** The tool the server downloads with. */
  fetch: 'curl' | 'wget' | null;
  node: NodeOnServer;
  found: FoundOnServer | null;
  /** Why the desk cannot use or install ahpd there; null when it can. */
  refusal: string | null;
  /** How the sign-in went through, which the host keeps. */
  signIn: SshHost['signIn'] | null;
  /** An ahpd config already there with no ahpd found, which the desk's ahpd will use. */
  ahpdConfig: string | null;
}

/** Signs in to the server and probes it; rejects with an `SshError` the form reads with `sshError`. */
export function probeSshServer(target: ProbeTarget, passphrase: string | null, password: string | null): Promise<Probe> {
  return invoke<Probe>('probe_ssh_server', { target, passphrase, password });
}

/** What Add host asks for an install over SSH. */
export interface SshInstall {
  target: ProbeTarget;
  backends: string[];
  /** The folder ahpd serves; the user's home on the server when none. */
  folder?: string;
  /** 9187 when none. */
  ahpdPort?: number;
  /** The tunnel keeps ahpd on the server's 127.0.0.1; an open port puts it on every address. */
  reach?: 'tunnel' | 'port';
}

/** The ahpd the desk connects to after an install or with a found one. */
export interface SshInstalled {
  token: string | null;
  version: string;
  prefix: string | null;
  node: string | null;
  ahpdPort: number;
  byDesk: boolean;
}

/** Installs ahpd on the server, or uses the one found there, calling `onEvent` with each step and line. */
export function installSshAhpd(request: SshInstall, onEvent: (event: InstallEvent) => void): Promise<SshInstalled> {
  const events = new Channel<InstallEvent>();
  events.onmessage = onEvent;
  return invoke<SshInstalled>('install_ssh_ahpd', { request, events });
}

/** Stops the install over SSH between steps, or in the middle of a long one. */
export function cancelSshInstall(): Promise<void> {
  return invoke('cancel_ssh_install');
}

/** Forgets the desk's key for a server, so the next connection asks to trust its key as a first one. */
export function forgetSshKey(address: string, port: number): Promise<void> {
  return invoke('forget_ssh_key', { address, port });
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

/** One event of an install: a line npm or ahpd wrote, or, over SSH, a named step starting or ending. */
export type InstallEvent =
  | { kind: 'line'; text: string }
  | { kind: 'step'; id: InstallStep; status: 'running' | 'done' | 'failed' | 'skipped' };

/** The steps of an install over SSH, in order; a found ahpd skips node, server, folder and unit. */
export type InstallStep = 'connect' | 'probe' | 'node' | 'server' | 'backends' | 'folder' | 'unit' | 'start' | 'ready';

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

/**
 * Saves the host. `token`: null keeps the one it has, `''` deletes it.
 * `sshPassword`: null keeps the one kept, `''` forgets it, a value keeps it.
 */
export function saveHost(host: Host, token: string | null, sshPassword: string | null): Promise<void> {
  return invoke('save_host', { host, token, sshPassword });
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
