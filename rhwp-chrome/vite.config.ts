import { defineConfig } from 'vite';
import { resolve } from 'path';
import { readFileSync } from 'fs';

// rhwp-studio를 Chrome 확장용으로 빌드
// 산출물: rhwp-chrome/dist/ → viewer.html + JS/CSS + WASM + 폰트

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
    port: 7701,
    fs: {
      allow: ['..'],
    },
  },
});
