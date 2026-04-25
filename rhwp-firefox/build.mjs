#!/usr/bin/env node

import { execFileSync } from 'child_process';
import {
  cpSync,
  existsSync,
  mkdirSync,
  readFileSync,
  renameSync,
  writeFileSync,
} from 'fs';
import { dirname, resolve } from 'path';
import { fileURLToPath } from 'url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const ROOT = resolve(__dirname, '..');
const DIST = resolve(__dirname, 'dist');

function run(file, args, cwd = __dirname) {
  console.log(`> ${file} ${args.join(' ')}`);
  execFileSync(file, args, {
    stdio: 'inherit',
    cwd,
  });
}

function copy(src, dest, options = {}) {
  if (!existsSync(src)) {
    console.warn(`  SKIP: ${src}`);
    return;
  }
  cpSync(src, dest, { recursive: true, ...options });
  console.log(`  COPY: ${src} -> ${dest}`);
}

const EXCLUDE_FROM_DIST = /\.(test|spec)\.[mc]?[jt]sx?$/i;
const viteCli = resolve(__dirname, 'node_modules', 'vite', 'bin', 'vite.js');

console.log('=== rhwp-firefox build start ===\n');

console.log('[1/4] Building viewer with Vite...');
const studioDir = resolve(ROOT, 'rhwp-studio');
run(process.execPath, [viteCli, 'build', '--config', resolve(__dirname, 'vite.config.ts')], studioDir);

const indexHtml = resolve(DIST, 'index.html');
const viewerHtml = resolve(DIST, 'viewer.html');
if (existsSync(indexHtml)) {
  renameSync(indexHtml, viewerHtml);
  console.log('  RENAME: index.html -> viewer.html');
}

if (existsSync(viewerHtml)) {
  const viewerContent = readFileSync(viewerHtml, 'utf-8');
  writeFileSync(viewerHtml, viewerContent.replace(
    '</head>',
    '  <script src="/firefox-compat.js"></script>\n  <script src="/dev-tools-inject.js"></script>\n</head>',
  ));
  console.log('  INJECT: firefox-compat.js + dev-tools-inject.js');
}

console.log('\n[2/4] Copying extension files...');
copy(resolve(__dirname, 'manifest.json'), resolve(DIST, 'manifest.json'));
copy(resolve(__dirname, 'background.js'), resolve(DIST, 'background.js'));
copy(resolve(__dirname, 'content-script.js'), resolve(DIST, 'content-script.js'));
copy(resolve(ROOT, 'rhwp-chrome', 'content-script.css'), resolve(DIST, 'content-script.css'));
copy(resolve(ROOT, 'rhwp-chrome', 'dev-tools-inject.js'), resolve(DIST, 'dev-tools-inject.js'));
copy(resolve(__dirname, 'firefox-compat.js'), resolve(DIST, 'firefox-compat.js'));
copy(resolve(__dirname, 'sw'), resolve(DIST, 'sw'), {
  filter: (src) => !EXCLUDE_FROM_DIST.test(src),
});
copy(resolve(__dirname, 'options.js'), resolve(DIST, 'options.js'));
copy(resolve(ROOT, 'rhwp-chrome', 'options.html'), resolve(DIST, 'options.html'));
copy(resolve(ROOT, 'rhwp-chrome', 'icons'), resolve(DIST, 'icons'));
copy(resolve(ROOT, 'rhwp-chrome', '_locales'), resolve(DIST, '_locales'));

mkdirSync(resolve(DIST, 'images'), { recursive: true });
copy(
  resolve(ROOT, 'rhwp-studio', 'public', 'images', 'icon_small_ko.svg'),
  resolve(DIST, 'images', 'icon_small_ko.svg'),
);
copy(resolve(ROOT, 'rhwp-studio', 'public', 'favicon.ico'), resolve(DIST, 'favicon.ico'));

console.log('\n[3/4] Copying WASM package...');
mkdirSync(resolve(DIST, 'wasm'), { recursive: true });
copy(resolve(ROOT, 'pkg', 'rhwp.js'), resolve(DIST, 'wasm', 'rhwp.js'));
copy(resolve(ROOT, 'pkg', 'rhwp.d.ts'), resolve(DIST, 'wasm', 'rhwp.d.ts'));
copy(resolve(ROOT, 'pkg', 'rhwp_bg.wasm'), resolve(DIST, 'wasm', 'rhwp_bg.wasm'));
copy(resolve(ROOT, 'pkg', 'rhwp_bg.wasm.d.ts'), resolve(DIST, 'wasm', 'rhwp_bg.wasm.d.ts'));

console.log('\n[4/4] Copying fonts...');
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

console.log('\n=== rhwp-firefox build complete ===');
console.log(`Output: ${DIST}`);
