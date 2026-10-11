/**
 * Each host's live state, from one AHP connection per host in a
 * `MultiHostClient` that reconnects on its own.
 */

import { AhpClient, RpcError, type AhpTransport } from '@microsoft/agent-host-protocol/client';
import { MultiHostClient } from '@microsoft/agent-host-protocol/hosts';
import { WebSocketTransport } from '@microsoft/agent-host-protocol/ws';
import { invoke } from '@tauri-apps/api/core';

import { hostToken, type PublishedStatus, type ShellHost } from './ipc.js';
import { OFFERED_PROTOCOL_VERSIONS, offering } from './protocol-versions.js';
import { withToken } from './url.js';

/** The AHP error a host answers `initialize` with when it speaks none of the versions offered. */
const NO_COMMON_VERSION = -32005;

/** What went wrong, in words a person can act on. */
export function explain(error: unknown): string {
  if (error instanceof RpcError && error.code === NO_COMMON_VERSION)
    return `This host speaks none of the protocol versions ${OFFERED_PROTOCOL_VERSIONS.join(', ')}. Update its server.`;
  return error instanceof Error ? error.message : String(error);
}

export type HostStatus =
  | { kind: 'connected'; server: string | null }
  | { kind: 'connecting' }
  | { kind: 'down'; error: string | null };

/** The server's own name for itself, from its answer to `initialize`. */
type ServerInfo = { name: string; version?: string };

const serverName = (info: ServerInfo) =>
  info.version === undefined ? info.name : `${info.name} ${info.version}`;

/** Passes every frame through, and tells `onServer` what the server says it is. */
function noticing(transport: AhpTransport, onServer: (name: string) => void): AhpTransport {
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
        if (info !== undefined && typeof info.name === 'string') onServer(serverName(info));
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
}

/**
 * The statuses; `redraw` is called on every change of any host. They live as
 * long as the host list window, which is as long as the app.
 */
export function createHostStatuses(redraw: () => void): HostStatuses {
  const multi = new MultiHostClient();
  // What every window last heard, so a change that moves no dot sends nothing.
  let published = '';
  const urls = new Map<string, string>();
  const servers = new Map<string, string>();
  // One sync at a time, so two quick changes never add the same host twice.
  let syncing: Promise<void> = Promise.resolve();

  void (async () => {
    for await (const _event of multi.hostEvents()) onChange();
  })();

  const add = (host: ShellHost) =>
    multi.addHost({
      id: host.id,
      label: host.name,
      // The token is read on each connect, so a saved new token takes effect on the next one.
      transportFactory: async (id) => {
        const token = await hostToken(id);
        const socket = await WebSocketTransport.connect(withToken(urls.get(id) ?? host.url, token));
        return noticing(offering(socket), (name) => {
          servers.set(id, name);
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

  /** Every host's dot, for the windows that list hosts but watch none (decision: the host list publishes). */
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

  function onChange() {
    redraw();
    publish();
  }

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
        case 'connected':
          return { kind: 'connected', server: servers.get(id) ?? null };
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
  };
  return statuses;
}

/** Connects once with `url` and `token` and returns the server's name, or throws. */
export async function testHost(url: string, token: string | null): Promise<string> {
  const timeout = <T,>(work: Promise<T>): Promise<T> =>
    Promise.race([
      work,
      new Promise<T>((_, reject) =>
        setTimeout(() => reject(new Error('No answer in 8 seconds.')), 8000),
      ),
    ]);
  let transport: AhpTransport;
  try {
    transport = await timeout(WebSocketTransport.connect(withToken(url, token)));
  } catch {
    // A browser socket says nothing about why: a wrong address, a down host and a refused token look the same.
    throw new Error('Could not connect. Check the address and the token.');
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
