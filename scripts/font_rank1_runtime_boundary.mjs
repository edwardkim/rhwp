#!/usr/bin/env node

import fs from 'node:fs';
import path from 'node:path';
import process from 'node:process';
import { fileURLToPath } from 'node:url';

import {
  assertTraceParity,
  canonicalJson,
  readJson,
  regularInput,
  rejectAbsolutePaths,
  repoRelative,
  requireEqual,
  runNative,
  runWasm,
  sha256,
  sha256File,
  summarizeTrace,
} from './font_rank8_trace_baseline.mjs';

const SCRIPT_PATH = fileURLToPath(import.meta.url);
const ROOT = path.resolve(path.dirname(SCRIPT_PATH), '..');
const TARGET_FACE = '문체부 바탕체';
const CANONICAL_FACE = 'MBatang';
const EXPECTED_HWPX_SHA256 = '8ded3aff6f0286ee5ee4ad9c66732026fa627220b529e5d0fa7b9d51bc3ddb3f';
const EXPECTED_HWP_SHA256 = '081b597eb85c8431f691236ce36517edf863c1f10563d09edf2617a48cfbcb6b';

function isObject(value) {
  return value !== null && typeof value === 'object' && !Array.isArray(value);
}

function compareText(left, right) {
  return Buffer.from(String(left), 'utf8').compare(Buffer.from(String(right), 'utf8'));
}

function counted(values) {
  const counts = new Map();
  for (const value of values) {
    const key = JSON.stringify(value);
    const current = counts.get(key) ?? { value, count: 0 };
    current.count += 1;
    counts.set(key, current);
  }
  return [...counts.values()].sort((left, right) => compareText(
    JSON.stringify(left.value),
    JSON.stringify(right.value),
  ));
}

export function summarizeBoundary(trace, expectedFace = TARGET_FACE) {
  const summary = summarizeTrace(trace, expectedFace);
  const records = trace.records;
  const requestedFaces = counted(records.map(record => record?.layoutName?.requestedFace));
  const normalizedFaces = counted(records.map(record => record?.layoutName?.normalizedFace));
  const layoutNameStepCounts = counted(records.map(record => record?.layoutName?.steps?.length));
  const aliasResolvedFaces = counted(records.map(record => record?.layoutMetric?.aliasResolvedFace));
  const languageSlots = counted(records.map(record => record?.document?.languageSlot));
  const paintRequestedFaces = {
    canvas2d: counted(records.map(record => record?.paint?.canvas2d?.requested)),
    canvaskit: counted(records.map(record => record?.paint?.canvaskit?.requested)),
    native: counted(records.map(record => record?.paint?.native?.requested)),
  };
  requireEqual(requestedFaces, [{ value: expectedFace, count: records.length }], 'requested faces');
  requireEqual(normalizedFaces, [{ value: expectedFace, count: records.length }], 'normalized faces');
  requireEqual(layoutNameStepCounts, [{ value: 0, count: records.length }], 'layout-name steps');
  requireEqual(aliasResolvedFaces, [{ value: expectedFace, count: records.length }], 'metric aliases');
  requireEqual(summary.metricEntries, [{ value: null, count: records.length }], 'metric entries');
  requireEqual(summary.matchKinds, [{ value: 'none', count: records.length }], 'metric match kinds');
  return {
    records: records.length,
    requestedFaces,
    normalizedFaces,
    layoutNameStepCounts,
    aliasResolvedFaces,
    languageSlots,
    metricEntries: summary.metricEntries,
    matchKinds: summary.matchKinds,
    widthSources: summary.widthSources,
    paintRequestedFaces,
    layoutHash: summary.layoutHash,
    normalizedHash: summary.normalizedHash,
    canonicalTraceSha256: summary.canonicalTraceSha256,
  };
}

function validateCanonicalEnvelope(value, kind, stage) {
  if (value?.schemaVersion !== 1 || value?.kind !== kind || value?.issue !== 4967) {
    throw new Error(`${stage} identity mismatch`);
  }
  const body = structuredClone(value);
  const claimed = body.canonicalSha256;
  delete body.canonicalSha256;
  if (claimed !== sha256(canonicalJson(body))) {
    throw new Error(`${stage} canonical SHA-256 drifted`);
  }
}

function validateQ0(q0) {
  validateCanonicalEnvelope(q0, 'font-rank1-qualification-baseline', 'W8-R1-Q0');
  if (q0?.stage !== 'W8-R1-Q0'
      || q0?.target?.documentFace !== TARGET_FACE
      || q0?.target?.canonicalFaceCandidate !== CANONICAL_FACE
      || q0?.gates?.existingMetricAnchorFound !== true
      || q0?.executionPolicy?.hyperVOracleRerun !== false) {
    throw new Error('W8-R1-Q0 decision boundary mismatch');
  }
}

function validateManifest(manifest, hwpxPath, hwpPath) {
  if (manifest?.schemaVersion !== 1
      || manifest?.kind !== 'font-rank1-runtime-boundary-fixture-manifest'
      || manifest?.issue !== 4967
      || manifest?.documentFace !== TARGET_FACE
      || manifest?.queueRank !== 1
      || manifest?.fontBytesEmbedded !== false
      || manifest?.conversion?.deterministicRepeatSha256Equal !== true
      || manifest?.conversion?.irDifference !== false
      || manifest?.conversion?.pageCountEqual !== true) {
    throw new Error('rank-1 runtime fixture manifest identity mismatch');
  }
  requireEqual(sha256File(hwpxPath), EXPECTED_HWPX_SHA256, 'HWPX fixture SHA-256');
  requireEqual(sha256File(hwpPath), EXPECTED_HWP_SHA256, 'HWP fixture SHA-256');
  requireEqual(manifest.hwpx?.sha256, EXPECTED_HWPX_SHA256, 'manifest HWPX SHA-256');
  requireEqual(manifest.hwp?.sha256, EXPECTED_HWP_SHA256, 'manifest HWP SHA-256');
}

async function traceFixture({ format, fixturePath, nativeBin, wasmJs, wasmBinary }) {
  const nativeEnvelope = runNative(nativeBin, fixturePath);
  const wasmTrace = await runWasm(wasmJs, wasmBinary, fixturePath);
  const nativeBoundary = summarizeBoundary(nativeEnvelope.trace);
  const wasmBoundary = summarizeBoundary(wasmTrace);
  const paritySha256 = assertTraceParity(nativeEnvelope.trace, wasmTrace);
  requireEqual(nativeBoundary, wasmBoundary, `${format} native/WASM boundary`);
  return {
    format,
    fixtureSha256: sha256File(fixturePath),
    trace: nativeBoundary,
    parity: {
      nativeWasmByteExact: true,
      canonicalTraceSha256: paritySha256,
      maxCharacters: 4096,
    },
  };
}

export function compareFormatBoundaries(hwpx, hwp) {
  const semantic = value => ({
    records: value.trace.records,
    requestedFaces: value.trace.requestedFaces,
    normalizedFaces: value.trace.normalizedFaces,
    layoutNameStepCounts: value.trace.layoutNameStepCounts,
    aliasResolvedFaces: value.trace.aliasResolvedFaces,
    metricEntries: value.trace.metricEntries,
    matchKinds: value.trace.matchKinds,
    widthSources: value.trace.widthSources,
    paintRequestedFaces: value.trace.paintRequestedFaces,
  });
  requireEqual(semantic(hwpx), semantic(hwp), 'HWP/HWPX runtime boundary semantics');
  return {
    semanticEqual: true,
    languageSlotsDiffer: canonicalJson(hwpx.trace.languageSlots)
      !== canonicalJson(hwp.trace.languageSlots),
    hwpxLanguageSlots: hwpx.trace.languageSlots,
    hwpLanguageSlots: hwp.trace.languageSlots,
  };
}

function parseArgs(args) {
  const result = {};
  for (let index = 0; index < args.length; index += 2) {
    const option = args[index];
    const value = args[index + 1];
    if (!option?.startsWith('--') || value === undefined) {
      throw new Error('every option requires a value');
    }
    result[option.slice(2)] = value;
  }
  for (const name of [
    'hwpx-fixture',
    'hwp-fixture',
    'fixture-manifest',
    'q0',
    'native-bin',
    'wasm-js',
    'wasm-binary',
    'output',
  ]) {
    if (!result[name]) throw new Error(`--${name} is required`);
  }
  return result;
}

async function main() {
  const args = parseArgs(process.argv.slice(2));
  const hwpxPath = regularInput(args['hwpx-fixture']);
  const hwpPath = regularInput(args['hwp-fixture']);
  const manifestPath = regularInput(args['fixture-manifest']);
  const q0Path = regularInput(args.q0);
  const nativeBin = regularInput(args['native-bin'], 512 * 1024 * 1024);
  const wasmJs = regularInput(args['wasm-js']);
  const wasmBinary = regularInput(args['wasm-binary']);
  const manifest = readJson(manifestPath);
  const q0 = readJson(q0Path);
  validateQ0(q0);
  validateManifest(manifest, hwpxPath, hwpPath);

  const hwpx = await traceFixture({
    format: 'hwpx', fixturePath: hwpxPath, nativeBin, wasmJs, wasmBinary,
  });
  const hwp = await traceFixture({
    format: 'hwp5', fixturePath: hwpPath, nativeBin, wasmJs, wasmBinary,
  });
  const formatComparison = compareFormatBoundaries(hwpx, hwp);
  const output = {
    schemaVersion: 1,
    kind: 'font-rank1-runtime-boundary-baseline',
    issue: 4967,
    stage: 'W8-R1-Q1',
    target: {
      documentFace: TARGET_FACE,
      canonicalFaceCandidate: CANONICAL_FACE,
      queueRank: 1,
    },
    inputs: {
      hwpxFixture: { artifact: repoRelative(hwpxPath), sha256: sha256File(hwpxPath) },
      hwpFixture: { artifact: repoRelative(hwpPath), sha256: sha256File(hwpPath) },
      fixtureManifest: { artifact: repoRelative(manifestPath), sha256: sha256File(manifestPath) },
      q0Baseline: { artifact: repoRelative(q0Path), sha256: sha256File(q0Path) },
      nativeBinary: { artifact: repoRelative(nativeBin), sha256: sha256File(nativeBin, 512 * 1024 * 1024) },
      wasmJs: { artifact: repoRelative(wasmJs), sha256: sha256File(wasmJs) },
      wasmBinary: { artifact: repoRelative(wasmBinary), sha256: sha256File(wasmBinary) },
    },
    formats: [hwpx, hwp],
    formatComparison,
    firstDivergence: {
      targetDecisionPlane: 'layout-name',
      observed: {
        requestedFace: TARGET_FACE,
        normalizedFace: TARGET_FACE,
        steps: 0,
        metricAliasResolvedFace: TARGET_FACE,
        metricEntry: null,
        matchKind: 'none',
      },
      expectedForQ2Hypothesis: {
        normalizedFace: CANONICAL_FACE,
        existingMetricAnchor: 'font-metric.e6fdb023c2acf414807d',
      },
      disposition: 'qualified-for-q2-layout-name-hypothesis',
    },
    executionPolicy: {
      fullCorpusRerun: false,
      hyperVOracleRerun: false,
      productMutation: false,
      publicFixturesOnly: true,
    },
    privacy: {
      absolutePathIncluded: false,
      fontBytesIncluded: false,
      privateCorpusAccessed: false,
      privateDocumentIdentityIncluded: false,
      fullTraceTracked: false,
    },
  };
  rejectAbsolutePaths(output);
  output.canonicalSha256 = sha256(canonicalJson(output));
  const outputPath = path.resolve(args.output);
  if (fs.existsSync(outputPath) && fs.lstatSync(outputPath).isSymbolicLink()) {
    throw new Error('refusing to overwrite a symlink output');
  }
  fs.mkdirSync(path.dirname(outputPath), { recursive: true });
  fs.writeFileSync(outputPath, `${JSON.stringify(output, null, 2)}\n`, { mode: 0o644 });
  fs.chmodSync(outputPath, 0o644);
  process.stdout.write(`${JSON.stringify({
    formats: output.formats.map(value => value.format),
    recordsPerFormat: output.formats.map(value => value.trace.records),
    nativeWasmByteExact: output.formats.every(value => value.parity.nativeWasmByteExact),
    targetDecisionPlane: output.firstDivergence.targetDecisionPlane,
    canonicalSha256: output.canonicalSha256,
  })}\n`);
}

if (process.argv[1] && path.resolve(process.argv[1]) === SCRIPT_PATH) {
  main().catch(error => {
    process.stderr.write(`${error instanceof Error ? error.message : String(error)}\n`);
    process.exitCode = 1;
  });
}
