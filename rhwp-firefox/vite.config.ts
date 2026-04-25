import { defineConfig } from 'vite';
import { resolve } from 'path';
import { readFileSync } from 'fs';

const studioPkg = JSON.parse(
  readFileSync(resolve(__dirname, '..', 'rhwp-studio', 'package.json'), 'utf-8'),
);

export default defineConfig({
  root: resolve(__dirname, '..', 'rhwp-studio'),
  publicDir: false,
  define: {
    __APP_VERSION__: JSON.stringify(studioPkg.version),
  },
  resolve: {
    alias: {
      '@': resolve(__dirname, '..', 'rhwp-studio', 'src'),
      '@wasm': resolve(__dirname, '..', 'pkg'),
    },
  },
  build: {
    outDir: resolve(__dirname, 'dist'),
    emptyOutDir: true,
    rollupOptions: {
      input: {
        viewer: resolve(__dirname, '..', 'rhwp-studio', 'index.html'),
      },
    },
    assetsInlineLimit: 0,
  },
  server: {
    host: '0.0.0.0',
    port: 7702,
    fs: {
      allow: ['..'],
    },
  },
});
