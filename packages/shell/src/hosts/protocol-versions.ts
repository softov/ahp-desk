/**
 * The versions the host list offers a host in `initialize`.
 *
 * Copied from ahpw's `src/connection/protocol-versions.ts`, which says why:
 * the SDK's host runtime offers only `PROTOCOL_VERSION`, so a host on an
 * earlier release it still supports refuses, and VS Code 1.141's host speaks
 * `0.10.0`, which is not a published release. The transport rewrites the
 * `initialize` request on its way out.
 */

import { SUPPORTED_PROTOCOL_VERSIONS } from '@microsoft/agent-host-protocol';
import type { AhpTransport } from '@microsoft/agent-host-protocol/client';

/** VS Code 1.141's protocol version, offered after `1.0.0`. */
const VSCODE_1_141_VERSION = '0.10.0';

/** The versions an `initialize` offers, most preferred first. */
export const OFFERED_PROTOCOL_VERSIONS: readonly string[] = Object.freeze(
  SUPPORTED_PROTOCOL_VERSIONS.flatMap((version) =>
    version === '1.0.0'
      ? [version, VSCODE_1_141_VERSION]
      : version === VSCODE_1_141_VERSION
        ? []
        : [version],
  ),
);

const record = (value: unknown): Record<string, unknown> | null =>
  typeof value === 'object' && value !== null && !Array.isArray(value)
    ? (value as Record<string, unknown>)
    : null;

const parsed = (text: string): unknown => {
  try {
    return JSON.parse(text);
  } catch {
    return null;
  }
};

/** An `initialize` request offering `OFFERED_PROTOCOL_VERSIONS`; any other frame as it came. */
export function offeringEveryVersion<T>(message: T): T {
  const body = record(typeof message === 'string' ? parsed(message) : message);
  if (body?.method !== 'initialize') return message;
  const params = { ...record(body.params), protocolVersions: [...OFFERED_PROTOCOL_VERSIONS] };
  const rewritten = { ...body, params };
  return (typeof message === 'string' ? JSON.stringify(rewritten) : rewritten) as T;
}

/** The same transport, with an `initialize` offering every version on its way out. */
export function offering(inner: AhpTransport): AhpTransport {
  return {
    send: (message) => inner.send(offeringEveryVersion(message)),
    close: () => inner.close(),
    recv: () => inner.recv(),
  };
}
