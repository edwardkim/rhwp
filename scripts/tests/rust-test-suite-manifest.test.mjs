import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
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
  deriveManifest,
  loadManifest,
  renderCargoTestBlock,
  renderHarness,
  validateAddedSourcePlacement,
  validateDerivedArtifactChanges,
  validateSourcePlacementAgainstBase,
  validateRepository,
} from '../rust-test-suite-manifest.mjs';

const ROOT = path.resolve(
  path.dirname(fileURLToPath(import.meta.url)),
  '../..',
);

function runGit(root, args) {
  execFileSync('git', args, { cwd: root, stdio: 'pipe' });
}

function writeRepositoryFixture(root) {
  const manifest = {
    version: 2,
    minimumNextestCases: 1,
    sharding: {
      suitePrefix: 'regression_suite_',
      suiteCount: 1,
      testAttributeWeight: 4096,
      maximumIntegrationTargets: 1,
    },
    nextestPriorities: [],
    sourceRoots: [{ path: 'tests/cases', recursive: true }],
    exceptions: [],
    suites: {
      regression_suite_001: ['tests/cases/existing.rs'],
    },
  };
  const policy = { ...manifest };
  delete policy.suites;
  mkdirSync(path.join(root, 'tests', 'cases'), { recursive: true });
  mkdirSync(path.join(root, 'tests', 'generated'), { recursive: true });
  mkdirSync(path.join(root, 'tests', 'suites'), { recursive: true });
  writeFileSync(
    path.join(root, 'tests', 'cases', 'existing.rs'),
    '#[test]\nfn existing_case() {}\n',
  );
  writeFileSync(
    path.join(root, 'tests', 'suites', 'manifest.json'),
    `${JSON.stringify(manifest, null, 2)}\n`,
  );
  writeFileSync(
    path.join(root, 'tests', 'suites', 'suite-policy.json'),
    `${JSON.stringify(policy, null, 2)}\n`,
  );
  writeFileSync(
    path.join(root, 'tests', 'generated', 'regression_suite_001.rs'),
    renderHarness('regression_suite_001', manifest.suites.regression_suite_001),
  );
  writeFileSync(
    path.join(root, 'Cargo.toml'),
    [
      '[package]',
      'name = "suite-fixture"',
      'version = "0.0.0"',
      'edition = "2021"',
      'autotests = false',
      '',
      renderCargoTestBlock(manifest),
      '',
    ].join('\n'),
  );
  runGit(root, ['init', '--quiet']);
  runGit(root, ['config', 'user.email', 'tests@example.invalid']);
  runGit(root, ['config', 'user.name', 'Rust suite test']);
  runGit(root, ['add', '.']);
  runGit(root, ['commit', '--quiet', '-m', 'base']);
  return manifest;
}

test('전체 integration source는 파생 산출물을 쓰지 않고 검증한다', () => {
  const policyPath = path.join(ROOT, 'tests', 'suites', 'suite-policy.json');
  const policyBefore = readFileSync(policyPath, 'utf8');
  const manifest = deriveManifest();
  const validation = validateRepository(ROOT, { derive: true });
  assert.deepEqual(validation.errors, []);
  assert.equal(readFileSync(policyPath, 'utf8'), policyBefore);
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

test('명시적으로 허용한 CLI module blocker는 generated suite에 배정한다', () => {
  for (const caseName of ['cli_json_contract', 'cli_catalog_contract']) {
    const plan = resolveCasePlan(caseName);
    assert.equal(plan.grouped, true, caseName);
    assert.match(plan.target, /^regression_suite_\d{3}$/);
  }
});

test('Native Skia 함수 게이트 case는 generated suite에서 feature별로 실행한다', () => {
  const plan = resolveCasePlan('issue_2225_missing_picture_placeholder');
  assert.equal(plan.grouped, true);
  assert.match(plan.target, /^regression_suite_\d{3}$/);
  assert.match(
    nextestArguments(plan).join(' '),
    /issue_2225_missing_picture_placeholder::/,
  );
});

test('CI slow archive case는 독립 target과 nextest 우선순위를 함께 유지한다', () => {
  const manifest = loadManifest();
  assert.deepEqual(manifest.nextestPriorities, [
    { case: 'overflow_cell_baseline', priority: 100 },
  ]);
  assert.equal(resolveCase('overflow_cell_baseline'), 'overflow_cell_baseline');
  const exception = manifest.exceptions.find(
    (entry) => entry.target === 'overflow_cell_baseline',
  );
  assert.deepEqual(exception, {
    target: 'overflow_cell_baseline',
    path: 'tests/overflow_cell_baseline.rs',
    manual: true,
    reasons: ['manual_isolation'],
  });
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

test('PR base에서 새로 추가된 integration source만 tests/cases 경로를 강제한다', () => {
  const errors = validateAddedSourcePlacement(
    [
      'tests/new_top_level.rs',
      'tests/cases/new_nested.rs',
      'mydocs/working/task.md',
    ],
    ['tests/existing.rs', 'tests/new_top_level.rs', 'tests/cases/new_nested.rs'],
  );
  assert.deepEqual(errors, [
    'PR base에 없는 신규 integration source는 tests/cases/ 아래에 두어야 합니다: tests/new_top_level.rs',
  ]);
});

test('PR은 파생 suite 산출물과 Cargo generated target 블록을 커밋할 수 없다', () => {
  const baseCargoToml = [
    '# BEGIN RHWP GENERATED TEST TARGETS',
    '[[test]]',
    'name = "regression_suite_001"',
    '# END RHWP GENERATED TEST TARGETS',
  ].join('\n');
  const headCargoToml = [
    '# BEGIN RHWP GENERATED TEST TARGETS',
    '[[test]]',
    'name = "regression_suite_002"',
    '# END RHWP GENERATED TEST TARGETS',
  ].join('\n');
  const errors = validateDerivedArtifactChanges(
    [
      'tests/cases/issue_5177_derived_suite_policy.rs',
      'tests/generated/regression_suite_012.rs',
      'tests/suites/manifest.json',
    ],
    { baseCargoToml, headCargoToml },
  );
  assert.equal(errors.length, 3);
  assert.match(errors[0], /tests\/generated\/regression_suite_012\.rs/);
  assert.match(errors[1], /tests\/suites\/manifest\.json/);
  assert.match(errors[2], /Cargo\.toml/);
});

test('PR base 검사도 커밋된 manifest 산출물을 실제 Git diff에서 거부한다', (t) => {
  const root = mkdtempSync(path.join(os.tmpdir(), 'rhwp-suite-base-diff-'));
  t.after(() => rmSync(root, { recursive: true, force: true }));
  const manifest = writeRepositoryFixture(root);
  writeFileSync(
    path.join(root, 'tests', 'suites', 'manifest.json'),
    `${JSON.stringify(manifest, null, 4)}\n`,
  );
  runGit(root, ['add', 'tests/suites/manifest.json']);
  runGit(root, ['commit', '--quiet', '-m', 'commit derived manifest']);

  const validation = validateRepository(root, { baseRef: 'HEAD~1' });
  assert.ok(
    validation.errors.some((error) => /PR에는 파생 Rust test 산출물을 커밋하지 마세요/.test(error)),
    validation.errors.join('\n'),
  );
});

test('PR base 검사는 manifest와 맞지 않는 Cargo generated target registry를 거부한다', (t) => {
  const root = mkdtempSync(path.join(os.tmpdir(), 'rhwp-suite-base-cargo-'));
  t.after(() => rmSync(root, { recursive: true, force: true }));
  writeRepositoryFixture(root);
  const cargoPath = path.join(root, 'Cargo.toml');
  writeFileSync(
    cargoPath,
    readFileSync(cargoPath, 'utf8').replace(
      'name = "regression_suite_001"',
      'name = "regression_suite_999"',
    ),
  );
  runGit(root, ['add', 'Cargo.toml']);
  runGit(root, ['commit', '--quiet', '-m', 'commit generated target']);

  const validation = validateRepository(root, { baseRef: 'HEAD~1' });
  assert.ok(
    validation.errors.some((error) => /Cargo\.toml generated test target block drift/.test(error)),
    validation.errors.join('\n'),
  );
});

test('CI가 삭제된 파생 산출물을 재생성해도 PR base 검사가 통과한다', (t) => {
  const root = mkdtempSync(path.join(os.tmpdir(), 'rhwp-suite-regenerated-output-'));
  t.after(() => rmSync(root, { recursive: true, force: true }));
  const manifest = writeRepositoryFixture(root);
  const generatedPath = path.join(
    root,
    'tests',
    'generated',
    'regression_suite_001.rs',
  );
  const manifestPath = path.join(root, 'tests', 'suites', 'manifest.json');

  runGit(root, ['rm', generatedPath, manifestPath]);
  runGit(root, ['commit', '--quiet', '-m', 'remove derived outputs']);

  // CI의 --prepare가 하는 일: HEAD에는 삭제로 남기되 작업 폴더에는 다시 생성한다.
  mkdirSync(path.dirname(generatedPath), { recursive: true });
  writeFileSync(generatedPath, renderHarness('regression_suite_001', manifest.suites.regression_suite_001));
  writeFileSync(manifestPath, `${JSON.stringify(manifest, null, 2)}\n`);

  const validation = validateRepository(root, { baseRef: 'HEAD~1' });
  assert.deepEqual(validation.errors, []);
});

test('기여자 가이드는 원본-only 제출을 안내한다', () => {
  const guide = readFileSync(path.join(ROOT, 'CONTRIBUTING.md'), 'utf8');
  for (const expected of [/tests\/cases/, /PR review|CI/, /tests\/generated/]) {
    assert.match(guide, expected);
  }
});

test('검토·개발 가이드는 파생 suite 준비 경로를 안내한다', () => {
  const reviewGuides = [
    'mydocs/manual/pr_review/local_validation.md',
    'mydocs/manual/dev_environment_guide.md',
  ].map((relativePath) => readFileSync(path.join(ROOT, relativePath), 'utf8'));
  for (const guide of reviewGuides) {
    assert.match(guide, /tests\/cases/);
    assert.match(guide, /PR review|PR 검토|CI/);
    assert.match(guide, /rust-test-suite-manifest\.mjs --prepare/);
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
    /rust-test-suite-manifest\.mjs --prepare/,
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

test('CI lint checkout은 PR base 3-way diff를 위해 전체 Git 계보를 가져온다', () => {
  const workflow = readFileSync(path.join(ROOT, '.github/workflows/ci.yml'), 'utf8');
  assert.match(
    workflow,
    /\n  lint:\n[\s\S]*?fetch-depth: 0\n/,
  );
  assert.doesNotMatch(
    workflow,
    /git fetch --no-tags --depth=1 origin "\$RHWP_TEST_POLICY_BASE_REF"/,
  );
});
