/**
 * ahpd's own update command over AHP: plan 09 task 03, released in ahpd 0.13.0
 * as its daemon/35. With `check` it answers, changing nothing, whether this
 * connection may update the daemon; with a version it installs that version
 * and restarts, answering before the restart drops the connection.
 */

/** The AHP method of ahpd's update command. */
export const UPDATE_METHOD = 'x-ahpd/update';

export interface UpdateParams {
  version?: string;
  check?: boolean;
  force?: boolean;
}

/**
 * What sends a request on a host's connection: the `HostClientHandle` of a
 * `MultiHostClient`, whose typed `request` knows only the protocol's own
 * methods, so ahpd's is sent through this looser shape.
 */
export interface Requester {
  request(method: string, params: unknown): Promise<unknown>;
}

/** Whether ahpd's answer to a check lets this connection update it. A refusal or a malformed answer is false. */
export function mayUpdate(answer: unknown): boolean {
  return typeof answer === 'object' && answer !== null && (answer as { may?: unknown }).may === true;
}

/** Sends the command on `client`. */
export function requestUpdate(client: Requester, params: UpdateParams): Promise<unknown> {
  return client.request(UPDATE_METHOD, params);
}

/**
 * Asks with `check` about `version`, the one the row would install, so ahpd
 * answers about that version and asks npm nothing of its own. An ahpd without
 * the command (`-32601`), a token it refuses, or a check that fails for any
 * other reason is false: the row keeps its notice without Update.
 */
export async function checkUpdate(client: Requester, version: string): Promise<boolean> {
  try {
    return mayUpdate(await requestUpdate(client, { version, check: true }));
  } catch {
    return false;
  }
}
