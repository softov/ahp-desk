/** `X.Y.Z` with an optional prerelease tag; anything else is not a version here. */
const VERSION = /^(\d+)\.(\d+)\.(\d+)(?:-([0-9A-Za-z.-]+))?$/;

/**
 * Whether `latest` is ahead of `current`, as ahpd's `newer()` in its
 * `update.ts`: a prerelease loses to its own release, and anything that does
 * not parse, as a local build's `unknown`, is not behind.
 */
export function newer(latest: string, current: string): boolean {
  const a = VERSION.exec(latest);
  const b = VERSION.exec(current);
  if (a === null || b === null) return false;
  for (let i = 1; i <= 3; i++) {
    if (Number(a[i]) !== Number(b[i])) return Number(a[i]) > Number(b[i]);
  }
  return b[4] !== undefined && a[4] === undefined;
}
