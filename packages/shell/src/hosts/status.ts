/**
 * Each host's live state, from one AHP connection per host in a
 * `MultiHostClient` that reconnects on its own.
 */

import { AhpClient, RpcError, type AhpTransport } from '@microsoft/agent-host-protocol/client';
import { MultiHostClient } from '@microsoft/agent-host-protocol/hosts';
import { WebSocketTransport } from '@microsoft/agent-host-protocol/ws';
import { invoke } from '@tauri-apps/api/core';

import { t } from '../i18n.js';
import { checkUpdate, requestUpdate, type Requester } from './ahpd-update.js';
import { findLocalAhpd, hostToken, updateSshAhpd, type PublishedStatus, type ShellHost } from './ipc.js';
import { OFFERED_PROTOCOL_VERSIONS, offering } from './protocol-versions.js';
import { withToken } from './url.js';

/** The AHP error a host answers `initialize` with when it speaks none of the versions offered. */
const NO_COMMON_VERSION = -32005;

/** What went wrong, in words a person can act on. */
export function explain(error: unknown): string {
  if (error instanceof RpcError && error.code === NO_COMMON_VERSION)
    return t('status.noCommonVersion', { versions: OFFERED_PROTOCOL_VERSIONS.join(', ') });
  return error instanceof Error ? error.message : String(error);
}

export type HostStatus =
  /** `ahpd`: the version, when the server is ahpd. */
  | { kind: 'connected'; server: string | null; ahpd: string | null }
  | { kind: 'connecting' }
  | { kind: 'down'; error: string | null };

/** The server's own name for itself, from its answer to `initialize`. */
type ServerInfo = { name: string; version?: string };

const serverName = (info: ServerInfo) =>
  info.version === undefined ? info.name : `${info.name} ${info.version}`;

/** How the desk can update a host's ahpd: over SSH (plan 09 task 05), or through ahpd's own update command (task 04). */
export type UpdateWay = { kind: 'ssh' } | { kind: 'ahpd' };

/**
 * How the desk can update a host's ahpd to `version`: over SSH for an ahpd the
 * desk installed there, else through ahpd's own update command when `may`,
 * its check, lets this token update it. A local ahpd the desk did not install
 * goes through that check too, as a remote one does.
 */
export function wayFor(host: ShellHost, version: string | null, may: boolean | null): UpdateWay | null {
  if (host.ssh?.byDesk === true) return { kind: 'ssh' };
  if (version === null) return null;
  return may === true ? { kind: 'ahpd' } : null;
}

/** How long an update through ahpd may take to come back on the new version: npm's install, and on Windows a helper that runs it after the daemon stops. */
const UPDATE_RETURN_MS = 3 * 60 * 1000;

/**
 * `SessionStatus.InProgress` of `@microsoft/agent-host-protocol`, written out
 * because `isolatedModules` cannot read an ambient const enum; `InputNeeded`
 * carries the same bit.
 */
const IN_PROGRESS = 1 << 3;

/** Passes every frame through, and tells `onServer` what the server says it is. */
function noticing(transport: AhpTransport, onServer: (info: ServerInfo) => void): AhpTransport {
  return {
    send: (message) => transport.send(message),
    close: () => transport.close(),
    async recv() {
      const frame = await transport.recv();
      if (frame !== null) {
        let message: unknown = null;
        if (frame.kind === 'parsed') message = frame.message;
        else if (frame.kind === 'text') {
          try {
            message = JSON.parse(frame.text);
          } catch {
            message = null;
          }
        }
        const info = (message as { result?: { serverInfo?: ServerInfo } } | null)?.result
          ?.serverInfo;
        if (info !== undefined && typeof info.name === 'string') onServer(info);
      }
      return frame;
    },
  };
}

export interface HostStatuses {
  /** Connects the hosts in `list`, drops the ones gone, and reconnects the ones whose address changed. */
  sync(list: readonly ShellHost[]): Promise<void>;
  status(id: string): HostStatus;
  /** Connects again now, as after a token change or a press on Retry. */
  retry(id: string): Promise<void>;
  /** The titles of the host's chats with a turn on, which an update would stop. */
  running(id: string): string[];
  /** How the desk can update the host's ahpd to `version`, the one its row names, or `null` when it cannot. */
  updateWay(host: ShellHost, version: string | null): Promise<UpdateWay | null>;
  /** Whether the host is a local one on an ahpd the desk did not install, which the person updates where it was installed. */
  selfUpdate(host: ShellHost): Promise<boolean>;
  /** Updates the host's ahpd to `version`, the one its row named; `force` once the person confirmed over running sessions. */
  updateHost(host: ShellHost, version: string, force: boolean): Promise<void>;
}

/**
 * The statuses; `redraw` is called on every change of any host. They live as
 * long as the host list window, which is as long as the app.
 */
export function createHostStatuses(redraw: () => void): HostStatuses {
  const multi = new MultiHostClient();
  // What every window last heard, so a change that moves no dot sends nothing.
  let published = '';
  let publishedRunning = '';
  const urls = new Map<string, string>();
  const servers = new Map<string, ServerInfo>();
  // One sync at a time, so two quick changes never add the same host twice.
  let syncing: Promise<void> = Promise.resolve();
  // Each connection's answer to ahpd's check about a version, kept until the host reconnects, so a redraw asks nothing.
  const checks = new Map<string, { client: unknown; version: string; may: Promise<boolean> }>();
  // Where each local host's ahpd came from, by the path set on it; asking spawns ahpd, so once a window.
  const origins = new Map<string, { path: string | null; installed: Promise<boolean> }>();

  void (async () => {
    for await (const _event of multi.hostEvents()) onChange();
  })();
  // A turn starts or ends in a host's session list, not in its connection's state.
  void (async () => {
    for await (const _event of multi.events()) publishRunning();
  })();

  const add = (host: ShellHost) =>
    multi.addHost({
      id: host.id,
      label: host.name,
      // The token is read on each connect, so a saved new token takes effect on the next one.
      transportFactory: async (id) => {
        const token = await hostToken(id);
        const socket = await WebSocketTransport.connect(withToken(urls.get(id) ?? host.url, token));
        return noticing(offering(socket), (info) => {
          servers.set(id, info);
          onChange();
        });
      },
    });

  const sync = async (list: readonly ShellHost[]) => {
    const ids = new Set(list.map((host) => host.id));
    for (const known of multi.hostsSnapshot()) {
      if (!ids.has(known.id)) {
        urls.delete(known.id);
        servers.delete(known.id);
        checks.delete(known.id);
        origins.delete(known.id);
        await multi.removeHost(known.id);
      }
    }
    for (const host of list) {
      const before = urls.get(host.id);
      urls.set(host.id, host.url);
      if (multi.host(host.id) === undefined) await add(host);
      else if (before !== host.url) await multi.reconnectHost(host.id);
    }
    onChange();
  };

  /** Every host's dot, for the windows that list hosts but watch none (decision `the-host-list-window-publishes-host-status`). */
  const publish = () => {
    const map: Record<string, PublishedStatus> = {};
    for (const known of multi.hostsSnapshot()) map[known.id] = statuses.status(known.id).kind;
    const text = JSON.stringify(map);
    if (text === published) return;
    published = text;
    invoke('publish_host_statuses', { map }).catch((err: unknown) =>
      console.error('publishing host statuses failed', err),
    );
  };

  /** Each host's running turns, for a window that asks before Restart stops them (decision `the-host-list-publishes-each-hosts-running-turns`). */
  function publishRunning() {
    const map: Record<string, string[]> = {};
    for (const known of multi.hostsSnapshot()) {
      const titles = running(known.id);
      if (titles.length > 0) map[known.id] = titles;
    }
    const text = JSON.stringify(map);
    if (text === publishedRunning) return;
    publishedRunning = text;
    invoke('publish_host_running', { map }).catch((err: unknown) =>
      console.error('publishing running turns failed', err),
    );
  }

  function onChange() {
    redraw();
    publish();
    publishRunning();
  }

  function running(id: string): string[] {
    return multi
      .aggregatedSessions()
      .filter((hosted) => hosted.hostId === id)
      .flatMap((hosted) => hosted.summary.chats ?? [])
      .filter((chat) => chat.status !== undefined && (chat.status & IN_PROGRESS) !== 0)
      .map((chat) => chat.title);
  }

  /** ahpd's check about `version` on the host's connection, or `null` while it is not a connected ahpd. */
  const checkOf = (id: string, version: string): Promise<boolean> | null => {
    const status = statuses.status(id);
    const handle = multi.client(id);
    if (status.kind !== 'connected' || status.ahpd === null || handle === undefined) return null;
    const client = handle.rawClient();
    const kept = checks.get(id);
    if (kept !== undefined && kept.client === client && kept.version === version) return kept.may;
    const may = checkUpdate(handle as unknown as Requester, version);
    checks.set(id, { client, version, may });
    return may;
  };

  /** Whether a local host's ahpd is the one the desk installed: only that one updates on its own. */
  const installedByDesk = (host: ShellHost): Promise<boolean> => {
    const path = host.ahpdPath ?? null;
    const kept = origins.get(host.id);
    if (kept !== undefined && kept.path === path) return kept.installed;
    const installed = findLocalAhpd(path).then(
      (found) => found?.origin === 'installed',
      () => false,
    );
    origins.set(host.id, { path, installed });
    return installed;
  };

  /** Resolves once the host is connected on `version`, or rejects after `UPDATE_RETURN_MS`. */
  const backOn = (id: string, version: string) =>
    new Promise<void>((resolve, reject) => {
      const until = Date.now() + UPDATE_RETURN_MS;
      const look = () => {
        const status = statuses.status(id);
        if (status.kind === 'connected' && status.ahpd === version) resolve();
        else if (Date.now() > until) reject(new Error(t('hosts.updateFailed', { why: t('status.noAnswer') })));
        else setTimeout(look, 500);
      };
      look();
    });

  const statuses: HostStatuses = {
    sync(list) {
      // A failed sync must not stop the next one.
      syncing = syncing.catch(() => undefined).then(() => sync(list));
      return syncing;
    },
    status(id) {
      const handle = multi.host(id);
      if (handle === undefined) return { kind: 'connecting' };
      switch (handle.state.status) {
        case 'connected': {
          const info = servers.get(id);
          return {
            kind: 'connected',
            server: info === undefined ? null : serverName(info),
            ahpd: info?.name === 'ahpd' && info.version !== undefined ? info.version : null,
          };
        }
        case 'connecting':
          return { kind: 'connecting' };
        default:
          return {
            kind: 'down',
            error: handle.lastError === null ? null : explain(handle.lastError),
          };
      }
    },
    retry: (id) => multi.reconnectHost(id),
    running,
    async updateWay(host, version) {
      const may = host.ssh?.byDesk === true || version === null ? null : checkOf(host.id, version);
      return wayFor(host, version, may === null ? null : await may);
    },
    async selfUpdate(host) {
      return host.kind === 'local' && !(await installedByDesk(host));
    },
    async updateHost(host, version, force) {
      if (host.ssh?.byDesk === true) {
        // The last lines npm and systemctl wrote, shown with a failure.
        const last: string[] = [];
        try {
          await updateSshAhpd(host.id, version, (event) => {
            if (event.kind !== 'line') return;
            last.push(event.text);
            if (last.length > 5) last.shift();
          });
        } catch (err) {
          const why = [err instanceof Error ? err.message : String(err), ...last].join('\n');
          throw new Error(t('hosts.updateFailed', { why }));
        }
        // The restart dropped the connection; take the new ahpd now rather than at the next retry.
        await multi.reconnectHost(host.id);
        return;
      }
      const handle = multi.client(host.id);
      const may = checkOf(host.id, version);
      if (handle === undefined || may === null || !(await may)) throw new Error(t('hosts.noUpdateWay'));
      try {
        await requestUpdate(handle as unknown as Requester, { version, force });
      } catch (err) {
        throw new Error(t('hosts.updateFailed', { why: explain(err) }));
      }
      // ahpd answered before its restart; the connection drops and comes back on its own, on the new version.
      checks.delete(host.id);
      await backOn(host.id, version);
    },
  };
  return statuses;
}

/** Connects once with `url` and `token` and returns the server's name, or throws. */
export async function testHost(url: string, token: string | null): Promise<string> {
  const timeout = <T,>(work: Promise<T>): Promise<T> =>
    Promise.race([
      work,
      new Promise<T>((_, reject) =>
        setTimeout(() => reject(new Error(t('status.noAnswer'))), 8000),
      ),
    ]);
  let transport: AhpTransport;
  try {
    transport = await timeout(WebSocketTransport.connect(withToken(url, token)));
  } catch {
    // A browser socket says nothing about why: a wrong address, a down host and a refused token look the same.
    throw new Error(t('status.couldNotConnect'));
  }
  const client = new AhpClient(transport);
  client.connect();
  try {
    const result = await timeout(
      client.initialize({ clientId: crypto.randomUUID(), protocolVersions: OFFERED_PROTOCOL_VERSIONS }),
    );
    return result.serverInfo === undefined
      ? `AHP ${result.protocolVersion}`
      : serverName(result.serverInfo);
  } finally {
    await client.shutdown();
  }
}
