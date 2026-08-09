'use strict';

const assert = require('node:assert/strict');
const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');
const test = require('node:test');

const {
  classifyChanges,
  runCli,
} = require('../ci-impact-classifier.cjs');

const FIXTURE_PATH = path.join(
  __dirname,
  'fixtures',
  'ci-impact-classifier-prs.json',
);
const HISTORICAL_PRS = JSON.parse(fs.readFileSync(FIXTURE_PATH, 'utf8'));

for (const fixture of HISTORICAL_PRS) {
  test(`historical PR #${fixture.pr}: ${fixture.title}`, () => {
    assert.deepEqual(
      classifyChanges({ eventName: 'pull_request', files: fixture.files }),
      fixture.expected,
    );
  });
}

test('review-only changes require no code worker', () => {
  assert.deepEqual(
    classifyChanges({
      eventName: 'pull_request',
      files: [
        { filename: 'mydocs/orders/20260802.md', status: 'modified' },
        { filename: 'README.md', status: 'modified' },
      ],
    }),
    {
      rust_required: 'false',
      frontend_mode: 'none',
      render_required: 'false',
      native_skia_required: 'false',
      codeql_languages: 'none',
      classification_status: 'classified',
      classifier_version: '2',
      reason: 'classified:review-only',
    },
  );
});

test('mixed Studio package and Rust changes union modes and CodeQL languages', () => {
  assert.deepEqual(
    classifyChanges({
      eventName: 'pull_request',
      files: [
        { filename: 'rhwp-studio/src/hwpctl/action.ts', status: 'modified' },
        { filename: 'src/parser/hwpx/mod.rs', status: 'modified' },
      ],
    }),
    {
      rust_required: 'true',
      frontend_mode: 'package',
      render_required: 'false',
      native_skia_required: 'false',
      codeql_languages: 'javascript-typescript,rust',
      classification_status: 'classified',
      classifier_version: '2',
      reason: 'classified:rust+studio-package',
    },
  );
});

test('Rust renderer changes require Rust, Native Skia, Canvas, and Rust CodeQL', () => {
  const result = classifyChanges({
    eventName: 'pull_request',
    files: [{ filename: 'src/renderer/layout/table.rs', status: 'modified' }],
  });

  assert.equal(result.rust_required, 'true');
  assert.equal(result.frontend_mode, 'none');
  assert.equal(result.render_required, 'true');
  assert.equal(result.native_skia_required, 'true');
  assert.equal(result.codeql_languages, 'rust');
  assert.equal(result.classification_status, 'classified');
});

test('Native Skia integration test and support changes run Rust and Native Skia without Canvas', () => {
  for (const filename of [
    'tests/cli_exit_codes_native.rs',
    'tests/issue_1144_native.rs',
    'tests/issue_2083_hide_fill_page_background.rs',
    'tests/issue_2225_missing_picture_placeholder.rs',
    'tests/issue_2292_chart_png_clip.rs',
    'tests/issue_2293_chart_png_text.rs',
    'tests/render_p37_direct_pdf_export.rs',
    'tests/support/cli_exit_code_support.rs',
    'tests/support/issue_1144_support.rs',
  ]) {
    const result = classifyChanges({
      eventName: 'pull_request',
      files: [{ filename, status: 'modified' }],
    });
    assert.equal(result.rust_required, 'true', filename);
    assert.equal(result.render_required, 'false', filename);
    assert.equal(result.native_skia_required, 'true', filename);
    assert.equal(result.codeql_languages, 'rust', filename);
    assert.equal(result.classification_status, 'classified', filename);
    assert.equal(result.classifier_version, '2', filename);
    assert.equal(result.reason, 'classified:native-skia-rust', filename);
  }
});

test('Rust test input changes keep default Rust tests alongside render gates', () => {
  for (const filename of [
    'tests/fixtures/fonts/RHWPExactFaceSmoke.ttc',
    'ttfs/opensource/NotoSansKR-Regular.ttf',
    'samples/render-p35-font-native-bitmap.hwpx',
  ]) {
    const result = classifyChanges({
      eventName: 'pull_request',
      files: [{ filename, status: 'modified' }],
    });
    assert.equal(result.rust_required, 'true', filename);
    assert.equal(result.render_required, 'true', filename);
    assert.equal(result.native_skia_required, 'true', filename);
    assert.equal(result.codeql_languages, 'none', filename);
    assert.equal(result.classification_status, 'classified', filename);
    assert.equal(result.classifier_version, '2', filename);
    assert.equal(result.reason, 'classified:rust-test-input', filename);
  }
});

test('frontend font assets and render tooling do not over-enable the Rust lane', () => {
  for (const filename of [
    'assets/fonts/NotoSansKR-Regular.woff2',
    'scripts/generate_exact_face_collection_fixture.py',
    'docs/text-ir-v2.md',
  ]) {
    const result = classifyChanges({
      eventName: 'pull_request',
      files: [{ filename, status: 'modified' }],
    });
    assert.equal(result.rust_required, 'false', filename);
    assert.equal(result.render_required, 'true', filename);
    assert.equal(result.native_skia_required, 'true', filename);
  }
});

test('Studio package configuration and broad runtime sources remain render-impacting', () => {
  for (const filename of [
    'rhwp-studio/package.json',
    'rhwp-studio/vite.config.ts',
    'rhwp-studio/src/style.css',
    'rhwp-studio/src/core/wasm-bridge.ts',
    'rhwp-studio/src/view/page-renderer.ts',
    'rhwp-studio/src/ui/hwp-password-dialog.ts',
    'rhwp-studio/src/engine/input-handler-keyboard.ts',
  ]) {
    const result = classifyChanges({
      eventName: 'pull_request',
      files: [{ filename, status: 'modified' }],
    });
    assert.equal(result.render_required, 'true', filename);
    assert.equal(result.classification_status, 'classified', filename);
    assert.equal(result.frontend_mode, 'package', filename);
  }
});

test('known command sources and non-render tests stay on the unit lane', () => {
  for (const filename of [
    'rhwp-studio/src/command/shortcut-map.ts',
    'rhwp-studio/src/engine/command.ts',
    'rhwp-studio/tests/shortcut-map.test.ts',
    'rhwp-studio/tests/canvaskit-readiness.test.ts',
    'rhwp-studio/tests/render-page.test.ts',
  ]) {
    const result = classifyChanges({
      eventName: 'pull_request',
      files: [{ filename, status: 'modified' }],
    });
    assert.equal(result.frontend_mode, 'unit', filename);
    assert.equal(result.render_required, 'false', filename);
  }
});

test('rename evaluates fail-closed before either path can be skipped', () => {
  const result = classifyChanges({
    eventName: 'pull_request',
    files: [{
      filename: 'rhwp-studio/src/command/new-name.ts',
      previous_filename: 'rhwp-studio/src/command/old-name.ts',
      status: 'renamed',
    }],
  });

  assert.equal(result.classification_status, 'full');
  assert.equal(result.reason, 'fail-closed:rename');
});

for (const [filename, expectedReason] of [
  ['Cargo.lock', 'fail-closed:cargo-contract'],
  ['.github/workflows/ci.yml', 'fail-closed:workflow-contract'],
  ['src/main.rs', 'fail-closed:main-render-boundary'],
  ['src/wasm_api.rs', 'fail-closed:wasm-contract'],
  ['scripts/ci-impact-classifier.cjs', 'fail-closed:classifier-contract'],
  ['rhwp-studio/tsconfig.ci-unit.json', 'fail-closed:frontend-unit-contract'],
  ['rhwp-studio/types/wasm-ci-unit-stub.d.ts', 'fail-closed:frontend-unit-contract'],
  ['web/new-entry.ts', 'fail-closed:unclassified-path'],
  ['unclassified/new-format.schema', 'fail-closed:unclassified-path'],
]) {
  test(`${filename} is fail-closed`, () => {
    const result = classifyChanges({
      eventName: 'pull_request',
      files: [{ filename, status: 'modified' }],
    });
    assert.equal(result.classification_status, 'full');
    assert.equal(result.reason, expectedReason);
    assert.equal(result.codeql_languages, 'javascript-typescript,python,rust');
  });
}

test('empty and forced file collection failures are full', () => {
  assert.equal(
    classifyChanges({ eventName: 'pull_request', files: [] }).reason,
    'fail-closed:file-list-empty',
  );
  assert.equal(
    classifyChanges({
      eventName: 'pull_request',
      files: [],
      forceFullReason: 'collection-error',
    }).reason,
    'fail-closed:collection-error',
  );
});

test('documented PR and push API boundaries are full', () => {
  const prFiles = Array.from(
    { length: 3000 },
    (_, index) => ({ filename: `mydocs/pr-${index}.md`, status: 'modified' }),
  );
  const pushFiles = Array.from(
    { length: 300 },
    (_, index) => ({ filename: `mydocs/push-${index}.md`, status: 'modified' }),
  );

  assert.equal(
    classifyChanges({ eventName: 'pull_request', files: prFiles }).reason,
    'fail-closed:pull_request-file-list-boundary',
  );
  assert.equal(
    classifyChanges({ eventName: 'push', files: pushFiles }).reason,
    'fail-closed:push-file-list-boundary',
  );
});

test('full fallback does not depend on input ordering', () => {
  const cargoFirst = [
    { filename: 'Cargo.toml', status: 'modified' },
    { filename: '.github/workflows/ci.yml', status: 'modified' },
  ];
  assert.deepEqual(
    classifyChanges({ eventName: 'pull_request', files: cargoFirst }),
    classifyChanges({ eventName: 'pull_request', files: cargoFirst.slice().reverse() }),
  );
});

test('CLI writes every classifier output to the GitHub output file', (t) => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), 'rhwp-ci-impact-'));
  t.after(() => fs.rmSync(directory, { recursive: true, force: true }));
  const inputPath = path.join(directory, 'input.json');
  const outputPath = path.join(directory, 'github-output.txt');
  fs.writeFileSync(inputPath, JSON.stringify({
    eventName: 'pull_request',
    files: [{ filename: 'rhwp-studio/src/command/shortcut-map.ts', status: 'modified' }],
  }));

  const result = runCli(['--input', inputPath, '--github-output', outputPath]);
  const output = fs.readFileSync(outputPath, 'utf8');

  for (const [key, value] of Object.entries(result)) {
    assert.match(output, new RegExp(`^${key}=${value}$`, 'm'));
  }
});
