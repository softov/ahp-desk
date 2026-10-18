import { useEffect, useState } from 'react';
import { onTunnelsChanged, sshTunnelStates, type TunnelState } from './ipc.js';

/** Every `ssh` host's tunnel, by host id, kept as the desk sends changes. */
export function useTunnels(): Record<string, TunnelState> {
  const [tunnels, setTunnels] = useState<Record<string, TunnelState>>({});
  useEffect(() => {
    let live = true;
    const stop = onTunnelsChanged((all) => {
      if (live) setTunnels(all);
    });
    sshTunnelStates()
      .then((all) => {
        if (live) setTunnels(all);
      })
      .catch((err: unknown) => console.error('tunnel states failed', err));
    return () => {
      live = false;
      void stop.then((unlisten) => unlisten());
    };
  }, []);
  return tunnels;
}
