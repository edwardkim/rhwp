#!/usr/bin/env node

import crypto from 'node:crypto';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import { pathToFileURL } from 'node:url';

const repoRoot = path.resolve(path.dirname(new URL(import.meta.url).pathname), '..');

const defaults = {
  fixture: path.join(repoRoot, 'mydocs/tech/investigations/issue-4963/fixtures/oracle_typesetting_fixture.hwpx'),
  manifest: path.join(repoRoot, 'mydocs/tech/investigations/issue-4963/fixtures/oracle_typesetting_fixture.manifest.json'),
  native: path.join(repoRoot, 'target/release/rhwp'),
  qkit: path.join(repoRoot, 'target/release/rhwp-q-kit'),
  pkg: path.join(repoRoot, 'pkg'),
  output: path.join(repoRoot, 'mydocs/tech/investigations/issue-4968/kerning_q1_baseline.json'),
};

export function canonical(value) {
  if (Array.isArray(value)) return value.map(canonical);
  if (value && typeof value === 'object') {
    return Object.fromEntries(Object.keys(value).sort().map((key) => [key, canonical(value[key])]));
  }
  return value;
}

export function canonicalJson(value) {
  return `${JSON.stringify(canonical(value))}\n`;
}

export function sha256(value) {
  return crypto.createHash('sha256').update(value).digest('hex');
}

export function normalizePlatformSentinels(value, differences = { count: 0 }) {
  if (Array.isArray(value)) return value.map((item) => normalizePlatformSentinels(item, differences));
  if (value && typeof value === 'object') {
    return Object.fromEntries(Object.entries(value).map(([key, item]) => [
      key,
      normalizePlatformSentinels(item, differences),
    ]));
  }
  if (typeof value === 'string'
      && /\/para:(?:18446744073709551615|4294967295)\/char:/.test(value)) {
    differences.count += 1;
    return value.replace(/\/para:(?:18446744073709551615|4294967295)\/char:/g, '/para:MAX/char:');
  }
  return value;
}

function visit(value, callback) {
  if (!value || typeof value !== 'object') return;
  callback(value);
  if (Array.isArray(value)) {
    for (const item of value) visit(item, callback);
  } else {
    for (const item of Object.values(value)) visit(item, callback);
  }
}

export function projectKerningOffRuns(tree) {
  const runs = [];
  visit(tree, (value) => {
    if (value.type !== 'textRun' || typeof value.text !== 'string') return;
    if (!/^BODY R(?:100|90|80) S(?:0|-5|-10) K0 \| $/.test(value.text)) return;
    if (Object.hasOwn(value.style ?? {}, 'kerning')
        || Object.hasOwn(value.paintStyle ?? {}, 'kerning')) {
      throw new Error('W9-Q1 current off-baseline unexpectedly exposes kerning');
    }
    runs.push({
      label: value.text.trimEnd(),
      positions: value.positions,
      styleSha256: sha256(canonicalJson(value.style)),
      paintStyleSha256: sha256(canonicalJson(value.paintStyle)),
    });
  });
  runs.sort((left, right) => left.label.localeCompare(right.label, 'en'));
  if (runs.length !== 9) throw new Error(`expected 9 BODY K0 runs, got ${runs.length}`);
  return runs;
}

function run(binary, args, options = {}) {
  const result = spawnSync(binary, args, {
    encoding: options.encoding ?? null,
    env: options.env ?? process.env,
    maxBuffer: 256 * 1024 * 1024,
  });
  if (result.status !== 0) {
    const stderr = Buffer.isBuffer(result.stderr) ? result.stderr.toString() : result.stderr;
    throw new Error(`${path.basename(binary)} failed (${result.status}): ${stderr}`);
  }
  return result.stdout;
}

function relative(file) {
  return path.relative(repoRoot, file).split(path.sep).join('/');
}

function fileDigest(file) {
  const bytes = fs.readFileSync(file);
  return { path: relative(file), bytes: bytes.length, sha256: sha256(bytes) };
}

function parseArgs(argv) {
  const options = { ...defaults };
  for (let index = 0; index < argv.length; index += 1) {
    const key = argv[index];
    if (!key.startsWith('--') || !(key.slice(2) in options)) {
      throw new Error(`unknown option: ${key}`);
    }
    const value = argv[++index];
    if (!value) throw new Error(`missing value for ${key}`);
    options[key.slice(2)] = path.resolve(value);
  }
  return options;
}

function assertSourceBoundary() {
  const sources = [
    ['src/renderer/style_resolver.rs', 'pub kerning: bool'],
    ['src/renderer/mod.rs', 'pub struct TextStyle'],
    ['src/renderer/layout/text_measurement.rs', 'pub(crate) fn resolved_to_text_style'],
    ['src/paint/paint_op.rs', 'pub struct PaintTextStyle'],
    ['src/paint/json.rs', '\\\"type\\\":\\\"glyphRun\\\"'],
    ['src/renderer/font_metrics_data.rs', 'pub struct FontMetric'],
  ];
  return sources.map(([name, needle]) => {
    const file = path.join(repoRoot, name);
    const text = fs.readFileSync(file, 'utf8');
    if (!text.includes(needle)) throw new Error(`source boundary marker missing: ${name}: ${needle}`);
    return fileDigest(file);
  });
}

function scrubRhwpEnv() {
  const env = { ...process.env };
  for (const key of Object.keys(env)) if (key.startsWith('RHWP_')) delete env[key];
  return env;
}

export async function generateBaseline(options) {
  const fixtureBytes = fs.readFileSync(options.fixture);
  const manifestBytes = fs.readFileSync(options.manifest);
  const manifest = JSON.parse(manifestBytes);
  if (manifest.inputSha256 !== sha256(fixtureBytes)) throw new Error('fixture digest does not match manifest');

  const env = scrubRhwpEnv();
  const nativeEnvelopeBytes = run(options.qkit, [
    'layer-tree', options.fixture, '--page', '0', '--json',
  ], { env });
  const nativeEnvelope = JSON.parse(nativeEnvelopeBytes);
  const nativeTree = nativeEnvelope.tree;

  const wasmJs = path.join(options.pkg, 'rhwp.js');
  const wasmBlob = path.join(options.pkg, 'rhwp_bg.wasm');
  const wasmModule = await import(`${pathToFileURL(wasmJs).href}?w9q1=${Date.now()}`);
  await wasmModule.default({ module_or_path: fs.readFileSync(wasmBlob) });
  const document = new wasmModule.HwpDocument(new Uint8Array(fixtureBytes));
  let wasmTreeText;
  let wasmSvg;
  try {
    wasmTreeText = document.getPageLayerTree(0);
    wasmSvg = document.renderPageSvg(0);
  } finally {
    document.free();
  }
  const wasmTree = JSON.parse(wasmTreeText);

  const nativeSentinels = { count: 0 };
  const wasmSentinels = { count: 0 };
  const normalizedNativeTree = normalizePlatformSentinels(nativeTree, nativeSentinels);
  const normalizedWasmTree = normalizePlatformSentinels(wasmTree, wasmSentinels);
  const nativeCanonical = canonicalJson(normalizedNativeTree);
  const wasmCanonical = canonicalJson(normalizedWasmTree);
  if (nativeCanonical !== wasmCanonical) throw new Error('native/WASM normalized layer-tree mismatch');

  const nativeRuns = projectKerningOffRuns(nativeTree);
  const wasmRuns = projectKerningOffRuns(wasmTree);
  if (canonicalJson(nativeRuns) !== canonicalJson(wasmRuns)) {
    throw new Error('native/WASM K0 run position mismatch');
  }

  const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), 'rhwp-w9-q1-'));
  let nativeSvg;
  try {
    const manifestText = run(options.native, [
      'export-svg', options.fixture, '--json', '-o', tempDir,
    ], { env, encoding: 'utf8' });
    const exportManifest = JSON.parse(manifestText);
    if (exportManifest.pageCount !== 1 || exportManifest.pages.length !== 1) {
      throw new Error('W5 fixture must export exactly one page');
    }
    nativeSvg = fs.readFileSync(exportManifest.pages[0].path);
  } finally {
    fs.rmSync(tempDir, { recursive: true, force: true });
  }
  const wasmSvgBytes = Buffer.from(wasmSvg);
  if (!nativeSvg.equals(wasmSvgBytes)) throw new Error('native/WASM SVG byte mismatch');

  const firstTextRun = (() => {
    let found;
    visit(nativeTree, (value) => {
      if (!found && value.type === 'textRun') found = value;
    });
    return found;
  })();
  if (!firstTextRun) throw new Error('fixture has no textRun');

  const nativeVersion = run(options.native, ['--version'], { env, encoding: 'utf8' }).trim();
  const output = {
    schemaVersion: 1,
    issue: 4968,
    stage: 'W9-Q1',
    status: 'complete',
    productMutationCount: 0,
    inputs: {
      fixture: fileDigest(options.fixture),
      manifest: fileDigest(options.manifest),
      sourceBoundary: assertSourceBoundary(),
    },
    build: {
      nativeVersion,
      wasmVersion: wasmModule.version(),
      wasmBlobBytes: fs.statSync(wasmBlob).size,
      wasmBlobSha256: sha256(fs.readFileSync(wasmBlob)),
      dockerWasmCommand: 'docker compose --env-file .env.docker run --rm wasm',
    },
    currentSerialization: {
      textStyleFields: Object.keys(firstTextRun.style).sort(),
      paintTextStyleFields: Object.keys(firstTextRun.paintStyle).sort(),
      textRunFields: Object.keys(firstTextRun).sort(),
      glyphRunFields: [
        'advances', 'bbox', 'bidiLevel', 'clusters', 'diagnostics', 'direction', 'glyphIds',
        'glyphTransforms', 'orientation', 'paintStyle', 'placement', 'positions', 'shapeKey',
        'source', 'type', 'variant', 'writingMode',
      ],
      kerningFieldPresent: false,
    },
    offBaseline: {
      page: 0,
      selectedRunCount: nativeRuns.length,
      selectedRuns: nativeRuns,
      nativeLayerTreeSha256: sha256(Buffer.from(JSON.stringify(nativeTree))),
      wasmLayerTreeSha256: sha256(Buffer.from(wasmTreeText)),
      rawLayerTreeByteEqual: Buffer.from(JSON.stringify(nativeTree)).equals(Buffer.from(wasmTreeText)),
      platformSentinel: {
        reason: 'target-usize-max-width',
        nativeOccurrenceCount: nativeSentinels.count,
        wasmOccurrenceCount: wasmSentinels.count,
        normalizedValue: 'para:MAX',
      },
      normalizedLayerTreeSha256: sha256(nativeCanonical),
      normalizedLayerTreeEqual: true,
      selectedRunPositionsEqual: true,
      svg: {
        bytes: nativeSvg.length,
        sha256: sha256(nativeSvg),
        byteEqual: true,
      },
    },
    capabilityContract: {
      request: ['disabled', 'enabled'],
      capability: ['gpos-kern', 'legacy-kern', 'unsupported'],
      disposition: ['not-requested', 'applied', 'no-pair-adjustment', 'fail-closed'],
      precedence: ['gpos-kern', 'legacy-kern', 'unsupported'],
      requiredTraceFields: [
        'request', 'capability', 'disposition', 'fontSourceSha256', 'faceIndex',
        'pairCountSeen', 'pairCountAdjusted', 'totalAdvanceAdjustment', 'fallbackReason',
      ],
      bounds: {
        maxFontBytes: 33554432,
        maxCodePointsPerRun: 4096,
        maxGlyphsPerRun: 4096,
        maxPairsPerRun: 4095,
        maxTraceRecordsPerRun: 4096,
        overflowDisposition: 'fail-closed',
        overflowLayout: 'preserve-current-advance',
      },
      backends: {
        commonLayout: 'decide-pair-adjustments-once',
        svg: 'replay-common-positions',
        canvas2d: 'replay-common-positions',
        canvaskit: 'replay-common-positions',
        nativeSkia: 'replay-common-positions',
        backendReshapingAllowed: false,
      },
    },
    engineDecision: {
      selected: 'rustybuzz-qualified-candidate',
      directDependencyAtQ1: false,
      currentWasmDependencyPresent: false,
      coverage: ['GPOS-kern', 'legacy-kern'],
      rejectedAlternative: 'limited-pair-parser',
      rejectionReasons: [
        'class-pair-and-extension-lookup-coverage-risk',
        'script-and-language-feature-routing-risk',
        'legacy-kern-format-coverage-risk',
      ],
      qualificationGates: [
        'Q2-public-pair-fixture',
        'Q3-native-wasm-size-delta',
        'Q3-bounded-performance',
        'glyph-identity-and-cluster-stability-or-fail-closed',
      ],
    },
    invariants: [
      'kerning-disabled-preserves-current-layout-and-serialization',
      'unsupported-or-untrusted-source-preserves-current-advance',
      'pair-decision-is-common-and-backends-do-not-reshape',
      'stored-and-fresh-lanes-remain-separate',
    ],
  };
  output.canonicalSha256 = sha256(canonicalJson(output));
  return output;
}

async function main() {
  const options = parseArgs(process.argv.slice(2));
  const output = await generateBaseline(options);
  fs.mkdirSync(path.dirname(options.output), { recursive: true });
  fs.writeFileSync(options.output, JSON.stringify(output, null, 2) + '\n');
  process.stdout.write(JSON.stringify({
    output: relative(options.output),
    canonicalSha256: output.canonicalSha256,
    selectedRunCount: output.offBaseline.selectedRunCount,
    normalizedLayerTreeEqual: output.offBaseline.normalizedLayerTreeEqual,
    svgByteEqual: output.offBaseline.svg.byteEqual,
  }) + '\n');
}

if (process.argv[1] && path.resolve(process.argv[1]) === path.resolve(new URL(import.meta.url).pathname)) {
  main().catch((error) => {
    process.stderr.write(`${error.stack ?? error}\n`);
    process.exitCode = 1;
  });
}
