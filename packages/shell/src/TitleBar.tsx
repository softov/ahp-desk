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

/** The host list's title bar: the name at the start, minimize, maximize or restore, and close at the end, as ahpw's. */
export function TitleBar() {
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
