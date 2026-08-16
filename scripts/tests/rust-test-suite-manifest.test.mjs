import assert from 'node:assert/strict';
import test from 'node:test';

import {
  cargoTestArguments,
  nextestArguments,
  resolveCase,
  resolveCasePlan,
} from '../run-rust-test.mjs';
import {
  loadManifest,
  renderCargoTestBlock,
  renderHarness,
  validateRepository,
} from '../rust-test-suite-manifest.mjs';

test('전체 integration source와 generated target 계약이 일치한다', () => {
  const manifest = loadManifest();
  const validation = validateRepository();
  assert.deepEqual(validation.errors, []);
  assert.equal(validation.sourceCount, validation.caseModuleCount);
  assert.equal(validation.suiteCount, manifest.sharding.suiteCount);
  assert.equal(
    validation.integrationTargetCount,
    validation.suiteCount + validation.exceptionCount,
  );
  assert.ok(
    validation.integrationTargetCount <= manifest.sharding.maximumIntegrationTargets,
  );
  assert.equal(validation.minimumNextestCases, manifest.minimumNextestCases);
});

test('일반 case는 weighted suite로 해석한다', () => {
  const target = resolveCase('issue_1035_alignment');
  assert.match(target, /^regression_suite_\d{3}$/);
  const plan = resolveCasePlan('issue_1035_alignment');
  assert.equal(plan.grouped, true);
  assert.match(nextestArguments(plan).join(' '), /issue_1035_alignment::/);
  assert.match(cargoTestArguments(plan).at(-1), /issue_1035_alignment::/);
});

test('경로 의존 case는 기존 target 이름을 유지한다', () => {
  const plan = resolveCasePlan('issue_4100_chart_data_edit');
  assert.deepEqual(plan, {
    caseName: 'issue_4100_chart_data_edit',
    target: 'issue_4100_chart_data_edit',
    grouped: false,
  });
  assert.equal(nextestArguments(plan).includes('-E'), false);
});

test('harness와 Cargo target 렌더링은 입력 순서와 무관하다', () => {
  const manifest = loadManifest();
  const [suite] = Object.keys(manifest.suites).sort();
  const sources = manifest.suites[suite];
  assert.equal(
    renderHarness(suite, [...sources].reverse()),
    renderHarness(suite, sources),
  );
  assert.match(renderCargoTestBlock(manifest), /\[\[test\]\]/);
});
