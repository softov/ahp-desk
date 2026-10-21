import { useEffect, useState } from 'react';
import { localStarting, onLocalStarting } from './ipc.js';

/** The local hosts whose ahpd the desk is starting, kept current. */
export function useLocalStarting(): ReadonlySet<string> {
  const [starting, setStarting] = useState<ReadonlySet<string>>(new Set());
  useEffect(() => {
    let live = true;
    const stop = onLocalStarting((ids) => {
      if (live) setStarting(new Set(ids));
    });
    localStarting()
      .then((ids) => {
        if (live) setStarting(new Set(ids));
      })
      .catch((err: unknown) => console.error('local starting failed', err));
    return () => {
      live = false;
      void stop.then((unlisten) => unlisten());
    };
  }, []);
  return starting;
}
