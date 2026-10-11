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
