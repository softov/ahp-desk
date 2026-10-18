import { t } from '../i18n.js';

/** The backends Add host offers, in `ahpd configure`'s order, and the bots. */
export const BACKENDS = [
  { name: 'Claude', pkg: '@ahpd/agent-claude' },
  { name: 'cofold', pkg: '@ahpd/agent-cofold' },
  { name: 'pi', pkg: '@ahpd/agent-pi' },
] as const;
export const BOT = '@ahpd/bot';

/**
 * Whether the host will have a backend: one ticked, or one a found ahpd has
 * ready. A ready plugin the desk does not offer counts, as the list cannot
 * tell a backend from another plugin.
 */
export function hasBackend(has: ReadonlySet<string>, backends: ReadonlySet<string>): boolean {
  return (
    BACKENDS.some((one) => backends.has(one.pkg) || has.has(one.pkg)) ||
    [...has].some((pkg) => pkg !== BOT && !BACKENDS.some((one) => one.pkg === pkg))
  );
}

/** What a found ahpd starts ticked with: nothing when it has a backend ready, else Claude. */
export function startingBackends(found: readonly string[]): Set<string> {
  const ready = found.filter((pkg) => pkg !== '' && pkg !== BOT);
  return new Set(ready.length > 0 ? [] : ['@ahpd/agent-claude']);
}

/** The backends' checkboxes; the ones a found ahpd has ready are ticked and fixed. */
export function BackendsField({
  has,
  backends,
  onChange,
  disabled,
}: {
  has: ReadonlySet<string>;
  backends: ReadonlySet<string>;
  onChange: (backends: Set<string>) => void;
  disabled: boolean;
}) {
  const toggle = (pkg: string, on: boolean) => {
    const next = new Set(backends);
    if (on) next.add(pkg);
    else next.delete(pkg);
    onChange(next);
  };
  return (
    <div className="field">
      <span>{t('local.backends')}</span>
      <div className="checks">
        {[...BACKENDS, { name: t('local.bot'), pkg: BOT }].map((one) => (
          <label className="check" key={one.pkg}>
            <input
              type="checkbox"
              checked={has.has(one.pkg) || backends.has(one.pkg)}
              onChange={(event) => toggle(one.pkg, event.target.checked)}
              disabled={disabled || has.has(one.pkg)}
            />
            <span>{one.name}</span>
            <small>{has.has(one.pkg) ? t('local.installed') : one.pkg}</small>
          </label>
        ))}
      </div>
    </div>
  );
}
