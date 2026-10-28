import { openUrl } from '@tauri-apps/plugin-opener';
import { useEffect, useState } from 'react';
import { AhpioLogo } from '../AhpioLogo.js';
import { t, useLanguage } from '../i18n.js';
import { WindowBar } from '../WindowBar.js';
import { about, type About } from './ipc.js';

const SITE = 'https://ahpio.com';
const RELEASES = 'https://github.com/softov/ahp-desk/releases';

/** The About window: the desk's, ahpw's and Node's versions, the license, and the site and releases. */
export function AboutWindow() {
  // A pick in the host list reaches this window through the language's storage event.
  useLanguage();
  const [versions, setVersions] = useState<About | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    about().then(setVersions, (err: unknown) => setError(String(err)));
  }, []);

  const link = (url: string, label: string) => (
    <a
      href={url}
      onClick={(event) => {
        event.preventDefault();
        openUrl(url).catch((err: unknown) => setError(String(err)));
      }}
    >
      {label}
    </a>
  );

  return (
    <>
      <WindowBar title={t('about.title')} />
      <main className="about">
        <AhpioLogo size={40} />
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
        {error !== null && <p className="result bad">{error}</p>}
        <p className="about-links">
          <span>{t('about.license')}</span>
          {link(SITE, 'ahpio.com')}
          {link(RELEASES, t('about.releases'))}
        </p>
      </main>
    </>
  );
}
