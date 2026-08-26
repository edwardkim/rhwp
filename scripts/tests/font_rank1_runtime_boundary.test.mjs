import assert from 'node:assert/strict';
import test from 'node:test';

import {
  compareFormatBoundaries,
  summarizeBoundary,
} from '../font_rank1_runtime_boundary.mjs';

const TARGET = '문체부 바탕체';

function record(character, widthSource, languageSlot = 0) {
  return {
    source: { character },
    document: { face: TARGET, languageSlot },
    layoutName: {
      requestedFace: TARGET,
      normalizedFace: TARGET,
      steps: [],
    },
    layoutMetric: {
      aliasResolvedFace: TARGET,
      metricEntry: null,
      matchKind: 'none',
      widthSource,
    },
    paint: {
      canvas2d: { requested: TARGET },
      canvaskit: { requested: TARGET },
      native: { requested: TARGET },
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

function formatBoundary(format, boundary) {
  return {
    format,
    trace: boundary,
    parity: { nativeWasmByteExact: true },
  };
}

test('rank-1 boundary keeps localized face unresolved before metric lookup', () => {
  const boundary = summarizeBoundary(trace([
    record('가', 'heuristicFullwidth', 0),
    record('A', 'heuristicHalfwidth', 1),
    record('(', 'heuristicNarrow', 5),
  ]));
  assert.deepEqual(boundary.normalizedFaces, [{ value: TARGET, count: 3 }]);
  assert.deepEqual(boundary.layoutNameStepCounts, [{ value: 0, count: 3 }]);
  assert.deepEqual(boundary.aliasResolvedFaces, [{ value: TARGET, count: 3 }]);
  assert.deepEqual(boundary.metricEntries, [{ value: null, count: 3 }]);
  assert.deepEqual(boundary.matchKinds, [{ value: 'none', count: 3 }]);
});

test('a silently normalized or metric-matched record fails the current-boundary contract', () => {
  const normalized = trace([record('가', 'heuristicFullwidth')]);
  normalized.records[0].layoutName.normalizedFace = 'MBatang';
  assert.throws(() => summarizeBoundary(normalized), /normalized faces mismatch/);

  const matched = trace([record('가', 'metricTable')]);
  matched.records[0].layoutMetric.metricEntry = { name: 'MBatang' };
  matched.records[0].layoutMetric.matchKind = 'exact';
  assert.throws(() => summarizeBoundary(matched), /metric entries mismatch/);
});

test('HWP and HWPX may differ in language slots but must share decision semantics', () => {
  const hwpxTrace = summarizeBoundary(trace([
    record('가', 'heuristicFullwidth', 1),
    record('A', 'heuristicHalfwidth', 1),
  ]));
  const hwpTrace = summarizeBoundary(trace([
    record('가', 'heuristicFullwidth', 0),
    record('A', 'heuristicHalfwidth', 0),
  ]));
  const comparison = compareFormatBoundaries(
    formatBoundary('hwpx', hwpxTrace),
    formatBoundary('hwp5', hwpTrace),
  );
  assert.equal(comparison.semanticEqual, true);
  assert.equal(comparison.languageSlotsDiffer, true);
});

test('HWP and HWPX width-source divergence fails closed', () => {
  const hwpxTrace = summarizeBoundary(trace([record('가', 'heuristicFullwidth')]));
  const hwpTrace = summarizeBoundary(trace([record('가', 'heuristicHalfwidth')]));
  assert.throws(
    () => compareFormatBoundaries(
      formatBoundary('hwpx', hwpxTrace),
      formatBoundary('hwp5', hwpTrace),
    ),
    /runtime boundary semantics mismatch/,
  );
});
