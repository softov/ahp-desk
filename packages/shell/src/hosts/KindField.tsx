import { t } from '../i18n.js';

export type AddKind = 'local' | 'url';

/**
 * Add host's Kind: This computer or URL. This computer is one host, so once it
 * is in the list it cannot be picked again.
 */
export function KindField({
  kind,
  localTaken,
  onKind,
}: {
  kind: AddKind;
  localTaken: boolean;
  onKind: (kind: AddKind) => void;
}) {
  return (
    <div className="field">
      <span>{t('add.kind')}</span>
      <div className="segmented" role="radiogroup" aria-label={t('add.kind')}>
        <button
          type="button"
          role="radio"
          aria-checked={kind === 'local'}
          disabled={localTaken}
          title={localTaken ? t('local.oneOnly') : undefined}
          onClick={() => onKind('local')}
        >
          {t('firstRun.local')}
        </button>
        <button type="button" role="radio" aria-checked={kind === 'url'} onClick={() => onKind('url')}>
          {t('add.url')}
        </button>
      </div>
    </div>
  );
}
