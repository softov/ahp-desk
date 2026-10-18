import type { ReactNode } from 'react';
import { t } from '../i18n.js';

/** The oldest ahpd the desk runs, as `local_ahpd::OLDEST_SUPPORTED`. */
export const NEEDED = '0.8.0';

/**
 * The ahpd field of Add host: what was found, in one line with its buttons
 * beside it, or an ahpd too old, with how to update it and Check again.
 */
export function FoundAhpdField({
  label,
  line,
  tooOld,
  update,
  onCheckAgain,
  actions,
  disabled,
}: {
  label: string;
  /** What was found, or what is going on, as its own paragraphs; not shown while too old. */
  line: ReactNode;
  /** The version of an ahpd too old for the desk, shown instead of `line`. */
  tooOld: string | null;
  /** The command that updates it, under the note. */
  update: string;
  onCheckAgain: () => void;
  /** Buttons after Check again, such as This computer's Change. */
  actions?: ReactNode;
  disabled: boolean;
}) {
  return (
    <div className="field">
      <span>{label}</span>
      <div className={tooOld !== null ? 'found' : 'found found--line'}>
        {tooOld === null ? (
          line
        ) : (
          <>
            <p>{t('local.tooOld', { version: tooOld, needed: NEEDED })}</p>
            <code>{update}</code>
          </>
        )}
        <div className="found-actions">
          {tooOld !== null && (
            <button type="button" onClick={onCheckAgain} disabled={disabled}>
              {t('local.checkAgain')}
            </button>
          )}
          {actions}
        </div>
      </div>
    </div>
  );
}
