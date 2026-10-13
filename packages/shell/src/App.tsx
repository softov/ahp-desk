import { useEffect, useReducer, useState } from 'react';
import { FirstRun } from './FirstRun.js';
import { HostForm } from './hosts/HostForm.js';
import { HostList } from './hosts/HostList.js';
import { listHosts, onAddAsked, onHostsChanged, type ShellHost } from './hosts/ipc.js';
import { createHostStatuses, type HostStatuses } from './hosts/status.js';
import { Footer } from './Footer.js';
import { useLanguage } from './i18n.js';
import { TitleBar } from './TitleBar.js';

type Screen = { kind: 'list' } | { kind: 'form'; host: ShellHost | null };

export function App() {
  // Every word below is read in this language; a pick redraws the whole window.
  useLanguage();
  // `null` until the list is read, so a desk with hosts never shows the first run.
  const [hosts, setHosts] = useState<ShellHost[] | null>(null);
  const [screen, setScreen] = useState<Screen>({ kind: 'list' });
  // With no host the desk is on its first run, which ends when the new host
  // opens, not when it is saved; removing the last host starts it again.
  const [firstRun, setFirstRun] = useState(false);
  // Each host's state lives in the client; a change only asks for a redraw.
  const [, redraw] = useReducer((n: number) => n + 1, 0);
  const [statuses] = useState<HostStatuses>(() => createHostStatuses(redraw));

  useEffect(() => {
    let live = true;
    const take = (list: ShellHost[]) => {
      setHosts(list);
      if (list.length === 0) setFirstRun(true);
    };
    void listHosts().then((list) => live && take(list));
    const stops = [onHostsChanged(take), onAddAsked(() => setScreen({ kind: 'form', host: null }))];
    return () => {
      live = false;
      for (const stop of stops) void stop.then((unlisten) => unlisten());
    };
  }, []);

  useEffect(() => {
    if (hosts === null) return;
    statuses.sync(hosts).catch((err: unknown) => console.error('host status sync failed', err));
  }, [hosts, statuses]);

  return (
    <>
      <TitleBar />
      <main className="shell">
        {hosts === null ? null : firstRun ? (
          <FirstRun
            onFinished={() => {
              setFirstRun(false);
              setScreen({ kind: 'list' });
            }}
          />
        ) : screen.kind === 'list' ? (
          <HostList
            hosts={hosts}
            statuses={statuses}
            onAdd={() => setScreen({ kind: 'form', host: null })}
            onEdit={(host) => setScreen({ kind: 'form', host })}
          />
        ) : (
          <HostForm
            host={screen.host}
            taken={hosts.map((host) => host.id)}
            onDone={(saved) => {
              setScreen({ kind: 'list' });
              // A new token is read on the next connect.
              if (saved !== null && screen.host !== null) void statuses.retry(saved.id);
            }}
          />
        )}
      </main>
      <Footer />
    </>
  );
}
