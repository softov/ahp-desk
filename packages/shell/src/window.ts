/**
 * The host list window's frame, which the OS does not draw: the rounded root,
 * squared while maximized, and the title bar as the place a drag moves the
 * window and a double click maximizes it, as ahpw's `embed/frame.ts`.
 */

import { getCurrentWindow } from '@tauri-apps/api/window';

/** What in the title bar takes its own clicks instead of moving the window. */
const INTERACTIVE = 'button, input, select, textarea, a, [role="menu"], [role="menuitem"], [role="menuitemradio"]';

export const controls = {
  minimize: (): Promise<void> => getCurrentWindow().minimize(),
  toggleMaximize: (): Promise<void> => getCurrentWindow().toggleMaximize(),
  close: (): Promise<void> => getCurrentWindow().close(),
  isMaximized: (): Promise<boolean> => getCurrentWindow().isMaximized(),
  /** Calls `handler` on each resize, a maximize and a restore among them; resolves to the stop. */
  onResized: (handler: () => void): Promise<() => void> => getCurrentWindow().onResized(() => handler()),
};

/** Starts the frame: called once, before the app renders. */
export function startFrame(): void {
  const html = document.documentElement;
  const sync = async () => {
    html.classList.toggle('is-maximized', await controls.isMaximized());
  };
  void sync();
  void controls.onResized(() => void sync());

  // As Tauri's own drag region: the second press of a double click maximizes, any other drags.
  document.addEventListener('mousedown', (event) => {
    if (event.button !== 0 || !(event.target instanceof Element)) return;
    if (event.target.closest('.titlebar') === null || event.target.closest(INTERACTIVE) !== null) return;
    event.preventDefault();
    void (event.detail === 2 ? controls.toggleMaximize() : getCurrentWindow().startDragging());
  });

  // WebKitGTK keeps the hover of a window that hid under the pointer, so the
  // close button came back from the tray red, with its tooltip; until the
  // pointer moves in again, nothing counts as hovered and no title shows.
  // On a show it sends a move at the old position, which is not the pointer's.
  let last = '';
  let stale: string | null = null;
  const away = () => {
    if (stale !== null) return;
    stale = last;
    html.classList.add('pointer-away');
    for (const el of document.querySelectorAll<HTMLElement>('[title]')) {
      el.dataset.awayTitle = el.title;
      el.removeAttribute('title');
    }
  };
  document.addEventListener('pointermove', (event) => {
    last = `${Math.round(event.clientX)},${Math.round(event.clientY)}`;
    if (stale === null || last === stale) return;
    stale = null;
    html.classList.remove('pointer-away');
    for (const el of document.querySelectorAll<HTMLElement>('[data-away-title]')) {
      el.title = el.dataset.awayTitle ?? '';
      delete el.dataset.awayTitle;
    }
  });
  window.addEventListener('blur', away);
  document.addEventListener('visibilitychange', () => {
    if (document.visibilityState === 'hidden') away();
  });
}
