import assert from 'node:assert/strict';
import { mkdtempSync, mkdirSync, writeFileSync, rmSync } from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import test from 'node:test';

import {
  OPTIONAL_CASES,
  REQUIRED_CASES,
  caseSourcePath,
  planPropRoundtrip,
} from '../run-prop-roundtrip.mjs';

function withTempRoot(run) {
  const root = mkdtempSync(path.join(os.tmpdir(), 'prop-roundtrip-'));
  try {
    mkdirSync(path.join(root, 'tests', 'cases'), { recursive: true });
    return run(root);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
}

test('required wiring case is prop_roundtrip_ci', () => {
  assert.deepEqual(REQUIRED_CASES, ['prop_roundtrip_ci']);
  assert.deepEqual(OPTIONAL_CASES, [
    'prop_edit_plan',
    'prop_hwpx_roundtrip',
    'prop_hwp5_roundtrip',
    'prop_m04f_catalog',
    'prop_m04f_skip',
    'prop_m04f_plans',
    'prop_m04f_exceptions',
    'prop_m04f_mutations',
  ]);
});

test('optional M04-2/M04-3 cases are skipped when source files are absent', () => {
  withTempRoot((root) => {
    writeFileSync(caseSourcePath('prop_roundtrip_ci', root), '');
    const plan = planPropRoundtrip(root);
    assert.deepEqual(plan.run, ['prop_roundtrip_ci']);
    assert.deepEqual(plan.skipped, OPTIONAL_CASES);
  });
});

test('optional cases are picked up when tests/cases sources exist', () => {
  withTempRoot((root) => {
    for (const caseName of [...REQUIRED_CASES, ...OPTIONAL_CASES]) {
      writeFileSync(caseSourcePath(caseName, root), '');
    }
    const plan = planPropRoundtrip(root);
    assert.deepEqual(plan.run, [
      'prop_roundtrip_ci',
      ...OPTIONAL_CASES,
    ]);
    assert.deepEqual(plan.skipped, []);
  });
});

test('hwpx only is picked up without requiring hwp5', () => {
  withTempRoot((root) => {
    writeFileSync(caseSourcePath('prop_roundtrip_ci', root), '');
    writeFileSync(caseSourcePath('prop_hwpx_roundtrip', root), '');
    const plan = planPropRoundtrip(root);
    assert.deepEqual(plan.run, ['prop_roundtrip_ci', 'prop_hwpx_roundtrip']);
    assert.deepEqual(
      plan.skipped,
      OPTIONAL_CASES.filter((name) => name !== 'prop_hwpx_roundtrip'),
    );
  });
});

test('missing required wiring case fails instead of skipping', () => {
  withTempRoot((root) => {
    assert.throws(
      () => planPropRoundtrip(root),
      /필수 property case 가 없습니다: prop_roundtrip_ci/,
    );
  });
});
