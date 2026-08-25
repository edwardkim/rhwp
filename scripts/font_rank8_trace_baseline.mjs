#!/usr/bin/env node

import { spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import fs from 'node:fs';
import path from 'node:path';
import process from 'node:process';
import { fileURLToPath, pathToFileURL } from 'node:url';

const SCRIPT_PATH = fileURLToPath(import.meta.url);
const ROOT = path.resolve(path.dirname(SCRIPT_PATH), '..');
const TARGET_FACE = 'KoPubWorld바탕체 Light';
const FIXTURE_SHA256 = 'f6edc8fc43dfd3256385e9752979c14a7041e50c06d36be47cef6e3486835084';
const MAX_INPUT_BYTES = 64 * 1024 * 1024;
const MAX_NATIVE_OUTPUT_BYTES = 64 * 1024 * 1024;
const ABSOLUTE_PATH = /^(?:\/|[A-Za-z]:[\\/]|\\\\)/;
const HWPUNIT_TO_PX = 96 / 7200;

function isObject(value) {
  return value !== null && typeof value === 'object' && !Array.isArray(value);
}

function compareText(left, right) {
  return Buffer.from(String(left), 'utf8').compare(Buffer.from(String(right), 'utf8'));
}

export function canonical(value) {
  if (Array.isArray(value)) return value.map(canonical);
  if (isObject(value)) {
    return Object.fromEntries(
      Object.keys(value).sort(compareText).map(key => [key, canonical(value[key])]),
    );
  }
  return value;
}

export function canonicalJson(value) {
  return `${JSON.stringify(canonical(value))}\n`;
}

export function sha256(value) {
  return createHash('sha256').update(value).digest('hex');
}

function regularInput(inputPath, maximumBytes = MAX_INPUT_BYTES) {
  const stats = fs.lstatSync(inputPath);
  if (stats.isSymbolicLink() || !stats.isFile()) {
    throw new Error(`input must be a regular non-symlink file: ${inputPath}`);
  }
  if (stats.size <= 0 || stats.size > maximumBytes) {
    throw new Error(`input byte limit exceeded: ${inputPath}`);
  }
  return path.resolve(inputPath);
}

function readJson(inputPath, maximumBytes = MAX_INPUT_BYTES) {
  return JSON.parse(fs.readFileSync(regularInput(inputPath, maximumBytes), 'utf8'));
}

function sha256File(inputPath, maximumBytes = MAX_INPUT_BYTES) {
  return sha256(fs.readFileSync(regularInput(inputPath, maximumBytes)));
}

function rejectAbsolutePaths(value, label = 'public') {
  if (Array.isArray(value)) {
    value.forEach((child, index) => rejectAbsolutePaths(child, `${label}[${index}]`));
  } else if (isObject(value)) {
    Object.entries(value).forEach(([key, child]) => (
      rejectAbsolutePaths(child, `${label}.${key}`)
    ));
  } else if (typeof value === 'string' && ABSOLUTE_PATH.test(value)) {
    throw new Error(`${label} exposes an absolute path`);
  }
}

function repoRelative(inputPath) {
  const relative = path.relative(ROOT, path.resolve(inputPath));
  if (!relative || relative.startsWith('..') || path.isAbsolute(relative)) {
    throw new Error(`artifact is outside the repository: ${inputPath}`);
  }
  return relative.split(path.sep).join('/');
}

function requireEqual(actual, expected, label) {
  if (canonicalJson(actual) !== canonicalJson(expected)) {
    throw new Error(`${label} mismatch`);
  }
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

function rounded(value) {
  return Math.round((value + Number.EPSILON) * 10) / 10;
}

export function summarizeTrace(trace, expectedFace = TARGET_FACE) {
  if (!isObject(trace) || trace.schemaVersion !== 1 || trace.status !== 'complete') {
    throw new Error('font trace must be a complete schema-v1 object');
  }
  if (!Array.isArray(trace.records) || trace.records.length === 0) {
    throw new Error('font trace records are missing');
  }
  const counts = trace.counts ?? {};
  if (counts.recordsEmitted !== trace.records.length
      || counts.recordsOmitted !== 0
      || counts.charactersSeen !== trace.records.length) {
    throw new Error('font trace is truncated or its counts are inconsistent');
  }
  if (!trace.records.every(record => record?.document?.face === expectedFace)) {
    throw new Error('font trace contains a non-target document face');
  }
  const metricEntries = counted(trace.records.map(record => record?.layoutMetric?.metricEntry));
  const matchKinds = counted(trace.records.map(record => record?.layoutMetric?.matchKind));
  const widthSources = counted(trace.records.map(record => record?.layoutMetric?.widthSource));
  return {
    records: trace.records.length,
    charactersSeen: counts.charactersSeen,
    recordsOmitted: counts.recordsOmitted,
    layoutHash: trace.layoutHash?.value ?? null,
    normalizedHash: trace.normalizedHash?.value ?? null,
    metricEntries,
    matchKinds,
    widthSources,
    backendSummary: trace.backendSummary,
    canonicalTraceSha256: sha256(canonicalJson(trace)),
  };
}

export function assertTraceParity(nativeTrace, wasmTrace) {
  const nativeCanonical = canonicalJson(nativeTrace);
  const wasmCanonical = canonicalJson(wasmTrace);
  if (nativeCanonical !== wasmCanonical) {
    throw new Error(
      `native/WASM trace mismatch: ${sha256(nativeCanonical)} != ${sha256(wasmCanonical)}`,
    );
  }
  return sha256(nativeCanonical);
}

export function summarizeFixedGeometry(layoutEnvelope, manifest) {
  if (!Array.isArray(layoutEnvelope?.runs) || layoutEnvelope.runs.length === 0) {
    throw new Error('text layout runs are missing');
  }
  const definitions = [
    { context: 'table-cell', parentParaIdx: 19, contentWidthHwpunit: 28980 },
    { context: 'text-box', parentParaIdx: 20, contentWidthHwpunit: 29434 },
  ];
  const matrix = new Map(manifest.semantic.matrix.map(entry => [entry.charPropertyId, entry]));
  const contexts = manifest.semantic.contexts.filter(entry => (
    entry.context === 'table-cell' || entry.context === 'text-box'
  ));
  const output = [];
  for (const definition of definitions) {
    const contextRows = contexts.filter(entry => entry.context === definition.context);
    if (contextRows.length !== 3) {
      throw new Error(`${definition.context} fixture context count mismatch`);
    }
    for (let representativeIndex = 0; representativeIndex < 3; representativeIndex += 1) {
      const runs = layoutEnvelope.runs.filter(run => (
        run?.parentParaIdx === definition.parentParaIdx
        && run?.cellParaIdx === representativeIndex
      ));
      if (runs.length === 0 || !runs.every(run => (
        Number.isFinite(run.x) && Number.isFinite(run.y)
        && Number.isFinite(run.w) && run.w >= 0
      ))) {
        throw new Error(`${definition.context}[${representativeIndex}] layout is incomplete`);
      }
      const lines = new Map();
      for (const run of runs) {
        const line = lines.get(run.y) ?? { minimumX: run.x, maximumX: run.x + run.w };
        line.minimumX = Math.min(line.minimumX, run.x);
        line.maximumX = Math.max(line.maximumX, run.x + run.w);
        lines.set(run.y, line);
      }
      const lineWidths = [...lines.values()].map(line => line.maximumX - line.minimumX);
      const maximumLineWidthPx = Math.max(...lineWidths);
      const contentWidthPx = definition.contentWidthHwpunit * HWPUNIT_TO_PX;
      const contextRow = contextRows[representativeIndex];
      const style = matrix.get(contextRow.charPropertyId);
      if (!style) throw new Error('fixed-context character property is missing');
      output.push({
        context: definition.context,
        representativeIndex,
        lineSegLane: contextRow.lineSegLane,
        ratio: style.ratio,
        spacing: style.spacing,
        kerning: style.kerning,
        contentWidthHwpunit: definition.contentWidthHwpunit,
        contentWidthPx: rounded(contentWidthPx),
        lineCount: lines.size,
        maximumLineWidthPx: rounded(maximumLineWidthPx),
        minimumSlackPx: rounded(contentWidthPx - maximumLineWidthPx),
        crossesFrame: maximumLineWidthPx > contentWidthPx,
      });
    }
  }
  return output;
}

function validateQ0(q0) {
  if (q0?.schemaVersion !== 1
      || q0?.kind !== 'font-rank8-qualification-baseline'
      || q0?.issue !== 4967
      || q0?.target?.documentFace !== TARGET_FACE
      || q0?.executionPolicy?.hyperVOracleRerun !== false) {
    throw new Error('W8-Q0 baseline identity mismatch');
  }
  const body = structuredClone(q0);
  const claimed = body.canonicalSha256;
  delete body.canonicalSha256;
  if (claimed !== sha256(canonicalJson(body))) {
    throw new Error('W8-Q0 canonical SHA-256 drifted');
  }
}

function validateFixture(manifest, fixturePath) {
  if (manifest?.schemaVersion !== 1
      || manifest?.kind !== 'font-oracle-typesetting-fixture-manifest'
      || manifest?.issue !== 4963
      || manifest?.semantic?.documentFace !== TARGET_FACE
      || manifest?.semantic?.queueRank !== 8
      || manifest?.semantic?.substitutionFace !== 'KoPubWorld돋움체 Light'
      || manifest?.semantic?.fontBytesEmbedded !== false) {
    throw new Error('rank-8 qualification fixture manifest identity mismatch');
  }
  if (manifest.inputSha256 !== sha256File(fixturePath)) {
    throw new Error('rank-8 qualification fixture SHA-256 drifted');
  }
  if (manifest.inputSha256 !== FIXTURE_SHA256) {
    throw new Error('rank-8 qualification fixture is not the sealed W5 artifact');
  }
  requireEqual(
    manifest.lineSegLaneCounts,
    { 'fresh-candidate-lane': 12, 'stored-line-lane': 14 },
    'rank-8 LineSeg lanes',
  );
}

function runNative(nativeBin, fixturePath) {
  const result = spawnSync(
    regularInput(nativeBin, 512 * 1024 * 1024),
    [fixturePath, '--page', '0', '--max-characters', '4096', '--json'],
    {
      cwd: ROOT,
      encoding: 'utf8',
      maxBuffer: MAX_NATIVE_OUTPUT_BYTES,
      shell: false,
    },
  );
  if (result.error) throw result.error;
  if (result.status !== 0) {
    throw new Error(`native trace failed (${result.status}): ${result.stderr}`);
  }
  const envelope = JSON.parse(result.stdout);
  if (envelope?.tool !== 'rhwp-q-font-trace' || !isObject(envelope.trace)) {
    throw new Error('native trace envelope identity mismatch');
  }
  return envelope;
}

function runLayout(layoutBin, fixturePath) {
  const result = spawnSync(
    regularInput(layoutBin, 512 * 1024 * 1024),
    [fixturePath, '--page', '0', '--json'],
    {
      cwd: ROOT,
      encoding: 'utf8',
      maxBuffer: MAX_NATIVE_OUTPUT_BYTES,
      shell: false,
    },
  );
  if (result.error) throw result.error;
  if (result.status !== 0) {
    throw new Error(`native text layout failed (${result.status}): ${result.stderr}`);
  }
  const envelope = JSON.parse(result.stdout);
  if (envelope?.tool !== 'rhwp-q-text-layout' || !Array.isArray(envelope.runs)) {
    throw new Error('native text-layout envelope identity mismatch');
  }
  return envelope;
}

async function runWasm(wasmJs, wasmBinary, fixturePath) {
  const module = await import(`${pathToFileURL(regularInput(wasmJs)).href}?w8=${Date.now()}`);
  await module.default({
    module_or_path: fs.readFileSync(regularInput(wasmBinary)),
  });
  const document = new module.HwpDocument(
    new Uint8Array(fs.readFileSync(regularInput(fixturePath))),
  );
  try {
    return JSON.parse(document.getFontDecisionTrace(0, JSON.stringify({ maxCharacters: 4096 })));
  } finally {
    document.free();
  }
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
    'fixture',
    'manifest',
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
  const fixturePath = regularInput(args.fixture);
  const manifestPath = regularInput(args.manifest);
  const q0Path = regularInput(args.q0);
  const nativeBin = regularInput(args['native-bin'], 512 * 1024 * 1024);
  const layoutBin = regularInput(args['layout-bin'], 512 * 1024 * 1024);
  const wasmJs = regularInput(args['wasm-js']);
  const wasmBinary = regularInput(args['wasm-binary']);
  const manifest = readJson(manifestPath);
  const q0 = readJson(q0Path);
  validateQ0(q0);
  validateFixture(manifest, fixturePath);

  const nativeEnvelope = runNative(nativeBin, fixturePath);
  const layoutEnvelope = runLayout(layoutBin, fixturePath);
  const wasmTrace = await runWasm(wasmJs, wasmBinary, fixturePath);
  const nativeSummary = summarizeTrace(nativeEnvelope.trace);
  const wasmSummary = summarizeTrace(wasmTrace);
  const paritySha256 = assertTraceParity(nativeEnvelope.trace, wasmTrace);
  requireEqual(nativeSummary, wasmSummary, 'native/WASM trace summary');
  const fixedGeometry = summarizeFixedGeometry(layoutEnvelope, manifest);

  const output = {
    schemaVersion: 1,
    kind: 'font-rank8-current-trace-baseline',
    issue: 4967,
    stage: 'W8-Q1',
    target: { documentFace: TARGET_FACE, queueRank: 8 },
    inputs: {
      fixture: {
        artifact: repoRelative(fixturePath),
        producerIssue: manifest.issue,
        consumerIssue: 4967,
        sha256: sha256File(fixturePath),
      },
      fixtureManifest: {
        artifact: repoRelative(manifestPath),
        sha256: sha256File(manifestPath),
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
    trace: nativeSummary,
    fixedGeometry,
    parity: {
      nativeWasmByteExact: true,
      canonicalTraceSha256: paritySha256,
      maxCharacters: 4096,
    },
    currentDisposition: {
      layoutMetricEntryPresent: nativeSummary.metricEntries.some(entry => entry.value !== null),
      matchKinds: nativeSummary.matchKinds,
      targetDecisionPlaneHypothesis: 'layout-metric',
      hypothesisStatus: 'qualification-required',
    },
    executionPolicy: {
      fullCorpusRerun: false,
      hyperVOracleRerun: false,
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
    records: output.trace.records,
    nativeWasmByteExact: true,
    canonicalSha256: output.canonicalSha256,
    hyperVOracleRerun: false,
  })}\n`);
}

if (process.argv[1] && path.resolve(process.argv[1]) === SCRIPT_PATH) {
  main().catch(error => {
    process.stderr.write(`${error instanceof Error ? error.message : String(error)}\n`);
    process.exitCode = 1;
  });
}
