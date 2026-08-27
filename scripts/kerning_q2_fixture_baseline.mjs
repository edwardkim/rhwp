#!/usr/bin/env node

import crypto from 'node:crypto';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import { pathToFileURL } from 'node:url';

import {
  canonicalJson,
  normalizePlatformSentinels,
  sha256,
} from './kerning_q1_baseline.mjs';

const repoRoot = path.resolve(path.dirname(new URL(import.meta.url).pathname), '..');
const defaults = {
  fixture: path.join(repoRoot, 'mydocs/tech/investigations/issue-4968/fixtures/kerning_pair_fixture.hwpx'),
  manifest: path.join(repoRoot, 'mydocs/tech/investigations/issue-4968/fixtures/kerning_pair_fixture.manifest.json'),
  boundary: path.join(repoRoot, 'mydocs/tech/investigations/issue-4968/kerning_capability_boundary.json'),
  adjudication: path.join(
    repoRoot,
    'mydocs/tech/investigations/issue-4968/kerning_q2_hancom_adjudication.json',
  ),
  native: path.join(repoRoot, 'target/release/rhwp'),
  qkit: path.join(repoRoot, 'target/release/rhwp-q-kit'),
  pkg: path.join(repoRoot, 'pkg'),
  output: path.join(repoRoot, 'mydocs/tech/investigations/issue-4968/kerning_q2_fixture_baseline.json'),
};

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

function scrubRhwpEnv() {
  const env = { ...process.env };
  for (const key of Object.keys(env)) if (key.startsWith('RHWP_')) delete env[key];
  return env;
}

function relative(file) {
  return path.relative(repoRoot, file).split(path.sep).join('/');
}

function digest(file) {
  const bytes = fs.readFileSync(file);
  return { path: relative(file), bytes: bytes.length, sha256: sha256(bytes) };
}

function parseArgs(argv) {
  const options = { ...defaults };
  for (let index = 0; index < argv.length; index += 1) {
    const key = argv[index];
    if (!key.startsWith('--') || !(key.slice(2) in options)) throw new Error(`unknown option: ${key}`);
    const value = argv[++index];
    if (!value) throw new Error(`missing value for ${key}`);
    options[key.slice(2)] = path.resolve(value);
  }
  return options;
}

function visit(value, callback) {
  if (!value || typeof value !== 'object') return;
  callback(value);
  for (const child of Array.isArray(value) ? value : Object.values(value)) visit(child, callback);
}

function round(value) {
  return Math.round(value * 1000) / 1000;
}

export function projectBodyRuns(tree, sample = 'AV To WA HH') {
  const rows = [];
  visit(tree, (value) => {
    if (value.type !== 'textRun' || typeof value.text !== 'string') return;
    const match = value.text.match(/^BODY R(100|90|80) S(0|-5|-10) K([01]) L(stored|fresh) \| /);
    if (!match) return;
    const sampleStart = value.text.indexOf(sample);
    const sampleLength = Array.from(sample).length;
    if (sampleStart < 0 || value.positions.length < sampleStart + sampleLength) {
      throw new Error(`sample position range missing: ${value.text}`);
    }
    const base = value.positions[sampleStart];
    rows.push({
      ratio: Number(match[1]),
      spacing: Number(match[2]),
      kerning: match[3] === '1',
      lane: match[4] === 'stored' ? 'stored-line-lane' : 'fresh-candidate-lane',
      fontFamily: value.style.fontFamily,
      fontSize: value.style.fontSize,
      sample,
      samplePositions: value.positions
        .slice(sampleStart, sampleStart + sampleLength + 1)
        .map((position) => round(position - base)),
    });
  });
  rows.sort((left, right) => (
    right.ratio - left.ratio
      || right.spacing - left.spacing
      || Number(left.kerning) - Number(right.kerning)
  ));
  if (rows.length !== 18) throw new Error(`expected 18 BODY pair runs, got ${rows.length}`);
  return rows;
}

export function assertCurrentOnOffIdentity(rows) {
  const groups = new Map();
  for (const row of rows) {
    const key = `${row.ratio}/${row.spacing}/${row.lane}`;
    const group = groups.get(key) ?? [];
    group.push(row);
    groups.set(key, group);
  }
  if (groups.size !== 9) throw new Error(`expected 9 ratio/spacing/lane groups, got ${groups.size}`);
  const summary = [];
  for (const [key, group] of groups) {
    if (group.length !== 2 || group[0].kerning === group[1].kerning) {
      throw new Error(`invalid on/off group: ${key}`);
    }
    if (canonicalJson(group[0].samplePositions) !== canonicalJson(group[1].samplePositions)) {
      throw new Error(`current product unexpectedly changes K0/K1 positions: ${key}`);
    }
    summary.push({
      ratio: group[0].ratio,
      spacing: group[0].spacing,
      lane: group[0].lane,
      currentOnOffEqual: true,
      positionsSha256: sha256(canonicalJson(group[0].samplePositions)),
    });
  }
  summary.sort((left, right) => right.ratio - left.ratio || right.spacing - left.spacing);
  return summary;
}

function checkedBoundary(file) {
  const boundary = JSON.parse(fs.readFileSync(file));
  const claimed = boundary.canonicalSha256;
  delete boundary.canonicalSha256;
  if (sha256(canonicalJson(boundary)) !== claimed) throw new Error('Q2 boundary canonical hash drift');
  if (boundary.publicFont.capability !== 'gpos-kern') throw new Error('public font must be GPOS kern');
  return { boundary: { ...boundary, canonicalSha256: claimed }, claimed };
}

function checkedAdjudication(file, manifest, boundary) {
  const adjudication = JSON.parse(fs.readFileSync(file));
  const claimed = adjudication.canonicalSha256;
  delete adjudication.canonicalSha256;
  if (sha256(canonicalJson(adjudication)) !== claimed) {
    throw new Error('Q2 Hancom adjudication canonical hash drift');
  }
  if (adjudication.issue !== 4968 || adjudication.stage !== 'W9-Q2') {
    throw new Error('Q2 Hancom adjudication identity mismatch');
  }
  if (adjudication.inputs.fixtureSha256 !== manifest.inputSha256
      || adjudication.inputs.fontSha256 !== boundary.publicFont.sha256) {
    throw new Error('Q2 Hancom adjudication input identity mismatch');
  }
  if (adjudication.adjudication.featureFlagSurvivesOpen !== true) {
    throw new Error('Q2 Hancom adjudication did not preserve the feature flag');
  }
  return { adjudication: { ...adjudication, canonicalSha256: claimed }, claimed };
}

function expectedScaledDeltas(boundary, fontSize) {
  const values = Object.fromEntries(boundary.publicFont.pairs.map((pair) => [pair.text, pair.totalXAdvance]));
  return [100, 90, 80].map((ratio) => ({
    ratio,
    spacingIndependent: true,
    pairs: Object.fromEntries(Object.entries(values).map(([pair, units]) => [
      pair,
      round(units * fontSize * (ratio / 100) / boundary.publicFont.unitsPerEm),
    ])),
  }));
}

export async function generateBaseline(options) {
  const fixtureBytes = fs.readFileSync(options.fixture);
  const manifest = JSON.parse(fs.readFileSync(options.manifest));
  if (manifest.inputSha256 !== sha256(fixtureBytes)) throw new Error('fixture digest mismatch');
  if (manifest.issue !== 4968 || manifest.stage !== 'W9-Q2') throw new Error('fixture identity mismatch');
  const { boundary, claimed: boundaryCanonicalSha256 } = checkedBoundary(options.boundary);
  if (manifest.semantic.fontSource.sha256 !== boundary.publicFont.sha256) {
    throw new Error('fixture and capability boundary font identity mismatch');
  }
  const {
    adjudication,
    claimed: adjudicationCanonicalSha256,
  } = checkedAdjudication(options.adjudication, manifest, boundary);

  const env = scrubRhwpEnv();
  const nativeEnvelope = JSON.parse(run(options.qkit, [
    'layer-tree', options.fixture, '--page', '0', '--json',
  ], { env }));
  const nativeTree = nativeEnvelope.tree;
  const wasmJs = path.join(options.pkg, 'rhwp.js');
  const wasmModule = await import(`${pathToFileURL(wasmJs).href}?w9q2=${Date.now()}`);
  await wasmModule.default({ module_or_path: fs.readFileSync(path.join(options.pkg, 'rhwp_bg.wasm')) });
  const document = new wasmModule.HwpDocument(new Uint8Array(fixtureBytes));
  let wasmTreeText;
  let wasmSvg;
  try {
    if (document.pageCount() !== 1) throw new Error('Q2 fixture must render as one page');
    wasmTreeText = document.getPageLayerTree(0);
    wasmSvg = document.renderPageSvg(0);
  } finally {
    document.free();
  }
  const wasmTree = JSON.parse(wasmTreeText);
  const nativeSentinels = { count: 0 };
  const wasmSentinels = { count: 0 };
  const nativeCanonical = canonicalJson(normalizePlatformSentinels(nativeTree, nativeSentinels));
  const wasmCanonical = canonicalJson(normalizePlatformSentinels(wasmTree, wasmSentinels));
  if (nativeCanonical !== wasmCanonical) throw new Error('native/WASM normalized Q2 tree mismatch');

  const nativeRows = projectBodyRuns(nativeTree);
  const wasmRows = projectBodyRuns(wasmTree);
  if (canonicalJson(nativeRows) !== canonicalJson(wasmRows)) throw new Error('native/WASM Q2 body mismatch');
  const currentGroups = assertCurrentOnOffIdentity(nativeRows);

  const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), 'rhwp-w9-q2-'));
  let nativeSvg;
  try {
    const exportManifest = JSON.parse(run(options.native, [
      'export-svg', options.fixture, '--json', '-o', tempDir,
    ], { env, encoding: 'utf8' }));
    if (exportManifest.pageCount !== 1) throw new Error('native Q2 SVG must have one page');
    nativeSvg = fs.readFileSync(exportManifest.pages[0].path);
  } finally {
    fs.rmSync(tempDir, { recursive: true, force: true });
  }
  const wasmSvgBytes = Buffer.from(wasmSvg);
  if (!nativeSvg.equals(wasmSvgBytes)) throw new Error('native/WASM Q2 SVG mismatch');

  const fontSize = nativeRows[0].fontSize;
  const output = {
    schemaVersion: 1,
    issue: 4968,
    stage: 'W9-Q2',
    status: 'complete',
    productMutationCount: 0,
    inputs: {
      fixture: digest(options.fixture),
      manifest: digest(options.manifest),
      capabilityBoundary: { ...digest(options.boundary), canonicalSha256: boundaryCanonicalSha256 },
      hancomAdjudication: {
        ...digest(options.adjudication),
        canonicalSha256: adjudicationCanonicalSha256,
      },
    },
    fixture: {
      pageCount: 1,
      matrixCount: manifest.semantic.matrix.length,
      contextCount: manifest.semantic.contexts.length,
      lineSegLaneCounts: manifest.lineSegLaneCounts,
      fontFamily: nativeRows[0].fontFamily,
      fontSize,
      pairText: manifest.semantic.pairText,
    },
    currentProductBaseline: {
      bodyRunCount: nativeRows.length,
      onOffGroupCount: currentGroups.length,
      allCurrentOnOffPositionsEqual: true,
      groups: currentGroups,
      rawLayerTreeByteEqual: Buffer.from(JSON.stringify(nativeTree)).equals(Buffer.from(wasmTreeText)),
      platformSentinel: {
        reason: 'target-usize-max-width',
        nativeOccurrenceCount: nativeSentinels.count,
        wasmOccurrenceCount: wasmSentinels.count,
      },
      normalizedLayerTreeSha256: sha256(nativeCanonical),
      normalizedLayerTreeEqual: true,
      svg: { bytes: nativeSvg.length, sha256: sha256(nativeSvg), byteEqual: true },
    },
    openTypeTruth: {
      capability: boundary.publicFont.capability,
      unitsPerEm: boundary.publicFont.unitsPerEm,
      pairDesignUnits: Object.fromEntries(
        boundary.publicFont.pairs.map((pair) => [pair.text, pair.totalXAdvance]),
      ),
      candidateScaledDeltasPx: expectedScaledDeltas(boundary, fontSize),
      applicationOrderStatus: adjudication.adjudication.applicationOrderStatus,
      candidateOrder: [
        'nominal-advance-and-pair-adjustment-in-font-units',
        'font-size-scale',
        'horizontal-ratio-scale',
        'letter-spacing-per-glyph',
      ],
    },
    edgeCoverage: Object.fromEntries(
      boundary.syntheticCases.map((item) => [item.case, {
        status: item.status,
        capability: item.capability,
        fallbackReason: item.fallbackReason,
      }]),
    ),
    hancomObservation: {
      version: adjudication.environment.hancomVersion,
      featureFlagSurvivesOpen: adjudication.adjudication.featureFlagSurvivesOpen,
      featureFlagCreatesPdfLayoutDifferential:
        adjudication.adjudication.featureFlagCreatesPdfLayoutDifferential,
      controlledGroupCount: adjudication.pdfLayout.controlledGroupCount,
      maximumAbsoluteOnOffDelta: adjudication.pdfLayout.maximumAbsoluteOnOffDelta,
      role: adjudication.adjudication.hancomObservationRole,
    },
    nextGate: 'maintainer-Q2-direction-approval-before-product-mutation',
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
    bodyRunCount: output.currentProductBaseline.bodyRunCount,
    normalizedLayerTreeEqual: output.currentProductBaseline.normalizedLayerTreeEqual,
    svgByteEqual: output.currentProductBaseline.svg.byteEqual,
    nextGate: output.nextGate,
  }) + '\n');
}

if (process.argv[1] && path.resolve(process.argv[1]) === path.resolve(new URL(import.meta.url).pathname)) {
  main().catch((error) => {
    process.stderr.write(`${error.stack ?? error}\n`);
    process.exitCode = 1;
  });
}
