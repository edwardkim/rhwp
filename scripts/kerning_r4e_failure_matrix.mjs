#!/usr/bin/env node

import fs from 'node:fs';
import path from 'node:path';
import { pathToFileURL } from 'node:url';

import {
  canonicalJson,
  normalizePlatformSentinels,
  sha256,
} from './kerning_q1_baseline.mjs';
import {
  assertBodyAxisScaling,
  buildWasmProbe,
  compareRuntimeParity,
  snapshotWasm,
} from './kerning_r4e_runtime_parity.mjs';

const repoRoot = path.resolve(path.dirname(new URL(import.meta.url).pathname), '..');
const maxFontBytes = 32 * 1024 * 1024;
const defaults = {
  fixture: path.join(repoRoot, 'mydocs/tech/investigations/issue-4968/fixtures/kerning_runtime_fixture.hwpx'),
  manifest: path.join(repoRoot, 'mydocs/tech/investigations/issue-4968/fixtures/kerning_runtime_fixture.manifest.json'),
  font: path.join(repoRoot, 'tests/fixtures/fonts/RHWPExactKerningSmoke.ttf'),
  unsupportedFont: path.join(repoRoot, 'tests/fixtures/fonts/RHWPBitmapSvgGlyphSmoke.ttf'),
  pkg: path.join(repoRoot, 'pkg'),
  nativeProbe: '',
  output: path.join(repoRoot, 'mydocs/tech/investigations/issue-4968/kerning_r4e_failure_matrix.json'),
};

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

function digest(bytes) {
  return { bytes: bytes.length, sha256: sha256(bytes) };
}

function registrationAttempt(document, charShapeId, languageIndex, fontBytes, faceIndex) {
  try {
    return {
      ok: true,
      value: JSON.parse(document.registerExactFontSource(
        charShapeId,
        languageIndex,
        fontBytes,
        faceIndex,
      )),
    };
  } catch (error) {
    return { ok: false, error: String(error) };
  }
}

function wasmFailureCase(wasmModule, fixtureBytes, definition) {
  const document = new wasmModule.HwpDocument(new Uint8Array(fixtureBytes));
  try {
    if (definition.beforeAttempt) definition.beforeAttempt(document);
    const before = snapshotWasm(document);
    const registration = registrationAttempt(
      document,
      8,
      definition.languageIndex,
      definition.fontBytes,
      definition.faceIndex,
    );
    const after = snapshotWasm(document);
    return { case: definition.case, registration, before, after };
  } finally {
    document.free();
  }
}

export async function buildWasmFailureMatrix(options) {
  const fixtureBytes = fs.readFileSync(options.fixture);
  const exactFont = new Uint8Array(fs.readFileSync(options.font));
  const unsupportedFont = new Uint8Array(fs.readFileSync(options.unsupportedFont));
  const wasmJs = path.join(options.pkg, 'rhwp.js');
  const wasmBlob = path.join(options.pkg, 'rhwp_bg.wasm');
  const wasmModule = await import(`${pathToFileURL(wasmJs).href}?r4e2=${Date.now()}`);
  await wasmModule.default({ module_or_path: fs.readFileSync(wasmBlob) });
  const definitions = [
    {
      case: 'malformed-sfnt',
      fontBytes: new Uint8Array(Buffer.from('not-an-sfnt')),
      faceIndex: 0,
      languageIndex: 1,
    },
    {
      case: 'pair-table-unsupported',
      fontBytes: unsupportedFont,
      faceIndex: 0,
      languageIndex: 1,
    },
    {
      case: 'unavailable-face-index',
      fontBytes: exactFont,
      faceIndex: 1,
      languageIndex: 1,
    },
    {
      case: 'invalid-language-index',
      fontBytes: exactFont,
      faceIndex: 0,
      languageIndex: 7,
    },
    {
      case: 'font-byte-limit-exceeded',
      fontBytes: new Uint8Array(maxFontBytes + 1),
      faceIndex: 0,
      languageIndex: 1,
    },
    {
      case: 'slot-conflict',
      fontBytes: unsupportedFont,
      faceIndex: 0,
      languageIndex: 1,
      beforeAttempt(document) {
        const first = registrationAttempt(document, 8, 1, exactFont, 0);
        if (!first.ok) throw new Error(`slot-conflict setup failed: ${first.error}`);
      },
    },
  ];
  return definitions.map((definition) => wasmFailureCase(wasmModule, fixtureBytes, definition));
}

function normalizedDigest(value) {
  const sentinels = { count: 0 };
  const normalized = normalizePlatformSentinels(value, sentinels);
  return { sha256: sha256(canonicalJson(normalized)), sentinelCount: sentinels.count };
}

function snapshotFingerprint(snapshot) {
  return {
    pageCount: snapshot.pageCount,
    renderTree: normalizedDigest(snapshot.renderTree),
    layerTree: normalizedDigest(snapshot.layerTree),
    svg: snapshot.svg,
    canvasCommandCount: snapshot.canvasCommandCount,
    canvasKit: normalizedDigest(snapshot.canvasKit),
    directReplayRequired: snapshot.canvasKit.directReplayRequired,
    hiddenOverlayViolations: snapshot.canvasKit.summary.hiddenOverlayViolations,
  };
}

function registrationReason(error) {
  const reasons = ['invalid-language-index', 'font-byte-limit-exceeded', 'slot-conflict'];
  const reason = reasons.find((candidate) => error.includes(candidate));
  if (!reason) throw new Error(`unstructured registration error: ${error}`);
  return reason;
}

function projectRegistration(registration) {
  if (!registration.ok) return { ok: false, reason: registrationReason(registration.error) };
  return {
    ok: true,
    status: registration.value.status,
    slot: registration.value.slot,
    handle: {
      faceIndex: registration.value.handle.faceIndex,
      bytes: registration.value.handle.fontBytes,
      fontSourceSha256: registration.value.handle.fontSourceSha256,
    },
    registry: {
      slotCount: registration.value.registry.slotCount,
      sourceCount: registration.value.registry.sourceCount,
      totalSourceBytes: registration.value.registry.totalSourceBytes,
    },
  };
}

function projectFailureCase(value) {
  const before = snapshotFingerprint(value.before);
  const after = snapshotFingerprint(value.after);
  if (canonicalJson(before) !== canonicalJson(after)) {
    throw new Error(`${value.case} changed render state`);
  }
  if (before.directReplayRequired !== true || before.hiddenOverlayViolations !== 0) {
    throw new Error(`${value.case} violated bounded direct replay`);
  }
  return {
    case: value.case,
    registration: projectRegistration(value.registration),
    renderState: before,
    unchanged: true,
  };
}

export function compareFailureMatrix(nativeMatrix, wasmMatrix) {
  const nativeProjection = nativeMatrix.map(projectFailureCase);
  const wasmProjection = wasmMatrix.map(projectFailureCase);
  if (canonicalJson(nativeProjection) !== canonicalJson(wasmProjection)) {
    throw new Error('native/WASM failure matrix mismatch');
  }
  const expected = new Map([
    ['malformed-sfnt', true],
    ['pair-table-unsupported', true],
    ['unavailable-face-index', true],
    ['invalid-language-index', false],
    ['font-byte-limit-exceeded', false],
    ['slot-conflict', false],
  ]);
  for (const item of nativeProjection) {
    if (item.registration.ok !== expected.get(item.case)) {
      throw new Error(`${item.case} registration disposition mismatch`);
    }
  }
  return nativeProjection;
}

export async function generateFailureEvidence(options) {
  const fixtureBytes = fs.readFileSync(options.fixture);
  const manifestBytes = fs.readFileSync(options.manifest);
  const exactFontBytes = fs.readFileSync(options.font);
  const unsupportedFontBytes = fs.readFileSync(options.unsupportedFont);
  const manifest = JSON.parse(manifestBytes);
  const nativeProbe = JSON.parse(fs.readFileSync(options.nativeProbe, 'utf8'));
  const wasmProbe = await buildWasmProbe(options);
  const runtimeParity = compareRuntimeParity(nativeProbe, wasmProbe, manifest);
  const axisScaling = assertBodyAxisScaling(runtimeParity.projection);
  const wasmFailureMatrix = await buildWasmFailureMatrix(options);
  const failureMatrix = compareFailureMatrix(nativeProbe.failureMatrix, wasmFailureMatrix);
  const output = {
    schemaVersion: 1,
    issue: 4968,
    stage: 'W9-Q3-5R4E-2',
    status: 'pass',
    inputs: {
      fixture: digest(fixtureBytes),
      manifest: digest(manifestBytes),
      exactFont: digest(exactFontBytes),
      unsupportedFont: digest(unsupportedFontBytes),
      projectionContractSha256: manifest.projectionContractSha256,
    },
    runtimeParityCanonicalSha256: runtimeParity.canonicalSha256,
    axisScaling,
    failureMatrix,
  };
  output.evidenceCanonicalSha256 = sha256(canonicalJson(output));
  return output;
}

async function main() {
  const options = parseArgs(process.argv.slice(2));
  const output = await generateFailureEvidence(options);
  fs.mkdirSync(path.dirname(options.output), { recursive: true });
  fs.writeFileSync(options.output, `${JSON.stringify(output, null, 2)}\n`);
  process.stdout.write(`${JSON.stringify({
    status: output.status,
    runtimeParityCanonicalSha256: output.runtimeParityCanonicalSha256,
    failureCaseCount: output.failureMatrix.length,
    evidenceCanonicalSha256: output.evidenceCanonicalSha256,
  })}\n`);
}

if (process.argv[1] && path.resolve(process.argv[1]) === path.resolve(new URL(import.meta.url).pathname)) {
  main().catch((error) => {
    process.stderr.write(`${error.stack ?? error}\n`);
    process.exitCode = 1;
  });
}
