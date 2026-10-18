import { useRef, useState } from 'react';
import { t } from '../i18n.js';
import { useOutsideClose } from '../useOutsideClose.js';
import { openHost, openSshTunnel, removeHost, sshSignIn, type ShellHost, type TunnelState } from './ipc.js';
import type { HostStatus, HostStatuses } from './status.js';
import { shortUrl } from './url.js';
import { useTunnels } from './useTunnels.js';

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
  onAdd,
  onEdit,
}: {
  hosts: readonly ShellHost[];
  statuses: HostStatuses;
  onAdd: () => void;
  onEdit: (host: ShellHost) => void;
}) {
  const [removing, setRemoving] = useState<string | null>(null);
  const [signingIn, setSigningIn] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const tunnels = useTunnels();

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
      {error !== null && (
        <p className="result bad" onClick={() => setError(null)}>
          {error}
        </p>
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
                {host.name}
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
                {tunnelDown ? t('tunnel.down') : statusText(status)}
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
                {tunnelDown && tunnel.needs === null && (
                  <button onClick={() => void run(openSshTunnel(host.id))}>{t('hosts.retry')}</button>
                )}
                {status.kind === 'down' && !tunnelDown && (
                  <button onClick={() => void run(statuses.retry(host.id))}>{t('hosts.retry')}</button>
                )}
              </span>
              <RowMenu onEdit={() => onEdit(host)} onRemove={() => setRemoving(host.id)} />
            </li>
          );
        })}
      </ul>
    </section>
  );
}
