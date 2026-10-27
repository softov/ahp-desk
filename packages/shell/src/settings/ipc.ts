/** The desk's own settings, kept in `settings.json` beside the host list. */

import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';

/** What closing the host list does when no app window is open. */
export type CloseBehavior = 'minimize-to-tray' | 'quit';

/** How hosts open: a window each, or one window with every host. */
export type Windows = 'per-host' | 'every-host';

export interface Settings {
  closeBehavior: CloseBehavior;
  windows: Windows;
}

export function getSettings(): Promise<Settings> {
  return invoke<Settings>('settings');
}

/** Saves the settings, and answers them as saved. */
export function saveSettings(settings: Settings): Promise<Settings> {
  return invoke<Settings>('save_settings', { settings });
}

/** Calls `handler` when the tray's Settings item asks for the Settings screen. */
export function onSettingsAsked(handler: () => void): Promise<UnlistenFn> {
  return listen('host-list-settings', () => handler());
}

/** "Launch at startup": the OS's login entry, and whether the first run applied its box. */
export interface StartWithOs {
  on: boolean;
  asked: boolean;
}

export function startWithOs(): Promise<StartWithOs> {
  return invoke<StartWithOs>('start_with_os');
}

/** Writes or removes the login entry. */
export function setStartWithOs(on: boolean): Promise<StartWithOs> {
  return invoke<StartWithOs>('set_start_with_os', { on });
}
