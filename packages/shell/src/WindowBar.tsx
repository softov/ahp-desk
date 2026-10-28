import { getCurrentWindow } from '@tauri-apps/api/window';
import { useEffect } from 'react';
import { AhpioLogo } from './AhpioLogo.js';
import { t } from './i18n.js';
import { controls } from './window.js';

/** A side window's title bar: the logo and the window's title, then minimize and close; it cannot maximize. */
export function WindowBar({ title }: { title: string }) {
  // The OS shows this title in the taskbar and Alt+Tab.
  useEffect(() => {
    getCurrentWindow()
      .setTitle(title)
      .catch((err: unknown) => console.error('window title failed', err));
  }, [title]);

  return (
    <header className="titlebar">
      <div className="titlebar__title">
        <AhpioLogo size={14} />
        <span>{title}</span>
      </div>
      <div className="titlebar__controls">
        <button type="button" className="control" title={t('window.minimize')} aria-label={t('window.minimize')} onClick={() => void controls.minimize()}>
          <svg width="10" height="10" viewBox="0 0 10 10" aria-hidden="true">
            <path d="M0 5.5h10" stroke="currentColor" />
          </svg>
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
