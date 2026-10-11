import { hostToken } from '@ahpd/desk-client';
import { useState, type FormEvent } from 'react';
import { saveHost, type ShellHost } from './ipc.js';
import { explain, testHost } from './status.js';
import { hostId, normalizeUrl } from './url.js';

type TestResult = { ok: true; text: string } | { ok: false; text: string } | null;

/**
 * Add host, or Edit host when `host` is given, for the kind URL.
 * In Edit, an empty token keeps the one saved.
 */
export function HostForm({
  host,
  taken,
  onDone,
}: {
  host: ShellHost | null;
  taken: readonly string[];
  onDone: (savedId: string | null) => void;
}) {
  const [name, setName] = useState(host?.name ?? '');
  const [address, setAddress] = useState(host?.url ?? '');
  const [token, setToken] = useState('');
  const [testing, setTesting] = useState(false);
  const [result, setResult] = useState<TestResult>(null);
  const [error, setError] = useState<string | null>(null);

  const url = normalizeUrl(address);
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
      await saveHost(
        { id, name: name.trim(), kind: 'url', url },
        token === '' ? (host === null ? '' : null) : token,
      );
      onDone(id);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    }
  };

  return (
    <form className="panel" onSubmit={save}>
      <header className="panel-head">
        <h1>{host === null ? 'Add host' : `Edit ${host.name}`}</h1>
      </header>
      <div className="fields">
        <label>
          <span>Name</span>
          <input
            value={name}
            onChange={(event) => setName(event.target.value)}
            placeholder="Office"
            autoFocus
          />
        </label>
        <label>
          <span>Address</span>
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
        <label>
          <span>Token</span>
          <input
            type="password"
            value={token}
            onChange={(event) => {
              setToken(event.target.value);
              setResult(null);
            }}
            placeholder={host === null ? '' : 'unchanged'}
            autoComplete="off"
          />
        </label>
        <p className="hint">The token is kept in the OS keyring.</p>
      </div>
      {result !== null && (
        <p className={result.ok ? 'result ok' : 'result bad'}>
          {result.ok ? `Connected: ${result.text}` : result.text}
        </p>
      )}
      {error !== null && <p className="result bad">{error}</p>}
      <footer className="actions">
        <button type="button" onClick={() => onDone(null)}>
          Cancel
        </button>
        <button type="button" onClick={test} disabled={url === '' || testing}>
          {testing ? 'Testing…' : 'Test'}
        </button>
        <button type="submit" className="primary" disabled={!ready}>
          Save
        </button>
      </footer>
    </form>
  );
}
