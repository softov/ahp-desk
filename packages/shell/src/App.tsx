import { useEffect, useReducer, useState } from 'react';
import { HostForm } from './hosts/HostForm.js';
import { HostList } from './hosts/HostList.js';
import { listHosts, onAddAsked, onHostsChanged, type ShellHost } from './hosts/ipc.js';
import { createHostStatuses, type HostStatuses } from './hosts/status.js';
import { Footer } from './Footer.js';
import { TitleBar } from './TitleBar.js';

type Screen = { kind: 'list' } | { kind: 'form'; host: ShellHost | null };

export function App() {
  const [hosts, setHosts] = useState<ShellHost[]>([]);
  const [screen, setScreen] = useState<Screen>({ kind: 'list' });
  // Each host's state lives in the client; a change only asks for a redraw.
  const [, redraw] = useReducer((n: number) => n + 1, 0);
  const [statuses] = useState<HostStatuses>(() => createHostStatuses(redraw));

  useEffect(() => {
    let live = true;
    void listHosts().then((list) => live && setHosts(list));
    const stops = [onHostsChanged((list) => setHosts(list)), onAddAsked(() => setScreen({ kind: 'form', host: null }))];
    return () => {
      live = false;
      for (const stop of stops) void stop.then((unlisten) => unlisten());
    };
  }, []);

  useEffect(() => {
    statuses.sync(hosts).catch((err: unknown) => console.error('host status sync failed', err));
  }, [hosts, statuses]);

  return (
    <>
      <TitleBar />
      <main className="shell">
        {screen.kind === 'list' ? (
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
            onDone={(savedId) => {
              setScreen({ kind: 'list' });
              // A new token is read on the next connect.
              if (savedId !== null && screen.host !== null) void statuses.retry(savedId);
            }}
          />
        )}
      </main>
      <Footer />
    </>
  );
}
