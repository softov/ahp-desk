import { useCallback, useEffect, useRef, useState, type FormEvent, type ReactNode } from 'react';
import { open as pickFile } from '@tauri-apps/plugin-dialog';
import {
  findLocalAhpd,
  installLocalAhpd,
  saveHost,
  startLocalAhpd,
  type FoundAhpd,
  type OnDesk,
} from './ipc.js';
import { t } from '../i18n.js';
import { InstallSteps } from './InstallSteps.js';
import type { Saved } from './HostForm.js';
import { hostId } from './url.js';

/** The backends Add host offers (decision 2), in `ahpd configure`'s order, and the bots. */
const BACKENDS = [
  { name: 'Claude', pkg: '@ahpd/agent-claude' },
  { name: 'cofold', pkg: '@ahpd/agent-cofold' },
  { name: 'pi', pkg: '@ahpd/agent-pi' },
] as const;
const BOT = '@ahpd/bot';

/** The oldest ahpd the desk runs, as `local_ahpd::OLDEST_SUPPORTED`. */
const NEEDED = '0.8.0';

/** Where the ahpd search stands: looking, done, or failed. */
type Search = { kind: 'looking' } | { kind: 'done'; found: FoundAhpd | null } | { kind: 'failed'; error: string };

/** What the form is doing: asking, or one of the steps of a save. */
type Phase = 'form' | 'installing' | 'starting';

/** The line naming the ahpd found. */
function foundLine(found: FoundAhpd): string {
  const words = { path: 'local.foundPath', set: 'local.foundSet', installed: 'local.foundInstalled' } as const;
  return t(words[found.origin], { version: found.version });
}

/**
 * Add host, kind "This computer": the ahpd found, or the install of one with
 * the backends picked, then the start. An ahpd too old for the desk is shown,
 * not replaced (decision 9).
 */
export function LocalForm({
  taken,
  firstRun = false,
  kindField,
  onDone,
}: {
  taken: readonly string[];
  firstRun?: boolean;
  /** Add host's Kind, first among the fields. */
  kindField?: ReactNode;
  onDone: (saved: Saved) => void;
}) {
  const [name, setName] = useState(() => t('firstRun.local'));
  const [ahpdPath, setAhpdPath] = useState<string | null>(null);
  const [search, setSearch] = useState<Search>({ kind: 'looking' });
  const [backends, setBackends] = useState<Set<string>>(() => new Set(['@ahpd/agent-claude']));
  const [onDesk, setOnDesk] = useState<OnDesk>({ startOnLaunch: true, stopOnQuit: false });
  const [phase, setPhase] = useState<Phase>('form');
  const [lines, setLines] = useState<string[]>([]);
  const [error, setError] = useState<string | null>(null);
  // Once saved, a failed start leaves the host in the list, down; the way out is the list.
  const [saved, setSaved] = useState<Saved>(null);
  const [retryInstall, setRetryInstall] = useState(false);
  // The window is small: while it installs, and after a failure, the steps and the buttons stay in view.
  const footer = useRef<HTMLElement>(null);
  const showing = lines.length > 0 || error !== null;
  useEffect(() => {
    if (showing) footer.current?.scrollIntoView({ block: 'end' });
  }, [showing, lines.length, error]);

  const look = useCallback((path: string | null) => {
    setSearch({ kind: 'looking' });
    findLocalAhpd(path).then(
      (found) => setSearch({ kind: 'done', found }),
      (err: unknown) => setSearch({ kind: 'failed', error: String(err) }),
    );
  }, []);
  useEffect(() => look(ahpdPath), [look, ahpdPath]);

  const change = async () => {
    const picked = await pickFile({ title: t('local.pickTitle'), multiple: false, directory: false });
    if (typeof picked === 'string') setAhpdPath(picked);
  };

  const toggle = (pkg: string, on: boolean) => {
    const next = new Set(backends);
    if (on) next.add(pkg);
    else next.delete(pkg);
    setBackends(next);
  };

  const found = search.kind === 'done' ? search.found : null;
  const install = retryInstall || (search.kind === 'done' && found === null);
  const usable = found?.supported === true || install;
  const anyBackend = BACKENDS.some((one) => backends.has(one.pkg));
  const ready = phase === 'form' && saved === null && name.trim() !== '' && usable && (!install || anyBackend);

  const save = async (event: FormEvent) => {
    event.preventDefault();
    if (!ready) return;
    setError(null);
    const id = hostId(name, taken);
    try {
      if (install) {
        setPhase('installing');
        setLines([]);
        await installLocalAhpd([...backends], (step) => setLines((all) => [...all, step.text]));
      }
      setPhase('starting');
      // The address is ahpd's, read when it starts; until then the default port stands in.
      await saveHost(
        { id, name: name.trim(), kind: 'local', url: 'ws://127.0.0.1:9187', ahpdPath: ahpdPath ?? undefined, onDesk },
        null,
      );
      setSaved({ id, name: name.trim() });
      await startLocalAhpd(id);
      onDone({ id, name: name.trim() });
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
      setPhase('form');
      // A failed install may leave ahpd with no backend: a retry installs again, backends and all.
      if (install) setRetryInstall(true);
    }
  };

  return (
    <form className="panel" onSubmit={save}>
      <header className="panel-head">
        <h1>{t('form.addTitle')}</h1>
      </header>
      <div className="fields">
        {kindField}
        <label>
          <span>{t('form.name')}</span>
          <input value={name} onChange={(event) => setName(event.target.value)} disabled={phase !== 'form'} />
        </label>
        <div className="field">
          <span>{t('local.ahpd')}</span>
          {/* One line, with Change beside it, except for the too-old note and its command. */}
          <div className={found !== null && !found.supported ? 'found' : 'found found--line'}>
            {search.kind === 'looking' && <p>{t('local.looking')}</p>}
            {search.kind === 'failed' && <p className="result bad">{search.error}</p>}
            {found !== null && found.supported && <p title={found.path}>{foundLine(found)}</p>}
            {found !== null && !found.supported && (
              <>
                <p>{t('local.tooOld', { version: found.version, needed: NEEDED })}</p>
                <code>npm i -g @ahpd/server</code>
              </>
            )}
            {install && <p>{t('local.none')}</p>}
            <div className="found-actions">
              {found !== null && !found.supported && (
                <button type="button" onClick={() => look(ahpdPath)} disabled={phase !== 'form'}>
                  {t('local.checkAgain')}
                </button>
              )}
              <button type="button" onClick={() => void change()} disabled={phase !== 'form'}>
                {t('local.change')}
              </button>
            </div>
          </div>
        </div>
        {install && (
          <div className="field">
            <span>{t('local.backends')}</span>
            <div className="checks">
              {BACKENDS.map((one) => (
                <label className="check" key={one.pkg}>
                  <input
                    type="checkbox"
                    checked={backends.has(one.pkg)}
                    onChange={(event) => toggle(one.pkg, event.target.checked)}
                    disabled={phase !== 'form'}
                  />
                  <span>{one.name}</span>
                  <small>{one.pkg}</small>
                </label>
              ))}
              <label className="check">
                <input
                  type="checkbox"
                  checked={backends.has(BOT)}
                  onChange={(event) => toggle(BOT, event.target.checked)}
                  disabled={phase !== 'form'}
                />
                <span>{t('local.bot')}</span>
                <small>{BOT}</small>
              </label>
            </div>
          </div>
        )}
        <label className="check">
          <input
            type="checkbox"
            checked={onDesk.startOnLaunch}
            onChange={(event) => setOnDesk({ ...onDesk, startOnLaunch: event.target.checked })}
            disabled={phase !== 'form'}
          />
          <span>{t('form.startOnLaunch')}</span>
        </label>
        <label className="check">
          <input
            type="checkbox"
            checked={onDesk.stopOnQuit}
            onChange={(event) => setOnDesk({ ...onDesk, stopOnQuit: event.target.checked })}
            disabled={phase !== 'form'}
          />
          <span>{t('form.stopOnQuit')}</span>
        </label>
      </div>
      {phase === 'installing' && <p className="result">{t('local.installing')}</p>}
      {phase === 'starting' && <p className="result">{t('local.starting')}</p>}
      {lines.length > 0 && <InstallSteps lines={lines} />}
      {error !== null && <p className="result bad">{error}</p>}
      <footer className="actions" ref={footer}>
        {saved !== null ? (
          <button type="button" className="primary" onClick={() => onDone(saved)}>
            {t('local.toList')}
          </button>
        ) : (
          <>
            <button type="button" onClick={() => onDone(null)} disabled={phase !== 'form'}>
              {firstRun ? t('form.back') : t('form.cancel')}
            </button>
            <button type="submit" className="primary" disabled={!ready}>
              {install ? t('local.installAndStart') : t('local.saveAndStart')}
            </button>
          </>
        )}
      </footer>
    </form>
  );
}
