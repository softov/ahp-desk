import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
import { App } from './App.js';
import './app.css';
import { startFrame } from './window.js';

// The window has no OS frame; the host list draws it.
startFrame();

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <App />
  </StrictMode>,
);
