import { useEffect, useRef, useState } from 'react';
import { t } from '../i18n.js';
import { useOutsideClose } from '../useOutsideClose.js';
import {
  installDeskUpdate,
  openHost,
  openSshTunnel,
  removeHost,
  sshSignIn,
  uninstallSshAhpd,
  type ShellHost,
  type SshHost,
  type TunnelState,
  type UninstallStep,
} from './ipc.js';
import { InstallSteps, withStep, type StepState } from './InstallSteps.js';
import type { HostStatus, HostStatuses, UpdateWay } from './status.js';
import { shortUrl } from './url.js';
import { useTunnels } from './useTunnels.js';
import { newer } from './versions.js';
import { ErrorText } from './LocalForm.js';

const UNINSTALL_WORDS = {
  connect: 'uninstall.connect',
  stop: 'uninstall.stop',
  unit: 'uninstall.unit',
  files: 'uninstall.files',
} as const;

/**
 * Remove's question on a host reached over SSH. For an ahpd the desk
 * installed: remove and uninstall it on the server, with the steps, or remove
 * from the desk only; a failure keeps the host and says why. For one the desk
 * did not install: the server's ahpd stays, and the question says so.
 */
function RemoveSshRow({ host, ssh, onDone }: { host: ShellHost; ssh: SshHost; onDone: () => void }) {
  const [phase, setPhase] = useState<'ask' | 'running' | 'failed'>('ask');
  const [steps, setSteps] = useState<StepState<UninstallStep>[]>([]);
  const [error, setError] = useState<string | null>(null);
  const server = ssh.address;
  const remove = () =>
    removeHost(host.id).catch((err: unknown) => {
      setError(err instanceof Error ? err.message : String(err));
      setPhase('failed');
    });
  const uninstall = async () => {
    setPhase('running');
    setSteps([]);
    setError(null);
    try {
      await uninstallSshAhpd(host.id, (event) => {
        if (event.kind === 'step') setSteps((all) => withStep(all, { id: event.id, status: event.status }));
      });
      await removeHost(host.id);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
      setPhase('failed');
    }
  };
  return (
    <li className="host confirm removing">
      <span className="question">{t('hosts.removeQuestion', { name: host.name })}</span>
      <p className="remove-note">
        {phase === 'running'
          ? t('hosts.uninstalling', { server })
          : ssh.byDesk
            ? t('hosts.uninstallNote', { server })
            : t('hosts.foundStays', { server })}
      </p>
      <InstallSteps lines={[]} steps={steps} wordOf={(id) => t(UNINSTALL_WORDS[id])} />
      {error !== null && <p className="result bad">{error}</p>}
      {phase !== 'running' && (
        <div className="remove-actions">
          <button onClick={onDone}>{t('form.cancel')}</button>
          {ssh.byDesk ? (
            <>
              <button onClick={() => void remove()}>{t('hosts.removeDeskOnly')}</button>
              {phase === 'ask' && (
                <button className="danger" onClick={() => void uninstall()}>
                  {t('hosts.removeAndUninstall')}
                </button>
              )}
            </>
          ) : (
            <button className="danger" onClick={() => void remove()}>
              {t('hosts.remove')}
            </button>
          )}
        </div>
      )}
    </li>
  );
}

/** A row's Sign in for a tunnel that needs what the person types, as after a restart. */
function SignInRow({ host, tunnel, onDone }: { host: ShellHost; tunnel: TunnelState; onDone: () => void }) {
  const [secret, setSecret] = useState('');
  const [error, setError] = useState<string | null>(null);
  const label = t(tunnel.needs === 'passphrase' ? 'tunnel.passphrase' : 'tunnel.password');
  return (
    <li className="host signing">
      <form
        className="sign-in"
        onSubmit={(event) => {
          event.preventDefault();
          sshSignIn(host.id, secret)
            .then(onDone)
            .catch((err: unknown) => setError(err instanceof Error ? err.message : String(err)));
        }}
      >
        <span className="question">{host.name}</span>
        <input
          type="password"
          aria-label={label}
          placeholder={label}
          value={secret}
          autoFocus
          onChange={(event) => setSecret(event.target.value)}
        />
        <button type="button" onClick={onDone}>
          {t('form.cancel')}
        </button>
        <button className="primary" type="submit" disabled={secret === ''}>
          {t('tunnel.signIn')}
        </button>
        {error !== null && <span className="result bad">{error}</span>}
      </form>
    </li>
  );
}

function statusText(status: HostStatus): string {
  switch (status.kind) {
    case 'connected':
      return status.server ?? t('status.connected');
    case 'connecting':
      return t('status.connecting');
    case 'down':
      return t('status.down');
  }
}

/** A down tunnel's words: its server key refused, changed or new, or just down. */
function tunnelWord(tunnel: TunnelState): 'tunnel.down' | 'tunnel.keyChanged' | 'tunnel.keyNew' {
  if (tunnel.key === null) return 'tunnel.down';
  return tunnel.key.reason === 'changedKey' ? 'tunnel.keyChanged' : 'tunnel.keyNew';
}

/** The ⋯ menu of one row; it closes on a click anywhere else. */
function RowMenu({ onEdit, onRemove }: { onEdit: () => void; onRemove: () => void }) {
  const [open, setOpen] = useState(false);
  const ref = useRef<HTMLDivElement>(null);
  useOutsideClose(ref, open, () => setOpen(false));
  return (
    <div className="menu" ref={ref}>
      <button className="more" aria-label={t('hosts.more')} onClick={() => setOpen(!open)}>
        ⋯
      </button>
      {open && (
        <div className="menu-items" role="menu">
          <button
            role="menuitem"
            onClick={() => {
              setOpen(false);
              onEdit();
            }}
          >
            {t('hosts.edit')}
          </button>
          <button
            role="menuitem"
            onClick={() => {
              setOpen(false);
              onRemove();
            }}
          >
            {t('hosts.remove')}
          </button>
        </div>
      )}
    </div>
  );
}

/** The host list: each host's state, Open or Retry, and its menu. */
export function HostList({
  hosts,
  statuses,
  latestAhpd,
  deskUpdate,
  onAdd,
  onEdit,
}: {
  hosts: readonly ShellHost[];
  statuses: HostStatuses;
  /** npm's latest `@ahpd/server`, or `null` until the desk has asked. */
  latestAhpd: string | null;
  /** The version of a newer desk ready to install, or `null`. */
  deskUpdate: string | null;
  onAdd: () => void;
  onEdit: (host: ShellHost) => void;
}) {
  const [removing, setRemoving] = useState<string | null>(null);
  const [signingIn, setSigningIn] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const tunnels = useTunnels();
  // How each host's ahpd can be updated, asked again when the list changes.
  const [ways, setWays] = useState<Record<string, UpdateWay | null>>({});
  useEffect(() => {
    let live = true;
    void Promise.all(hosts.map(async (host) => [host.id, await statuses.updateWay(host)] as const)).then((all) => {
      if (live) setWays(Object.fromEntries(all));
    });
    return () => {
      live = false;
    };
  }, [hosts, statuses]);
  // The host whose update waits on a click over running sessions, and the ones updating.
  const [confirmingUpdate, setConfirmingUpdate] = useState<string | null>(null);
  const [updating, setUpdating] = useState<ReadonlySet<string>>(new Set());
  const update = (host: ShellHost, version: string, force: boolean) => {
    setConfirmingUpdate(null);
    setUpdating((all) => new Set(all).add(host.id));
    void run(statuses.updateHost(host, version, force)).finally(() =>
      setUpdating((all) => {
        const next = new Set(all);
        next.delete(host.id);
        return next;
      }),
    );
  };

  const run = (work: Promise<void>) =>
    work.catch((err: unknown) => setError(err instanceof Error ? err.message : String(err)));

  return (
    <section className="panel">
      <header className="panel-head">
        <h1>{t('hosts.title')}</h1>
        <button className="primary" onClick={onAdd}>
          {t('hosts.add')}
        </button>
      </header>
      {deskUpdate !== null && (
        <div className="desk-update">
          <span>{t('update.ready', { version: deskUpdate })}</span>
          <button className="primary" onClick={() => void run(installDeskUpdate())}>
            {t('update.restart')}
          </button>
        </div>
      )}
      {error !== null && (
        <div onClick={() => setError(null)}>
          <ErrorText text={error} />
        </div>
      )}
      <ul className="hosts">
        {hosts.map((host) => {
          const status = statuses.status(host.id);
          // A tunnelled host is down while its tunnel is: the row says why.
          const tunnel = host.kind === 'ssh' ? tunnels[host.id] : undefined;
          const tunnelDown = tunnel !== undefined && !tunnel.on;
          if (signingIn === host.id && tunnel !== undefined && tunnel.needs !== null) {
            return <SignInRow key={host.id} host={host} tunnel={tunnel} onDone={() => setSigningIn(null)} />;
          }
          if (removing === host.id && host.ssh !== undefined) {
            return <RemoveSshRow key={host.id} host={host} ssh={host.ssh} onDone={() => setRemoving(null)} />;
          }
          // The local host updates by its own rule (plan 03), never from this notice.
          const behind =
            host.kind !== 'local' &&
            status.kind === 'connected' &&
            status.ahpd !== null &&
            latestAhpd !== null &&
            newer(latestAhpd, status.ahpd);
          if (confirmingUpdate === host.id && latestAhpd !== null) {
            return (
              <li key={host.id} className="host confirm">
                <span className="question">
                  {t('hosts.updateQuestion', { name: host.name, sessions: statuses.running(host.id).join(', ') })}
                </span>
                <button onClick={() => setConfirmingUpdate(null)}>{t('form.cancel')}</button>
                <button className="primary" onClick={() => update(host, latestAhpd, true)}>
                  {t('hosts.update')}
                </button>
              </li>
            );
          }
          if (removing === host.id) {
            return (
              <li key={host.id} className="host confirm">
                <span className="question">{t('hosts.removeQuestion', { name: host.name })}</span>
                <button onClick={() => setRemoving(null)}>{t('form.cancel')}</button>
                <button
                  className="danger"
                  onClick={() => {
                    setRemoving(null);
                    void run(removeHost(host.id));
                  }}
                >
                  {t('hosts.remove')}
                </button>
              </li>
            );
          }
          return (
            <li key={host.id} className="host">
              <span
                className={`dot ${status.kind}`}
                role="img"
                aria-label={t(`status.${status.kind}`)}
                title={t(`status.${status.kind}`)}
              />
              <span className="name">
                <span className="name-text" title={host.name}>
                  {host.name}
                </span>
                {host.tokenInFile === true && (
                  <span className="warn" title={t('hosts.noKeyring')}>
                    {t('hosts.tokenInFile')}
                  </span>
                )}
                {host.sshPasswordInFile === true && (
                  <span className="warn" title={t('hosts.noKeyring')}>
                    {t('hosts.passwordInFile')}
                  </span>
                )}
              </span>
              <span
                className="state"
                title={tunnelDown ? (tunnel.error ?? undefined) : status.kind === 'down' ? (status.error ?? undefined) : undefined}
              >
                {tunnelDown ? t(tunnelWord(tunnel)) : statusText(status)}
              </span>
              <span className="address" title={host.url}>
                {shortUrl(host.url)}
              </span>
              <span className="action">
                {status.kind === 'connected' && !tunnelDown && (
                  <button onClick={() => void run(openHost(host.id))}>{t('hosts.open')}</button>
                )}
                {tunnelDown && tunnel.needs !== null && (
                  <button onClick={() => setSigningIn(host.id)}>{t('tunnel.signIn')}</button>
                )}
                {/* Retry asks the server again with nothing new; the key is answered in Edit host. */}
                {tunnelDown && tunnel.key !== null && (
                  <button onClick={() => onEdit(host)}>{t('tunnel.checkKey')}</button>
                )}
                {tunnelDown && tunnel.needs === null && tunnel.key === null && (
                  <button onClick={() => void run(openSshTunnel(host.id))}>{t('hosts.retry')}</button>
                )}
                {status.kind === 'down' && !tunnelDown && (
                  <button onClick={() => void run(statuses.retry(host.id))}>{t('hosts.retry')}</button>
                )}
              </span>
              <RowMenu onEdit={() => onEdit(host)} onRemove={() => setRemoving(host.id)} />
              {/* Under the whole row, from the name on, so the words and Update both fit. */}
              {behind && (
                <span className="row-note">
                  <span
                    className="note-text"
                    title={updating.has(host.id) ? undefined : t('hosts.ahpdAvailable', { version: latestAhpd })}
                  >
                    {updating.has(host.id) ? t('hosts.updating') : t('hosts.ahpdAvailable', { version: latestAhpd })}
                  </span>
                  {!updating.has(host.id) && (ways[host.id] ?? null) !== null && (
                    <button
                      className="link"
                      onClick={() =>
                        statuses.running(host.id).length === 0
                          ? update(host, latestAhpd, false)
                          : setConfirmingUpdate(host.id)
                      }
                    >
                      {t('hosts.update')}
                    </button>
                  )}
                </span>
              )}
            </li>
          );
        })}
      </ul>
    </section>
  );
}
