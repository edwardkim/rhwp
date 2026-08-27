import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import test from 'node:test';

import {
  assertBodyAxisScaling,
  compareRuntimeParity,
  projectRuntimeProbe,
} from '../kerning_r4e_runtime_parity.mjs';

const repoRoot = path.resolve(path.dirname(new URL(import.meta.url).pathname), '../..');
const manifest = JSON.parse(fs.readFileSync(path.join(
  repoRoot,
  'mydocs/tech/investigations/issue-4968/fixtures/kerning_runtime_fixture.manifest.json',
), 'utf8'));

function stableSourceKey(context) {
  if (context.context === 'body') {
    return `section:0/para:${context.charPropertyId - 6}/char:0`;
  }
  const cellId = context.context === 'table-cell' ? 19 : 20;
  return `section:0/para:${context.charPropertyId - 7}/char:0/cell:${cellId}:0:0:${context.charPropertyId - 7}:0`;
}

function layerTree(afterRegistration, sentinel) {
  const matrix = new Map(manifest.semantic.matrix.map((row) => [row.charPropertyId, row]));
  return {
    type: 'root',
    children: [
      ...manifest.semantic.contexts.map((context) => {
        const requested = matrix.get(context.charPropertyId).kerning;
        const end = afterRegistration && requested ? 0.9 : 1;
        return {
          type: 'textRun',
          source: { stableSourceKey: stableSourceKey(context) },
          positions: [0, end],
          bbox: { width: end },
        };
      }),
      {
        type: 'textRun',
        source: { stableSourceKey: `section:0/para:${sentinel}/char:0` },
        positions: [0],
        bbox: { width: 0 },
      },
    ],
  };
}

function registrations() {
  return manifest.semantic.exactSourceRegistration.slots.map((slot, index) => ({
    handle: {
      faceIndex: 0,
      fontBytes: 1236,
      fontSourceSha256: manifest.semantic.fontSource.sha256,
    },
    ok: true,
    registry: {
      generation: index + 1,
      slotCount: index + 1,
      sourceCount: 1,
      totalSourceBytes: 1236,
    },
    slot,
    status: 'registered',
  }));
}

function snapshot(afterRegistration, sentinel) {
  const tree = layerTree(afterRegistration, sentinel);
  return {
    pageCount: 1,
    renderTree: { source: `section:0/para:${sentinel}/char:0` },
    layerTree: tree,
    svg: { bytes: 1, sha256: afterRegistration ? 'b' : 'a' },
    canvasCommandCount: 32,
    canvasKit: {
      directReplayRequired: true,
      hiddenCanvas2dOverlayAllowed: false,
      summary: { hiddenOverlayViolations: 0 },
    },
  };
}

function probe(sentinel) {
  return {
    projectionContractSha256: manifest.projectionContractSha256,
    registration: registrations(),
    k0: snapshot(false, sentinel),
    k1: snapshot(true, sentinel),
  };
}

test('runtime projection keeps only public allowlisted evidence', () => {
  const projection = projectRuntimeProbe(probe('18446744073709551615'), manifest);
  assert.equal(projection.k0.rows.length, 22);
  assert.equal(projection.k1.rows.length, 22);
  assert.equal(projection.registration.registry.slotCount, 18);
  assert.doesNotMatch(
    JSON.stringify(projection),
    /"(?:stableSourceKey|fontBytes|sourcePath|text)"\s*:/,
  );
});

test('native64 and wasm32 sentinels normalize without hiding layout drift', () => {
  const nativeProbe = probe('18446744073709551615');
  const wasmProbe = probe('4294967295');
  const result = compareRuntimeParity(nativeProbe, wasmProbe, manifest);
  assert.deepEqual(result.effects, { unchangedOffRows: 11, changedOnRows: 11 });
  assert.equal(result.diagnostics.k0.renderTree.sentinelCount, 1);

  wasmProbe.k1.layerTree.children[0].positions[1] = 0.8;
  assert.throws(
    () => compareRuntimeParity(nativeProbe, wasmProbe, manifest),
    /canonical projection mismatch/,
  );
});

test('body axes apply pair delta once after ratio and independently of spacing', () => {
  const projection = projectRuntimeProbe(probe('18446744073709551615'), manifest);
  for (const row of projection.k1.rows.filter((item) => item.context === 'body')) {
    if (!row.kerningRequested) continue;
    const expectedDelta = { 100: 1.6, 90: 1.44, 80: 1.28 }[row.ratio];
    row.measurement.totalWidth = 1 - expectedDelta;
  }
  for (const row of projection.k1.rows.filter((item) => item.context === 'body')) {
    if (!row.kerningRequested) row.measurement.totalWidth = 1;
  }
  assert.deepEqual(assertBodyAxisScaling(projection), {
    unit: 'layer-milli-px',
    deltas: [
      { ratio: 100, pairDeltaMilliPx: 1600 },
      { ratio: 90, pairDeltaMilliPx: 1440 },
      { ratio: 80, pairDeltaMilliPx: 1280 },
    ],
  });

  projection.k1.rows.find((row) => row.context === 'body'
    && row.ratio === 100 && row.spacing === -5 && row.kerningRequested).measurement.totalWidth = -0.5;
  assert.throws(() => assertBodyAxisScaling(projection), /spacing changed pair delta/);
});
