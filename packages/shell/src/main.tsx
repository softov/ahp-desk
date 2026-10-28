import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
import { AboutWindow } from './about/AboutWindow.js';
import { App } from './App.js';
import './app.css';
import { UpdatesWindow } from './updates/UpdatesWindow.js';
import { startFrame } from './window.js';

// One bundle, a page per window: a side window loads it with its view as the
// hash, and only the host list, `App`, starts the desk's work.
const side = location.hash === '#about' ? <AboutWindow /> : location.hash === '#updates' ? <UpdatesWindow /> : null;

// The window has no OS frame; the page draws it.
startFrame({ maximizes: side === null });

createRoot(document.getElementById('root')!).render(
  <StrictMode>{side ?? <App />}</StrictMode>,
);
