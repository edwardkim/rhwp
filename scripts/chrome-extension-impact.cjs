'use strict';
const fs = require('node:fs');

const requiredPrefixes = ['rhwp-chrome/', 'rhwp-shared/sw/', 'rhwp-studio/src/', 'src/', 'crates/', 'pkg/', 'assets/fonts/'];
const requiredFiles = new Set([
  '.github/workflows/ci.yml', '.github/workflows/chrome-browser-cache.yml',
  'scripts/chrome-extension-impact.cjs', 'scripts/tests/chrome-extension-impact.test.cjs',
  'scripts/tests/test_chrome_extension_workflow.py', 'scripts/ci-impact-classifier.cjs',
  'scripts/frontend-extension-dist.test.mjs', 'scripts/wasm-pack-locked.sh',
  'Cargo.toml', 'Cargo.lock', 'rust-toolchain.toml',
  'rhwp-studio/package.json', 'rhwp-studio/package-lock.json', 'rhwp-studio/index.html',
  'rhwp-studio/public/theme-init.js', 'rhwp-studio/public/locale-init.js', 'rhwp-studio/public/print.html',
  'rhwp-studio/public/images/icon_small_ko.svg', 'rhwp-studio/public/images/icon_small_ko_dark.svg',
  'rhwp-studio/public/icons/icon-256.png', 'rhwp-studio/public/favicon.ico',
  'samples/hwp3-pagedef-1915.hwp', 'samples/hwpx_sample2.hwpx', 'samples/re-font-dotum-empty-hancom.hwp',
]);
const unrelatedPrefixes = ['mydocs/', 'rhwp-firefox/', 'rhwp-safari/', 'rhwp-vscode/', 'npm/editor/',
  'rhwp-studio/tests/', 'rhwp-studio/e2e/', 'samples/', 'pdf/', 'tests/', 'gym/', 'assets/screenshots/',
  'assets/chrome/', 'assets/edge/', 'assets/logo/'];
function result(required, reason) {
  return { chrome_extension_e2e_required: String(required), chrome_extension_e2e_reason: reason };
}
function classifyChromeExtension(input) {
  if (!input || input.forceFullReason) return result(true, input?.forceFullReason || 'invalid-input');
  if (input.eventName !== 'pull_request') return result(true, 'manual-tag-or-unknown-event');
  if (!Array.isArray(input.files) || input.files.length === 0) return result(true, 'empty-or-invalid-files');
  if (input.files.length >= 3000 || (input.expectedFileCount != null && input.expectedFileCount !== input.files.length)) {
    return result(true, 'incomplete-file-list');
  }
  const paths = [];
  for (const file of input.files) {
    if (!file || typeof file.filename !== 'string' || !file.filename
      || file.filename.startsWith('/') || file.filename.includes('..') || file.filename.includes('\\')) {
      return result(true, 'invalid-path');
    }
    paths.push(file.filename);
    if (file.status === 'renamed' && !file.previous_filename) return result(true, 'missing-rename-source');
    if (file.previous_filename) {
      if (typeof file.previous_filename !== 'string' || file.previous_filename.startsWith('/')
        || file.previous_filename.includes('..') || file.previous_filename.includes('\\')) {
        return result(true, 'invalid-rename-source');
      }
      paths.push(file.previous_filename);
    }
  }
  for (const filename of paths.sort()) {
    if (filename.endsWith('.md') || filename === 'LICENSE') continue;
    if (requiredFiles.has(filename) || requiredPrefixes.some(prefix => filename.startsWith(prefix))) {
      return result(true, `extension-input:${filename}`);
    }
    if (unrelatedPrefixes.some(prefix => filename.startsWith(prefix))
      || /^scripts\/tests\/test_gym_.*\.py$/.test(filename)) continue;
    return result(true, `unclassified-path:${filename}`);
  }
  return result(false, 'unrelated-paths');
}
module.exports = { classifyChromeExtension };
if (require.main === module) {
  let output;
  try { output = classifyChromeExtension(JSON.parse(fs.readFileSync(process.argv[2], 'utf8'))); }
  catch { output = result(true, 'classification-error'); }
  console.log(JSON.stringify(output));
  if (process.env.GITHUB_OUTPUT) fs.appendFileSync(process.env.GITHUB_OUTPUT,
    Object.entries(output).map(([key, value]) => `${key}=${value.replace(/[\r\n]/g, ' ')}\n`).join(''));
}
