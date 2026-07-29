import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { existsSync, lstatSync, readFileSync, readdirSync, readlinkSync, statSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';

const ROOT = path.resolve(fileURLToPath(new URL('..', import.meta.url)));
const CANONICAL_DIR = path.join(ROOT, 'assets/fonts');
const EXPECTED_FONT_COUNT = 36;
const EXPECTED_TOTAL_BYTES = 22_651_296;
const EXPECTED_NOTO_SANS_KR_SHA256 = 'd1bf8649914a4fe9477a8735bf056383e44e466141fb3d61897252e06d900c1a';
const VSCODE_FONT_FILES = [
  'NotoSerifKR-Regular.woff2',
  'NotoSerifKR-Bold.woff2',
  'NotoSansKR-Regular.woff2',
  'NotoSansKR-Bold.woff2',
  'NotoSansKR-ExtraLight.woff2',
  'Pretendard-Regular.woff2',
  'Pretendard-Bold.woff2',
  'D2Coding-Regular.woff2',
  'NanumGothic-Regular.woff2',
  'NanumMyeongjo-Regular.woff2',
  'GowunBatang-Regular.woff2',
  'GowunDodum-Regular.woff2',
].sort();
const VSCODE_RUNTIME_LICENSE_FILES = [
  ['canvaskit-wasm-LICENSE.txt', 'canvaskit-wasm/LICENSE'],
  ['noble-hashes-LICENSE.txt', '@noble/hashes/LICENSE'],
];

function fontFiles(directory) {
  return readdirSync(directory).filter((file) => file.endsWith('.woff2')).sort();
}

function sha256(file) {
  return createHash('sha256').update(readFileSync(file)).digest('hex');
}

function assertDirectoryMatchesSource(directory, expectedFiles, label) {
  assert.ok(existsSync(directory), `${label} directory must exist: ${directory}`);
  assert.deepEqual(fontFiles(directory), expectedFiles, `${label} font filenames must match`);
  for (const file of expectedFiles) {
    const source = path.join(CANONICAL_DIR, file);
    const artifact = path.join(directory, file);
    assert.equal(statSync(artifact).size, statSync(source).size, `${label}/${file} byte size must match`);
    assert.equal(sha256(artifact), sha256(source), `${label}/${file} SHA-256 must match`);
  }
}

test('canonical font inventory and license contract', () => {
  const files = fontFiles(CANONICAL_DIR);
  assert.equal(files.length, EXPECTED_FONT_COUNT);
  assert.equal(
    files.reduce((total, file) => total + statSync(path.join(CANONICAL_DIR, file)).size, 0),
    EXPECTED_TOTAL_BYTES,
  );
  assert.ok(statSync(path.join(CANONICAL_DIR, 'FONTS.md')).size > 0);
  assert.ok(statSync(path.join(CANONICAL_DIR, 'SourceHanSerifK-OFL.txt')).size > 0);
  assert.equal(sha256(path.join(CANONICAL_DIR, 'NotoSansKR-Regular.woff2')), EXPECTED_NOTO_SANS_KR_SHA256);
});

test('Studio font link targets the canonical directory', () => {
  const relative = 'rhwp-studio/public/fonts';
  const target = '../../assets/fonts';
  const link = path.join(ROOT, relative);
  assert.ok(lstatSync(link).isSymbolicLink(), `${relative} must be a symlink`);
  assert.equal(readlinkSync(link), target, `${relative} must target ${target}`);
});

test('Studio and browser extension distributions preserve all canonical fonts', () => {
  const expected = fontFiles(CANONICAL_DIR);
  assertDirectoryMatchesSource(path.join(ROOT, 'rhwp-studio/dist/fonts'), expected, 'Studio dist');
  assertDirectoryMatchesSource(path.join(ROOT, 'rhwp-chrome/dist/fonts'), expected, 'Chrome dist');
  assertDirectoryMatchesSource(path.join(ROOT, 'rhwp-firefox/dist/fonts'), expected, 'Firefox dist');
});

test('VS Code distribution preserves the approved font subset', () => {
  assertDirectoryMatchesSource(
    path.join(ROOT, 'rhwp-vscode/dist/media/fonts'),
    VSCODE_FONT_FILES,
    'VS Code dist',
  );
});

test('VS Code CanvasKit auto font planning is limited to packaged fonts', () => {
  const viewerSource = readFileSync(
    path.join(ROOT, 'rhwp-vscode/src/webview/viewer.ts'),
    'utf8',
  );
  const planStart = viewerSource.indexOf('const canvasKitFontPlan');
  const planEnd = viewerSource.indexOf('const rendererSession', planStart);
  assert.ok(planStart >= 0 && planEnd > planStart, 'VS Code CanvasKit font plan must be declared');
  const planSource = viewerSource.slice(planStart, planEnd);
  assert.match(planSource, /availableLocalFiles:\s*vscodeBundledFontFiles/);
  assert.match(planSource, /disableExternalWebFonts:\s*true/);
});

test('VS Code VSIX distribution preserves full runtime dependency license notices', () => {
  const vscodeRoot = path.join(ROOT, 'rhwp-vscode');
  assert.match(
    readFileSync(path.join(vscodeRoot, '.vscodeignore'), 'utf8'),
    /^node_modules\/\*\*$/m,
    'VSIX packaging must not rely on excluded node_modules license files',
  );
  for (const [artifactName, packageLicense] of VSCODE_RUNTIME_LICENSE_FILES) {
    const source = path.join(vscodeRoot, 'node_modules', packageLicense);
    const artifact = path.join(vscodeRoot, 'dist', 'licenses', artifactName);
    assert.ok(existsSync(artifact), `VS Code license artifact must exist: ${artifactName}`);
    assert.deepEqual(
      readFileSync(artifact),
      readFileSync(source),
      `${artifactName} must preserve the complete dependency license notice`,
    );
  }
});
