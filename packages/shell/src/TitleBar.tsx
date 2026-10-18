import { useEffect, useRef, useState } from 'react';
import { LANG_NAMES, LANGS, setLanguage, t, useLanguage, type Lang } from './i18n.js';
import { Mark } from './Mark.js';
import { useOutsideClose } from './useOutsideClose.js';
import { controls } from './window.js';

/** The language as the picker's button shows it. */
const SHORT: Record<Lang, string> = { en: 'EN', 'pt-BR': 'PT', es: 'ES' };

/** The language picker: the current language, and a menu of the three, each in itself. */
function LanguagePicker() {
  const lang = useLanguage();
  const [open, setOpen] = useState(false);
  const ref = useRef<HTMLDivElement>(null);
  useOutsideClose(ref, open, () => setOpen(false));
  return (
    <div className="menu" ref={ref}>
      <button
        type="button"
        className="control control--language"
        title={t('window.language')}
        aria-label={t('window.language')}
        aria-haspopup="menu"
        aria-expanded={open}
        onClick={() => setOpen(!open)}
      >
        {SHORT[lang]}
      </button>
      {open && (
        <div className="menu-items" role="menu">
          {LANGS.map((each) => (
            <button
              key={each}
              type="button"
              role="menuitemradio"
              aria-checked={each === lang}
              lang={each}
              onClick={() => {
                setOpen(false);
                setLanguage(each);
              }}
            >
              {LANG_NAMES[each]}
            </button>
          ))}
        </div>
      )}
    </div>
  );
}

/** The host list's title bar: the name at the start; Settings, the language, minimize, maximize or restore, and close at the end. */
export function TitleBar({ onSettings }: { onSettings?: () => void }) {
  const [maximized, setMaximized] = useState(false);

  useEffect(() => {
    let live = true;
    const sync = async () => {
      const now = await controls.isMaximized();
      if (live) setMaximized(now);
    };
    void sync();
    const stop = controls.onResized(() => void sync());
    return () => {
      live = false;
      void stop.then((unlisten) => unlisten());
    };
  }, []);

  return (
    <header className="titlebar">
      <span className="titlebar__name">
        <Mark size={14} />
        ahp-desk
      </span>
      <div className="titlebar__controls">
        {onSettings !== undefined && (
          <button type="button" className="control" title={t('window.settings')} aria-label={t('window.settings')} onClick={onSettings}>
            <svg width="10" height="10" viewBox="0 0 10 10" aria-hidden="true">
              <path
                d="M8.18 3.80 L9.51 4.09 L9.51 5.91 L8.18 6.20 L8.10 6.40 L8.83 7.54 L7.54 8.83 L6.40 8.10 L6.20 8.18 L5.91 9.51 L4.09 9.51 L3.80 8.18 L3.60 8.10 L2.46 8.83 L1.17 7.54 L1.90 6.40 L1.82 6.20 L0.49 5.91 L0.49 4.09 L1.82 3.80 L1.90 3.60 L1.17 2.46 L2.46 1.17 L3.60 1.90 L3.80 1.82 L4.09 0.49 L5.91 0.49 L6.20 1.82 L6.40 1.90 L7.54 1.17 L8.83 2.46 L8.10 3.60 Z"
                fill="none"
                stroke="currentColor"
                strokeLinejoin="round"
                strokeWidth="0.9"
              />
              <circle cx="5" cy="5" r="1.4" fill="none" stroke="currentColor" strokeWidth="0.9" />
            </svg>
          </button>
        )}
        <LanguagePicker />
        <button type="button" className="control" title={t('window.minimize')} aria-label={t('window.minimize')} onClick={() => void controls.minimize()}>
          <svg width="10" height="10" viewBox="0 0 10 10" aria-hidden="true">
            <path d="M0 5.5h10" stroke="currentColor" />
          </svg>
        </button>
        <button
          type="button"
          className="control"
          title={maximized ? t('window.restore') : t('window.maximize')}
          aria-label={maximized ? t('window.restore') : t('window.maximize')}
          onClick={() => void controls.toggleMaximize()}
        >
          {maximized ? (
            <svg width="10" height="10" viewBox="0 0 10 10" aria-hidden="true">
              <path d="M2.5 2.5V.5h7v7h-2M.5 2.5h7v7h-7z" fill="none" stroke="currentColor" />
            </svg>
          ) : (
            <svg width="10" height="10" viewBox="0 0 10 10" aria-hidden="true">
              <path d="M.5.5h9v9h-9z" fill="none" stroke="currentColor" />
            </svg>
          )}
        </button>
        <button type="button" className="control control--close" title={t('window.close')} aria-label={t('window.close')} onClick={() => void controls.close()}>
          <svg width="10" height="10" viewBox="0 0 10 10" aria-hidden="true">
            <path d="M.5.5l9 9M9.5.5l-9 9" stroke="currentColor" />
          </svg>
        </button>
      </div>
    </header>
  );
}
