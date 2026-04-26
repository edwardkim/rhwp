#!/usr/bin/env node
// rhwp-firefox 빌드 스크립트
// 1. rhwp-studio를 Vite로 빌드 → dist/
// 2. WASM, 폰트, 확장 파일(manifest, background, content-script)을 dist/에 복사
// 3. dist/ 폴더가 곧 Firefox 확장 프로그램

import { execFileSync } from 'child_process';
import { cpSync, mkdirSync, existsSync, renameSync, rmSync } from 'fs';
import { resolve, dirname } from 'path';
import { fileURLToPath } from 'url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const ROOT = resolve(__dirname, '..');
const DIST = resolve(__dirname, 'dist');

// [Task #354] execSync (shell 보간) → execFileSync (인자 배열, shell:false) 전환.
// CodeQL 경고 (js/shell-command-injection-from-environment) 해소 + 공백 포함
// 경로에서도 인자 배열이 안전하게 처리됨.
function run(cmd, args, cwd = __dirname) {
  console.log(`> ${cmd} ${args.join(' ')}`);
  execFileSync(cmd, args, { stdio: 'inherit', cwd, shell: false });
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

console.log('=== rhwp-firefox 빌드 시작 ===\n');

if (existsSync(DIST)) {
  rmSync(DIST, { recursive: true, force: true });
  console.log(`  CLEAN: ${DIST}`);
}

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
// sw/ 는 download-interceptor-common.js 심링크(→ rhwp-shared/sw/)를 포함하므로
// dereference: true 로 실체 파일 복사 (패키징된 확장에서 상대 경로 의존성 제거).
copy(resolve(__dirname, 'sw'), resolve(DIST, 'sw'), {
  filter: (src) => !EXCLUDE_FROM_DIST.test(src),
  dereference: true,
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
