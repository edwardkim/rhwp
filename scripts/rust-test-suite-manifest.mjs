#!/usr/bin/env node

import { spawnSync } from 'node:child_process';
import {
  existsSync,
  mkdirSync,
  readFileSync,
  readdirSync,
  renameSync,
  statSync,
  writeFileSync,
} from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const SCRIPT_PATH = fileURLToPath(import.meta.url);
export const ROOT = path.resolve(path.dirname(SCRIPT_PATH), '..');
export const MANIFEST_RELATIVE_PATH = 'tests/suites/manifest.json';
const RUST_MODULE_NAME = /^[a-z][a-z0-9_]*$/;

function assertRecord(value, label) {
  if (value === null || typeof value !== 'object' || Array.isArray(value)) {
    throw new Error(`${label}은 객체여야 합니다.`);
  }
}

function assertBudget(value, label) {
  if (!Number.isInteger(value) || value < 0) {
    throw new Error(`${label}은 0 이상의 정수여야 합니다.`);
  }
}

export function loadManifest(root = ROOT) {
  const manifestPath = path.join(root, MANIFEST_RELATIVE_PATH);
  const manifest = JSON.parse(readFileSync(manifestPath, 'utf8'));

  if (manifest.version !== 1) {
    throw new Error(`지원하지 않는 manifest version: ${manifest.version}`);
  }
  assertRecord(manifest.budgets, 'budgets');
  assertBudget(
    manifest.budgets.nonAutomaticIntegrationTargets,
    'budgets.nonAutomaticIntegrationTargets',
  );
  assertBudget(
    manifest.budgets.standaloneIssueTargets,
    'budgets.standaloneIssueTargets',
  );
  assertRecord(manifest.suites, 'suites');
  assertRecord(manifest.automaticIssueAssignment, 'automaticIssueAssignment');

  const { firstSuite, suitePrefix, maxCasesPerSuite } =
    manifest.automaticIssueAssignment;
  if (typeof firstSuite !== 'string' || !RUST_MODULE_NAME.test(firstSuite)) {
    throw new Error(`잘못된 automaticIssueAssignment.firstSuite: ${firstSuite}`);
  }
  if (
    typeof suitePrefix !== 'string' ||
    !RUST_MODULE_NAME.test(`${suitePrefix}002`)
  ) {
    throw new Error(`잘못된 automaticIssueAssignment.suitePrefix: ${suitePrefix}`);
  }
  if (!Number.isInteger(maxCasesPerSuite) || maxCasesPerSuite < 2) {
    throw new Error(
      'automaticIssueAssignment.maxCasesPerSuite는 2 이상의 정수여야 합니다.',
    );
  }
  if (!Array.isArray(manifest.suites[firstSuite])) {
    throw new Error(`자동 배정 첫 suite가 manifest에 없습니다: ${firstSuite}`);
  }

  return manifest;
}

export function buildCaseIndex(manifest) {
  const index = new Map();

  for (const [suite, cases] of Object.entries(manifest.suites).sort()) {
    if (!RUST_MODULE_NAME.test(suite)) {
      throw new Error(`잘못된 suite 이름: ${suite}`);
    }
    if (!Array.isArray(cases) || cases.length === 0) {
      throw new Error(`${suite} suite에는 case가 하나 이상 있어야 합니다.`);
    }

    for (const caseName of cases) {
      if (typeof caseName !== 'string' || !RUST_MODULE_NAME.test(caseName)) {
        throw new Error(`잘못된 case 이름: ${String(caseName)}`);
      }
      if (index.has(caseName)) {
        throw new Error(
          `중복 case: ${caseName} (${index.get(caseName)}, ${suite})`,
        );
      }
      index.set(caseName, suite);
    }
  }

  return index;
}

function automaticSuiteOrdinal(suite, assignment) {
  if (suite === assignment.firstSuite) {
    return 1;
  }
  if (!suite.startsWith(assignment.suitePrefix)) {
    return null;
  }

  const suffix = suite.slice(assignment.suitePrefix.length);
  return /^\d{3}$/.test(suffix) ? Number.parseInt(suffix, 10) : null;
}

function automaticSuiteNames(manifest) {
  const assignment = manifest.automaticIssueAssignment;
  return Object.keys(manifest.suites)
    .map((suite) => [suite, automaticSuiteOrdinal(suite, assignment)])
    .filter(([, ordinal]) => ordinal !== null)
    .sort((left, right) => left[1] - right[1])
    .map(([suite]) => suite);
}

export function selectAutomaticSuite(manifest) {
  const assignment = manifest.automaticIssueAssignment;
  const suites = automaticSuiteNames(manifest);

  for (const suite of suites) {
    if (manifest.suites[suite].length < assignment.maxCasesPerSuite) {
      return suite;
    }
  }

  const usedOrdinals = new Set(
    suites.map((suite) => automaticSuiteOrdinal(suite, assignment)),
  );
  let ordinal = 2;
  while (usedOrdinals.has(ordinal)) {
    ordinal += 1;
  }
  return `${assignment.suitePrefix}${String(ordinal).padStart(3, '0')}`;
}

export function renderHarness(suite, cases) {
  const modules = [...cases]
    .sort((left, right) => left.localeCompare(right))
    .map(
      (caseName) =>
        `#[path = "suites/${suite}/${caseName}.rs"]\nmod ${caseName};`,
    )
    .join('\n\n');

  return [
    '//! `tests/suites/manifest.json`에서 자동 생성된 integration test harness다.',
    '//! 직접 수정하지 말고 manifest를 갱신한 뒤 생성기를 실행한다.',
    '//!',
    '//! Issue 회귀 테스트의 링크 단위를 줄이는 통합 suite.',
    '',
    modules,
    '',
  ].join('\n');
}

function rustFiles(directory) {
  if (!existsSync(directory)) {
    return [];
  }
  return readdirSync(directory, { withFileTypes: true })
    .filter((entry) => entry.isFile() && entry.name.endsWith('.rs'))
    .map((entry) => entry.name.slice(0, -3))
    .sort((left, right) => left.localeCompare(right));
}

function automaticIntegrationTargets(testsDirectory) {
  return readdirSync(testsDirectory, { withFileTypes: true }).filter((entry) => {
    if (entry.isFile()) {
      return entry.name.endsWith('.rs');
    }
    return (
      entry.isDirectory() &&
      existsSync(path.join(testsDirectory, entry.name, 'main.rs'))
    );
  }).length;
}

function writeManifest(manifest, root) {
  manifest.suites = Object.fromEntries(
    Object.entries(manifest.suites)
      .sort(([left], [right]) => left.localeCompare(right))
      .map(([suite, cases]) => [
        suite,
        [...cases].sort((left, right) => left.localeCompare(right)),
      ]),
  );
  writeFileSync(
    path.join(root, MANIFEST_RELATIVE_PATH),
    `${JSON.stringify(manifest, null, 2)}\n`,
    'utf8',
  );
}

function gitPaths(arguments_, root) {
  const result = spawnSync('git', arguments_, {
    cwd: root,
    encoding: 'utf8',
    shell: false,
  });
  if (result.error) {
    throw result.error;
  }
  if (result.status !== 0) {
    throw new Error(result.stderr.trim() || `git ${arguments_.join(' ')} 실패`);
  }
  return result.stdout.split('\0').filter(Boolean);
}

export function discoverNewIssueTests(root = ROOT) {
  const candidates = new Set([
    ...gitPaths(
      ['diff', '--name-only', '--diff-filter=A', '-z', 'HEAD', '--', 'tests'],
      root,
    ),
    ...gitPaths(
      ['ls-files', '--others', '--exclude-standard', '-z', '--', 'tests'],
      root,
    ),
  ]);

  return [...candidates]
    .filter((relativePath) => {
      const normalized = relativePath.split(path.sep).join('/');
      return /^tests\/issue_[a-z0-9_]+\.rs$/.test(normalized);
    })
    .sort((left, right) => left.localeCompare(right));
}

export function adoptIssueTests(inputPaths, root = ROOT) {
  const manifest = loadManifest(root);
  const caseIndex = buildCaseIndex(manifest);
  const testsDirectory = path.join(root, 'tests');
  const uniquePaths = [...new Set(inputPaths)].sort((left, right) =>
    left.localeCompare(right),
  );
  const plans = [];

  for (const inputPath of uniquePaths) {
    const sourcePath = path.isAbsolute(inputPath)
      ? path.normalize(inputPath)
      : path.resolve(root, inputPath);
    if (path.dirname(sourcePath) !== testsDirectory) {
      throw new Error(`top-level tests 파일만 자동 배정할 수 있습니다: ${inputPath}`);
    }
    if (!existsSync(sourcePath) || !statSync(sourcePath).isFile()) {
      throw new Error(`자동 배정할 파일이 없습니다: ${inputPath}`);
    }

    const extension = path.extname(sourcePath);
    const caseName = path.basename(sourcePath, extension);
    if (extension !== '.rs' || !/^issue_[a-z0-9_]+$/.test(caseName)) {
      throw new Error(`잘못된 issue test 파일명: ${inputPath}`);
    }
    if (caseIndex.has(caseName)) {
      throw new Error(`이미 suite에 등록된 case입니다: ${caseName}`);
    }

    const suite = selectAutomaticSuite(manifest);
    manifest.suites[suite] ??= [];
    const targetPath = path.join(
      testsDirectory,
      'suites',
      suite,
      `${caseName}.rs`,
    );
    if (existsSync(targetPath)) {
      throw new Error(`자동 배정 대상 파일이 이미 있습니다: ${targetPath}`);
    }

    manifest.suites[suite].push(caseName);
    caseIndex.set(caseName, suite);
    plans.push({ caseName, sourcePath, suite, targetPath });
  }

  for (const plan of plans) {
    mkdirSync(path.dirname(plan.targetPath), { recursive: true });
    renameSync(plan.sourcePath, plan.targetPath);
    process.stdout.write(
      `[RustTestSuite] 자동 배정: ${plan.caseName} -> ${plan.suite}\n`,
    );
  }
  if (plans.length > 0) {
    writeManifest(manifest, root);
  }

  return plans;
}

export function validateRepository(
  root = ROOT,
  { checkGenerated = true } = {},
) {
  const errors = [];
  let manifest;
  let caseIndex;

  try {
    manifest = loadManifest(root);
    caseIndex = buildCaseIndex(manifest);
  } catch (error) {
    return {
      errors: [error instanceof Error ? error.message : String(error)],
      suiteCount: 0,
      caseCount: 0,
      automaticIntegrationTargets: 0,
      standaloneIssueTargets: 0,
    };
  }

  const testsDirectory = path.join(root, 'tests');
  const suitesDirectory = path.join(testsDirectory, 'suites');
  const declaredSuites = new Set(Object.keys(manifest.suites));
  const autoSuites = automaticSuiteNames(manifest);
  const automaticCaseCount = autoSuites.reduce(
    (count, suite) => count + manifest.suites[suite].length,
    0,
  );
  const minimumAutomaticSuiteCount = Math.ceil(
    automaticCaseCount / manifest.automaticIssueAssignment.maxCasesPerSuite,
  );
  const physicalSuites = readdirSync(suitesDirectory, { withFileTypes: true })
    .filter((entry) => entry.isDirectory())
    .map((entry) => entry.name)
    .sort((left, right) => left.localeCompare(right));

  for (const suite of physicalSuites) {
    if (!declaredSuites.has(suite)) {
      errors.push(`manifest에 없는 suite 디렉터리: ${suite}`);
    }
  }

  for (const [suite, cases] of Object.entries(manifest.suites).sort()) {
    if (
      autoSuites.includes(suite) &&
      cases.length > manifest.automaticIssueAssignment.maxCasesPerSuite
    ) {
      errors.push(
        `${suite} case 상한 초과: ${cases.length} > ` +
          manifest.automaticIssueAssignment.maxCasesPerSuite,
      );
    }
    const suiteDirectory = path.join(suitesDirectory, suite);
    const declaredCases = [...cases].sort((left, right) =>
      left.localeCompare(right),
    );
    const physicalCases = rustFiles(suiteDirectory);

    if (!existsSync(suiteDirectory)) {
      errors.push(`suite 디렉터리가 없습니다: ${suite}`);
      continue;
    }
    for (const caseName of declaredCases) {
      if (!physicalCases.includes(caseName)) {
        errors.push(`manifest case 파일이 없습니다: ${suite}/${caseName}.rs`);
      }
    }
    for (const caseName of physicalCases) {
      if (!declaredCases.includes(caseName)) {
        errors.push(`manifest에 없는 case 파일: ${suite}/${caseName}.rs`);
      }
    }

    if (checkGenerated) {
      const harnessPath = path.join(testsDirectory, `${suite}.rs`);
      const expected = renderHarness(suite, cases);
      const actual = existsSync(harnessPath)
        ? readFileSync(harnessPath, 'utf8')
        : null;
      if (actual !== expected) {
        errors.push(
          `생성 harness가 manifest와 다릅니다: tests/${suite}.rs ` +
            '(node scripts/rust-test-suite-manifest.mjs --generate)',
        );
      }
    }
  }

  const targetCount = automaticIntegrationTargets(testsDirectory);
  const integrationTargetBudget =
    manifest.budgets.nonAutomaticIntegrationTargets + autoSuites.length;
  const issueTargetCount = readdirSync(testsDirectory, { withFileTypes: true }).filter(
    (entry) => {
      const targetName = entry.name.endsWith('.rs')
        ? entry.name.slice(0, -3)
        : entry.name;
      return (
        entry.isFile() &&
        entry.name.startsWith('issue_') &&
        entry.name.endsWith('.rs') &&
        !declaredSuites.has(targetName)
      );
    },
  ).length;

  if (autoSuites.length > minimumAutomaticSuiteCount) {
    errors.push(
      `자동 suite가 불필요하게 분산됐습니다: ${autoSuites.length} > ` +
        minimumAutomaticSuiteCount,
    );
  }
  if (targetCount > integrationTargetBudget) {
    errors.push(
      `integration target 예산 초과: ${targetCount} > ` +
        integrationTargetBudget,
    );
  }
  if (issueTargetCount > manifest.budgets.standaloneIssueTargets) {
    errors.push(
      `standalone issue target 예산 초과: ${issueTargetCount} > ` +
        manifest.budgets.standaloneIssueTargets +
        ' (node scripts/rust-test-suite-manifest.mjs --adopt-new)',
    );
  }

  return {
    errors,
    suiteCount: Object.keys(manifest.suites).length,
    caseCount: caseIndex.size,
    automaticSuiteCount: autoSuites.length,
    automaticIntegrationTargets: targetCount,
    integrationTargetBudget,
    standaloneIssueTargets: issueTargetCount,
  };
}

export function generateHarnesses(root = ROOT) {
  const validation = validateRepository(root, { checkGenerated: false });
  if (validation.errors.length > 0) {
    throw new Error(validation.errors.join('\n'));
  }

  const manifest = loadManifest(root);
  for (const [suite, cases] of Object.entries(manifest.suites).sort()) {
    const harnessPath = path.join(root, 'tests', `${suite}.rs`);
    writeFileSync(harnessPath, renderHarness(suite, cases), 'utf8');
    process.stdout.write(`[RustTestSuite] 생성: tests/${suite}.rs\n`);
  }
}

function adoptNewIssueTests(root = ROOT) {
  const newIssueTests = discoverNewIssueTests(root);
  if (newIssueTests.length === 0) {
    process.stdout.write('[RustTestSuite] 새 top-level issue case 없음\n');
    return [];
  }
  return adoptIssueTests(newIssueTests, root);
}

function printValidation(validation) {
  if (validation.errors.length > 0) {
    for (const error of validation.errors) {
      process.stderr.write(`[RustTestSuite] 오류: ${error}\n`);
    }
    process.exitCode = 1;
    return;
  }

  process.stdout.write(
    '[RustTestSuite] 확인 완료: ' +
      `${validation.suiteCount} suite, ${validation.caseCount} cases, ` +
      `${validation.automaticIntegrationTargets}/${validation.integrationTargetBudget} ` +
      'integration targets, ' +
      `${validation.standaloneIssueTargets} standalone issue targets\n`,
  );
}

if (process.argv[1] && path.resolve(process.argv[1]) === SCRIPT_PATH) {
  try {
    const command = process.argv[2];
    if (command === '--generate') {
      adoptNewIssueTests();
      generateHarnesses();
      printValidation(validateRepository());
    } else if (command === '--adopt-new') {
      adoptNewIssueTests();
      generateHarnesses();
      printValidation(validateRepository());
    } else if (command === '--adopt') {
      const inputPaths = process.argv.slice(3);
      if (inputPaths.length === 0) {
        throw new Error('--adopt에는 하나 이상의 top-level issue 파일이 필요합니다.');
      }
      adoptIssueTests(inputPaths);
      generateHarnesses();
      printValidation(validateRepository());
    } else if (command === '--check') {
      printValidation(validateRepository());
    } else {
      process.stderr.write(
        '사용법: node scripts/rust-test-suite-manifest.mjs ' +
          '--check|--generate|--adopt-new|--adopt <파일...>\n',
      );
      process.exitCode = 2;
    }
  } catch (error) {
    process.stderr.write(
      `[RustTestSuite] 오류: ${error instanceof Error ? error.message : String(error)}\n`,
    );
    process.exitCode = 1;
  }
}
