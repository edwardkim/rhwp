import { defineConfig } from 'vite';
import { resolve, extname, join } from 'path';
import { readFileSync, readFile } from 'fs';
import { VitePWA } from 'vite-plugin-pwa';

const configDir = import.meta.dirname;
const pkg = JSON.parse(readFileSync(resolve(configDir, 'package.json'), 'utf-8'));
const subsecondWasmDir = resolve(
  configDir,
  '..',
  'target',
  'rhwp-subsecond-vite',
);
const useSubsecondWasm = process.env.RHWP_SUBSECOND === '1';
// Tauri 데스크톱 빌드에서는 서비스워커(PWA)가 불필요하고 캐시 충돌 위험이 있어 제외한다.
const isDesktopBuild = process.env.RHWP_DESKTOP === '1';
/**
 * hwpctrl 플러그인 포함 여부.
 *
 * `RHWP_WITHOUT_HWPCTRL=1` 로 빌드하면 studio 는 `npm/hwpctrl-ocx` 에 **빌드 시점에도** 묶이지
 * 않는다 — `main.ts` 의 동적 import 가 상수 분기 안에 있어 통째로 tree-shake 되고, 산출물에
 * `studio-plugin` 청크 자체가 남지 않는다. studio 만 떼어 배포할 때 쓴다.
 */
const withHwpctrl = process.env.RHWP_WITHOUT_HWPCTRL !== '1';
/**
 * AI 도우미 포함 여부. `RHWP_WITHOUT_AI=1` 이면 `main.ts` 의 동적 import 분기가 tree-shake 되어
 * AI 청크와 제공자 SDK 가 산출물에 남지 않는다.
 */
const withAi = process.env.RHWP_WITHOUT_AI !== '1';

export default defineConfig({
  define: {
    __APP_VERSION__: JSON.stringify(pkg.version),
    // 셀프 호스팅 빌드에서 외부(CDN) 웹폰트 로드를 빌드 시점에 끈다.
    // 확장 storage 설정(disableExternalWebFonts)이 있으면 그 값이 우선한다.
    __RHWP_DISABLE_EXTERNAL_WEBFONTS__: JSON.stringify(
      process.env.RHWP_DISABLE_EXTERNAL_WEBFONTS === '1',
    ),
    __RHWP_HWPCTRL__: JSON.stringify(withHwpctrl),
    __RHWP_AI__: JSON.stringify(withAi),
  },
  resolve: {
    alias: {
      '@': resolve(configDir, 'src'),
      '@wasm/rhwp.js': useSubsecondWasm
        ? resolve(subsecondWasmDir, 'rhwp-subsecond.js')
        : resolve(configDir, '..', 'pkg', 'rhwp.js'),
      '@wasm': resolve(configDir, '..', 'pkg'),
      // 플러그인 패키지 — 동적 import 로만 들어오므로 별도 청크가 된다.
      // 올리지 않으면 코드도 로드되지 않는다(플러그인 없는 studio 의 초기 비용 0).
      '@rhwp/hwpctrl/studio-plugin': resolve(configDir, '..', 'npm', 'hwpctrl-ocx', 'src', 'studio-plugin.mjs'),
    },
  },
  // AI SDK 는 질문할 때 동적 import 로만 들어온다. 개발 서버가 그때 처음 발견하면 의존성을
  // 다시 최적화하며 페이지를 새로 고쳐 진행 중인 대화와 열린 문서를 잃는다 — 시작할 때 미리 묶는다.
  optimizeDeps: withAi
    ? { include: ['@anthropic-ai/sdk', '@anthropic-ai/sdk/lib/transform-json-schema'] }
    : {},
  server: {
    host: '127.0.0.1',
    port: 7700,
    proxy: useSubsecondWasm ? {
      '/_dioxus': {
        target: 'http://127.0.0.1:7711',
        ws: true,
      },
      '/wasm': {
        target: 'http://127.0.0.1:7711',
      },
    } : undefined,
    fs: {
      // [Task #741 후속] 외부 file path 그림 영역 영역 samples/ dir 영역 영역 fetch 가능 영역.
      allow: [
        configDir,
        resolve(configDir, '..', 'pkg'),
        subsecondWasmDir,
        resolve(configDir, '..', 'samples'),
        resolve(configDir, '..', 'npm', 'editor'),
      ],
    },
    watch: {
      ignored: ['**/librhwp-subsecond-patch-*.wasm'],
    },
  },
  build: {
    // 현재 약 2.2 MB인 Studio 진입 청크는 허용하되 추가 증가는 계속 알린다.
    chunkSizeWarningLimit: 2500,
    rolldownOptions: {
      checks: { bundlerTimings: false },
      onwarn(warning, defaultHandler) {
        const message = warning.message.replaceAll('\\', '/');
        // CanvasKit의 Node 전용 분기에 있는 fs/path 참조만 경고에서 제외한다.
        if (
          message.includes('/canvaskit-wasm/') &&
          /Module "(fs|path)" has been externalized for browser compatibility/.test(message)
        ) return;
        defaultHandler(warning);
      },
    },
  },
  plugins: [
    {
      name: 'ignore-subsecond-patch-artifacts',
      handleHotUpdate(context) {
        if (/librhwp-subsecond-patch-\d+\.wasm$/.test(context.file)) {
          return [];
        }
      },
    },
    // [Task #741 후속] dev 서버 영역 영역 /samples/* 경로 영역 영역 parent samples/ dir 영역
    // 영역 정적 serve 영역 — wasm-bridge.ts 영역 영역 외부 image fetch 영역 영역 영역.
    {
      name: 'serve-samples-dir',
      configureServer(server) {
        const samplesDir = resolve(configDir, '..', 'samples');
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
    ...(isDesktopBuild ? [] : [VitePWA({
      registerType: 'autoUpdate',
      includeAssets: ['favicon.ico', 'icons/*.png'],
      manifest: {
        name: 'rhwp-studio',
        short_name: 'rhwp',
        description: 'HWP/HWPX/HML 뷰어·에디터 — 알(R), 모두의 한글',
        lang: 'ko',
        theme_color: '#2b6cb0',
        background_color: '#ffffff',
        display: 'standalone',
        start_url: '/rhwp/',
        scope: '/rhwp/',
        file_handlers: [
          {
            action: '/rhwp/',
            accept: {
              'application/x-hwp': ['.hwp'],
              'application/hwp+zip': ['.hwpx'],
              'application/xml': ['.hml'],
              'text/xml': ['.hml'],
            },
          },
        ],
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
    })]),
  ],
});
