import assert from 'node:assert/strict';
import { mkdtempSync, mkdirSync, writeFileSync, rmSync } from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import test from 'node:test';

import {
  ADAPTERS,
  REQUIRED_CASES,
  adapterSourcePath,
  isExported,
  planAdapterDiff,
  rustflagsFor,
} from '../run-adapter-diff.mjs';

function withTempRoot(run) {
  const root = mkdtempSync(path.join(os.tmpdir(), 'adapter-diff-'));
  try {
    mkdirSync(path.join(root, 'src', 'render_backend'), { recursive: true });
    mkdirSync(path.join(root, 'tests', 'cases'), { recursive: true });
    return run(root);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
}

const MOD_CORE = [
  'pub mod backends;',
  'pub mod svg_adapter;',
  'pub use backends::{NullBackend, TraceBackend};',
  'pub use svg_adapter::SvgBackend;',
  '',
].join('\n');

function writeCore(root, mod = MOD_CORE) {
  writeFileSync(path.join(root, 'src', 'render_backend', 'svg_adapter.rs'), '');
  writeFileSync(path.join(root, 'src', 'render_backend', 'backends.rs'), '');
  writeFileSync(path.join(root, 'src', 'render_backend', 'mod.rs'), mod);
}

test('required wiring case is adapter_diff', () => {
  assert.deepEqual(REQUIRED_CASES, ['adapter_diff']);
  assert.deepEqual(
    ADAPTERS.map((adapter) => adapter.name),
    ['svg', 'null', 'trace', 'png', 'skia'],
  );
});

test('optional png/skia are skipped when source files are absent', () => {
  withTempRoot((root) => {
    writeCore(root);
    const plan = planAdapterDiff(root);
    assert.deepEqual(plan.present, ['svg', 'null', 'trace']);
    assert.deepEqual(
      plan.skipped.map((item) => `${item.name}:${item.reason}`),
      ['png:missing', 'skia:missing'],
    );
    assert.deepEqual(plan.cfgs, []);
    assert.deepEqual(plan.run, ['adapter_diff']);
  });
});

test('optional adapters are picked up when source and export exist', () => {
  withTempRoot((root) => {
    writeCore(
      root,
      `${MOD_CORE}pub mod png_adapter;\npub use png_adapter::PngBackend;\n` +
        'pub mod skia_adapter;\npub use skia_adapter::SkiaBackend;\n',
    );
    writeFileSync(adapterSourcePath('src/render_backend/png_adapter.rs', root), '');
    writeFileSync(adapterSourcePath('src/render_backend/skia_adapter.rs', root), '');
    const plan = planAdapterDiff(root);
    assert.deepEqual(plan.present, ['svg', 'null', 'trace', 'png', 'skia']);
    assert.deepEqual(plan.skipped, []);
    assert.deepEqual(plan.cfgs, ['rhwp_has_png_backend', 'rhwp_has_skia_backend']);
  });
});

test('png file without export is skipped honestly', () => {
  withTempRoot((root) => {
    writeCore(root);
    writeFileSync(adapterSourcePath('src/render_backend/png_adapter.rs', root), '');
    const plan = planAdapterDiff(root);
    assert.ok(!plan.present.includes('png'));
    assert.deepEqual(
      plan.skipped.filter((item) => item.name === 'png'),
      [{ name: 'png', reason: 'unexported' }],
    );
  });
});

test('missing required svg fails instead of skipping', () => {
  withTempRoot((root) => {
    writeFileSync(path.join(root, 'src', 'render_backend', 'backends.rs'), '');
    writeFileSync(
      path.join(root, 'src', 'render_backend', 'mod.rs'),
      'pub use backends::{NullBackend, TraceBackend};\n',
    );
    assert.throws(
      () => planAdapterDiff(root),
      /필수 어댑터 원본이 없습니다: svg/,
    );
  });
});

test('isExported accepts pub use and pub mod', () => {
  assert.equal(isExported('pub use png_adapter::PngBackend;', ADAPTERS[3]), true);
  assert.equal(isExported('pub mod png_adapter;', ADAPTERS[3]), true);
  assert.equal(isExported('pub use svg_adapter::SvgBackend;', ADAPTERS[3]), false);
});

test('rustflagsFor appends cfg flags', () => {
  assert.equal(rustflagsFor([]), '');
  assert.equal(
    rustflagsFor(['rhwp_has_png_backend'], ''),
    '--cfg rhwp_has_png_backend',
  );
  assert.equal(
    rustflagsFor(['rhwp_has_png_backend'], '-C debuginfo=1'),
    '-C debuginfo=1 --cfg rhwp_has_png_backend',
  );
});
