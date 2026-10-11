import react from '@vitejs/plugin-react';
import { defineConfig } from 'vite';

/** Tauri loads this dev server from `build.devUrl`, so the port is fixed. */
export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  server: {
    host: '127.0.0.1',
    port: 5175,
    strictPort: true,
  },
  build: {
    outDir: 'dist',
    target: 'es2022',
  },
});
