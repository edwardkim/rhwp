import assert from 'node:assert/strict';
import test from 'node:test';

import { compareFailureMatrix } from '../kerning_r4e_failure_matrix.mjs';

function snapshot(sentinel) {
  return {
    pageCount: 1,
    renderTree: { source: `section:0/para:${sentinel}/char:0` },
    layerTree: { source: `section:0/para:${sentinel}/char:0` },
    svg: { bytes: 10, sha256: 'svg' },
    canvasCommandCount: 3,
    canvasKit: {
      directReplayRequired: true,
      summary: { hiddenOverlayViolations: 0 },
    },
  };
}

function registration(ok, reason) {
  if (!ok) return { ok: false, error: `registration failed: ${reason}` };
  return {
    ok: true,
    value: {
      status: 'registered',
      slot: { charShapeId: 8, languageIndex: 1 },
      handle: { faceIndex: 0, fontBytes: 3, fontSourceSha256: 'font' },
      registry: { slotCount: 1, sourceCount: 1, totalSourceBytes: 3 },
    },
  };
}

function matrix(sentinel) {
  return [
    ['malformed-sfnt', true],
    ['pair-table-unsupported', true],
    ['unavailable-face-index', true],
    ['invalid-language-index', false],
    ['font-byte-limit-exceeded', false],
    ['slot-conflict', false],
  ].map(([name, ok]) => ({
    case: name,
    registration: registration(ok, name),
    before: snapshot(sentinel),
    after: snapshot(sentinel),
  }));
}

test('failure matrix normalizes only target sentinel and preserves render state', () => {
  const projection = compareFailureMatrix(
    matrix('18446744073709551615'),
    matrix('4294967295'),
  );
  assert.equal(projection.length, 6);
  assert.equal(projection.filter((item) => item.registration.ok).length, 3);
  assert.equal(projection.filter((item) => !item.registration.ok).length, 3);
  assert.ok(projection.every((item) => item.unchanged));
  assert.doesNotMatch(JSON.stringify(projection), /"fontBytes"\s*:/);
});

test('failure matrix rejects a render mutation and an unstructured error', () => {
  const native = matrix('18446744073709551615');
  const wasm = matrix('4294967295');
  wasm[0].after.svg.sha256 = 'drift';
  assert.throws(() => compareFailureMatrix(native, wasm), /changed render state/);

  const unstructured = matrix('4294967295');
  unstructured[3].registration.error = 'opaque failure';
  assert.throws(() => compareFailureMatrix(native, unstructured), /unstructured registration/);
});
