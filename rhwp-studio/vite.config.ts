import { defineConfig } from 'vite';
import { resolve, extname, join } from 'path';
import { readFileSync, readFile } from 'fs';
import { VitePWA } from 'vite-plugin-pwa';

const pkg = JSON.parse(readFileSync(resolve(__dirname, 'package.json'), 'utf-8'));

export default defineConfig({
  define: {
    __APP_VERSION__: JSON.stringify(pkg.version),
  },
  resolve: {
    alias: {
      '@': resolve(__dirname, 'src'),
      '@wasm': resolve(__dirname, '..', 'pkg'),
    },
  },
  server: {
    host: '127.0.0.1',
    port: 7700,
    fs: {
      // [Task #741 후속] 외부 file path 그림 영역 영역 samples/ dir 영역 영역 fetch 가능 영역.
      allow: [__dirname, resolve(__dirname, '..', 'pkg'), resolve(__dirname, '..', 'samples')],
    },
  },
  plugins: [
    // [Task #741 후속] dev 서버 영역 영역 /samples/* 경로 영역 영역 parent samples/ dir 영역
    // 영역 정적 serve 영역 — wasm-bridge.ts 영역 영역 외부 image fetch 영역 영역 영역.
    {
      name: 'serve-samples-dir',
      configureServer(server) {
        const samplesDir = resolve(__dirname, '..', 'samples');
        server.middlewares.use('/samples', (req, res, next) => {
          if (!req.url) return next();
          // URL decode + sanitize (path traversal 차단)
          const reqPath = decodeURIComponent(req.url.split('?')[0]);
          const relPath = reqPath.replace(/^\/+/, '');
          if (relPath.includes('..')) { res.statusCode = 403; return res.end(); }
          const full = join(samplesDir, relPath);
          if (!full.startsWith(samplesDir)) { res.statusCode = 403; return res.end(); }
          readFile(full, (err: NodeJS.ErrnoException | null, data: Buffer) => {
            if (err) { res.statusCode = 404; return res.end(); }
            const ext = extname(full).toLowerCase();
            const mime: Record<string, string> = {
              '.gif': 'image/gif', '.jpg': 'image/jpeg', '.jpeg': 'image/jpeg',
              '.png': 'image/png', '.bmp': 'image/bmp', '.webp': 'image/webp',
            };
            res.setHeader('Content-Type', mime[ext] ?? 'application/octet-stream');
            // [Task #741 후속] OS 영역 절대 경로 영역 영역 response header 영역 노출 — JS
            // 영역 영역 dialog 영역 영역 한컴 viewer 정합 (D:\\... 영역 영역 영역 의 영역 영역) 영역.
            res.setHeader('X-File-Path', encodeURI(full));
            res.setHeader('Access-Control-Expose-Headers', 'X-File-Path');
            res.end(data);
          });
        });
      },
    },
    VitePWA({
      registerType: 'autoUpdate',
      includeAssets: ['favicon.ico', 'icons/*.png'],
      manifest: {
        name: 'rhwp-studio',
        short_name: 'rhwp',
        description: 'HWP/HWPX 뷰어·에디터 — 알(R), 모두의 한글',
        lang: 'ko',
        theme_color: '#2b6cb0',
        background_color: '#ffffff',
        display: 'standalone',
        start_url: '/rhwp/',
        scope: '/rhwp/',
        icons: [
          { src: 'icons/icon-128.png', sizes: '128x128', type: 'image/png' },
          { src: 'icons/icon-192.png', sizes: '192x192', type: 'image/png' },
          { src: 'icons/icon-256.png', sizes: '256x256', type: 'image/png' },
          { src: 'icons/icon-512.png', sizes: '512x512', type: 'image/png' },
          { src: 'icons/icon-512.png', sizes: '512x512', type: 'image/png', purpose: 'any maskable' },
        ],
      },
      workbox: {
        // WASM (~12 MB) is kept out of precache to avoid blocking SW installation;
        // CacheFirst at runtime still gives offline access after the first load.
        globPatterns: ['**/*.{js,css,html,png,svg,ico,woff,woff2,ttf,otf}'],
        maximumFileSizeToCacheInBytes: 20 * 1024 * 1024,
        runtimeCaching: [
          {
            urlPattern: /\.wasm$/,
            handler: 'CacheFirst',
            options: {
              cacheName: 'wasm-cache',
              expiration: { maxEntries: 5, maxAgeSeconds: 30 * 24 * 60 * 60 },
            },
          },
        ],
      },
      devOptions: {
        enabled: false,
      },
    }),
  ],
});
