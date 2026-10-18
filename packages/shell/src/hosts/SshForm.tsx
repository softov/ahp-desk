import { useEffect, useRef, useState, type FormEvent, type ReactNode } from 'react';
import { open as pickFile } from '@tauri-apps/plugin-dialog';
import {
  cancelSshInstall,
  confirmSshKey,
  forgetSshKey,
  installSshAhpd,
  openSshTunnel,
  pickSshLocalPort,
  probeSshServer,
  saveHost,
  sshError,
  sshKeyringWorks,
  type InstallEvent,
  type Probe,
  type ProbeTarget,
  type RefusedKey,
} from './ipc.js';
import { t } from '../i18n.js';
import { BackendsField, hasBackend, startingBackends } from './BackendsField.js';
import { FoundAhpdField } from './FoundAhpd.js';
import { InstallSteps, withStep, type StepState } from './InstallSteps.js';
import { ErrorText } from './LocalForm.js';
import { ServerKey } from './ServerKey.js';
import type { Saved } from './HostForm.js';
import { ahpdPort, hostId, sshTarget } from './url.js';

/** What the form is doing: asking, checking the server, stopped on its key, or one of the steps of a save. */
type Phase = 'form' | 'connecting' | 'key' | 'probed' | 'installing' | 'saving';

/** The port in an ahpd's address, when it listens beyond the server's own 127.0.0.1. */
function openPort(url: string | null): number | null {
  if (url === null) return null;
  try {
    const parsed = new URL(url);
    if (['127.0.0.1', 'localhost', '[::1]'].includes(parsed.hostname)) return null;
    return parsed.port === '' ? null : Number(parsed.port);
  } catch {
    return null;
  }
}

/** The ahpd's own port, from its address. */
function portOf(url: string | null): number | null {
  if (url === null) return null;
  try {
    const port = new URL(url).port;
    return port === '' ? null : Number(port);
  } catch {
    return null;
  }
}

/**
 * Add host, kind "A server over SSH": signs in, trusts the server's key,
 * shows what the probe found, then uses the ahpd found or installs one, saves
 * the host and connects (plan 10 task 07).
 */
export function SshForm({
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
  /** The host saved and connecting, or `null` when the form was left. */
  onDone: (saved: Saved) => void;
  /** Saved but its tunnel did not open: the way out is the host list, where the host shows down. */
  onToList: () => void;
}) {
  // The name follows the host until one is typed.
  const [typedName, setTypedName] = useState<string | null>(null);
  const [address, setAddress] = useState('');
  const [port, setPort] = useState('22');
  const [user, setUser] = useState('');
  const [keyFile, setKeyFile] = useState<string | null>(null);
  const [password, setPassword] = useState('');
  const [remember, setRemember] = useState(false);
  const [passphrase, setPassphrase] = useState('');
  const [needsPassphrase, setNeedsPassphrase] = useState(false);
  const [keyring, setKeyring] = useState<boolean | null>(null);
  const [folder, setFolder] = useState('');
  const [portText, setPortText] = useState('9187');
  const [backends, setBackends] = useState<Set<string>>(() => new Set(['@ahpd/agent-claude']));
  const [reach, setReach] = useState<'tunnel' | 'port'>('tunnel');
  const [phase, setPhase] = useState<Phase>('form');
  const [refused, setRefused] = useState<RefusedKey | null>(null);
  const [probe, setProbe] = useState<Probe | null>(null);
  // Fixed at the first check: what was typed for the server is held under this id until the desk quits.
  const [id, setId] = useState<string | null>(null);
  const [steps, setSteps] = useState<StepState[]>([]);
  const [lines, setLines] = useState<string[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [saved, setSaved] = useState<Saved>(null);
  // The window is small: while it installs, and after a failure, the steps and the buttons stay in view.
  const footer = useRef<HTMLElement>(null);
  const showing = steps.length > 0 || error !== null || refused !== null;
  useEffect(() => {
    if (showing) footer.current?.scrollIntoView({ block: 'end' });
  }, [showing, steps.length, lines.length, error]);

  useEffect(() => {
    sshKeyringWorks().then(setKeyring, (err: unknown) => setError(String(err)));
  }, []);

  const name = typedName ?? address.trim();
  const busy = phase === 'connecting' || phase === 'installing' || phase === 'saving';
  const found = probe?.found ?? null;
  const has = new Set(found?.plugins ?? []);
  const foundPort = portOf(found?.url ?? null);
  // A found ahpd is reached as it listens: an open port only when it already listens beyond 127.0.0.1.
  const portOffered = found === null || openPort(found.url) !== null;
  const remotePort = found === null ? ahpdPort(portText) : foundPort;
  const ready =
    phase === 'probed' &&
    saved === null &&
    name !== '' &&
    probe !== null &&
    probe.refusal === null &&
    (found === null || found.supported) &&
    hasBackend(has, backends) &&
    (found !== null || (remotePort !== null && folder.trim() !== ''));

  /** Any change to the server asks for a new check. */
  const server = (set: (value: string) => void) => (value: string) => {
    set(value);
    setProbe(null);
    setRefused(null);
    if (!busy) setPhase('form');
  };

  const target = (): ProbeTarget | null => {
    const checked = sshTarget(address, port, user);
    if (!checked.ok) {
      setError(
        t(checked.field === 'host' ? 'ssh.wrongHost' : checked.field === 'port' ? 'ssh.wrongPort' : 'ssh.wrongUser'),
      );
      return null;
    }
    const hostIdNow = id ?? hostId(name === '' ? checked.host : name, taken);
    if (id === null) setId(hostIdNow);
    return {
      id: hostIdNow,
      address: checked.host,
      port: checked.port,
      user: checked.user,
      keyFile: keyFile ?? undefined,
    };
  };

  const check = async () => {
    setError(null);
    setRefused(null);
    const to = target();
    if (to === null) return;
    setPhase('connecting');
    try {
      const probed = await probeSshServer(to, passphrase === '' ? null : passphrase, password === '' ? null : password);
      setProbe(probed);
      setFolder((typed) => (typed === '' ? probed.home : typed));
      if (probed.found !== null) {
        setBackends(startingBackends(probed.found.plugins));
        if (openPort(probed.found.url) === null) setReach('tunnel');
      }
      setPhase('probed');
    } catch (err) {
      setPhase('form');
      const why = sshError(err);
      if (why === null) setError(err instanceof Error ? err.message : String(err));
      else if (why.kind === 'connect' && why.reason !== 'failed') {
        setRefused(why);
        setPhase('key');
      } else if (why.kind === 'connect') setError(t('ssh.failed', { why: why.why }));
      else if (why.kind === 'passphrase' || (why.kind === 'needsPerson' && why.what === 'passphrase')) {
        setNeedsPassphrase(true);
        setError(t('ssh.needsPassphrase'));
      } else if (why.kind === 'needsPerson') setError(t('ssh.refused', { tried: why.what }));
      else setError(t('ssh.refused', { tried: why.tried.join(', ') }));
    }
  };

  const trust = async () => {
    if (refused === null) return;
    const to = target();
    if (to === null) return;
    try {
      await confirmSshKey(to.address, to.port, refused.offered.sha256);
      await check();
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    }
  };

  const forget = async () => {
    const to = target();
    if (to === null) return;
    try {
      await forgetSshKey(to.address, to.port);
      await check();
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    }
  };

  const changeKey = async () => {
    const picked = await pickFile({ title: t('ssh.pickKey'), multiple: false, directory: false });
    if (typeof picked === 'string') server((value) => setKeyFile(value))(picked);
  };

  const save = async (event: FormEvent) => {
    event.preventDefault();
    if (phase === 'form' || phase === 'key') {
      await check();
      return;
    }
    const to = target();
    if (!ready || to === null || probe === null) return;
    setError(null);
    setSteps([]);
    setLines([]);
    setPhase('installing');
    try {
      const heard = (event: InstallEvent) => {
        if (event.kind === 'line') setLines((all) => [...all, event.text]);
        else setSteps((all) => withStep(all, { id: event.id, status: event.status }));
      };
      const installed = await installSshAhpd(
        {
          target: to,
          backends: [...backends].filter((pkg) => !has.has(pkg)),
          folder: found === null ? folder.trim() : undefined,
          ahpdPort: found === null && remotePort !== null ? remotePort : undefined,
          reach,
        },
        heard,
      );
      setPhase('saving');
      const localPort = reach === 'tunnel' ? await pickSshLocalPort() : undefined;
      const host = to.address.includes(':') ? `[${to.address}]` : to.address;
      await saveHost(
        {
          id: to.id,
          name,
          kind: reach === 'tunnel' ? 'ssh' : 'url',
          url: `ws://${reach === 'tunnel' ? '127.0.0.1' : host}:${localPort ?? installed.ahpdPort}`,
          ssh: {
            address: to.address,
            port: to.port,
            user: to.user,
            signIn: probe.signIn ?? undefined,
            keyFile: to.keyFile,
            reach,
            remotePort: installed.ahpdPort,
            folder: found === null ? folder.trim() : undefined,
            byDesk: installed.byDesk,
            localPort,
          },
        },
        installed.token ?? '',
        remember && password !== '' ? password : null,
      );
      setSaved({ id: to.id, name });
      if (reach === 'tunnel') await openSshTunnel(to.id);
      onDone({ id: to.id, name });
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
      setPhase('probed');
    }
  };

  const serverWords = `${address.trim()}:${port.trim()}`;

  return (
    <form className="panel" onSubmit={save}>
      <header className="panel-head">
        <h1>{t('form.addTitle')}</h1>
      </header>
      <div className="fields">
        {kindField}
        <label>
          <span>{t('form.name')}</span>
          <input value={name} onChange={(event) => setTypedName(event.target.value)} disabled={busy} />
        </label>
        <div className="field">
          <span>{t('ssh.host')}</span>
          <div className="ssh-server">
            <input
              aria-label={t('ssh.host')}
              value={address}
              onChange={(event) => server(setAddress)(event.target.value)}
              placeholder="server.example"
              spellCheck={false}
              disabled={busy}
              autoFocus
            />
            <label>
              <span>{t('ssh.port')}</span>
              <input
                className="ssh-port"
                value={port}
                onChange={(event) => server(setPort)(event.target.value)}
                inputMode="numeric"
                disabled={busy}
              />
            </label>
            <label>
              <span>{t('ssh.user')}</span>
              <input
                className="ssh-user"
                value={user}
                onChange={(event) => server(setUser)(event.target.value)}
                spellCheck={false}
                disabled={busy}
              />
            </label>
          </div>
        </div>
        <div className="field">
          <span>{t('ssh.keyFile')}</span>
          <div className="found found--line">
            <p title={keyFile ?? undefined} className={keyFile === null ? 'muted' : undefined}>
              {keyFile ?? t('ssh.keyNone')}
            </p>
            <div className="found-actions">
              <button type="button" onClick={() => void changeKey()} disabled={busy}>
                {t('local.change')}
              </button>
              {keyFile !== null && (
                <button type="button" onClick={() => server(() => setKeyFile(null))('')} disabled={busy}>
                  {t('ssh.clear')}
                </button>
              )}
            </div>
          </div>
        </div>
        {needsPassphrase && (
          <>
            <label>
              <span>{t('ssh.passphrase')}</span>
              <input
                type="password"
                value={passphrase}
                onChange={(event) => setPassphrase(event.target.value)}
                autoComplete="off"
                disabled={busy}
              />
            </label>
            <p className="hint">{t('ssh.heldHint')}</p>
          </>
        )}
        <label>
          <span>{t('ssh.password')}</span>
          <input
            type="password"
            value={password}
            onChange={(event) => setPassword(event.target.value)}
            autoComplete="off"
            disabled={busy}
          />
        </label>
        <p className="hint">{password !== '' && !remember ? t('ssh.heldHint') : t('ssh.passwordHint')}</p>
        <label className="check">
          <input
            type="checkbox"
            checked={remember}
            onChange={(event) => setRemember(event.target.checked)}
            disabled={busy || keyring === null}
          />
          <span>{keyring === false ? t('ssh.rememberPlain') : t('ssh.remember')}</span>
        </label>
        {refused !== null && (
          <ServerKey
            server={serverWords}
            address={address.trim()}
            port={Number(port)}
            refused={refused}
            busy={busy}
            onTrust={() => void trust()}
            onForget={() => void forget()}
          />
        )}
        {probe !== null && (
          <>
            <FoundAhpdField
              label={t('ssh.ahpd')}
              line={
                found !== null ? (
                  <div className="found">
                    <p title={found.path}>{t('ssh.found', { version: found.version, path: found.path })}</p>
                    {!found.running && !found.byDesk && <p className="hint-inline">{t('ssh.foundOff')}</p>}
                  </div>
                ) : (
                  <div className="found">
                    <p>{t('ssh.none', { os: probe.pretty ?? probe.os, arch: probe.arch })}</p>
                    {probe.node.kind === 'own' && <p>{t('ssh.nodeOwn', { version: probe.node.version })}</p>}
                    {probe.node.kind === 'download' && <p>{t('ssh.nodeDownload')}</p>}
                    {probe.ahpdConfig !== null && (
                      <p className="hint-inline">{t('ssh.ahpdConfig', { path: probe.ahpdConfig })}</p>
                    )}
                    {!probe.linger && probe.refusal === null && (
                      <>
                        <p className="hint-inline">{t('ssh.linger', { user: probe.user })}</p>
                        <code>sudo loginctl enable-linger {probe.user}</code>
                      </>
                    )}
                    {probe.refusal !== null && <p className="result bad">{probe.refusal}</p>}
                  </div>
                )
              }
              tooOld={found !== null && !found.supported ? found.version : null}
              update={
                found?.byDesk ? 'npm install --prefix ~/.local/share/ahpd @ahpd/server@latest' : 'npm i -g @ahpd/server'
              }
              onCheckAgain={() => void check()}
              disabled={busy}
            />
            {(found === null || found.supported) && probe.refusal === null && (
              <>
                {found === null ? (
                  <label>
                    <span>{t('ssh.folder')}</span>
                    <input
                      value={folder}
                      onChange={(event) => setFolder(event.target.value)}
                      spellCheck={false}
                      disabled={busy}
                    />
                  </label>
                ) : (
                  <div className="field">
                    <span>{t('ssh.folder')}</span>
                    <div className="field-text">
                      <p>{t('ssh.folderFound')}</p>
                    </div>
                  </div>
                )}
                {found === null ? (
                  <label>
                    <span>{t('ssh.ahpdPort')}</span>
                    <input
                      className="ssh-port"
                      value={portText}
                      onChange={(event) => setPortText(event.target.value)}
                      inputMode="numeric"
                      disabled={busy}
                    />
                  </label>
                ) : (
                  <div className="field">
                    <span>{t('ssh.ahpdPort')}</span>
                    <div className="field-text">
                      <p>{foundPort?.toString() ?? t('ssh.ahpdPortFound')}</p>
                    </div>
                  </div>
                )}
                {found === null && remotePort === null && <p className="hint bad">{t('ssh.wrongAhpdPort')}</p>}
                <BackendsField has={has} backends={backends} onChange={setBackends} disabled={busy} />
                <div className="field">
                  <span id="ssh-reach">{t('ssh.reach')}</span>
                  <div className="reach" role="radiogroup" aria-labelledby="ssh-reach">
                    <label className="check">
                      <input
                        type="radio"
                        name="reach"
                        checked={reach === 'tunnel'}
                        onChange={() => setReach('tunnel')}
                        disabled={busy}
                      />
                      <span>
                        {t('ssh.reachTunnel')} <small>{t('ssh.reachTunnelNote')}</small>
                      </span>
                    </label>
                    {portOffered && (
                      <label className="check">
                        <input
                          type="radio"
                          name="reach"
                          checked={reach === 'port'}
                          onChange={() => setReach('port')}
                          disabled={busy}
                        />
                        <span>
                          {t('ssh.reachPort')} <small>{t('ssh.reachPortNote')}</small>
                        </span>
                      </label>
                    )}
                  </div>
                </div>
              </>
            )}
          </>
        )}
      </div>
      {phase === 'connecting' && <p className="result">{t('ssh.checking', { server: serverWords })}</p>}
      {phase === 'installing' && <p className="result">{t('ssh.installing')}</p>}
      <InstallSteps lines={lines} steps={steps} />
      {error !== null && <ErrorText text={error} />}
      <footer className="actions" ref={footer}>
        {saved !== null ? (
          <button type="button" className="primary" onClick={onToList}>
            {t('local.toList')}
          </button>
        ) : (
          <>
            {phase === 'installing' ? (
              <button type="button" onClick={() => void cancelSshInstall()}>
                {t('form.cancel')}
              </button>
            ) : (
              <button type="button" onClick={() => onDone(null)} disabled={busy}>
                {firstRun ? t('form.back') : t('form.cancel')}
              </button>
            )}
            {probe === null ? (
              <button type="submit" className="primary" disabled={busy || phase === 'key'}>
                {t('ssh.check')}
              </button>
            ) : (
              <button type="submit" className="primary" disabled={!ready}>
                {found === null ? t('ssh.install') : t('ssh.useFound')}
              </button>
            )}
          </>
        )}
      </footer>
    </form>
  );
}
