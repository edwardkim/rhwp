#!/usr/bin/env node

import {
  existsSync,
  readFileSync,
  readdirSync,
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
    manifest.budgets.automaticIntegrationTargets,
    'budgets.automaticIntegrationTargets',
  );
  assertBudget(
    manifest.budgets.standaloneIssueTargets,
    'budgets.standaloneIssueTargets',
  );
  assertRecord(manifest.suites, 'suites');

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

  if (targetCount > manifest.budgets.automaticIntegrationTargets) {
    errors.push(
      `integration target 예산 초과: ${targetCount} > ` +
        manifest.budgets.automaticIntegrationTargets,
    );
  }
  if (issueTargetCount > manifest.budgets.standaloneIssueTargets) {
    errors.push(
      `standalone issue target 예산 초과: ${issueTargetCount} > ` +
        manifest.budgets.standaloneIssueTargets,
    );
  }

  return {
    errors,
    suiteCount: Object.keys(manifest.suites).length,
    caseCount: caseIndex.size,
    automaticIntegrationTargets: targetCount,
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
      `${validation.automaticIntegrationTargets} integration targets, ` +
      `${validation.standaloneIssueTargets} standalone issue targets\n`,
  );
}

if (process.argv[1] && path.resolve(process.argv[1]) === SCRIPT_PATH) {
  try {
    const command = process.argv[2];
    if (command === '--generate') {
      generateHarnesses();
      printValidation(validateRepository());
    } else if (command === '--check') {
      printValidation(validateRepository());
    } else {
      process.stderr.write(
        '사용법: node scripts/rust-test-suite-manifest.mjs --check|--generate\n',
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
