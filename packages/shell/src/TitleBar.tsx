import { useEffect, useState } from 'react';
import { Mark } from './Mark.js';
import { controls } from './window.js';

/** The host list's title bar: the name at the start, minimize, maximize or restore, and close at the end, as ahpw's. */
export function TitleBar() {
  const [maximized, setMaximized] = useState(false);

  useEffect(() => {
    let live = true;
    const sync = async () => {
      const now = await controls.isMaximized();
      if (live) setMaximized(now);
    };
    void sync();
    const stop = controls.onResized(() => void sync());
    return () => {
      live = false;
      void stop.then((unlisten) => unlisten());
    };
  }, []);

  return (
    <header className="titlebar">
      <span className="titlebar__name">
        <Mark size={14} />
        ahp-desk
      </span>
      <div className="titlebar__controls">
        <button type="button" className="control" title="Minimize" aria-label="Minimize" onClick={() => void controls.minimize()}>
          <svg width="10" height="10" viewBox="0 0 10 10" aria-hidden="true">
            <path d="M0 5.5h10" stroke="currentColor" />
          </svg>
        </button>
        <button
          type="button"
          className="control"
          title={maximized ? 'Restore' : 'Maximize'}
          aria-label={maximized ? 'Restore' : 'Maximize'}
          onClick={() => void controls.toggleMaximize()}
        >
          {maximized ? (
            <svg width="10" height="10" viewBox="0 0 10 10" aria-hidden="true">
              <path d="M2.5 2.5V.5h7v7h-2M.5 2.5h7v7h-7z" fill="none" stroke="currentColor" />
            </svg>
          ) : (
            <svg width="10" height="10" viewBox="0 0 10 10" aria-hidden="true">
              <path d="M.5.5h9v9h-9z" fill="none" stroke="currentColor" />
            </svg>
          )}
        </button>
        <button type="button" className="control control--close" title="Close" aria-label="Close" onClick={() => void controls.close()}>
          <svg width="10" height="10" viewBox="0 0 10 10" aria-hidden="true">
            <path d="M.5.5l9 9M9.5.5l-9 9" stroke="currentColor" />
          </svg>
        </button>
      </div>
    </header>
  );
}
