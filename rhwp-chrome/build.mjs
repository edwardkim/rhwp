#!/usr/bin/env node
// rhwp-chrome 빌드 스크립트
// 1. rhwp-studio를 Vite로 빌드 → dist/
// 2. WASM, 폰트, 확장 파일(manifest, background, content-script)을 dist/에 복사
// 3. dist/ 폴더가 곧 Chrome 확장 프로그램

import { execFileSync } from 'child_process';
import { cpSync, mkdirSync, existsSync, renameSync } from 'fs';
import { resolve, dirname } from 'path';
import { fileURLToPath } from 'url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const ROOT = resolve(__dirname, '..');
const DIST = resolve(__dirname, 'dist');

/**
 * Shell 우회로 명령어 실행 — 인자가 shell 에 의해 해석되지 않아 경로의 공백/특수문자 안전.
 * (CodeQL js/shell-command-injection-from-environment fix, alert #16)
 *
 * Windows 의 npx 같은 .cmd 스크립트 실행을 위해 shell: true 가 필요한 경우는
 * 별도 처리. 본 스크립트는 Linux/macOS 빌드 전제 (Docker WASM + native).
 */
function run(file, args, cwd = __dirname) {
  console.log(`> ${file} ${args.join(' ')}`);
  execFileSync(file, args, { stdio: 'inherit', cwd });
}

function copy(src, dest, options = {}) {
  if (!existsSync(src)) {
    console.warn(`  SKIP (not found): ${src}`);
    return;
  }
  cpSync(src, dest, { recursive: true, ...options });
  console.log(`  COPY: ${src} → ${dest}`);
}

/** 배포본에서 제외할 파일 패턴 (테스트 등). */
const EXCLUDE_FROM_DIST = /\.(test|spec)\.[mc]?[jt]sx?$/i;

console.log('=== rhwp-chrome 빌드 시작 ===\n');

// 1. Vite 빌드 (rhwp-studio → dist/)
console.log('[1/4] Vite 빌드...');
const studioDir = resolve(ROOT, 'rhwp-studio');
run('npx', ['vite', 'build', '--config', resolve(__dirname, 'vite.config.ts')], studioDir);

// index.html → viewer.html 이름 변경
const indexHtml = resolve(DIST, 'index.html');
const viewerHtml = resolve(DIST, 'viewer.html');
if (existsSync(indexHtml)) {
  renameSync(indexHtml, viewerHtml);
  console.log('  RENAME: index.html → viewer.html');
}

// viewer.html에 DevTools 스크립트 주입 (확장 뷰어 탭에서도 rhwpDev 사용 가능)
import { readFileSync, writeFileSync } from 'fs';
const viewerContent = readFileSync(viewerHtml, 'utf-8');
writeFileSync(viewerHtml, viewerContent.replace(
  '</head>',
  '  <script src="/dev-tools-inject.js"></script>\n</head>'
));
console.log('  INJECT: dev-tools-inject.js → viewer.html');

// 2. 확장 파일 복사
console.log('\n[2/4] 확장 파일 복사...');
copy(resolve(__dirname, 'manifest.json'), resolve(DIST, 'manifest.json'));
copy(resolve(__dirname, 'background.js'), resolve(DIST, 'background.js'));
copy(resolve(__dirname, 'content-script.js'), resolve(DIST, 'content-script.js'));
copy(resolve(__dirname, 'content-script.css'), resolve(DIST, 'content-script.css'));
copy(resolve(__dirname, 'dev-tools-inject.js'), resolve(DIST, 'dev-tools-inject.js'));
copy(resolve(__dirname, 'sw'), resolve(DIST, 'sw'), {
  filter: (src) => !EXCLUDE_FROM_DIST.test(src),
});
copy(resolve(__dirname, 'options.html'), resolve(DIST, 'options.html'));
copy(resolve(__dirname, 'options.js'), resolve(DIST, 'options.js'));

// 아이콘
mkdirSync(resolve(DIST, 'icons'), { recursive: true });
copy(resolve(__dirname, 'icons'), resolve(DIST, 'icons'));

// i18n
copy(resolve(__dirname, '_locales'), resolve(DIST, '_locales'));

// rhwp-studio 리소스 (CSS에서 참조)
mkdirSync(resolve(DIST, 'images'), { recursive: true });
copy(resolve(ROOT, 'rhwp-studio', 'public', 'images', 'icon_small_ko.svg'), resolve(DIST, 'images', 'icon_small_ko.svg'));
copy(resolve(ROOT, 'rhwp-studio', 'public', 'favicon.ico'), resolve(DIST, 'favicon.ico'));

// 3. WASM 복사
console.log('\n[3/4] WASM 복사...');
mkdirSync(resolve(DIST, 'wasm'), { recursive: true });
copy(resolve(ROOT, 'pkg', 'rhwp.js'), resolve(DIST, 'wasm', 'rhwp.js'));
copy(resolve(ROOT, 'pkg', 'rhwp.d.ts'), resolve(DIST, 'wasm', 'rhwp.d.ts'));
copy(resolve(ROOT, 'pkg', 'rhwp_bg.wasm'), resolve(DIST, 'wasm', 'rhwp_bg.wasm'));
copy(resolve(ROOT, 'pkg', 'rhwp_bg.wasm.d.ts'), resolve(DIST, 'wasm', 'rhwp_bg.wasm.d.ts'));

// 4. 폰트 복사 (필수 폰트만)
console.log('\n[4/4] 폰트 복사...');
mkdirSync(resolve(DIST, 'fonts'), { recursive: true });
const essentialFonts = [
  'Pretendard-Regular.woff2',
  'Pretendard-Bold.woff2',
  'NotoSansKR-Regular.woff2',
  'NotoSansKR-Bold.woff2',
  'NotoSerifKR-Regular.woff2',
  'NotoSerifKR-Bold.woff2',
  'GowunBatang-Regular.woff2',
  'GowunBatang-Bold.woff2',
  'GowunDodum-Regular.woff2',
  'NanumGothic-Regular.woff2',
  'NanumGothic-Bold.woff2',
  'NanumMyeongjo-Regular.woff2',
  'NanumMyeongjo-Bold.woff2',
  'D2Coding-Regular.woff2',
];
for (const font of essentialFonts) {
  copy(resolve(ROOT, 'web', 'fonts', font), resolve(DIST, 'fonts', font));
}

console.log('\n=== 빌드 완료 ===');
console.log(`출력: ${DIST}`);
