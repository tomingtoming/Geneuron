/// <reference types="vitest" />
import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [react()],
  base: '/geneuron/',  // Add base URL for GitHub Pages
  server: {
    port: 3000,
  },
  build: {
    outDir: 'dist',
    target: 'esnext',
    sourcemap: true,
  },
  optimizeDeps: {
    include: ['three', '@tensorflow/tfjs'],
  },
  test: {
    globals: true,
    environment: 'jsdom',
    setupFiles: ['./src/setupTests.ts'],
  }
});