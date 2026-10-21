import { useCallback, useEffect, useRef, useState, type FormEvent, type ReactNode } from 'react';
import { open as pickFile } from '@tauri-apps/plugin-dialog';
import {
  findLocalAhpd,
  addLocalBackends,
  installLocalAhpd,
  localHomeFolder,
  saveHost,
  serveLocalFolder,
  startLocalAhpd,
  type FoundAhpd,
  type InstallEvent,
  ON_DESK_DEFAULTS,
  type OnDesk,
} from './ipc.js';
import { t } from '../i18n.js';
import { BackendsField, hasBackend, startingBackends } from './BackendsField.js';
import { FoundAhpdField } from './FoundAhpd.js';
import { InstallSteps } from './InstallSteps.js';
import type { Saved } from './HostForm.js';
import { hostId } from './url.js';

/** Where the ahpd search stands: looking, done, or failed. */
type Search = { kind: 'looking' } | { kind: 'done'; found: FoundAhpd | null } | { kind: 'failed'; error: string };

/** What the form is doing: asking, or one of the steps of a save. */
type Phase = 'form' | 'installing' | 'adding' | 'starting';

/** An error's first line, and what ahpd wrote under it, such as its log, in a box. */
export function ErrorText({ text }: { text: string }) {
  const [head, ...rest] = text.split('\n');
  return (
    <>
      <p className="result bad">{head}</p>
      {rest.length > 0 && <InstallSteps lines={rest} />}
    </>
  );
}

/** The line naming the ahpd found. */
function foundLine(found: FoundAhpd): string {
  const words = { path: 'local.foundPath', set: 'local.foundSet', installed: 'local.foundInstalled' } as const;
  return t(words[found.origin], { version: found.version });
}

/**
 * Add host, kind "This computer": the ahpd found, or the install of one with
 * the backends picked, then the start. An ahpd too old for the desk is shown,
 * not replaced (decision `an-ahpd-too-old-on-path-is-shown-not-replaced`).
 */
export function LocalForm({
  taken,
  firstRun = false,
  kindField,
  onDone,
  onToList,
}: {
  taken: readonly string[];
  firstRun?: boolean;
  /** Add host's Kind, first among the fields. */
  kindField?: ReactNode;
  /** The host saved and on, or `null` when the form was left. */
  onDone: (saved: Saved) => void;
  /** Saved but not started: the way out is the host list, where the host shows down. */
  onToList: () => void;
}) {
  const [name, setName] = useState(() => t('firstRun.local'));
  const [ahpdPath, setAhpdPath] = useState<string | null>(null);
  const [search, setSearch] = useState<Search>({ kind: 'looking' });
  const [backends, setBackends] = useState<Set<string>>(() => new Set(['@ahpd/agent-claude']));
  const [onDesk, setOnDesk] = useState<OnDesk>(ON_DESK_DEFAULTS);
  // The folder to serve, the home folder until one is picked.
  const [folder, setFolder] = useState<string | null>(null);
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
  useEffect(() => {
    localHomeFolder().then(
      (home) => setFolder((picked) => picked ?? home),
      (err: unknown) => setError(String(err)),
    );
  }, []);

  const change = async () => {
    const picked = await pickFile({ title: t('local.pickTitle'), multiple: false, directory: false });
    if (typeof picked === 'string') setAhpdPath(picked);
  };

  const changeFolder = async () => {
    const picked = await pickFile({ title: t('local.pickFolderTitle'), multiple: false, directory: true, defaultPath: folder ?? undefined });
    if (typeof picked === 'string') setFolder(picked);
  };

  const found = search.kind === 'done' ? search.found : null;
  const install = retryInstall || (search.kind === 'done' && found === null);
  const usable = found?.supported === true || install;
  // What the found ahpd has ready, ticked and fixed; the desk's own install starts with none.
  const has = new Set(install ? [] : (found?.plugins ?? []));
  const added = [...backends].filter((pkg) => !has.has(pkg));
  const ready =
    phase === 'form' && saved === null && name.trim() !== '' && usable && hasBackend(has, backends) && folder !== null;

  // A found ahpd that already has a backend gets nothing ticked for it; one with none starts on Claude.
  const foundPlugins = found?.plugins.join(' ');
  useEffect(() => {
    if (foundPlugins === undefined) return;
    setBackends(startingBackends(foundPlugins.split(' ')));
  }, [foundPlugins]);

  const save = async (event: FormEvent) => {
    event.preventDefault();
    if (!ready) return;
    setError(null);
    const id = hostId(name, taken);
    try {
      // The local install sends lines only; steps are the SSH install's.
      const heard = (event: InstallEvent) => {
        if (event.kind === 'line') setLines((all) => [...all, event.text]);
      };
      if (install) {
        setPhase('installing');
        setLines([]);
        await installLocalAhpd([...backends], heard);
      } else if (added.length > 0) {
        setPhase('adding');
        setLines([]);
        await addLocalBackends(ahpdPath, added, heard);
      }
      setPhase('starting');
      // ahpd writes the folder into its own config before its first start (decision `the-folder-is-written-by-an-ahpd-command`).
      if (folder !== null) await serveLocalFolder(ahpdPath, folder);
      // The address is ahpd's, read when it starts; until then the default port stands in.
      await saveHost(
        { id, name: name.trim(), kind: 'local', url: 'ws://127.0.0.1:9187', ahpdPath: ahpdPath ?? undefined, onDesk },
        null,
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
        <FoundAhpdField
          label={t('local.ahpd')}
          line={
            <>
              {search.kind === 'looking' && <p>{t('local.looking')}</p>}
              {search.kind === 'failed' && <p className="result bad">{search.error}</p>}
              {found !== null && found.supported && <p title={found.path}>{foundLine(found)}</p>}
              {install && <p>{t('local.none')}</p>}
            </>
          }
          tooOld={found !== null && !found.supported ? found.version : null}
          update="npm i -g @ahpd/server"
          onCheckAgain={() => look(ahpdPath)}
          actions={
            <button type="button" onClick={() => void change()} disabled={phase !== 'form'}>
              {t('local.change')}
            </button>
          }
          disabled={phase !== 'form'}
        />
        {usable && (
          <div className="field">
            <span>{t('local.folder')}</span>
            <div className="found found--line">
              <p title={folder ?? undefined}>{folder ?? ''}</p>
              <div className="found-actions">
                <button type="button" onClick={() => void changeFolder()} disabled={phase !== 'form' || folder === null}>
                  {t('local.change')}
                </button>
              </div>
            </div>
          </div>
        )}
        {usable && <BackendsField has={has} backends={backends} onChange={setBackends} disabled={phase !== 'form'} />}
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
        {/* Only an ahpd the desk installs updates itself (plan 03 decision 6), so a found one has no box. */}
        {install && (
          <label className="check">
            <input
              type="checkbox"
              checked={onDesk.autoUpdate}
              onChange={(event) => setOnDesk({ ...onDesk, autoUpdate: event.target.checked })}
              disabled={phase !== 'form'}
            />
            <span>{t('form.autoUpdate')}</span>
          </label>
        )}
      </div>
      {phase === 'installing' && <p className="result">{t('local.installing')}</p>}
      {phase === 'adding' && <p className="result">{t('local.adding')}</p>}
      {phase === 'starting' && <p className="result">{t('local.starting')}</p>}
      {lines.length > 0 && <InstallSteps lines={lines} />}
      {error !== null && <ErrorText text={error} />}
      <footer className="actions" ref={footer}>
        {saved !== null ? (
          <button type="button" className="primary" onClick={onToList}>
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
