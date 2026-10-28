/** What the Updates window asks the desk: its update check on demand, and the window itself. */

import { invoke } from '@tauri-apps/api/core';

/** Asks for a newer desk now and downloads it; answers the ready version, or `null` when up to date. */
export function checkDeskUpdate(): Promise<string | null> {
  return invoke<string | null>('check_desk_update');
}

/** Opens the Updates window beside the host list, or brings it to the front. */
export function openUpdatesWindow(): Promise<void> {
  return invoke('open_updates_window');
}
