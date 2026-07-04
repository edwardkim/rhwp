#!/usr/bin/env node
// rhwp-chrome 빌드 스크립트
// 1. rhwp-studio를 Vite로 빌드 → dist/
// 2. WASM, 폰트, 확장 파일(manifest, background, content-script)을 dist/에 복사
// 3. dist/ 폴더가 곧 Chrome 확장 프로그램

import { execFileSync } from 'child_process';
import { cpSync, mkdirSync, existsSync, renameSync, rmSync, readdirSync } from 'fs';
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

// [#1444] 다크테마 FOUC 방지 스크립트. vite 는 publicDir:false 라 public/ 을 복사하지
// 않으므로 viewer.html 의 <script src="/theme-init.js"> 가 가리키는 파일을 개별 복사한다.
// 확장 CSP('self')는 인라인을 금지하므로 인라인 대신 이 외부 파일을 쓴다.
copy(resolve(ROOT, 'rhwp-studio', 'public', 'theme-init.js'), resolve(DIST, 'theme-init.js'));

// rhwp-studio 리소스 (CSS에서 참조)
mkdirSync(resolve(DIST, 'images'), { recursive: true });
copy(resolve(ROOT, 'rhwp-studio', 'public', 'images', 'icon_small_ko.svg'), resolve(DIST, 'images', 'icon_small_ko.svg'));
// [#1444] 다크 모드 아이콘 스프라이트 (base.css 다크 테마에서 참조). 누락 시 viewer 404.
copy(resolve(ROOT, 'rhwp-studio', 'public', 'images', 'icon_small_ko_dark.svg'), resolve(DIST, 'images', 'icon_small_ko_dark.svg'));
copy(resolve(ROOT, 'rhwp-studio', 'public', 'favicon.ico'), resolve(DIST, 'favicon.ico'));

// 3. WASM 복사
console.log('\n[3/4] WASM 복사...');
mkdirSync(resolve(DIST, 'wasm'), { recursive: true });
copy(resolve(ROOT, 'pkg', 'rhwp.js'), resolve(DIST, 'wasm', 'rhwp.js'));
copy(resolve(ROOT, 'pkg', 'rhwp.d.ts'), resolve(DIST, 'wasm', 'rhwp.d.ts'));
copy(resolve(ROOT, 'pkg', 'rhwp_bg.wasm'), resolve(DIST, 'wasm', 'rhwp_bg.wasm'));
copy(resolve(ROOT, 'pkg', 'rhwp_bg.wasm.d.ts'), resolve(DIST, 'wasm', 'rhwp_bg.wasm.d.ts'));

// 4. 폰트 복사 (web/fonts 전체 woff2)
// FontLoader(rhwp-studio)가 참조하는 폰트와 자동 일치시키기 위해 하드코딩 목록 대신
// web/fonts 의 woff2 전부를 복사한다. 목록을 고정하면 폴백 폰트 추가 시 누락된다 (#1484).
console.log('\n[4/4] 폰트 복사...');
mkdirSync(resolve(DIST, 'fonts'), { recursive: true });
const fontDir = resolve(ROOT, 'web', 'fonts');
const fontFiles = readdirSync(fontDir).filter((f) => f.endsWith('.woff2'));
for (const font of fontFiles) {
  copy(resolve(fontDir, font), resolve(DIST, 'fonts', font));
}
console.log(`  woff2 ${fontFiles.length}개 복사`);

console.log('\n=== 빌드 완료 ===');
console.log(`출력: ${DIST}`);
