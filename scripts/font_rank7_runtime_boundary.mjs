#!/usr/bin/env node

import { spawnSync } from 'node:child_process';
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
  runWasm,
  sha256,
  sha256File,
  summarizeFixedGeometry,
  summarizeTrace,
} from './font_rank8_trace_baseline.mjs';

const SCRIPT_PATH = fileURLToPath(import.meta.url);
const ROOT = path.resolve(path.dirname(SCRIPT_PATH), '..');
const TARGET_FACE = 'KoPubWorld돋움체 Light';
const SUBSTITUTION_FACE = 'KoPubWorld바탕체 Light';
const EXPECTED_HWPX_SHA256 = '1cc8062c6fd0da39cfddc4182115226717516d4250e693b43596293374236f9e';
const EXPECTED_HWP_SHA256 = '3a844e0530ecede89301ab1f3c2381865412f8472aa08733cdb9d1d25223ee7f';
const EXPECTED_GENERATOR_MANIFEST_SHA256 = '38778e8ee3ecc8a9c5177c082eea2528214c5b4a7d71c66c009873922befd9df';
const MAX_CHILD_OUTPUT_BYTES = 64 * 1024 * 1024;
const CHILD_TIMEOUT_MS = 60_000;

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
  validateCanonicalEnvelope(q0, 'font-rank7-qualification-baseline', 'W8-R7-Q0');
  if (q0?.stage !== 'W8-R7-Q0'
      || q0?.target?.documentFace !== TARGET_FACE
      || q0?.target?.queueRank !== 7
      || q0?.gates?.layoutProjectionAbsent !== true
      || q0?.gates?.registrySupplyOnly !== true
      || q0?.executionPolicy?.hyperVOracleRerun !== false) {
    throw new Error('W8-R7-Q0 decision boundary mismatch');
  }
}

function validateGeneratorManifest(manifest, hwpxPath) {
  if (manifest?.schemaVersion !== 1
      || manifest?.kind !== 'font-oracle-typesetting-fixture-manifest'
      || manifest?.issue !== 4963
      || manifest?.semantic?.documentFace !== TARGET_FACE
      || manifest?.semantic?.queueRank !== 7
      || manifest?.semantic?.substitutionFace !== SUBSTITUTION_FACE
      || manifest?.semantic?.fontBytesEmbedded !== false) {
    throw new Error('rank-7 generator fixture manifest identity mismatch');
  }
  requireEqual(manifest.inputSha256, EXPECTED_HWPX_SHA256, 'generator fixture SHA-256');
  requireEqual(sha256File(hwpxPath), EXPECTED_HWPX_SHA256, 'HWPX fixture SHA-256');
  requireEqual(
    manifest.lineSegLaneCounts,
    { 'fresh-candidate-lane': 12, 'stored-line-lane': 14 },
    'rank-7 LineSeg lanes',
  );
}

function validateRuntimeManifest(manifest, generatorManifestPath, hwpxPath, hwpPath) {
  if (manifest?.schemaVersion !== 1
      || manifest?.kind !== 'font-rank7-runtime-boundary-fixture-manifest'
      || manifest?.issue !== 4967
      || manifest?.producerIssue !== 4963
      || manifest?.consumerIssue !== 4967
      || manifest?.documentFace !== TARGET_FACE
      || manifest?.substitutionFace !== SUBSTITUTION_FACE
      || manifest?.queueRank !== 7
      || manifest?.fontBytesEmbedded !== false
      || manifest?.conversion?.deterministicRepeatSha256Equal !== true
      || manifest?.conversion?.irDifference !== false
      || manifest?.conversion?.pageCountEqual !== true) {
    throw new Error('rank-7 runtime fixture manifest identity mismatch');
  }
  requireEqual(sha256File(hwpxPath), EXPECTED_HWPX_SHA256, 'runtime HWPX SHA-256');
  requireEqual(sha256File(hwpPath), EXPECTED_HWP_SHA256, 'runtime HWP SHA-256');
  requireEqual(manifest.hwpx?.sha256, EXPECTED_HWPX_SHA256, 'manifest HWPX SHA-256');
  requireEqual(manifest.hwp?.sha256, EXPECTED_HWP_SHA256, 'manifest HWP SHA-256');
  requireEqual(
    sha256File(generatorManifestPath),
    EXPECTED_GENERATOR_MANIFEST_SHA256,
    'generator manifest SHA-256',
  );
  requireEqual(
    manifest.producerFixtureManifest?.sha256,
    EXPECTED_GENERATOR_MANIFEST_SHA256,
    'runtime manifest producer SHA-256',
  );
}

function runChild(binary, args, label) {
  const result = spawnSync(
    regularInput(binary, 512 * 1024 * 1024),
    args,
    {
      cwd: ROOT,
      encoding: 'utf8',
      maxBuffer: MAX_CHILD_OUTPUT_BYTES,
      shell: false,
      timeout: CHILD_TIMEOUT_MS,
      killSignal: 'SIGKILL',
    },
  );
  if (result.error) throw result.error;
  if (result.status !== 0) {
    throw new Error(`${label} failed (${result.status}): ${result.stderr}`);
  }
  return JSON.parse(result.stdout);
}

function runNative(nativeBin, fixturePath) {
  const envelope = runChild(
    nativeBin,
    [fixturePath, '--page', '0', '--max-characters', '4096', '--json'],
    'native trace',
  );
  if (envelope?.tool !== 'rhwp-q-font-trace' || typeof envelope.trace !== 'object') {
    throw new Error('native trace envelope identity mismatch');
  }
  return envelope;
}

function runLayout(layoutBin, fixturePath) {
  const envelope = runChild(layoutBin, [fixturePath, '--page', '0', '--json'], 'text layout');
  if (envelope?.tool !== 'rhwp-q-text-layout' || !Array.isArray(envelope.runs)) {
    throw new Error('text-layout envelope identity mismatch');
  }
  return envelope;
}

export function summarizeBoundary(trace, format) {
  const summary = summarizeTrace(trace, TARGET_FACE);
  const records = trace.records;
  const requestedFaces = counted(records.map(record => record?.layoutName?.requestedFace));
  const normalizedFaces = counted(records.map(record => record?.layoutName?.normalizedFace));
  const aliasResolvedFaces = counted(records.map(record => record?.layoutMetric?.aliasResolvedFace));
  const substitutionFaces = counted(records.map(record => record?.document?.substFont ?? null));
  const layoutNameStepKinds = counted(records.flatMap(
    record => (record?.layoutName?.steps ?? []).map(step => step?.kind),
  ));
  const cssFamilyChains = counted(records.map(record => record?.layoutName?.cssFamilyChain));
  const paintRequested = {
    canvas2d: counted(records.map(record => record?.paint?.canvas2d?.requested)),
    canvaskit: counted(records.map(record => record?.paint?.canvaskit?.requested)),
    native: counted(records.map(record => record?.paint?.native?.requested)),
  };
  requireEqual(requestedFaces, [{ value: TARGET_FACE, count: records.length }], `${format} requested`);
  requireEqual(normalizedFaces, [{ value: TARGET_FACE, count: records.length }], `${format} normalized`);
  requireEqual(aliasResolvedFaces, [{ value: TARGET_FACE, count: records.length }], `${format} alias`);
  requireEqual(summary.metricEntries, [{ value: null, count: records.length }], `${format} metric`);
  requireEqual(summary.matchKinds, [{ value: 'none', count: records.length }], `${format} match`);
  return {
    records: records.length,
    requestedFaces,
    normalizedFaces,
    aliasResolvedFaces,
    substitutionFaces,
    layoutNameStepKinds,
    cssFamilyChains,
    metricEntries: summary.metricEntries,
    matchKinds: summary.matchKinds,
    widthSources: summary.widthSources,
    paintRequested,
    canonicalTraceSha256: summary.canonicalTraceSha256,
  };
}

export function layoutMetricProjection(trace) {
  return trace.records.map(record => ({
    source: record.source,
    layoutMetric: record.layoutMetric,
  }));
}

export function layoutRunProjection(layoutEnvelope) {
  return layoutEnvelope.runs.map(run => {
    const projected = structuredClone(run);
    delete projected.fontFamily;
    return projected;
  });
}

export function compareFormatBoundaries(hwpx, hwp) {
  requireEqual(hwpx.layoutMetricProjection, hwp.layoutMetricProjection, 'HWP/HWPX metric projection');
  requireEqual(hwpx.layoutRunProjection, hwp.layoutRunProjection, 'HWP/HWPX layout run projection');
  requireEqual(hwpx.fixedGeometry, hwp.fixedGeometry, 'HWP/HWPX fixed geometry');
  requireEqual(
    hwpx.boundary.substitutionFaces,
    [{ value: SUBSTITUTION_FACE, count: hwpx.boundary.records }],
    'HWPX document substitution',
  );
  requireEqual(
    hwp.boundary.substitutionFaces,
    [{ value: null, count: hwp.boundary.records }],
    'HWP document substitution',
  );
  requireEqual(
    hwpx.boundary.layoutNameStepKinds,
    [{ value: 'documentSubstFont', count: hwpx.boundary.records }],
    'HWPX layout-name metadata step',
  );
  requireEqual(hwp.boundary.layoutNameStepKinds, [], 'HWP layout-name metadata step');
  return {
    layoutMetricProjectionEqual: true,
    layoutRunProjectionEqual: true,
    fixedGeometryEqual: true,
    substitutionMetadataDiffers: true,
    substitutionAffectsLayoutMetric: false,
    substitutionAffectsPaintCandidateChain: true,
    layoutMetricProjectionSha256: sha256(canonicalJson(hwpx.layoutMetricProjection)),
    layoutRunProjectionSha256: sha256(canonicalJson(hwpx.layoutRunProjection)),
  };
}

async function traceFixture({ format, fixturePath, nativeBin, layoutBin, wasmJs, wasmBinary, manifest }) {
  const nativeEnvelope = runNative(nativeBin, fixturePath);
  const wasmTrace = await runWasm(wasmJs, wasmBinary, fixturePath);
  const layoutEnvelope = runLayout(layoutBin, fixturePath);
  const boundary = summarizeBoundary(nativeEnvelope.trace, format);
  const wasmBoundary = summarizeBoundary(wasmTrace, format);
  const paritySha256 = assertTraceParity(nativeEnvelope.trace, wasmTrace);
  requireEqual(boundary, wasmBoundary, `${format} native/WASM boundary`);
  return {
    format,
    fixtureSha256: sha256File(fixturePath),
    boundary,
    fixedGeometry: summarizeFixedGeometry(layoutEnvelope, manifest),
    layoutMetricProjection: layoutMetricProjection(nativeEnvelope.trace),
    layoutRunProjection: layoutRunProjection(layoutEnvelope),
    parity: {
      nativeWasmByteExact: true,
      canonicalTraceSha256: paritySha256,
      maxCharacters: 4096,
    },
  };
}

function publicFormat(value) {
  return {
    format: value.format,
    fixtureSha256: value.fixtureSha256,
    boundary: value.boundary,
    fixedGeometry: value.fixedGeometry,
    projections: {
      layoutMetricSha256: sha256(canonicalJson(value.layoutMetricProjection)),
      layoutRunsSha256: sha256(canonicalJson(value.layoutRunProjection)),
    },
    parity: value.parity,
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
    'generator-manifest',
    'fixture-manifest',
    'q0',
    'native-bin',
    'layout-bin',
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
  const generatorManifestPath = regularInput(args['generator-manifest']);
  const fixtureManifestPath = regularInput(args['fixture-manifest']);
  const q0Path = regularInput(args.q0);
  const nativeBin = regularInput(args['native-bin'], 512 * 1024 * 1024);
  const layoutBin = regularInput(args['layout-bin'], 512 * 1024 * 1024);
  const wasmJs = regularInput(args['wasm-js']);
  const wasmBinary = regularInput(args['wasm-binary']);
  const generatorManifest = readJson(generatorManifestPath);
  const fixtureManifest = readJson(fixtureManifestPath);
  const q0 = readJson(q0Path);
  validateQ0(q0);
  validateGeneratorManifest(generatorManifest, hwpxPath);
  validateRuntimeManifest(
    fixtureManifest,
    generatorManifestPath,
    hwpxPath,
    hwpPath,
  );

  const hwpx = await traceFixture({
    format: 'hwpx',
    fixturePath: hwpxPath,
    nativeBin,
    layoutBin,
    wasmJs,
    wasmBinary,
    manifest: generatorManifest,
  });
  const hwp = await traceFixture({
    format: 'hwp5',
    fixturePath: hwpPath,
    nativeBin,
    layoutBin,
    wasmJs,
    wasmBinary,
    manifest: generatorManifest,
  });
  const formatComparison = compareFormatBoundaries(hwpx, hwp);

  const output = {
    schemaVersion: 1,
    kind: 'font-rank7-runtime-boundary-baseline',
    issue: 4967,
    stage: 'W8-R7-Q1',
    target: { documentFace: TARGET_FACE, queueRank: 7 },
    inputs: {
      hwpxFixture: { artifact: repoRelative(hwpxPath), sha256: sha256File(hwpxPath) },
      hwpFixture: { artifact: repoRelative(hwpPath), sha256: sha256File(hwpPath) },
      generatorManifest: {
        artifact: repoRelative(generatorManifestPath),
        sha256: sha256File(generatorManifestPath),
      },
      fixtureManifest: {
        artifact: repoRelative(fixtureManifestPath),
        sha256: sha256File(fixtureManifestPath),
      },
      q0Baseline: { artifact: repoRelative(q0Path), sha256: sha256File(q0Path) },
      nativeBinary: {
        artifact: repoRelative(nativeBin),
        sha256: sha256File(nativeBin, 512 * 1024 * 1024),
      },
      layoutBinary: {
        artifact: repoRelative(layoutBin),
        sha256: sha256File(layoutBin, 512 * 1024 * 1024),
      },
      wasmJs: { artifact: repoRelative(wasmJs), sha256: sha256File(wasmJs) },
      wasmBinary: { artifact: repoRelative(wasmBinary), sha256: sha256File(wasmBinary) },
    },
    formats: [publicFormat(hwpx), publicFormat(hwp)],
    formatComparison,
    firstDivergence: {
      targetDecisionPlane: 'layout-metric',
      observed: {
        requestedFace: TARGET_FACE,
        normalizedFace: TARGET_FACE,
        metricAliasResolvedFace: TARGET_FACE,
        metricEntry: null,
        matchKind: 'none',
        widthSources: hwpx.boundary.widthSources,
      },
      formatMetadataBoundary: {
        hwpxDocumentSubstitution: SUBSTITUTION_FACE,
        hwpDocumentSubstitution: null,
        affectsLayoutMetric: false,
        affectsPaintCandidateChain: true,
      },
      expectedForQ2Hypothesis: {
        metricSource: 'official exact KoPubWorld Dotum Light hmtx',
        sourceFontSha256: '069494cce21a4222c88e537f256b6f46fee209375aba769f82431b2d382bc84f',
      },
      disposition: 'qualified-for-q2-layout-metric-hypothesis',
    },
    executionPolicy: {
      fullCorpusRerun: false,
      hyperVOracleRerun: false,
      productMutation: false,
      publicFixturesOnly: true,
      maxCharacters: 4096,
      childTimeoutMs: CHILD_TIMEOUT_MS,
      childOutputLimitBytes: MAX_CHILD_OUTPUT_BYTES,
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
    recordsPerFormat: output.formats.map(value => value.boundary.records),
    nativeWasmByteExact: output.formats.every(value => value.parity.nativeWasmByteExact),
    targetDecisionPlane: output.firstDivergence.targetDecisionPlane,
    substitutionAffectsLayoutMetric: output.formatComparison.substitutionAffectsLayoutMetric,
    canonicalSha256: output.canonicalSha256,
  })}\n`);
}

if (process.argv[1] && path.resolve(process.argv[1]) === SCRIPT_PATH) {
  main().catch(error => {
    process.stderr.write(`${error instanceof Error ? error.message : String(error)}\n`);
    process.exitCode = 1;
  });
}
