import { useEffect, useRef, useState } from 'react';
import { t } from '../i18n.js';
import { installDeskUpdate, localAhpdStatus, onDeskUpdateAsked, type ShellHost } from './ipc.js';
import type { HostStatuses } from './status.js';

/**
 * The ready desk update and its Restart, on the host list and on About.
 * Restart stops each local ahpd the desk started, as the installer replaces
 * the Node it runs on: the turns running there are named first, and a second
 * Restart installs. The tray's item comes here too, so it asks the same question.
 */
export function DeskUpdateBar({
  version,
  hosts,
  statuses,
  onError,
}: {
  version: string;
  hosts: readonly ShellHost[];
  statuses: HostStatuses;
  onError: (message: string) => void;
}) {
  const [question, setQuestion] = useState<string | null>(null);
  const run = (work: Promise<void>) =>
    work.catch((err: unknown) => onError(err instanceof Error ? err.message : String(err)));

  const restart = async () => {
    const busy: string[] = [];
    for (const host of hosts) {
      const running = statuses.running(host.id);
      if (host.kind !== 'local' || running.length === 0) continue;
      if ((await localAhpdStatus(host.id)).startedByDesk) busy.push(`${host.name}: ${running.join(', ')}`);
    }
    if (busy.length === 0) return installDeskUpdate();
    setQuestion(busy.join('; '));
  };
  const asked = useRef(() => run(restart()));
  asked.current = () => run(restart());
  useEffect(() => {
    const stop = onDeskUpdateAsked(() => void asked.current());
    return () => void stop.then((unlisten) => unlisten());
  }, []);

  return (
    <div className="desk-update">
      <span>
        {question === null ? t('update.ready', { version }) : t('update.restartQuestion', { sessions: question })}
      </span>
      <span className="desk-update-buttons">
        {question !== null && <button onClick={() => setQuestion(null)}>{t('form.cancel')}</button>}
        <button className="primary" onClick={() => void run(question === null ? restart() : installDeskUpdate())}>
          {t('update.restart')}
        </button>
      </span>
    </div>
  );
}
