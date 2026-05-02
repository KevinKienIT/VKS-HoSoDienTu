import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import * as path from 'path';

export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      '@': path.resolve(__dirname, 'src')
    }
  },
  build: {
    outDir: 'dist',
    emptyOutDir: true,
    rollupOptions: {
      output: {
        manualChunks(id) {
          if (id.includes('react-pdf') || id.includes('pdfjs-dist')) {
            return 'pdf-viewer';
          }

          if (id.includes('@tauri-apps/api') || id.includes('@tauri-apps/plugin-')) {
            return 'tauri-runtime';
          }
        }
      }
    }
  }
});
