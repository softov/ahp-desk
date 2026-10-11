/** The host commands only the host list may call; a UI reads hosts through `@ahpd/desk-client`. */

import type { DeskHost } from '@ahpd/desk-client';
import { invoke } from '@tauri-apps/api/core';

/** A host as the host list sees it: with the flag for a token kept in the plain file. */
export interface ShellHost extends DeskHost {
  tokenInFile?: boolean;
}

export function listHosts(): Promise<ShellHost[]> {
  return invoke<ShellHost[]>('hosts');
}

/** Saves the host. `token`: null keeps the one it has, `''` deletes it. */
export function saveHost(host: DeskHost, token: string | null): Promise<void> {
  return invoke('save_host', { host, token });
}

export function removeHost(id: string): Promise<void> {
  return invoke('remove_host', { id });
}
