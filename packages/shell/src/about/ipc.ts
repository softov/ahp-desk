/** What About asks the desk: its versions, its update check on demand, and its window. */

import { invoke } from '@tauri-apps/api/core';

/** The desk's version, the bundled ahpw's and the bundled Node's; `null` when the build carries none. */
export interface About {
  desk: string;
  ahpw: string | null;
  node: string | null;
}

export function about(): Promise<About> {
  return invoke<About>('about');
}

/** Asks for a newer desk now and downloads it; answers the ready version, or `null` when up to date. */
export function checkDeskUpdate(): Promise<string | null> {
  return invoke<string | null>('check_desk_update');
}

/** Opens the About window beside the host list, or brings it to the front. */
export function openAboutWindow(): Promise<void> {
  return invoke('open_about_window');
}
