import { defineConfig } from 'vite';

/** One self-contained script, which Rust reads in with `include_str!` and injects into each app window. */
export default defineConfig({
  build: {
    outDir: 'dist',
    target: 'es2022',
    lib: {
      entry: 'src/main.ts',
      formats: ['iife'],
      name: 'ahpwEmbedder',
      fileName: () => 'ahpw-embedder.js',
    },
  },
});
