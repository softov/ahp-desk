import { describe, expect, it } from 'vitest';
import { checkUpdate, mayUpdate, requestUpdate, UPDATE_METHOD, type Requester } from './ahpd-update.js';
import type { ShellHost } from './ipc.js';
import { wayFor } from './status.js';

/** A connection that answers each request with `answer`, or throws it, and keeps what it was sent. */
function fake(answer: unknown, fails = false): Requester & { sent: [string, unknown][] } {
  const sent: [string, unknown][] = [];
  return {
    sent,
    request(method, params) {
      sent.push([method, params]);
      return fails ? Promise.reject(answer) : Promise.resolve(answer);
    },
  };
}

describe('mayUpdate', () => {
  it.each([
    [{ current: '0.12.0', latest: '0.13.0', may: true }, true],
    [{ current: '0.12.0', latest: '0.13.0', may: false, why: 'a checkout' }, false],
    [{ may: 'true' }, false],
    [null, false],
    ['may', false],
  ])('%j is %s', (answer, expected) => {
    expect(mayUpdate(answer)).toBe(expected);
  });
});

describe('requestUpdate', () => {
  it('sends x-ahpd/update with the params given', async () => {
    const client = fake({ from: '0.12.0', to: '0.13.0', restarting: true });
    expect(await requestUpdate(client, { version: '0.13.0', force: true })).toEqual({ from: '0.12.0', to: '0.13.0', restarting: true });
    expect(client.sent).toEqual([[UPDATE_METHOD, { version: '0.13.0', force: true }]]);
  });
});

describe('checkUpdate', () => {
  it('asks with check about the version the row names, and is true when ahpd says may', async () => {
    const client = fake({ current: '0.12.0', latest: '0.13.0', may: true });
    expect(await checkUpdate(client, '0.13.0')).toBe(true);
    expect(client.sent).toEqual([[UPDATE_METHOD, { version: '0.13.0', check: true }]]);
  });

  it('is false for a refused token, an ahpd without the command, and a refusal by check', async () => {
    expect(await checkUpdate(fake({ code: -32009, message: 'only the deployment token' }, true), '0.13.0')).toBe(false);
    expect(await checkUpdate(fake({ code: -32601, message: 'Method not found' }, true), '0.13.0')).toBe(false);
    expect(await checkUpdate(fake({ current: '0.12.0', latest: '0.13.0', may: false, why: 'a link' }), '0.13.0')).toBe(false);
  });
});

describe('wayFor', () => {
  const local: ShellHost = { id: 'l', name: 'This computer', kind: 'local', url: 'ws://127.0.0.1:9187' };
  const sshByDesk: ShellHost = {
    id: 's',
    name: 'Server',
    kind: 'ssh',
    url: 'ws://127.0.0.1:40000',
    ssh: { address: 'server.example', port: 22, user: 'person', reach: 'tunnel', remotePort: 9187, byDesk: true },
  };

  it('is over SSH for an ahpd the desk installed there, with or without a check', () => {
    expect(wayFor(sshByDesk, '0.13.0', null)).toEqual({ kind: 'ssh' });
    expect(wayFor(sshByDesk, null, null)).toEqual({ kind: 'ssh' });
  });

  it("is ahpd's own command for a local host whose check answers true, the desk's install or not", () => {
    expect(wayFor(local, '0.13.0', true)).toEqual({ kind: 'ahpd' });
  });

  it('is none for a check that answers false or none, and with no version', () => {
    expect(wayFor(local, '0.13.0', false)).toBeNull();
    expect(wayFor(local, '0.13.0', null)).toBeNull();
    expect(wayFor(local, null, true)).toBeNull();
  });
});
