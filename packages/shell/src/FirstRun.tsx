import { useState } from 'react';
import { HostForm } from './hosts/HostForm.js';
import { openHost } from './hosts/ipc.js';
import { t } from './i18n.js';
import { Mark } from './Mark.js';

type Step =
  | { kind: 'welcome' }
  | { kind: 'where' }
  | { kind: 'form' }
  | { kind: 'opening'; id: string; name: string; error: string | null };

/**
 * The first run, for a desk with no host (decision: the first run adds a URL
 * host): a welcome, where the host is, the host form with Test, and Open.
 * It ends when the new host's window opens; the desk then shows the host list.
 */
export function FirstRun({ onFinished }: { onFinished: () => void }) {
  const [step, setStep] = useState<Step>({ kind: 'welcome' });

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
          <Mark size={56} />
          <h1>{t('firstRun.welcomeTitle')}</h1>
          <p>{t('firstRun.welcomeText')}</p>
          <button className="primary" onClick={() => setStep({ kind: 'where' })}>
            {t('firstRun.start')}
          </button>
        </section>
      );
    case 'where':
      return (
        <section className="panel">
          <header className="panel-head">
            <h1>{t('firstRun.whereTitle')}</h1>
          </header>
          <div className="choices">
            <button className="choice" onClick={() => setStep({ kind: 'form' })}>
              <strong>{t('firstRun.remote')}</strong>
              <span>{t('firstRun.remoteHint')}</span>
            </button>
            <button className="choice" disabled>
              <strong>
                {t('firstRun.local')}
                <span className="badge">{t('firstRun.soon')}</span>
              </strong>
              <span>{t('firstRun.localHint')}</span>
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
    case 'opening':
      return (
        <section className="first-run">
          <Mark size={56} />
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
