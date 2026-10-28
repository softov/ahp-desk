/** What About asks the desk: its versions, and its window. */

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

/** Opens the About window beside the host list, or brings it to the front. */
export function openAboutWindow(): Promise<void> {
  return invoke('open_about_window');
}
