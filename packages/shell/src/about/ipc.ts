/** What the About screen asks the desk: its versions, and its update check on demand. */

import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';

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

/** Calls `handler` when the tray's About item asks for the About screen. */
export function onAboutAsked(handler: () => void): Promise<UnlistenFn> {
  return listen('host-list-about', () => handler());
}
