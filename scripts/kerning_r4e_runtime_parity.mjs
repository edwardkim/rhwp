#!/usr/bin/env node

import fs from 'node:fs';
import path from 'node:path';
import { pathToFileURL } from 'node:url';

import {
  canonicalJson,
  normalizePlatformSentinels,
  sha256,
} from './kerning_q1_baseline.mjs';

const repoRoot = path.resolve(path.dirname(new URL(import.meta.url).pathname), '..');

const defaults = {
  fixture: path.join(repoRoot, 'mydocs/tech/investigations/issue-4968/fixtures/kerning_runtime_fixture.hwpx'),
  manifest: path.join(repoRoot, 'mydocs/tech/investigations/issue-4968/fixtures/kerning_runtime_fixture.manifest.json'),
  font: path.join(repoRoot, 'tests/fixtures/fonts/RHWPExactKerningSmoke.ttf'),
  pkg: path.join(repoRoot, 'pkg'),
  nativeProbe: '',
  output: path.join(repoRoot, 'mydocs/tech/investigations/issue-4968/kerning_r4e_runtime_parity.json'),
};

function visit(value, callback) {
  if (!value || typeof value !== 'object') return;
  callback(value);
  for (const child of Array.isArray(value) ? value : Object.values(value)) visit(child, callback);
}

function parseArgs(argv) {
  const options = { ...defaults };
  for (let index = 0; index < argv.length; index += 1) {
    const key = argv[index];
    const name = key.startsWith('--') ? key.slice(2) : '';
    if (!(name in options)) throw new Error(`unknown option: ${key}`);
    const value = argv[++index];
    if (!value) throw new Error(`missing value for ${key}`);
    options[name] = path.resolve(value);
  }
  if (!options.nativeProbe) throw new Error('--nativeProbe is required');
  return options;
}

function readJson(file) {
  return JSON.parse(fs.readFileSync(file, 'utf8'));
}

function digest(bytes) {
  return { bytes: bytes.length, sha256: sha256(bytes) };
}

function snapshotWasm(document) {
  const svg = document.renderPageSvg(0);
  return {
    pageCount: document.pageCount(),
    renderTree: JSON.parse(document.getPageRenderTree(0)),
    layerTree: JSON.parse(document.getPageLayerTree(0)),
    svg: digest(Buffer.from(svg)),
    canvasCommandCount: document.renderPageCanvas(0),
    canvasKit: JSON.parse(document.getCanvasKitReplayPlan(0, 'default')),
  };
}

export async function buildWasmProbe(options) {
  const fixtureBytes = fs.readFileSync(options.fixture);
  const fontBytes = fs.readFileSync(options.font);
  const manifest = readJson(options.manifest);
  if (manifest.inputSha256 !== sha256(fixtureBytes)) throw new Error('fixture digest mismatch');
  if (manifest.semantic.fontSource.sha256 !== sha256(fontBytes)) throw new Error('font digest mismatch');

  const wasmJs = path.join(options.pkg, 'rhwp.js');
  const wasmBlob = path.join(options.pkg, 'rhwp_bg.wasm');
  const wasmModule = await import(`${pathToFileURL(wasmJs).href}?r4e=${Date.now()}`);
  await wasmModule.default({ module_or_path: fs.readFileSync(wasmBlob) });
  const document = new wasmModule.HwpDocument(new Uint8Array(fixtureBytes));
  try {
    const k0 = snapshotWasm(document);
    const registration = manifest.semantic.exactSourceRegistration.slots.map((slot) => JSON.parse(
      document.registerExactFontSource(
        slot.charShapeId,
        slot.languageIndex,
        new Uint8Array(fontBytes),
        manifest.semantic.exactSourceRegistration.faceIndex,
      ),
    ));
    const k1 = snapshotWasm(document);
    return {
      schemaVersion: 1,
      issue: 4968,
      stage: 'W9-Q3-5R4E-1',
      projectionContractSha256: manifest.projectionContractSha256,
      registration,
      k0,
      k1,
    };
  } finally {
    document.free();
  }
}

function sourceOffset(stableSourceKey) {
  const match = /^section:0\/para:\d+\/char:(\d+)/.exec(stableSourceKey);
  if (!match) throw new Error(`unexpected runtime source key: ${stableSourceKey}`);
  return Number(match[1]);
}

function contextMatchesRun(context, stableSourceKey) {
  if (context.context === 'body') {
    return stableSourceKey.startsWith(`section:0/para:${context.charPropertyId - 6}/char:`)
      && !stableSourceKey.includes('/cell:');
  }
  const cellId = context.context === 'table-cell' ? 19 : 20;
  const innerParagraph = context.charPropertyId - 7;
  return stableSourceKey.includes(`/cell:${cellId}:0:0:${innerParagraph}:0`);
}

function projectRows(layerTree, manifest) {
  const runs = [];
  visit(layerTree, (value) => {
    if (value.type === 'textRun'
        && typeof value.source?.stableSourceKey === 'string'
        && Array.isArray(value.positions)
        && value.positions.length > 1) runs.push(value);
  });
  const matrix = new Map(manifest.semantic.matrix.map((row) => [row.charPropertyId, row]));
  return manifest.semantic.contexts.map((context) => {
    const fragments = runs
      .filter((run) => contextMatchesRun(context, run.source.stableSourceKey))
      .map((run) => ({ offset: sourceOffset(run.source.stableSourceKey), run }))
      .sort((left, right) => left.offset - right.offset);
    if (fragments.length === 0) {
      throw new Error(`missing ${context.context} char shape ${context.charPropertyId}`);
    }
    const row = matrix.get(context.charPropertyId);
    if (!row) throw new Error(`missing matrix row ${context.charPropertyId}`);
    const lineStarts = fragments.map((fragment) => fragment.offset);
    const lineBoundaries = fragments.map(
      (fragment) => fragment.offset + fragment.run.positions.length - 1,
    );
    const positions = fragments.map((fragment) => fragment.run.positions);
    const totalWidth = positions.reduce((total, values) => total + values.at(-1), 0);
    const bboxWidth = fragments.reduce((total, fragment) => total + fragment.run.bbox.width, 0);
    return {
      context: context.context,
      charShapeId: context.charPropertyId,
      languageIndex: manifest.semantic.exactSourceRegistration.languageIndex,
      lineSegLane: context.lineSegLane,
      ratio: row.ratio,
      spacing: row.spacing,
      kerningRequested: row.kerning,
      paragraphRef: `paragraph:${context.paragraphId}`,
      measurement: { totalWidth },
      line: { starts: lineStarts, boundaries: lineBoundaries },
      layout: { positions, bboxWidth },
    };
  });
}

function projectRegistration(registrations) {
  if (!Array.isArray(registrations) || registrations.length === 0) {
    throw new Error('registration evidence is empty');
  }
  const first = registrations[0];
  const last = registrations.at(-1);
  return {
    source: {
      sha256: first.handle.fontSourceSha256,
      bytes: first.handle.fontBytes,
      faceIndex: first.handle.faceIndex,
    },
    slots: registrations.map((value) => ({
      charShapeId: value.slot.charShapeId,
      languageIndex: value.slot.languageIndex,
      status: value.status,
    })),
    registry: {
      slotCount: last.registry.slotCount,
      sourceCount: last.registry.sourceCount,
      totalSourceBytes: last.registry.totalSourceBytes,
    },
  };
}

function projectSnapshot(snapshot, manifest) {
  return {
    pageCount: snapshot.pageCount,
    rows: projectRows(snapshot.layerTree, manifest),
    svg: { sha256: snapshot.svg.sha256 },
  };
}

export function projectRuntimeProbe(probe, manifest) {
  return {
    registration: projectRegistration(probe.registration),
    k0: projectSnapshot(probe.k0, manifest),
    k1: projectSnapshot(probe.k1, manifest),
  };
}

function normalizedDigest(value) {
  const differences = { count: 0 };
  const normalized = normalizePlatformSentinels(value, differences);
  return { sha256: sha256(canonicalJson(normalized)), sentinelCount: differences.count };
}

function assertNoForbiddenFields(projection, manifest) {
  const serialized = canonicalJson(projection);
  for (const field of manifest.semantic.canonicalProjectionContract.forbiddenFields) {
    const pattern = new RegExp(`"${field}"\\s*:`, 'i');
    if (pattern.test(serialized)) throw new Error(`forbidden projection field: ${field}`);
  }
}

function rowKey(row) {
  return [row.context, row.charShapeId, row.languageIndex, row.lineSegLane].join('|');
}

function rowMap(rows) {
  return new Map(rows.map((row) => [rowKey(row), row]));
}

function assertK0IdentityAndK1Effect(projection) {
  const before = rowMap(projection.k0.rows);
  let unchangedOffRows = 0;
  let changedOnRows = 0;
  for (const after of projection.k1.rows) {
    const prior = before.get(rowKey(after));
    if (!prior) throw new Error(`missing K0 row ${rowKey(after)}`);
    const equal = canonicalJson(prior) === canonicalJson(after);
    if (after.kerningRequested) {
      if (!equal) changedOnRows += 1;
    } else {
      if (!equal) throw new Error(`kerning-off row changed: ${rowKey(after)}`);
      unchangedOffRows += 1;
    }
  }
  if (unchangedOffRows !== 11) throw new Error(`expected 11 unchanged off rows, got ${unchangedOffRows}`);
  if (changedOnRows === 0) throw new Error('exact registration changed no kerning-requested row');
  return { unchangedOffRows, changedOnRows };
}

function snapshotDiagnostics(nativeSnapshot, wasmSnapshot) {
  const nativeRenderTree = normalizedDigest(nativeSnapshot.renderTree);
  const wasmRenderTree = normalizedDigest(wasmSnapshot.renderTree);
  const nativeLayerTree = normalizedDigest(nativeSnapshot.layerTree);
  const wasmLayerTree = normalizedDigest(wasmSnapshot.layerTree);
  const nativeCanvasKit = normalizedDigest(nativeSnapshot.canvasKit);
  const wasmCanvasKit = normalizedDigest(wasmSnapshot.canvasKit);
  if (nativeRenderTree.sha256 !== wasmRenderTree.sha256) throw new Error('render-tree parity mismatch');
  if (nativeLayerTree.sha256 !== wasmLayerTree.sha256) throw new Error('layer-tree parity mismatch');
  if (nativeCanvasKit.sha256 !== wasmCanvasKit.sha256) throw new Error('CanvasKit plan parity mismatch');
  if (nativeSnapshot.canvasCommandCount !== wasmSnapshot.canvasCommandCount) {
    throw new Error('Canvas command-count parity mismatch');
  }
  if (nativeSnapshot.svg.sha256 !== wasmSnapshot.svg.sha256
      || nativeSnapshot.svg.bytes !== wasmSnapshot.svg.bytes) throw new Error('SVG parity mismatch');
  for (const snapshot of [nativeSnapshot, wasmSnapshot]) {
    if (snapshot.canvasKit.directReplayRequired !== true
        || snapshot.canvasKit.hiddenCanvas2dOverlayAllowed !== false
        || snapshot.canvasKit.summary.hiddenOverlayViolations !== 0) {
      throw new Error('CanvasKit direct replay invariant failed');
    }
  }
  return {
    renderTree: nativeRenderTree,
    layerTree: nativeLayerTree,
    canvasKit: nativeCanvasKit,
    canvasCommandCount: nativeSnapshot.canvasCommandCount,
    svg: nativeSnapshot.svg,
    directReplayRequired: true,
    hiddenOverlayViolations: 0,
  };
}

export function compareRuntimeParity(nativeProbe, wasmProbe, manifest) {
  if (nativeProbe.projectionContractSha256 !== manifest.projectionContractSha256
      || wasmProbe.projectionContractSha256 !== manifest.projectionContractSha256) {
    throw new Error('projection contract digest mismatch');
  }
  const nativeProjection = projectRuntimeProbe(nativeProbe, manifest);
  const wasmProjection = projectRuntimeProbe(wasmProbe, manifest);
  assertNoForbiddenFields(nativeProjection, manifest);
  assertNoForbiddenFields(wasmProjection, manifest);
  const nativeCanonical = canonicalJson(nativeProjection);
  const wasmCanonical = canonicalJson(wasmProjection);
  if (nativeCanonical !== wasmCanonical) throw new Error('native/WASM canonical projection mismatch');
  const effects = assertK0IdentityAndK1Effect(nativeProjection);
  if (nativeProjection.k0.pageCount !== nativeProjection.k1.pageCount) {
    throw new Error('exact registration changed page count');
  }
  if (nativeProjection.k0.svg.sha256 === nativeProjection.k1.svg.sha256) {
    throw new Error('exact registration did not change SVG');
  }
  return {
    projection: nativeProjection,
    canonicalSha256: sha256(nativeCanonical),
    effects,
    diagnostics: {
      k0: snapshotDiagnostics(nativeProbe.k0, wasmProbe.k0),
      k1: snapshotDiagnostics(nativeProbe.k1, wasmProbe.k1),
    },
  };
}

export async function generateRuntimeParity(options) {
  const fixtureBytes = fs.readFileSync(options.fixture);
  const manifestBytes = fs.readFileSync(options.manifest);
  const fontBytes = fs.readFileSync(options.font);
  const manifest = JSON.parse(manifestBytes);
  const nativeProbe = readJson(options.nativeProbe);
  const wasmProbe = await buildWasmProbe(options);
  const comparison = compareRuntimeParity(nativeProbe, wasmProbe, manifest);
  const wasmBlob = fs.readFileSync(path.join(options.pkg, 'rhwp_bg.wasm'));
  const output = {
    schemaVersion: 1,
    issue: 4968,
    stage: 'W9-Q3-5R4E-1',
    status: 'pass',
    inputs: {
      fixture: digest(fixtureBytes),
      manifest: digest(manifestBytes),
      font: digest(fontBytes),
      projectionContractSha256: manifest.projectionContractSha256,
    },
    build: {
      wasm: digest(wasmBlob),
      command: 'docker compose --env-file .env.docker run --rm wasm',
    },
    ...comparison,
  };
  output.evidenceCanonicalSha256 = sha256(canonicalJson(output));
  return output;
}

async function main() {
  const options = parseArgs(process.argv.slice(2));
  const output = await generateRuntimeParity(options);
  fs.mkdirSync(path.dirname(options.output), { recursive: true });
  fs.writeFileSync(options.output, `${JSON.stringify(output, null, 2)}\n`);
  process.stdout.write(`${JSON.stringify({
    status: output.status,
    canonicalSha256: output.canonicalSha256,
    evidenceCanonicalSha256: output.evidenceCanonicalSha256,
    unchangedOffRows: output.effects.unchangedOffRows,
    changedOnRows: output.effects.changedOnRows,
  })}\n`);
}

if (process.argv[1] && path.resolve(process.argv[1]) === path.resolve(new URL(import.meta.url).pathname)) {
  main().catch((error) => {
    process.stderr.write(`${error.stack ?? error}\n`);
    process.exitCode = 1;
  });
}
