/** A bare address is what somebody types; `ws://` is the scheme to guess, as ahpapp does. */
export function normalizeUrl(raw: string): string {
  const trimmed = raw.trim();
  if (trimmed === '') return trimmed;
  return /^wss?:\/\//i.test(trimmed) ? trimmed : `ws://${trimmed}`;
}

/**
 * The address with the token as `?tkn=`. A webview's WebSocket cannot set an
 * `Authorization` header, and ahpd reads the token from either.
 */
export function withToken(url: string, token: string | null): string {
  if (token === null || token === '') return url;
  const separator = url.includes('?') ? '&' : '?';
  return `${url}${separator}tkn=${encodeURIComponent(token)}`;
}

/** The address as the list shows it: without the scheme, when it is plain `ws://`. */
export function shortUrl(url: string): string {
  return url.replace(/^ws:\/\//i, '');
}

/** An id for a new host from its name: lower case, dashes, unique among `taken`. */
export function hostId(name: string, taken: readonly string[]): string {
  const base =
    name
      .toLowerCase()
      .normalize('NFKD')
      .replace(/[^a-z0-9]+/g, '-')
      .replace(/^-+|-+$/g, '') || 'host';
  let id = base;
  for (let n = 2; taken.includes(id); n += 1) id = `${base}-${n}`;
  return id;
}

/** The server Add host over SSH names, its fields trimmed, or the field that is wrong. */
export function sshTarget(
  host: string,
  port: string,
  user: string,
): { ok: true; host: string; port: number; user: string } | { ok: false; field: 'host' | 'port' | 'user' } {
  const name = host.trim();
  // A name, an IPv4 or an IPv6 address: no spaces, nothing a shell or URL would split on.
  if (name === '' || !/^[A-Za-z0-9.:_-]+$/.test(name)) return { ok: false, field: 'host' };
  const number = portOf(port);
  if (number === null) return { ok: false, field: 'port' };
  const login = user.trim();
  if (login === '' || /\s/.test(login)) return { ok: false, field: 'user' };
  return { ok: true, host: name, port: number, user: login };
}

/** The port ahpd listens on, on the server: from 1 to 65535, or `null`. */
export function ahpdPort(text: string): number | null {
  return portOf(text);
}

function portOf(text: string): number | null {
  const trimmed = text.trim();
  if (!/^\d+$/.test(trimmed)) return null;
  const port = Number(trimmed);
  return port >= 1 && port <= 65535 ? port : null;
}

/**
 * The folder ahpd serves on the server, as an absolute path: `~` and `~/…`
 * from the server user's home, an absolute path as typed; `null` for anything
 * else, which ahpd would read from wherever it runs.
 */
export function serverFolder(text: string, home: string): string | null {
  const typed = text.trim();
  const base = home.replace(/\/+$/, '');
  if (typed === '~' || typed === '~/') return base;
  if (typed.startsWith('~/')) return `${base}/${typed.slice(2)}`.replace(/\/+$/, '');
  if (typed.startsWith('/')) return typed.length > 1 ? typed.replace(/\/+$/, '') : typed;
  return null;
}
