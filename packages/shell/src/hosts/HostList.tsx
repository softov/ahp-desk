import { openHost } from '@ahpd/desk-client';
import { useEffect, useRef, useState } from 'react';
import { removeHost, type ShellHost } from './ipc.js';
import type { HostStatus, HostStatuses } from './status.js';
import { shortUrl } from './url.js';

function statusText(status: HostStatus): string {
  switch (status.kind) {
    case 'connected':
      return status.server ?? 'connected';
    case 'connecting':
      return 'connecting';
    case 'down':
      return 'down';
  }
}

/** The ⋯ menu of one row; it closes on a click anywhere else. */
function RowMenu({ onEdit, onRemove }: { onEdit: () => void; onRemove: () => void }) {
  const [open, setOpen] = useState(false);
  const ref = useRef<HTMLDivElement>(null);
  useEffect(() => {
    if (!open) return;
    const close = (event: MouseEvent) => {
      if (!ref.current?.contains(event.target as Node)) setOpen(false);
    };
    document.addEventListener('mousedown', close);
    return () => document.removeEventListener('mousedown', close);
  }, [open]);
  return (
    <div className="menu" ref={ref}>
      <button className="more" aria-label="More" onClick={() => setOpen(!open)}>
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
            Edit
          </button>
          <button
            role="menuitem"
            onClick={() => {
              setOpen(false);
              onRemove();
            }}
          >
            Remove
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
  const [error, setError] = useState<string | null>(null);

  const run = (work: Promise<void>) =>
    work.catch((err: unknown) => setError(err instanceof Error ? err.message : String(err)));

  return (
    <section className="panel">
      <header className="panel-head">
        <h1>Hosts</h1>
        <button className="primary" onClick={onAdd}>
          + Add host
        </button>
      </header>
      {error !== null && (
        <p className="result bad" onClick={() => setError(null)}>
          {error}
        </p>
      )}
      {hosts.length === 0 ? (
        <p className="empty">No hosts yet. Add an ahpd, or any AHP server, by its address.</p>
      ) : (
        <ul className="hosts">
          {hosts.map((host) => {
            const status = statuses.status(host.id);
            if (removing === host.id) {
              return (
                <li key={host.id} className="host confirm">
                  <span className="question">Remove {host.name}?</span>
                  <button onClick={() => setRemoving(null)}>Cancel</button>
                  <button
                    className="danger"
                    onClick={() => {
                      setRemoving(null);
                      void run(removeHost(host.id));
                    }}
                  >
                    Remove
                  </button>
                </li>
              );
            }
            return (
              <li key={host.id} className="host">
                <span
                  className={`dot ${status.kind}`}
                  role="img"
                  aria-label={status.kind}
                  title={status.kind}
                />
                <span className="name">
                  {host.name}
                  {host.tokenInFile === true && (
                    <span className="warn" title="No OS keyring was available.">
                      The token is in a plain file.
                    </span>
                  )}
                </span>
                <span
                  className="state"
                  title={status.kind === 'down' ? (status.error ?? undefined) : undefined}
                >
                  {statusText(status)}
                </span>
                <span className="address" title={host.url}>
                  {shortUrl(host.url)}
                </span>
                <span className="action">
                  {status.kind === 'connected' && (
                    <button onClick={() => void run(openHost(host.id))}>Open</button>
                  )}
                  {status.kind === 'down' && (
                    <button onClick={() => void run(statuses.retry(host.id))}>Retry</button>
                  )}
                </span>
                <RowMenu onEdit={() => onEdit(host)} onRemove={() => setRemoving(host.id)} />
              </li>
            );
          })}
        </ul>
      )}
    </section>
  );
}
