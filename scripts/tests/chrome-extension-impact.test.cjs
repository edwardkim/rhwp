'use strict';
const test = require('node:test');
const assert = require('node:assert/strict');
const { classifyChromeExtension: classify } = require('../chrome-extension-impact.cjs');
const input = (...files) => ({ eventName: 'pull_request', files: files.map(filename => ({ filename, status: 'modified' })) });
test('packaged surfaces, shared worker, runtime, WASM and fixtures require Chrome', () => {
  for (const file of ['rhwp-chrome/options.js', 'rhwp-shared/sw/settings-store.js',
    'rhwp-studio/src/ui/print-surface.ts', 'rhwp-studio/public/print.html',
    'rhwp-studio/public/images/icon_small_ko_dark.svg', 'src/parser/hwpx/mod.rs', 'Cargo.lock',
    'samples/hwpx_sample2.hwpx', '.github/workflows/ci.yml']) {
    assert.equal(classify(input(file)).chrome_extension_e2e_required, 'true', file);
  }
});
test('unrelated products, documents and Studio tests do not require Chrome', () => {
  assert.equal(classify(input('rhwp-firefox/background.js', 'rhwp-safari/manifest.json',
    'rhwp-vscode/src/extension.ts', 'npm/editor/src/index.ts', 'mydocs/plan.md',
    'rhwp-studio/tests/test.ts', 'rhwp-studio/e2e/foo.mjs', 'rhwp-chrome/README.md',
    'scripts/tests/test_gym_work_receipt_pack.py')).chrome_extension_e2e_required, 'false');
});
test('renames evaluate both sides and missing old path runs conservatively', () => {
  for (const file of [
    { filename: 'mydocs/old.md', previous_filename: 'rhwp-chrome/options.js', status: 'renamed' },
    { filename: 'rhwp-chrome/options.js', previous_filename: 'mydocs/old.md', status: 'renamed' },
    { filename: 'mydocs/old.md', status: 'renamed' },
  ]) assert.equal(classify({ eventName: 'pull_request', files: [file] }).chrome_extension_e2e_required, 'true');
});
test('missing, truncated, malformed and explicit runs never skip', () => {
  for (const value of [null, input(), { ...input('README.md'), expectedFileCount: 2 },
    { ...input('README.md'), eventName: 'workflow_dispatch' },
    { ...input('README.md'), eventName: 'push' },
    { ...input('README.md'), forceFullReason: 'collection-error' },
    input('../invalid'), input('unknown.config'),
    { eventName: 'pull_request', files: Array.from({ length: 3000 }, () => ({ filename: 'README.md' })) },
  ]) assert.equal(classify(value).chrome_extension_e2e_required, 'true');
});
