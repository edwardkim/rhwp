#!/usr/bin/env node

import {
  existsSync,
  mkdirSync,
  readFileSync,
  readdirSync,
  rmSync,
  statSync,
  writeFileSync,
} from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const SCRIPT_PATH = fileURLToPath(import.meta.url);
export const ROOT = path.resolve(path.dirname(SCRIPT_PATH), '..');
export const MANIFEST_RELATIVE_PATH = 'tests/suites/manifest.json';
const CARGO_MANIFEST_RELATIVE_PATH = 'Cargo.toml';
const GENERATED_TEST_DIRECTORY = 'tests/generated';
const CARGO_BLOCK_START = '# BEGIN RHWP GENERATED TEST TARGETS';
const CARGO_BLOCK_END = '# END RHWP GENERATED TEST TARGETS';
const RUST_MODULE_NAME = /^[a-z][a-z0-9_]*$/;

function assertRecord(value, label) {
  if (value === null || typeof value !== 'object' || Array.isArray(value)) {
    throw new Error(`${label}은 객체여야 합니다.`);
  }
}

function normalizeRelativePath(value, label) {
  if (typeof value !== 'string' || value.length === 0) {
    throw new Error(`${label}은 비어 있지 않은 문자열이어야 합니다.`);
  }
  const normalized = value.replaceAll('\\', '/');
  if (
    path.posix.isAbsolute(normalized) ||
    normalized === '..' ||
    normalized.startsWith('../') ||
    normalized.includes('/../')
  ) {
    throw new Error(`${label}은 저장소 내부 상대 경로여야 합니다: ${value}`);
  }
  return path.posix.normalize(normalized);
}

function assertPositiveInteger(value, label) {
  if (!Number.isInteger(value) || value <= 0) {
    throw new Error(`${label}은 양의 정수여야 합니다.`);
  }
}

export function loadManifest(root = ROOT) {
  const manifestPath = path.join(root, MANIFEST_RELATIVE_PATH);
  const manifest = JSON.parse(readFileSync(manifestPath, 'utf8'));

  if (manifest.version !== 2) {
    throw new Error(`지원하지 않는 manifest version: ${manifest.version}`);
  }
  assertPositiveInteger(manifest.minimumNextestCases, 'minimumNextestCases');
  assertRecord(manifest.sharding, 'sharding');
  const {
    suitePrefix,
    suiteCount,
    testAttributeWeight,
    maximumIntegrationTargets,
  } = manifest.sharding;
  if (
    typeof suitePrefix !== 'string' ||
    !RUST_MODULE_NAME.test(`${suitePrefix}001`)
  ) {
    throw new Error(`잘못된 sharding.suitePrefix: ${suitePrefix}`);
  }
  assertPositiveInteger(suiteCount, 'sharding.suiteCount');
  assertPositiveInteger(testAttributeWeight, 'sharding.testAttributeWeight');
  assertPositiveInteger(
    maximumIntegrationTargets,
    'sharding.maximumIntegrationTargets',
  );
  if (maximumIntegrationTargets < suiteCount) {
    throw new Error('maximumIntegrationTargets는 suiteCount 이상이어야 합니다.');
  }
  if (!Array.isArray(manifest.nextestPriorities)) {
    throw new Error('nextestPriorities는 배열이어야 합니다.');
  }
  for (const [index, entry] of manifest.nextestPriorities.entries()) {
    assertRecord(entry, `nextestPriorities[${index}]`);
    if (!RUST_MODULE_NAME.test(entry.case)) {
      throw new Error(`잘못된 nextest priority case: ${entry.case}`);
    }
    assertPositiveInteger(entry.priority, `nextestPriorities[${index}].priority`);
  }

  if (!Array.isArray(manifest.sourceRoots) || manifest.sourceRoots.length === 0) {
    throw new Error('sourceRoots에는 하나 이상의 검색 루트가 필요합니다.');
  }
  for (const [index, sourceRoot] of manifest.sourceRoots.entries()) {
    assertRecord(sourceRoot, `sourceRoots[${index}]`);
    sourceRoot.path = normalizeRelativePath(
      sourceRoot.path,
      `sourceRoots[${index}].path`,
    );
    if (typeof sourceRoot.recursive !== 'boolean') {
      throw new Error(`sourceRoots[${index}].recursive는 boolean이어야 합니다.`);
    }
  }

  if (!Array.isArray(manifest.exceptions)) {
    throw new Error('exceptions는 배열이어야 합니다.');
  }
  for (const [index, exception] of manifest.exceptions.entries()) {
    assertRecord(exception, `exceptions[${index}]`);
    if (!RUST_MODULE_NAME.test(exception.target)) {
      throw new Error(`잘못된 exception target: ${exception.target}`);
    }
    exception.path = normalizeRelativePath(
      exception.path,
      `exceptions[${index}].path`,
    );
    if (!exception.path.endsWith('.rs')) {
      throw new Error(`exception은 Rust 파일이어야 합니다: ${exception.path}`);
    }
    if (exception.manual !== undefined && typeof exception.manual !== 'boolean') {
      throw new Error(`exceptions[${index}].manual은 boolean이어야 합니다.`);
    }
    if (
      exception.reasons !== undefined &&
      (!Array.isArray(exception.reasons) ||
        exception.reasons.some((reason) => typeof reason !== 'string'))
    ) {
      throw new Error(`exceptions[${index}].reasons는 문자열 배열이어야 합니다.`);
    }
  }

  assertRecord(manifest.suites, 'suites');
  for (const [suite, sources] of Object.entries(manifest.suites)) {
    if (!RUST_MODULE_NAME.test(suite)) {
      throw new Error(`잘못된 suite 이름: ${suite}`);
    }
    if (!Array.isArray(sources)) {
      throw new Error(`${suite} suite는 source 경로 배열이어야 합니다.`);
    }
    manifest.suites[suite] = sources.map((source, index) => {
      const normalized = normalizeRelativePath(source, `${suite}[${index}]`);
      if (!normalized.endsWith('.rs')) {
        throw new Error(`suite source는 Rust 파일이어야 합니다: ${normalized}`);
      }
      return normalized;
    });
  }

  return manifest;
}

function suiteName(manifest, index) {
  return `${manifest.sharding.suitePrefix}${String(index + 1).padStart(3, '0')}`;
}

function caseNameForSource(source) {
  const caseName = path.posix.basename(source, '.rs');
  if (!RUST_MODULE_NAME.test(caseName)) {
    throw new Error(`Rust module로 사용할 수 없는 test 파일명: ${source}`);
  }
  return caseName;
}

function walkRustFiles(directory, recursive) {
  return readdirSync(directory, { withFileTypes: true }).flatMap((entry) => {
    const entryPath = path.join(directory, entry.name);
    if (entry.isDirectory()) {
      return recursive ? walkRustFiles(entryPath, true) : [];
    }
    return entry.isFile() && entry.name.endsWith('.rs') ? [entryPath] : [];
  });
}

export function discoverSourceFiles(manifest, root = ROOT) {
  const sources = new Set();
  for (const sourceRoot of manifest.sourceRoots) {
    const directory = path.join(root, sourceRoot.path);
    if (!existsSync(directory) || !statSync(directory).isDirectory()) {
      throw new Error(`test source root가 없습니다: ${sourceRoot.path}`);
    }
    for (const file of walkRustFiles(directory, sourceRoot.recursive)) {
      const relative = path.relative(root, file).split(path.sep).join('/');
      if (!relative.startsWith(`${GENERATED_TEST_DIRECTORY}/`)) {
        sources.add(relative);
      }
    }
  }
  return [...sources].sort((left, right) => left.localeCompare(right));
}

const TEST_ATTRIBUTE =
  /^\s*#\s*\[\s*(?:(?:tokio|async_std|rstest|test_case|wasm_bindgen_test)::)?(?:test|rstest|test_case|wasm_bindgen_test)\b[^\]]*\]/gm;
const CASE_ATTRIBUTE = /^\s*#\s*\[\s*case(?:\s*\([^\]]*\))?\s*\]/gm;
const MODULE_BLOCKERS = [
  ['root_mod', /^\s*mod\s+[A-Za-z_][A-Za-z0-9_]*\s*;/m],
  ['crate_path', /\bcrate::/],
  ['path_attr', /#\s*\[\s*path\s*=/],
  ['macro_export', /#\s*\[\s*macro_export\s*\]/],
  ['macro_use_extern', /#\s*\[\s*macro_use\s*\]\s*extern\s+crate/],
  ['link_symbol', /#\s*\[\s*(?:no_mangle|export_name|link_section)\b/],
  [
    'crate_only_inner_attr',
    /^\s*#!\s*\[\s*(?:feature|no_main|no_std|recursion_limit|type_length_limit)\b/m,
  ],
  ['main_fn', /^\s*(?:pub\s+)?fn\s+main\s*\(/m],
  ['process_environment', /\b(?:set_var|remove_var|set_current_dir)\s*\(/],
  [
    'global_init',
    /#\s*\[\s*(?:ctor|global_allocator)\b|\bctor::ctor\b/,
  ],
];

export function sourceMetrics(source, manifest, root = ROOT) {
  const text = readFileSync(path.join(root, source), 'utf8');
  const testAttributes = (text.match(TEST_ATTRIBUTE) ?? []).length;
  const caseAttributes = (text.match(CASE_ATTRIBUTE) ?? []).length;
  const staticTests = testAttributes + caseAttributes;
  const bytes = Buffer.byteLength(text);
  const blockers = MODULE_BLOCKERS.filter(([, pattern]) => pattern.test(text)).map(
    ([name]) => name,
  );
  return {
    source,
    caseName: caseNameForSource(source),
    staticTests,
    bytes,
    weight:
      bytes + Math.max(1, staticTests) * manifest.sharding.testAttributeWeight,
    blockers,
  };
}

export function buildCaseIndex(manifest) {
  const index = new Map();
  const register = (source, target) => {
    const caseName = caseNameForSource(source);
    if (index.has(caseName)) {
      throw new Error(
        `중복 test case module: ${caseName} (${index.get(caseName)}, ${target})`,
      );
    }
    index.set(caseName, target);
  };

  for (const exception of manifest.exceptions) {
    register(exception.path, exception.target);
  }
  for (const [suite, sources] of Object.entries(manifest.suites).sort()) {
    for (const source of sources) {
      register(source, suite);
    }
  }
  return index;
}

function sortedManifest(manifest) {
  manifest.exceptions.sort((left, right) => left.target.localeCompare(right.target));
  manifest.suites = Object.fromEntries(
    Object.entries(manifest.suites)
      .sort(([left], [right]) => left.localeCompare(right))
      .map(([suite, sources]) => [
        suite,
        [...sources].sort((left, right) => left.localeCompare(right)),
      ]),
  );
  return manifest;
}

function writeManifest(manifest, root = ROOT) {
  writeFileSync(
    path.join(root, MANIFEST_RELATIVE_PATH),
    `${JSON.stringify(sortedManifest(manifest), null, 2)}\n`,
    'utf8',
  );
}

function exceptionForMetric(metric, manual = false) {
  return {
    target: metric.caseName,
    path: metric.source,
    manual,
    reasons: manual ? ['manual_isolation'] : [...metric.blockers].sort(),
  };
}

function emptySuiteRecords(manifest) {
  return Array.from({ length: manifest.sharding.suiteCount }, (_, index) => ({
    name: suiteName(manifest, index),
    sources: [],
    weight: 0,
  }));
}

function lightestSuite(records) {
  return [...records].sort(
    (left, right) =>
      left.weight - right.weight ||
      left.sources.length - right.sources.length ||
      left.name.localeCompare(right.name),
  )[0];
}

export function rebalanceManifest(manifest, root = ROOT) {
  const sources = discoverSourceFiles(manifest, root);
  const metrics = sources.map((source) => sourceMetrics(source, manifest, root));
  const manualPaths = new Set(
    manifest.exceptions.filter((entry) => entry.manual).map((entry) => entry.path),
  );
  const exceptions = [];
  const candidates = [];

  for (const metric of metrics) {
    if (manualPaths.has(metric.source) || metric.blockers.length > 0) {
      exceptions.push(exceptionForMetric(metric, manualPaths.has(metric.source)));
    } else {
      candidates.push(metric);
    }
  }

  const suites = emptySuiteRecords(manifest);
  candidates
    .sort(
      (left, right) =>
        right.weight - left.weight || left.source.localeCompare(right.source),
    )
    .forEach((metric) => {
      const suite = lightestSuite(suites);
      suite.sources.push(metric.source);
      suite.weight += metric.weight;
    });

  manifest.exceptions = exceptions;
  manifest.suites = Object.fromEntries(
    suites.map((suite) => [suite.name, suite.sources]),
  );
  writeManifest(manifest, root);
  process.stdout.write(
    `[RustTestSuite] 전체 재배정: ${candidates.length} sources / ` +
      `${suites.length} suites / ${exceptions.length} exceptions\n`,
  );
  return manifest;
}

function declaredSourcePaths(manifest) {
  return new Set([
    ...manifest.exceptions.map((entry) => entry.path),
    ...Object.values(manifest.suites).flat(),
  ]);
}

function suiteRecordsWithWeights(manifest, root = ROOT) {
  return Object.entries(manifest.suites).map(([name, sources]) => ({
    name,
    sources,
    weight: sources.reduce(
      (total, source) => total + sourceMetrics(source, manifest, root).weight,
      0,
    ),
  }));
}

export function assignSources(manifest, requestedSources, root = ROOT) {
  const discovered = new Set(discoverSourceFiles(manifest, root));
  const declared = declaredSourcePaths(manifest);
  const caseIndex = buildCaseIndex(manifest);
  const suites = suiteRecordsWithWeights(manifest, root);
  const metrics = [...new Set(requestedSources)]
    .map((source) => normalizeRelativePath(source, 'source'))
    .sort((left, right) => left.localeCompare(right))
    .map((source) => {
      if (!discovered.has(source)) {
        throw new Error(`sourceRoots에서 test source를 찾을 수 없습니다: ${source}`);
      }
      if (declared.has(source)) {
        throw new Error(`이미 manifest에 등록된 test source입니다: ${source}`);
      }
      return sourceMetrics(source, manifest, root);
    })
    .sort(
      (left, right) =>
        right.weight - left.weight || left.source.localeCompare(right.source),
    );

  for (const metric of metrics) {
    let assignedTarget;
    if (caseIndex.has(metric.caseName)) {
      throw new Error(`중복 test case module: ${metric.caseName}`);
    }
    if (metric.blockers.length > 0) {
      manifest.exceptions.push(exceptionForMetric(metric));
      assignedTarget = metric.caseName;
      process.stdout.write(
        `[RustTestSuite] 예외 배정: ${metric.source} (${metric.blockers.join(', ')})\n`,
      );
    } else {
      const suite = lightestSuite(suites);
      suite.sources.push(metric.source);
      suite.weight += metric.weight;
      manifest.suites[suite.name].push(metric.source);
      assignedTarget = suite.name;
      process.stdout.write(
        `[RustTestSuite] 자동 배정: ${metric.source} -> ${suite.name}\n`,
      );
    }
    caseIndex.set(metric.caseName, assignedTarget);
  }

  if (metrics.length > 0) {
    writeManifest(manifest, root);
  }
  return metrics.length;
}

export function assignUnlistedSources(manifest, root = ROOT) {
  const declared = declaredSourcePaths(manifest);
  const unlisted = discoverSourceFiles(manifest, root).filter(
    (source) => !declared.has(source),
  );
  if (unlisted.length === 0) {
    process.stdout.write('[RustTestSuite] 새 test source 없음\n');
    return 0;
  }
  return assignSources(manifest, unlisted, root);
}

export function syncManifestSources(manifest, root = ROOT) {
  const discovered = new Set(discoverSourceFiles(manifest, root));
  let removed = 0;

  manifest.exceptions = manifest.exceptions.filter((entry) => {
    if (discovered.has(entry.path)) {
      return true;
    }
    removed += 1;
    process.stdout.write(`[RustTestSuite] 제거 반영: ${entry.path}\n`);
    return false;
  });
  for (const [suite, sources] of Object.entries(manifest.suites)) {
    manifest.suites[suite] = sources.filter((source) => {
      if (discovered.has(source)) {
        return true;
      }
      removed += 1;
      process.stdout.write(`[RustTestSuite] 제거 반영: ${source} (${suite})\n`);
      return false;
    });
  }
  if (removed > 0) {
    writeManifest(manifest, root);
  }

  const added = assignUnlistedSources(manifest, root);
  return { removed, added };
}

export function renderHarness(suite, sources) {
  const modules = [...sources]
    .sort((left, right) => left.localeCompare(right))
    .map((source) => {
      const modulePath = path.posix.relative(GENERATED_TEST_DIRECTORY, source);
      const caseName = caseNameForSource(source);
      return `#[path = "${modulePath}"]\nmod ${caseName};`;
    })
    .join('\n\n');

  return [
    '//! `tests/suites/manifest.json`에서 자동 생성된 integration test harness다.',
    '//! 직접 수정하지 말고 suite manifest 생성기를 사용한다.',
    `//! suite: ${suite}`,
    '',
    modules,
    '',
  ].join('\n');
}

export function renderCargoTestBlock(manifest) {
  const lines = [
    CARGO_BLOCK_START,
    '# tests/suites/manifest.json에서 생성한다. 직접 수정하지 않는다.',
  ];
  for (const suite of Object.keys(manifest.suites).sort()) {
    lines.push(
      '',
      '[[test]]',
      `name = "${suite}"`,
      `path = "${GENERATED_TEST_DIRECTORY}/${suite}.rs"`,
    );
  }
  for (const exception of [...manifest.exceptions].sort((left, right) =>
    left.target.localeCompare(right.target),
  )) {
    lines.push(
      '',
      '[[test]]',
      `name = "${exception.target}"`,
      `path = "${exception.path}"`,
    );
    if (Array.isArray(exception.requiredFeatures) && exception.requiredFeatures.length > 0) {
      lines.push(
        `required-features = [${exception.requiredFeatures
          .map((feature) => `"${feature}"`)
          .join(', ')}]`,
      );
    }
  }
  lines.push('', CARGO_BLOCK_END);
  return lines.join('\n');
}

function cargoBlockBounds(cargoManifest) {
  const start = cargoManifest.indexOf(CARGO_BLOCK_START);
  const endMarker = cargoManifest.indexOf(CARGO_BLOCK_END, start);
  if (start < 0 || endMarker < 0) {
    throw new Error('Cargo.toml에 generated test target marker가 없습니다.');
  }
  return { start, end: endMarker + CARGO_BLOCK_END.length };
}

function updateCargoManifest(manifest, root = ROOT) {
  const cargoPath = path.join(root, CARGO_MANIFEST_RELATIVE_PATH);
  const cargoManifest = readFileSync(cargoPath, 'utf8');
  const bounds = cargoBlockBounds(cargoManifest);
  const generatedBlock = renderCargoTestBlock(manifest);
  writeFileSync(
    cargoPath,
    cargoManifest.slice(0, bounds.start) +
      generatedBlock +
      cargoManifest.slice(bounds.end),
    'utf8',
  );
}

function inspectRepository(
  manifest,
  root = ROOT,
  { checkGenerated = true, checkCargo = true } = {},
) {
  const errors = [];
  const discovered = discoverSourceFiles(manifest, root);
  const discoveredSet = new Set(discovered);
  const declared = declaredSourcePaths(manifest);
  let caseIndex = new Map();

  try {
    caseIndex = buildCaseIndex(manifest);
  } catch (error) {
    errors.push(error instanceof Error ? error.message : String(error));
  }
  for (const entry of manifest.nextestPriorities) {
    if (!caseIndex.has(entry.case)) {
      errors.push(`nextest priority case가 manifest에 없습니다: ${entry.case}`);
    }
  }

  for (const source of discovered) {
    if (!declared.has(source)) {
      errors.push(
        `manifest에 없는 test source: ${source} ` +
          '(node scripts/rust-test-suite-manifest.mjs --generate)',
      );
    }
  }
  for (const source of declared) {
    if (!discoveredSet.has(source)) {
      errors.push(`존재하지 않거나 sourceRoots 밖인 test source: ${source}`);
    }
  }

  const expectedSuiteNames = new Set(
    Array.from({ length: manifest.sharding.suiteCount }, (_, index) =>
      suiteName(manifest, index),
    ),
  );
  for (const suite of expectedSuiteNames) {
    if (!Object.hasOwn(manifest.suites, suite)) {
      errors.push(`필수 generated suite가 없습니다: ${suite}`);
    } else if (manifest.suites[suite].length === 0) {
      errors.push(`generated suite가 비어 있습니다: ${suite}`);
    }
  }
  for (const suite of Object.keys(manifest.suites)) {
    if (!expectedSuiteNames.has(suite)) {
      errors.push(`정책 밖 generated suite: ${suite}`);
    }
  }

  const metrics = discovered.map((source) => sourceMetrics(source, manifest, root));
  const metricsBySource = new Map(metrics.map((metric) => [metric.source, metric]));
  const exceptionPaths = new Set(manifest.exceptions.map((entry) => entry.path));
  for (const [suite, sources] of Object.entries(manifest.suites)) {
    for (const source of sources) {
      const metric = metricsBySource.get(source);
      if (metric && metric.blockers.length > 0) {
        errors.push(
          `module 통합 불가 source가 ${suite}에 포함됨: ${source} ` +
            `(${metric.blockers.join(', ')})`,
        );
      }
    }
  }
  for (const metric of metrics) {
    if (metric.blockers.length > 0 && !exceptionPaths.has(metric.source)) {
      errors.push(`통합 예외 등록이 필요한 source: ${metric.source}`);
    }
  }

  if (checkGenerated) {
    const generatedDirectory = path.join(root, GENERATED_TEST_DIRECTORY);
    const expectedHarnesses = new Set(
      Object.keys(manifest.suites).map((suite) => `${suite}.rs`),
    );
    const physicalHarnesses = existsSync(generatedDirectory)
      ? readdirSync(generatedDirectory, { withFileTypes: true })
          .filter((entry) => entry.isFile() && entry.name.endsWith('.rs'))
          .map((entry) => entry.name)
      : [];
    for (const harness of physicalHarnesses) {
      if (!expectedHarnesses.has(harness)) {
        errors.push(`manifest에 없는 generated harness: ${harness}`);
      }
    }
    for (const [suite, sources] of Object.entries(manifest.suites)) {
      const harnessPath = path.join(generatedDirectory, `${suite}.rs`);
      const expected = renderHarness(suite, sources);
      const actual = existsSync(harnessPath)
        ? readFileSync(harnessPath, 'utf8')
        : null;
      if (actual !== expected) {
        errors.push(`generated harness drift: ${suite}.rs`);
      }
    }
  }

  if (checkCargo) {
    const cargoManifest = readFileSync(
      path.join(root, CARGO_MANIFEST_RELATIVE_PATH),
      'utf8',
    );
    if (!/^autotests\s*=\s*false\s*$/m.test(cargoManifest)) {
      errors.push('Cargo.toml package에 autotests = false가 필요합니다.');
    }
    try {
      const bounds = cargoBlockBounds(cargoManifest);
      const actual = cargoManifest.slice(bounds.start, bounds.end);
      const expected = renderCargoTestBlock(manifest);
      if (actual !== expected) {
        errors.push('Cargo.toml generated test target block drift');
      }
    } catch (error) {
      errors.push(error instanceof Error ? error.message : String(error));
    }
  }

  const suiteWeights = Object.entries(manifest.suites).map(([suite, sources]) => ({
    suite,
    weight: sources.reduce(
      (total, source) => total + (metricsBySource.get(source)?.weight ?? 0),
      0,
    ),
  }));
  const weights = suiteWeights.map((entry) => entry.weight).sort((a, b) => a - b);
  const integrationTargetCount =
    Object.keys(manifest.suites).length + manifest.exceptions.length;
  if (integrationTargetCount > manifest.sharding.maximumIntegrationTargets) {
    errors.push(
      `integration target 예산 초과: ${integrationTargetCount} > ` +
        manifest.sharding.maximumIntegrationTargets,
    );
  }
  return {
    errors,
    sourceCount: discovered.length,
    staticTestAttributes: metrics.reduce(
      (total, metric) => total + metric.staticTests,
      0,
    ),
    suiteCount: Object.keys(manifest.suites).length,
    exceptionCount: manifest.exceptions.length,
    integrationTargetCount,
    maximumIntegrationTargets: manifest.sharding.maximumIntegrationTargets,
    caseModuleCount: caseIndex.size,
    minimumNextestCases: manifest.minimumNextestCases,
    minimumSuiteWeight: weights[0] ?? 0,
    maximumSuiteWeight: weights.at(-1) ?? 0,
  };
}

export function validateRepository(root = ROOT) {
  try {
    return inspectRepository(loadManifest(root), root);
  } catch (error) {
    return {
      errors: [error instanceof Error ? error.message : String(error)],
      sourceCount: 0,
      staticTestAttributes: 0,
      suiteCount: 0,
      exceptionCount: 0,
      integrationTargetCount: 0,
      maximumIntegrationTargets: 0,
      caseModuleCount: 0,
      minimumNextestCases: 0,
      minimumSuiteWeight: 0,
      maximumSuiteWeight: 0,
    };
  }
}

export function generateArtifacts(manifest, root = ROOT) {
  const inspection = inspectRepository(manifest, root, {
    checkGenerated: false,
    checkCargo: false,
  });
  if (inspection.errors.length > 0) {
    throw new Error(inspection.errors.join('\n'));
  }

  const generatedDirectory = path.join(root, GENERATED_TEST_DIRECTORY);
  mkdirSync(generatedDirectory, { recursive: true });
  const expectedHarnesses = new Set(
    Object.keys(manifest.suites).map((suite) => `${suite}.rs`),
  );
  for (const entry of readdirSync(generatedDirectory, { withFileTypes: true })) {
    if (
      entry.isFile() &&
      entry.name.endsWith('.rs') &&
      !expectedHarnesses.has(entry.name)
    ) {
      rmSync(path.join(generatedDirectory, entry.name));
    }
  }
  for (const [suite, sources] of Object.entries(manifest.suites).sort()) {
    writeFileSync(
      path.join(generatedDirectory, `${suite}.rs`),
      renderHarness(suite, sources),
      'utf8',
    );
  }
  updateCargoManifest(manifest, root);
  process.stdout.write(
    `[RustTestSuite] 생성: ${Object.keys(manifest.suites).length} harnesses, ` +
      `${manifest.exceptions.length} exceptions\n`,
  );
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
      `${validation.sourceCount} sources / ` +
      `${validation.staticTestAttributes} static test attrs / ` +
      `${validation.suiteCount} suites + ${validation.exceptionCount} exceptions = ` +
      `${validation.integrationTargetCount}/${validation.maximumIntegrationTargets} ` +
      'integration targets / ' +
      `nextest 최소 ${validation.minimumNextestCases} cases / ` +
      `weight ${validation.minimumSuiteWeight}..${validation.maximumSuiteWeight}\n`,
  );
}

if (process.argv[1] && path.resolve(process.argv[1]) === SCRIPT_PATH) {
  try {
    const command = process.argv[2];
    if (command === '--rebalance') {
      const manifest = rebalanceManifest(loadManifest());
      generateArtifacts(manifest);
      printValidation(validateRepository());
    } else if (command === '--generate' || command === '--adopt-new') {
      const manifest = loadManifest();
      assignUnlistedSources(manifest);
      generateArtifacts(manifest);
      printValidation(validateRepository());
    } else if (command === '--sync') {
      const manifest = loadManifest();
      syncManifestSources(manifest);
      generateArtifacts(manifest);
      printValidation(validateRepository());
    } else if (command === '--adopt') {
      const requested = process.argv.slice(3).map((inputPath) =>
        path.isAbsolute(inputPath)
          ? path.relative(ROOT, inputPath).split(path.sep).join('/')
          : inputPath,
      );
      if (requested.length === 0) {
        throw new Error('--adopt에는 하나 이상의 test source가 필요합니다.');
      }
      const manifest = loadManifest();
      assignSources(manifest, requested);
      generateArtifacts(manifest);
      printValidation(validateRepository());
    } else if (command === '--check') {
      printValidation(validateRepository());
    } else {
      process.stderr.write(
        '사용법: node scripts/rust-test-suite-manifest.mjs ' +
          '--check|--generate|--sync|--rebalance|--adopt-new|--adopt <파일...>\n',
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
