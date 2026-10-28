import { useState } from 'react';
import { t } from '../i18n.js';
import { installDeskUpdate, localAhpdStatus, type ShellHost } from './ipc.js';

/**
 * The ready desk update and its Restart, on the host list and in the Updates window.
 * Restart stops each local ahpd the desk started, as the installer replaces
 * the Node it runs on: the turns running there are named first, and a second
 * Restart installs. The tray's item opens the Updates window, so it asks the same question.
 */
export function DeskUpdateBar({
  version,
  hosts,
  running,
  onError,
}: {
  version: string;
  hosts: readonly ShellHost[];
  /** The titles of a host's chats with a turn on. */
  running: (id: string) => string[];
  onError: (message: string) => void;
}) {
  const [question, setQuestion] = useState<string | null>(null);
  const run = (work: Promise<void>) =>
    work.catch((err: unknown) => onError(err instanceof Error ? err.message : String(err)));

  const restart = async () => {
    const busy: string[] = [];
    for (const host of hosts) {
      const titles = running(host.id);
      if (host.kind !== 'local' || titles.length === 0) continue;
      if ((await localAhpdStatus(host.id)).startedByDesk) busy.push(`${host.name}: ${titles.join(', ')}`);
    }
    if (busy.length === 0) return installDeskUpdate();
    setQuestion(busy.join('; '));
  };
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
