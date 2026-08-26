import assert from 'node:assert/strict';
import test from 'node:test';

import {
  compareFormatBoundaries,
  layoutMetricProjection,
  layoutRunProjection,
  summarizeBoundary,
} from '../font_rank7_runtime_boundary.mjs';

const TARGET = 'KoPubWorld돋움체 Light';
const SUBSTITUTION = 'KoPubWorld바탕체 Light';

function record(character, widthSource, substitution = null) {
  const family = substitution ? `${TARGET},${substitution}` : TARGET;
  return {
    source: { character, codePoint: character.codePointAt(0) },
    document: { face: TARGET, substFont: substitution },
    layoutName: {
      requestedFace: TARGET,
      normalizedFace: TARGET,
      cssFamilyChain: substitution ? [TARGET, substitution] : [TARGET],
      steps: substitution ? [{ kind: 'documentSubstFont' }] : [],
    },
    layoutMetric: {
      aliasResolvedFace: TARGET,
      metricEntry: null,
      matchKind: 'none',
      widthSource,
      baseAdvanceHwpunit: 500,
      finalAdvanceHwpunit: 500,
    },
    paint: {
      canvas2d: { requested: family },
      canvaskit: { requested: family },
      native: { requested: family },
    },
  };
}

function trace(records) {
  return {
    schemaVersion: 1,
    status: 'complete',
    counts: {
      charactersSeen: records.length,
      recordsEmitted: records.length,
      recordsOmitted: 0,
    },
    layoutHash: { value: 'a'.repeat(64) },
    normalizedHash: { value: 'b'.repeat(64) },
    backendSummary: {},
    records,
  };
}

function formatResult(format, substitution) {
  const rawTrace = trace([
    record('가', 'heuristicFullwidth', substitution),
    record('A', 'heuristicHalfwidth', substitution),
  ]);
  const layout = {
    runs: [{ text: '가A', w: 10, fontFamily: substitution ? `${TARGET},${substitution}` : TARGET }],
  };
  return {
    format,
    boundary: summarizeBoundary(rawTrace, format),
    layoutMetricProjection: layoutMetricProjection(rawTrace),
    layoutRunProjection: layoutRunProjection(layout),
    fixedGeometry: [{ context: 'table-cell', maximumLineWidthPx: 10 }],
  };
}

test('rank-7 current boundary keeps the face unresolved at layout metric', () => {
  const boundary = summarizeBoundary(
    trace([record('가', 'heuristicFullwidth', SUBSTITUTION)]),
    'hwpx',
  );
  assert.deepEqual(boundary.normalizedFaces, [{ value: TARGET, count: 1 }]);
  assert.deepEqual(boundary.aliasResolvedFaces, [{ value: TARGET, count: 1 }]);
  assert.deepEqual(boundary.metricEntries, [{ value: null, count: 1 }]);
  assert.deepEqual(boundary.matchKinds, [{ value: 'none', count: 1 }]);
});

test('HWPX substitution metadata may differ while metric and geometry stay equal', () => {
  const comparison = compareFormatBoundaries(
    formatResult('hwpx', SUBSTITUTION),
    formatResult('hwp5', null),
  );
  assert.equal(comparison.layoutMetricProjectionEqual, true);
  assert.equal(comparison.layoutRunProjectionEqual, true);
  assert.equal(comparison.substitutionMetadataDiffers, true);
  assert.equal(comparison.substitutionAffectsLayoutMetric, false);
  assert.equal(comparison.substitutionAffectsPaintCandidateChain, true);
});

test('a metric match fails the current unresolved boundary contract', () => {
  const value = trace([record('가', 'metricTable', SUBSTITUTION)]);
  value.records[0].layoutMetric.metricEntry = { name: 'unexpected' };
  value.records[0].layoutMetric.matchKind = 'exact';
  assert.throws(() => summarizeBoundary(value, 'hwpx'), /metric mismatch/);
});

test('format metric projection divergence fails closed', () => {
  const hwpx = formatResult('hwpx', SUBSTITUTION);
  const hwp = formatResult('hwp5', null);
  hwp.layoutMetricProjection[0].layoutMetric.finalAdvanceHwpunit = 501;
  assert.throws(() => compareFormatBoundaries(hwpx, hwp), /metric projection mismatch/);
});

test('fontFamily-only layout difference is excluded but geometry change is not', () => {
  const hwpx = formatResult('hwpx', SUBSTITUTION);
  const hwp = formatResult('hwp5', null);
  assert.deepEqual(hwpx.layoutRunProjection, hwp.layoutRunProjection);
  hwp.layoutRunProjection[0].w = 11;
  assert.throws(() => compareFormatBoundaries(hwpx, hwp), /layout run projection mismatch/);
});
