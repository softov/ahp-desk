import { useEffect, useState, type FormEvent, type ReactNode } from 'react';
import { hostToken, saveHost, sshKeyringWorks, sshSignIn, type OnDesk, type ShellHost } from './ipc.js';
import { t } from '../i18n.js';
import { explain, testHost } from './status.js';
import { hostId, normalizeUrl } from './url.js';

type TestResult = { ok: true; text: string } | { ok: false; text: string } | null;

/** The host `HostForm` saved, or `null` when it was left. */
export type Saved = { id: string; name: string } | null;

/**
 * Add host, or Edit host when `host` is given. A URL host has an address and
 * a token; a local host, whose address and token come from its ahpd, has the
 * start and quit settings instead. A host reached over SSH shows its server,
 * which only a new Add host changes, and takes its password. In Edit, an
 * empty token or password keeps the one saved. In the first run, the way out
 * is Back and saving opens the host.
 */
export function HostForm({
  host,
  taken,
  firstRun = false,
  kindField,
  onDone,
}: {
  host: ShellHost | null;
  taken: readonly string[];
  firstRun?: boolean;
  /** Add host's Kind, first among the fields. */
  kindField?: ReactNode;
  onDone: (saved: Saved) => void;
}) {
  const [name, setName] = useState(host?.name ?? '');
  const [address, setAddress] = useState(host?.url ?? '');
  const [token, setToken] = useState('');
  const [testing, setTesting] = useState(false);
  const [result, setResult] = useState<TestResult>(null);
  const [error, setError] = useState<string | null>(null);
  const local = host?.kind === 'local';
  const tunnel = host?.kind === 'ssh';
  const ssh = host?.ssh;
  const [password, setPassword] = useState('');
  const [remember, setRemember] = useState(true);
  const [keyring, setKeyring] = useState<boolean | null>(null);
  useEffect(() => {
    if (ssh !== undefined) sshKeyringWorks().then(setKeyring, (err: unknown) => setError(String(err)));
  }, [ssh]);
  const [onDesk, setOnDesk] = useState<OnDesk>(host?.onDesk ?? { startOnLaunch: true, stopOnQuit: false });

  // A tunnel's address is its kept local port, never typed.
  const url = local || tunnel ? (host?.url ?? '') : normalizeUrl(address);
  const ready = name.trim() !== '' && url !== '';

  /** The token to use: the one typed, or in Edit the one saved. */
  const tokenToUse = async () =>
    token !== '' ? token : host === null ? null : await hostToken(host.id);

  const test = async () => {
    setTesting(true);
    setResult(null);
    try {
      setResult({ ok: true, text: await testHost(url, await tokenToUse()) });
    } catch (err) {
      setResult({ ok: false, text: explain(err) });
    } finally {
      setTesting(false);
    }
  };

  const save = async (event: FormEvent) => {
    event.preventDefault();
    if (!ready) return;
    const id = host?.id ?? hostId(name, taken);
    try {
      if (local) {
        await saveHost({ id, name: name.trim(), kind: 'local', url, ahpdPath: host?.ahpdPath, onDesk }, null, null);
      } else {
        // A password not remembered replaces the one kept, held by the desk until it quits.
        const sshPassword = password === '' ? null : remember ? password : '';
        await saveHost(
          { id, name: name.trim(), kind: tunnel ? 'ssh' : 'url', url, ssh },
          token === '' ? (host === null ? '' : null) : token,
          ssh === undefined ? null : sshPassword,
        );
        if (tunnel && password !== '' && !remember) await sshSignIn(id, password);
      }
      onDone({ id, name: name.trim() });
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    }
  };

  return (
    <form className="panel" onSubmit={save}>
      <header className="panel-head">
        <h1>{host === null ? t('form.addTitle') : t('form.editTitle', { name: host.name })}</h1>
      </header>
      <div className="fields">
        {kindField}
        <label>
          <span>{t('form.name')}</span>
          <input
            value={name}
            onChange={(event) => setName(event.target.value)}
            placeholder={t('form.namePlaceholder')}
            autoFocus
          />
        </label>
        {local ? (
          <>
            <label className="check">
              <input
                type="checkbox"
                checked={onDesk.startOnLaunch}
                onChange={(event) => setOnDesk({ ...onDesk, startOnLaunch: event.target.checked })}
              />
              <span>{t('form.startOnLaunch')}</span>
            </label>
            <label className="check">
              <input
                type="checkbox"
                checked={onDesk.stopOnQuit}
                onChange={(event) => setOnDesk({ ...onDesk, stopOnQuit: event.target.checked })}
              />
              <span>{t('form.stopOnQuit')}</span>
            </label>
            <p className="hint">{t('form.stopOnQuitHint')}</p>
          </>
        ) : (
          <>
            {ssh !== undefined && (
              <div className="field">
                <span>{t('ssh.host')}</span>
                <div className="field-text">
                  <p>
                    {t('ssh.details', {
                      user: ssh.user,
                      address: ssh.address,
                      port: String(ssh.port),
                      remotePort: String(ssh.remotePort),
                    })}
                  </p>
                  <p>
                    {tunnel ? t('ssh.detailsTunnel', { localPort: String(ssh.localPort ?? '') }) : t('ssh.detailsPort')}
                  </p>
                  {ssh.folder !== undefined && <p title={ssh.folder}>{ssh.folder}</p>}
                </div>
              </div>
            )}
            {!tunnel && (
              <label>
                <span>{t('form.address')}</span>
                <input
                  value={address}
                  onChange={(event) => {
                    setAddress(event.target.value);
                    setResult(null);
                  }}
                  placeholder="wss://host.example:9187"
                  spellCheck={false}
                />
              </label>
            )}
            <label>
              <span>{t('form.token')}</span>
              <input
                type="password"
                value={token}
                onChange={(event) => {
                  setToken(event.target.value);
                  setResult(null);
                }}
                placeholder={host === null ? '' : t('form.tokenUnchanged')}
                autoComplete="off"
              />
            </label>
            <p className="hint">{t('form.tokenHint')}</p>
            {ssh !== undefined && (
              <>
                <label>
                  <span>{t('ssh.password')}</span>
                  <input
                    type="password"
                    value={password}
                    onChange={(event) => setPassword(event.target.value)}
                    placeholder={t('ssh.passwordKept')}
                    autoComplete="off"
                  />
                </label>
                <label className="check">
                  <input
                    type="checkbox"
                    checked={remember}
                    onChange={(event) => setRemember(event.target.checked)}
                    disabled={keyring === null}
                  />
                  <span>{keyring === false ? t('ssh.rememberPlain') : t('ssh.remember')}</span>
                </label>
              </>
            )}
          </>
        )}
      </div>
      {result !== null && (
        <p className={result.ok ? 'result ok' : 'result bad'}>
          {result.ok ? t('form.connected', { server: result.text }) : result.text}
        </p>
      )}
      {error !== null && <p className="result bad">{error}</p>}
      <footer className="actions">
        <button type="button" onClick={() => onDone(null)}>
          {firstRun ? t('form.back') : t('form.cancel')}
        </button>
        {!local && (
          <button type="button" onClick={test} disabled={url === '' || testing}>
            {testing ? t('form.testing') : t('form.test')}
          </button>
        )}
        <button type="submit" className="primary" disabled={!ready}>
          {firstRun ? t('form.saveAndOpen') : t('form.save')}
        </button>
      </footer>
    </form>
  );
}
