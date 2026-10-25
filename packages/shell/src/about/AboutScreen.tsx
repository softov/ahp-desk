import { openUrl } from '@tauri-apps/plugin-opener';
import { useEffect, useState } from 'react';
import { DeskUpdateBar } from '../hosts/DeskUpdateBar.js';
import type { ShellHost } from '../hosts/ipc.js';
import type { HostStatuses } from '../hosts/status.js';
import { t } from '../i18n.js';
import { about, checkDeskUpdate, type About } from './ipc.js';

const SITE = 'https://ahpio.com';
const RELEASES = 'https://github.com/softov/ahp-desk/releases';

/** Check now's state; a found update shows as the ready bar, from the desk's own event. */
type Check = { kind: 'idle' } | { kind: 'checking' } | { kind: 'upToDate' } | { kind: 'failed'; why: string };

/** The desk's, ahpw's and Node's versions and the desk's update state; Back returns to the host list. */
export function AboutScreen({
  deskUpdate,
  hosts,
  statuses,
  onDone,
}: {
  deskUpdate: string | null;
  hosts: readonly ShellHost[];
  statuses: HostStatuses;
  onDone: () => void;
}) {
  const [versions, setVersions] = useState<About | null>(null);
  const [check, setCheck] = useState<Check>({ kind: 'idle' });
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    about().then(setVersions, (err: unknown) => setError(String(err)));
  }, []);

  const checkNow = () => {
    setCheck({ kind: 'checking' });
    checkDeskUpdate().then(
      (ready) => setCheck(ready === null ? { kind: 'upToDate' } : { kind: 'idle' }),
      (err: unknown) => setCheck({ kind: 'failed', why: err instanceof Error ? err.message : String(err) }),
    );
  };

  const open = (url: string) => openUrl(url).catch((err: unknown) => setError(String(err)));

  return (
    <section className="panel">
      <header className="panel-head">
        <h1>{t('about.title')}</h1>
      </header>
      {deskUpdate !== null ? (
        <DeskUpdateBar version={deskUpdate} hosts={hosts} statuses={statuses} onError={setError} />
      ) : (
        <div className="desk-update">
          <span className={check.kind === 'failed' ? 'bad' : undefined}>
            {check.kind === 'checking'
              ? t('about.checking')
              : check.kind === 'upToDate'
                ? t('about.upToDate')
                : check.kind === 'failed'
                  ? check.why
                  : ''}
          </span>
          <span className="desk-update-buttons">
            <button className="primary" onClick={checkNow} disabled={check.kind === 'checking'}>
              {t('about.check')}
            </button>
          </span>
        </div>
      )}
      {versions !== null && (
        <dl className="about-versions">
          <dt>ahp-desk</dt>
          <dd>{versions.desk}</dd>
          <dt>ahpw</dt>
          <dd>{versions.ahpw ?? '-'}</dd>
          <dt>Node</dt>
          <dd>{versions.node ?? '-'}</dd>
        </dl>
      )}
      <p className="about-links">
        <span>{t('about.license')}</span>
        <a
          href={SITE}
          onClick={(event) => {
            event.preventDefault();
            void open(SITE);
          }}
        >
          ahpio.com
        </a>
        <a
          href={RELEASES}
          onClick={(event) => {
            event.preventDefault();
            void open(RELEASES);
          }}
        >
          {t('about.releases')}
        </a>
      </p>
      {error !== null && <p className="result bad">{error}</p>}
      <footer className="actions">
        <button type="button" onClick={onDone}>
          {t('form.back')}
        </button>
      </footer>
    </section>
  );
}
