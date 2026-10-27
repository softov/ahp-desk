import { useEffect, useState } from 'react';
import { HostForm } from './hosts/HostForm.js';
import { LocalForm } from './hosts/LocalForm.js';
import { SshForm } from './hosts/SshForm.js';
import { openHost } from './hosts/ipc.js';
import { t } from './i18n.js';
import { AhpioLogo } from './AhpioLogo.js';
import { setStartWithOs, startWithOs } from './settings/ipc.js';

type Step =
  | { kind: 'welcome' }
  | { kind: 'where' }
  | { kind: 'form' }
  | { kind: 'local' }
  | { kind: 'ssh' }
  | { kind: 'opening'; id: string; name: string; error: string | null };

/**
 * The first run, for a desk with no host: a welcome, where the host is,
 * the host form with Test, This computer's install and start, or a server
 * over SSH, and Open.
 * It ends when the new host's window opens; the desk then shows the host list.
 */
export function FirstRun({ onFinished }: { onFinished: () => void }) {
  const [step, setStep] = useState<Step>({ kind: 'welcome' });
  // "Launch at startup", checked the first time; a first run that comes back
  // after the last host is removed shows the login entry as it is.
  const [atLogin, setAtLogin] = useState<boolean | null>(null);
  const [error, setError] = useState<string | null>(null);
  useEffect(() => {
    startWithOs().then(
      (state) => setAtLogin(state.asked ? state.on : true),
      (err: unknown) => setError(String(err)),
    );
  }, []);

  const start = () => {
    if (atLogin === null) return;
    setError(null);
    setStartWithOs(atLogin).then(
      () => setStep({ kind: 'where' }),
      (err: unknown) => setError(err instanceof Error ? err.message : String(err)),
    );
  };

  const open = (id: string, name: string) => {
    setStep({ kind: 'opening', id, name, error: null });
    openHost(id).then(onFinished, (err: unknown) =>
      setStep({ kind: 'opening', id, name, error: err instanceof Error ? err.message : String(err) }),
    );
  };

  switch (step.kind) {
    case 'welcome':
      return (
        <section className="first-run">
          <AhpioLogo size={56} />
          <h1>{t('firstRun.welcomeTitle')}</h1>
          <p>{t('firstRun.welcomeText')}</p>
          <label className="check">
            <input
              type="checkbox"
              checked={atLogin ?? false}
              disabled={atLogin === null}
              onChange={(event) => setAtLogin(event.target.checked)}
            />
            <span>{t('settings.startWithOs')}</span>
          </label>
          <button className="primary" onClick={start} disabled={atLogin === null}>
            {t('firstRun.start')}
          </button>
          {error !== null && <p className="result bad">{error}</p>}
        </section>
      );
    case 'where':
      return (
        <section className="first-run">
          <h1>{t('firstRun.whereTitle')}</h1>
          <div className="choices">
            <button className="choice" onClick={() => setStep({ kind: 'form' })}>
              <strong>📡︎ {t('firstRun.remote')}</strong>
              <span>{t('firstRun.remoteHint')}</span>
            </button>
            <button className="choice" onClick={() => setStep({ kind: 'local' })}>
              <strong>🖳 {t('firstRun.local')}</strong>
              <span>{t('firstRun.localHint')}</span>
            </button>
            <button className="choice" onClick={() => setStep({ kind: 'ssh' })}>
              <strong>🖧 {t('firstRun.ssh')}</strong>
              <span>{t('firstRun.sshHint')}</span>
            </button>
          </div>
          <footer className="actions">
            <button type="button" onClick={() => setStep({ kind: 'welcome' })}>
              {t('form.back')}
            </button>
          </footer>
        </section>
      );
    case 'form':
      return (
        <HostForm
          host={null}
          taken={[]}
          firstRun
          onDone={(saved) => (saved === null ? setStep({ kind: 'where' }) : open(saved.id, saved.name))}
        />
      );
    case 'local':
      return (
        <LocalForm
          taken={[]}
          firstRun
          onDone={(saved) => (saved === null ? setStep({ kind: 'where' }) : open(saved.id, saved.name))}
          onToList={onFinished}
        />
      );
    case 'ssh':
      return (
        <SshForm
          taken={[]}
          firstRun
          onDone={(saved) => (saved === null ? setStep({ kind: 'where' }) : open(saved.id, saved.name))}
          onToList={onFinished}
        />
      );
    case 'opening':
      return (
        <section className="first-run">
          <AhpioLogo size={56} />
          {step.error === null ? (
            <p>{t('firstRun.opening', { name: step.name })}</p>
          ) : (
            <>
              <p className="result bad">{step.error}</p>
              <div className="actions">
                <button onClick={onFinished}>{t('firstRun.toList')}</button>
                <button className="primary" onClick={() => open(step.id, step.name)}>
                  {t('hosts.retry')}
                </button>
              </div>
            </>
          )}
        </section>
      );
  }
}
