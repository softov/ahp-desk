import { useEffect, useState } from 'react';
import { t } from '../i18n.js';
import { getSettings, saveSettings, setStartWithOs, startWithOs, type CloseBehavior, type Settings, type Windows } from './ipc.js';

const CLOSE: readonly { value: CloseBehavior; key: 'settings.closeTray' | 'settings.closeQuit' }[] = [
  { value: 'minimize-to-tray', key: 'settings.closeTray' },
  { value: 'quit', key: 'settings.closeQuit' },
];

const WINDOWS: readonly { value: Windows; key: 'settings.windowsPerHost' | 'settings.windowsEveryHost' }[] = [
  { value: 'per-host', key: 'settings.windowsPerHost' },
  { value: 'every-host', key: 'settings.windowsEveryHost' },
];

/** The desk's settings, each saved as it changes; Back returns to the host list. */
export function SettingsScreen({ onDone }: { onDone: () => void }) {
  const [settings, setSettings] = useState<Settings | null>(null);
  // The OS's login entry; null until it is read.
  const [atLogin, setAtLogin] = useState<boolean | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    getSettings().then(setSettings, (err: unknown) => setError(String(err)));
    startWithOs().then((state) => setAtLogin(state.on), (err: unknown) => setError(String(err)));
  }, []);

  const changeAtLogin = (on: boolean) => {
    setError(null);
    setStartWithOs(on).then((state) => setAtLogin(state.on), (err: unknown) => setError(String(err)));
  };

  const change = (next: Settings) => {
    setError(null);
    saveSettings(next).then(setSettings, (err: unknown) => setError(String(err)));
  };

  return (
    <section className="panel">
      <header className="panel-head">
        <h1>{t('settings.title')}</h1>
      </header>
      {settings !== null && (
        <div className="fields">
          <fieldset className="choice">
            <legend>{t('settings.close')}</legend>
            {CLOSE.map((one) => (
              <label className="check" key={one.value}>
                <input
                  type="radio"
                  name="close"
                  checked={settings.closeBehavior === one.value}
                  onChange={() => change({ ...settings, closeBehavior: one.value })}
                />
                <span>{t(one.key)}</span>
              </label>
            ))}
          </fieldset>
          <fieldset className="choice">
            <legend>{t('settings.windows')}</legend>
            {WINDOWS.map((one) => (
              <label className="check" key={one.value}>
                <input
                  type="radio"
                  name="windows"
                  checked={settings.windows === one.value}
                  onChange={() => change({ ...settings, windows: one.value })}
                />
                <span>{t(one.key)}</span>
              </label>
            ))}
          </fieldset>
          {atLogin !== null && (
            <label className="check check--alone">
              <input type="checkbox" checked={atLogin} onChange={(event) => changeAtLogin(event.target.checked)} />
              <span>{t('settings.startWithOs')}</span>
            </label>
          )}
        </div>
      )}
      {error !== null && <p className="result bad">{error}</p>}
      <footer className="actions">
        <button type="button" onClick={onDone}>
          {t('form.back')}
        </button>
      </footer>
    </section>
  );
}
