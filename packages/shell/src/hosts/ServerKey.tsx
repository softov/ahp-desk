import { t } from '../i18n.js';
import type { ConnectError, Fingerprint } from './ipc.js';

/** A server key the connection refused: one the desk has not seen, or one that changed. */
export type RefusedKey = Extract<ConnectError, { reason: 'newKey' | 'changedKey' }>;

function KeyLine({ fingerprint }: { fingerprint: Fingerprint }) {
  return (
    <code className="fingerprint">
      {fingerprint.keyType} {fingerprint.sha256}
    </code>
  );
}

/**
 * The server key Check the server stopped on: a new one, with Trust, or a
 * changed one, with both fingerprints and Forget the old key, or the
 * `ssh-keygen -R` that removes it when it came from `~/.ssh/known_hosts`.
 */
export function ServerKey({
  server,
  address,
  port,
  refused,
  busy,
  onTrust,
  onForget,
}: {
  /** The server as the person typed it, `host:port`. */
  server: string;
  address: string;
  port: number;
  refused: RefusedKey;
  busy: boolean;
  onTrust: () => void;
  onForget: () => void;
}) {
  if (refused.reason === 'newKey')
    return (
      <div className="server-key">
        <strong>{t('key.newTitle')}</strong>
        <p>{t('key.newText', { server, type: refused.offered.keyType })}</p>
        <KeyLine fingerprint={refused.offered} />
        <p className="hint">{t('key.newCheck')}</p>
        <div className="found-actions">
          <button type="button" className="primary" onClick={onTrust} disabled={busy}>
            {t('key.trust')}
          </button>
        </div>
      </div>
    );
  return (
    <div className="server-key bad">
      <strong>{t('key.changedTitle')}</strong>
      <p>{t('key.changedText', { server })}</p>
      <KeyLine fingerprint={refused.known} />
      <p>{t('key.offered')}</p>
      <KeyLine fingerprint={refused.offered} />
      {refused.inKnownHosts ? (
        <>
          <p>{t('key.knownHosts')}</p>
          <code>ssh-keygen -R {port === 22 ? address : `'[${address}]:${port}'`}</code>
        </>
      ) : (
        <div className="found-actions">
          <button type="button" onClick={onForget} disabled={busy}>
            {t('key.forget')}
          </button>
        </div>
      )}
    </div>
  );
}
