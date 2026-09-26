'use strict';
const test = require('node:test');
const assert = require('node:assert/strict');
const { classifyChromeExtension: classify } = require('../chrome-extension-impact.cjs');
const { classifyChanges } = require('../ci-impact-classifier.cjs');
const input = (...files) => ({ eventName: 'pull_request', baseRef: 'devel', files: files.map(filename => ({ filename, status: 'modified' })) });

const nativeOnlyFiles = [
  'src/main.rs', 'src/cli/document_io.rs', 'src/cli/commands/edit/mod.rs',
  'src/bin/rhwp-agent/main.rs', 'src/tools/font_metric_gen.rs',
  'bindings/Native/src/lib.rs', 'scripts/package-swift-xcframework.sh',
  'scripts/frontend-vscode-outline.test.mjs',
];

test('devel skips verified binary and native consumers without promoting the frontend lane', () => {
  for (const filename of nativeOnlyFiles) {
    for (const status of ['added', 'modified', 'removed']) {
      const value = { ...input(filename), files: [{ filename, status }] };
      assert.equal(classify(value).chrome_extension_e2e_required, 'false', `${status}:${filename}`);
    }
  }
  for (const filename of nativeOnlyFiles.filter(file => file.endsWith('.rs'))) {
    assert.equal(classifyChanges(input(filename)).frontend_mode, 'none', filename);
  }
});

test('shared Rust and build contracts stay required alongside native-only changes', () => {
  for (const file of ['src/lib.rs', 'src/parser/mod.rs', 'src/service/open.rs',
    'src/wasm_api.rs', 'src/renderer/layout.rs', 'crates/rhwp-contracts/src/lib.rs',
    'Cargo.toml', 'Cargo.lock', 'build.rs', 'bindings/Native/Cargo.toml']) {
    assert.equal(classify(input(...nativeOnlyFiles, file)).chrome_extension_e2e_required, 'true', file);
  }
});

test('main runs the complete Chrome suite even for native, unrelated or review-only changes', () => {
  for (const file of [...nativeOnlyFiles, 'rhwp-firefox/background.js', 'mydocs/review.md']) {
    const result = classify({ ...input(file), baseRef: 'main' });
    assert.equal(result.chrome_extension_e2e_required, 'true', file);
    assert.equal(result.chrome_extension_e2e_reason, 'main-release-validation', file);
  }
});

test('missing or unsupported target branch cannot opt out of Chrome validation', () => {
  for (const baseRef of [undefined, '', 'feature', 'refs/heads/devel']) {
    assert.equal(classify({ ...input('mydocs/review.md'), baseRef }).chrome_extension_e2e_required, 'true');
  }
});

test('renames across the native/shared boundary evaluate both sides', () => {
  for (const [filename, previous_filename, required] of [
    ['src/cli/new.rs', 'src/cli/old.rs', false],
    ['src/cli/new.rs', 'src/parser/old.rs', true],
    ['src/parser/new.rs', 'src/cli/old.rs', true],
    ['bindings/Native/src/new.rs', 'crates/rhwp-contracts/src/old.rs', true],
  ]) {
    const value = { ...input(), files: [{ filename, previous_filename, status: 'renamed' }] };
    assert.equal(classify(value).chrome_extension_e2e_required, String(required), `${previous_filename} -> ${filename}`);
  }
});
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
    'rhwp-studio/public/samples/demo.hwpx',
    'scripts/tests/test_gym_work_receipt_pack.py')).chrome_extension_e2e_required, 'false');
});
test('renames evaluate both sides and missing old path runs conservatively', () => {
  for (const file of [
    { filename: 'mydocs/old.md', previous_filename: 'rhwp-chrome/options.js', status: 'renamed' },
    { filename: 'rhwp-chrome/options.js', previous_filename: 'mydocs/old.md', status: 'renamed' },
    { filename: 'mydocs/old.md', status: 'renamed' },
  ]) assert.equal(classify({ ...input(), files: [file] }).chrome_extension_e2e_required, 'true');
});
test('missing, truncated, malformed and explicit runs never skip', () => {
  for (const value of [null, input(), { ...input('README.md'), expectedFileCount: 2 },
    { ...input('README.md'), eventName: 'workflow_dispatch' },
    { ...input('README.md'), eventName: 'push' },
    { ...input('README.md'), forceFullReason: 'collection-error' },
    input('../invalid'), input('unknown.config'),
    { ...input(), files: Array.from({ length: 3000 }, () => ({ filename: 'README.md' })) },
  ]) assert.equal(classify(value).chrome_extension_e2e_required, 'true');
});
