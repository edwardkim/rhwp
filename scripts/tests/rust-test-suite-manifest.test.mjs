import assert from 'node:assert/strict';
import {
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';

import {
  cargoTestArguments,
  nextestArguments,
  resolveCase,
  resolveCasePlan,
} from '../run-rust-test.mjs';
import {
  assignSources,
  loadManifest,
  renderCargoTestBlock,
  renderHarness,
  validateSourcePlacementAgainstBase,
  validateRepository,
} from '../rust-test-suite-manifest.mjs';

const ROOT = path.resolve(
  path.dirname(fileURLToPath(import.meta.url)),
  '../..',
);

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

test('nextest 우선순위 case는 generated suite에서도 유지된다', () => {
  const manifest = loadManifest();
  assert.deepEqual(manifest.nextestPriorities, [
    { case: 'overflow_cell_baseline', priority: 100 },
  ]);
  assert.match(resolveCase('overflow_cell_baseline'), /^regression_suite_\d{3}$/);
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

test('신규 source는 선택된 suite에 정확히 한 번만 배정한다', (t) => {
  const root = mkdtempSync(path.join(os.tmpdir(), 'rhwp-suite-assign-'));
  t.after(() => rmSync(root, { recursive: true, force: true }));
  mkdirSync(path.join(root, 'tests', 'suites'), { recursive: true });
  writeFileSync(path.join(root, 'tests', 'first.rs'), '#[test] fn first() {}');
  writeFileSync(path.join(root, 'tests', 'second.rs'), '#[test] fn second() {}');
  const manifest = {
    version: 2,
    minimumNextestCases: 1,
    sharding: {
      suitePrefix: 'regression_suite_',
      suiteCount: 2,
      testAttributeWeight: 4096,
      maximumIntegrationTargets: 2,
    },
    nextestPriorities: [],
    sourceRoots: [{ path: 'tests', recursive: false }],
    exceptions: [],
    suites: {
      regression_suite_001: [],
      regression_suite_002: [],
    },
  };
  assignSources(manifest, ['tests/first.rs', 'tests/second.rs'], root);
  const assigned = Object.values(manifest.suites).flat();
  assert.equal(assigned.filter((source) => source === 'tests/first.rs').length, 1);
  assert.equal(assigned.filter((source) => source === 'tests/second.rs').length, 1);
});

test('PR base에 없던 integration source는 tests/cases 밖에서 거부한다', () => {
  const baseManifest = {
    exceptions: [{ path: 'tests/legacy.rs' }],
    suites: { regression_suite_001: ['tests/cases/existing.rs'] },
  };
  const errors = validateSourcePlacementAgainstBase(
    [
      'tests/legacy.rs',
      'tests/new_top_level.rs',
      'tests/cases/new_nested.rs',
    ],
    baseManifest,
  );
  assert.deepEqual(errors, [
    'PR base에 없는 신규 integration source는 tests/cases/ 아래에 두어야 합니다: tests/new_top_level.rs',
  ]);
});

test('개발자 가이드가 자동 sharding 진입점을 안내한다', () => {
  const guides = [
    'CONTRIBUTING.md',
    'mydocs/manual/pr_review/local_validation.md',
    'mydocs/manual/dev_environment_guide.md',
  ].map((relativePath) =>
    readFileSync(path.join(ROOT, relativePath), 'utf8'),
  );
  for (const guide of guides) {
    assert.match(guide, /rust-test-suite-manifest\.mjs --generate/);
    assert.match(guide, /rust-test-suite-manifest\.mjs --sync/);
    assert.match(guide, /tests\/generated/);
  }
});

test('CI가 PR base를 integration과 source unit 정책 검사에 전달한다', () => {
  const workflow = readFileSync(path.join(ROOT, '.github/workflows/ci.yml'), 'utf8');
  assert.match(
    workflow,
    /RHWP_TEST_POLICY_BASE_REF: \$\{\{ github\.event\.pull_request\.base\.sha \|\| '' \}\}/,
  );
  assert.match(
    workflow,
    /rust-test-suite-manifest\.mjs --check "\$\{base_args\[@\]\}"/,
  );
  assert.match(
    workflow,
    /rust-unit-test-tiers\.mjs --check "\$\{base_args\[@\]\}"/,
  );
});
