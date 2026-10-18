/**
 * The script the desk injects into every app window before the page's own:
 * it hands ahpw its embedder. Built into one file, `dist/ahpw-embedder.js`.
 */

import type { Embedder } from '@ahpd/web/embed';
import { createEmbedder } from './embedder.js';

/** Drops `focus=` from the window's address, once the host it names is shown. */
function forgetFocus(): void {
  const url = new URL(window.location.href);
  url.searchParams.delete('focus');
  window.history.replaceState(window.history.state, '', url);
}

(globalThis as { ahpwEmbedder?: Embedder }).ahpwEmbedder = createEmbedder(window.location, forgetFocus);
