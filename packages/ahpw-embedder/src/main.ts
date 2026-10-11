/**
 * The script the desk injects into every app window before the page's own:
 * it hands ahpw its embedder. Built into one file, `dist/ahpw-embedder.js`.
 */

import type { Embedder } from '@ahpd/web/embed';
import { createEmbedder } from './embedder.js';

(globalThis as { ahpwEmbedder?: Embedder }).ahpwEmbedder = createEmbedder();
