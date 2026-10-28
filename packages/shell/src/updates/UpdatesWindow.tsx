import { useEffect, useState } from 'react';
import { about } from '../about/ipc.js';
import { DeskUpdateBar } from '../hosts/DeskUpdateBar.js';
import { deskUpdate, hostRunning, listHosts, onDeskUpdateReady, onHostRunning, onHostsChanged, type ShellHost } from '../hosts/ipc.js';
import { t, useLanguage } from '../i18n.js';
import { WindowBar } from '../WindowBar.js';
import { checkDeskUpdate } from './ipc.js';

/** Check now's state; a found update shows as the ready bar, from the desk's own event. */
type Check = { kind: 'idle' } | { kind: 'checking' } | { kind: 'upToDate' } | { kind: 'failed'; why: string };

/**
 * The Updates window: the running desk's version, then Check now and what it
 * found, or with an update ready its Restart, which names the turns a local
 * ahpd's stop would end from the host list's published map.
 */
export function UpdatesWindow() {
  // A pick in the host list reaches this window through the language's storage event.
  useLanguage();
  const [version, setVersion] = useState<string | null>(null);
  const [ready, setReady] = useState<string | null>(null);
  const [hosts, setHosts] = useState<ShellHost[]>([]);
  const [running, setRunning] = useState<Record<string, string[]>>({});
  const [check, setCheck] = useState<Check>({ kind: 'idle' });
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const fail = (err: unknown) => setError(String(err));
    about().then((versions) => setVersion(versions.desk), fail);
    deskUpdate().then((found) => found !== null && setReady(found), fail);
    listHosts().then(setHosts, fail);
    hostRunning().then(setRunning, fail);
    const stops = [onDeskUpdateReady(setReady), onHostsChanged(setHosts), onHostRunning(setRunning)];
    return () => {
      for (const stop of stops) void stop.then((unlisten) => unlisten());
    };
  }, []);

  const checkNow = () => {
    setCheck({ kind: 'checking' });
    checkDeskUpdate().then(
      (found) => setCheck(found === null ? { kind: 'upToDate' } : { kind: 'idle' }),
      (err: unknown) => setCheck({ kind: 'failed', why: err instanceof Error ? err.message : String(err) }),
    );
  };

  return (
    <>
      <WindowBar title={t('updates.title')} />
      <main className="updates">
        {version !== null && <p className="updates-version">ahp-desk {version}</p>}
        {/* With an update ready, the bar is the state; Check now has nothing to add. */}
        {ready !== null ? (
          <DeskUpdateBar version={ready} hosts={hosts} running={(id) => running[id] ?? []} onError={setError} />
        ) : (
          <div className="updates-check">
            <button onClick={checkNow} disabled={check.kind === 'checking'}>
              {t('updates.check')}
            </button>
            <span className={check.kind === 'failed' ? 'bad' : undefined} role="status">
              {check.kind === 'checking' && t('updates.checking')}
              {check.kind === 'upToDate' && t('updates.upToDate')}
              {check.kind === 'failed' && check.why}
            </span>
          </div>
        )}
        {error !== null && <p className="result bad">{error}</p>}
      </main>
    </>
  );
}
