import { useEffect, useReducer, useState } from 'react';
import { FirstRun } from './FirstRun.js';
import { HostForm } from './hosts/HostForm.js';
import { HostList } from './hosts/HostList.js';
import { KindField, type AddKind } from './hosts/KindField.js';
import { LocalForm } from './hosts/LocalForm.js';
import { SshForm } from './hosts/SshForm.js';
import {
  listHosts,
  onAddAsked,
  onEditAsked,
  onHostsChanged,
  onTunnelsChanged,
  onUpdatesChanged,
  setTrayWords,
  sshTunnelStates,
  updates,
  type ShellHost,
} from './hosts/ipc.js';
import { createHostStatuses, type HostStatuses } from './hosts/status.js';
import { Footer } from './Footer.js';
import { t, useLanguage } from './i18n.js';
import { TitleBar } from './TitleBar.js';
import { onSettingsAsked } from './settings/ipc.js';
import { SettingsScreen } from './settings/SettingsScreen.js';

type Screen = { kind: 'list' } | { kind: 'form'; host: ShellHost | null } | { kind: 'settings' };

export function App() {
  // Every word below is read in this language; a pick redraws the whole window.
  const lang = useLanguage();
  // `null` until the list is read, so a desk with hosts never shows the first run.
  const [hosts, setHosts] = useState<ShellHost[] | null>(null);
  const [screen, setScreen] = useState<Screen>({ kind: 'list' });
  // Add host opens on This computer while the list has none; a desk has one local host.
  const [addKind, setAddKind] = useState<AddKind>('local');
  const localTaken = hosts?.some((host) => host.kind === 'local') ?? false;
  const add = () => {
    setAddKind(localTaken ? 'url' : 'local');
    setScreen({ kind: 'form', host: null });
  };
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
    const stops = [
      onHostsChanged(take),
      onAddAsked(() => setScreen({ kind: 'form', host: null })),
      // Read again rather than from state: this listener outlives the list it was set up with.
      onEditAsked((id) => {
        void listHosts().then((list) => {
          const host = list.find((one) => one.id === id);
          if (live && host !== undefined) setScreen({ kind: 'form', host });
        });
      }),
      onSettingsAsked(() => setScreen({ kind: 'settings' })),
    ];
    return () => {
      live = false;
      for (const stop of stops) void stop.then((unlisten) => unlisten());
    };
  }, []);

  // The tray menu has no words of its own; it takes the host list's, and again on each pick.
  useEffect(() => {
    setTrayWords({ hosts: t('tray.hosts'), settings: t('tray.settings'), quit: t('tray.quit') }).catch((err: unknown) =>
      console.error('tray words failed', err),
    );
  }, [lang]);

  useEffect(() => {
    if (hosts === null) return;
    statuses.sync(hosts).catch((err: unknown) => console.error('host status sync failed', err));
  }, [hosts, statuses]);

  // npm's latest @ahpd/server, which each host row compares with its ahpd.
  const [latestAhpd, setLatestAhpd] = useState<string | null>(null);
  useEffect(() => {
    const take = (view: { ahpd: { latest: string } | null }) => setLatestAhpd(view.ahpd?.latest ?? null);
    updates().then(take, (err: unknown) => console.error('updates failed', err));
    const stop = onUpdatesChanged(take);
    return () => void stop.then((unlisten) => unlisten());
  }, []);

  // A tunnel that comes up is connected through at once, not at the client's next retry.
  useEffect(() => {
    const on = new Set<string>();
    const stop = (async () => {
      for (const [id, state] of Object.entries(await sshTunnelStates())) if (state.on) on.add(id);
      return onTunnelsChanged((all) => {
        for (const [id, state] of Object.entries(all)) {
          if (state.on && !on.has(id))
            statuses.retry(id).catch((err: unknown) => console.error('host retry failed', err));
          if (state.on) on.add(id);
          else on.delete(id);
        }
      });
    })();
    return () => void stop.then((unlisten) => unlisten());
  }, [statuses]);

  return (
    <>
      {/* Settings opens from the host list; the first run has no way back to it. */}
      <TitleBar onSettings={firstRun ? undefined : () => setScreen({ kind: 'settings' })} />
      <main className="shell">
        {hosts === null ? null : firstRun ? (
          <FirstRun
            onFinished={() => {
              setFirstRun(false);
              setScreen({ kind: 'list' });
            }}
          />
        ) : screen.kind === 'settings' ? (
          <SettingsScreen onDone={() => setScreen({ kind: 'list' })} />
        ) : screen.kind === 'list' ? (
          <HostList
            hosts={hosts}
            statuses={statuses}
            latestAhpd={latestAhpd}
            onAdd={add}
            onEdit={(host) => setScreen({ kind: 'form', host })}
          />
        ) : screen.host === null && addKind === 'local' && !localTaken ? (
          <LocalForm
            taken={hosts.map((host) => host.id)}
            kindField={<KindField kind={addKind} localTaken={localTaken} onKind={setAddKind} />}
            onDone={() => setScreen({ kind: 'list' })}
            onToList={() => setScreen({ kind: 'list' })}
          />
        ) : screen.host === null && addKind === 'ssh' ? (
          <SshForm
            taken={hosts.map((host) => host.id)}
            kindField={<KindField kind={addKind} localTaken={localTaken} onKind={setAddKind} />}
            onDone={() => setScreen({ kind: 'list' })}
            onToList={() => setScreen({ kind: 'list' })}
          />
        ) : (
          <HostForm
            host={screen.host}
            kindField={
              screen.host === null ? (
                <KindField kind="url" localTaken={localTaken} onKind={setAddKind} />
              ) : undefined
            }
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
