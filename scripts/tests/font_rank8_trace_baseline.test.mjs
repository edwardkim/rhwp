import assert from 'node:assert/strict';
import test from 'node:test';

import {
  assertTraceParity,
  canonicalJson,
  sha256,
  summarizeFixedGeometry,
  summarizeTrace,
} from '../font_rank8_trace_baseline.mjs';

function trace(records, overrides = {}) {
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
    backendSummary: { layout: { status: 'complete', reasons: [] } },
    records,
    ...overrides,
  };
}

function record(character, widthSource) {
  return {
    source: { character },
    document: { face: 'KoPubWorld바탕체 Light' },
    layoutMetric: {
      metricEntry: null,
      matchKind: 'none',
      widthSource,
    },
  };
}

test('complete target trace is summarized without promoting a metric entry', () => {
  const value = trace([
    record('가', 'heuristicFullwidth'),
    record('A', 'heuristicHalfwidth'),
    record('(', 'heuristicNarrow'),
  ]);
  const summary = summarizeTrace(value);
  assert.equal(summary.records, 3);
  assert.deepEqual(summary.metricEntries, [{ value: null, count: 3 }]);
  assert.deepEqual(summary.matchKinds, [{ value: 'none', count: 3 }]);
  assert.equal(summary.canonicalTraceSha256, sha256(canonicalJson(value)));
});

test('native and WASM trace require byte-exact canonical parity', () => {
  const left = trace([record('가', 'heuristicFullwidth')]);
  const right = JSON.parse(JSON.stringify(left));
  assert.equal(assertTraceParity(left, right), sha256(canonicalJson(left)));
  right.records[0].layoutMetric.widthSource = 'embeddedMetric';
  assert.throws(() => assertTraceParity(left, right), /native\/WASM trace mismatch/);
});

test('truncated and mixed-face traces fail closed', () => {
  const truncated = trace([record('가', 'heuristicFullwidth')], {
    status: 'truncated',
  });
  assert.throws(() => summarizeTrace(truncated), /complete schema-v1/);

  const mixed = trace([record('가', 'heuristicFullwidth')]);
  mixed.records[0].document.face = '다른 글꼴';
  assert.throws(() => summarizeTrace(mixed), /non-target/);
});

test('fixed-context geometry preserves actual frame slack and LineSeg lane', () => {
  const manifest = {
    semantic: {
      matrix: [
        { charPropertyId: 7, ratio: 100, spacing: 0, kerning: false },
        { charPropertyId: 16, ratio: 90, spacing: -5, kerning: true },
        { charPropertyId: 24, ratio: 80, spacing: -10, kerning: true },
      ],
      contexts: [
        { context: 'table-cell', charPropertyId: 7, lineSegLane: 'stored-line-lane' },
        { context: 'text-box', charPropertyId: 7, lineSegLane: 'fresh-candidate-lane' },
        { context: 'table-cell', charPropertyId: 16, lineSegLane: 'fresh-candidate-lane' },
        { context: 'text-box', charPropertyId: 16, lineSegLane: 'stored-line-lane' },
        { context: 'table-cell', charPropertyId: 24, lineSegLane: 'stored-line-lane' },
        { context: 'text-box', charPropertyId: 24, lineSegLane: 'fresh-candidate-lane' },
      ],
    },
  };
  const runs = [];
  for (const parentParaIdx of [19, 20]) {
    for (let cellParaIdx = 0; cellParaIdx < 3; cellParaIdx += 1) {
      runs.push({ parentParaIdx, cellParaIdx, x: 10, y: cellParaIdx, w: 400 });
    }
  }
  const geometry = summarizeFixedGeometry({ runs }, manifest);
  assert.equal(geometry.length, 6);
  assert.equal(geometry[0].contentWidthHwpunit, 28980);
  assert.equal(geometry[0].minimumSlackPx, -13.6);
  assert.equal(geometry[0].crossesFrame, true);
  assert.equal(geometry[3].contentWidthHwpunit, 29434);
  assert.equal(geometry[3].minimumSlackPx, -7.5);
  assert.equal(geometry[3].lineSegLane, 'fresh-candidate-lane');
});
