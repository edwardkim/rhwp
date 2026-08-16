import assert from 'node:assert/strict';
import test from 'node:test';

import { resolveCase } from '../run-rust-test.mjs';
import {
  loadManifest,
  renderHarness,
  selectAutomaticSuite,
  validateRepository,
} from '../rust-test-suite-manifest.mjs';

test('suite manifest와 저장소 구조가 일치한다', () => {
  const validation = validateRepository();
  assert.deepEqual(validation.errors, []);
  assert.equal(validation.suiteCount, 1);
  assert.equal(validation.caseCount, 20);
});

test('pilot case를 단일 integration target으로 해석한다', () => {
  assert.equal(resolveCase('issue_1035_alignment'), 'issue_regression_pilot');
});

test('harness 생성 순서는 manifest 배열 순서와 무관하다', () => {
  const manifest = loadManifest();
  const cases = manifest.suites.issue_regression_pilot;
  assert.equal(
    renderHarness('issue_regression_pilot', [...cases].reverse()),
    renderHarness('issue_regression_pilot', cases),
  );
});

test('새 issue case는 현재 suite를 채운 뒤 다음 suite로 분리한다', () => {
  const manifest = loadManifest();
  assert.equal(selectAutomaticSuite(manifest), 'issue_regression_pilot');

  const fullManifest = structuredClone(manifest);
  fullManifest.suites.issue_regression_pilot = Array.from(
    { length: fullManifest.automaticIssueAssignment.maxCasesPerSuite },
    (_, index) => `issue_full_${index}`,
  );
  assert.equal(selectAutomaticSuite(fullManifest), 'issue_regression_002');
});
