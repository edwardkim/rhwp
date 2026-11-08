import { defineConfig } from 'vite';
import { resolve } from 'path';
import { readFileSync } from 'fs';

// rhwp-studio를 Firefox 확장용으로 빌드
// 산출물: rhwp-firefox/dist/ → viewer.html + JS/CSS + WASM + 폰트

// rhwp-studio 의 package.json 버전을 __APP_VERSION__ 으로 주입
// (rhwp-studio/vite.config.ts 와 동일 패턴 — about-dialog 가 ReferenceError 나지 않도록)
const studioPkg = JSON.parse(
  readFileSync(resolve(__dirname, '..', 'rhwp-studio', 'package.json'), 'utf-8'),
);

export default defineConfig({
  root: resolve(__dirname, '..', 'rhwp-studio'),
  publicDir: false, // public/ 폴더 제외 (samples, images 등 불필요)
  define: {
    __APP_VERSION__: JSON.stringify(studioPkg.version),
    // AI 도우미는 확장 배포본에 넣지 않는다 — 확장 CSP·키 보관 정책을 따로 정하기 전까지.
    // 정의하지 않으면 studio 의 `__RHWP_AI__` 참조가 시작 시 ReferenceError 가 된다.
    __RHWP_AI__: 'false',
  },
  resolve: {
    alias: {
      '@': resolve(__dirname, '..', 'rhwp-studio', 'src'),
      '@wasm': resolve(__dirname, '..', 'pkg'),
    },
  },
  build: {
    outDir: resolve(__dirname, 'dist'),
    emptyDir: true,
    rollupOptions: {
      input: {
        viewer: resolve(__dirname, '..', 'rhwp-studio', 'index.html'),
      },
    },
    // WASM inline 방지 — 별도 파일로 유지
    assetsInlineLimit: 0,
  },
  // 개발 서버 (확장 디버깅용)
  server: {
    host: '0.0.0.0',
    port: 7702,
    fs: {
      allow: ['..'],
    },
  },
});
