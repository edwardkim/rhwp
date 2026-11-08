import test from 'node:test';
import assert from 'node:assert/strict';
import type { LayerFormObjectOp } from '../src/core/types.ts';
import { formDrawingReplayRejection } from '../src/view/canvaskit/form-replay.ts';

const MAX_FLOAT32 = 3.4028234663852886e38;
function form(): LayerFormObjectOp {
  return { type: 'formObject', bbox: { x: 10, y: 20, width: 80, height: 24 },
    drawing: { primitives: [{ kind: 'rect', bbox: { x: 10, y: 20, width: 80, height: 24 }, color: '#ffffff' }],
      label: { text: 'display label', x: 13, baseline: 38, fontFamily: 'prepared', fontSize: 12,
        bold: false, italic: true, color: '#000000' } } };
}

test('form admission checks only the serialized producer appearance without mutating it', () => {
  const op = { ...form(), formType: 'future-producer-kind', caption: 'x'.repeat(5000),
    text: 'unused raw value', value: -2147483648, foreColor: 'x'.repeat(129), enabled: false };
  const original = structuredClone(op);
  assert.equal(formDrawingReplayRejection(op), null);
  assert.deepEqual(op, original);
  assert.equal(formDrawingReplayRejection({ ...op, drawing: undefined }), 'missingProducerDrawing');
  assert.equal(formDrawingReplayRejection({ ...op, drawing: { primitives: [], label: null } }), null);
});

test('form scalars and derived endpoints share the portable f32 boundary', () => {
  const op = form();
  for (const value of [NaN, Infinity, -Infinity, 1e100, -1e100]) {
    assert.equal(formDrawingReplayRejection({ ...op, bbox: { ...op.bbox, x: value } }), 'invalidGeometry');
    assert.equal(formDrawingReplayRejection({ ...op, drawing: { primitives: [],
      label: { ...op.drawing!.label!, baseline: value } } }), 'invalidGeometry');
  }
  assert.equal(formDrawingReplayRejection({ ...op, bbox: { x: 0, y: 0, width: MAX_FLOAT32, height: 1 } }), null);
  assert.equal(formDrawingReplayRejection({ ...op, bbox: { x: MAX_FLOAT32, y: 0, width: MAX_FLOAT32, height: 1 } }), 'invalidGeometry');
  for (const width of [0, -1]) {
    assert.equal(formDrawingReplayRejection({ ...op, bbox: { ...op.bbox, width } }), 'invalidGeometry');
  }
  const circle = { kind: 'circle' as const, x: MAX_FLOAT32, y: 0, radius: MAX_FLOAT32, color: '#000' };
  assert.equal(formDrawingReplayRejection({ ...op, drawing: { primitives: [circle] } }), 'invalidGeometry');
});

test('form budgets apply to rendered colors, UTF-16 labels and all polyline points', () => {
  const op = form(), rect = op.drawing!.primitives[0];
  for (const [count, expected] of [[512, null], [513, 'formPrimitiveBudgetExceeded']] as const) {
    assert.equal(formDrawingReplayRejection({ ...op, drawing: { primitives: Array(count).fill(rect) } }), expected);
  }
  const line = { kind: 'polyline' as const, points: Array<[number, number]>(2048).fill([1, 2]),
    width: 1, closed: false, color: '#000' };
  assert.equal(formDrawingReplayRejection({ ...op, drawing: { primitives: [line, line] } }), null);
  assert.equal(formDrawingReplayRejection({ ...op, drawing: { primitives: [line, line, { ...line, points: [[0, 0]] }] } }), 'formPrimitiveBudgetExceeded');
  for (const field of ['text', 'fontFamily'] as const) {
    for (const [count, expected] of [[2048, null], [2049, 'formTextBudgetExceeded']] as const) {
      assert.equal(formDrawingReplayRejection({ ...op, drawing: { primitives: [],
        label: { ...op.drawing!.label!, [field]: '😀'.repeat(count) } } }), expected);
    }
  }
  for (const [length, expected] of [[128, null], [129, 'formColorBudgetExceeded']] as const) {
    assert.equal(formDrawingReplayRejection({ ...op, drawing: { primitives: [{ ...rect, color: 'x'.repeat(length) }] } }), expected);
  }
});

test('malformed form arrays and primitives fail closed before native replay', () => {
  const op = form();
  const invalid = [
    { primitives: null }, { primitives: [null] }, { primitives: [{ kind: 'unknown' }] },
    { primitives: [{ kind: 'rect', bbox: null, color: '#000' }] },
    { primitives: [{ kind: 'polyline', points: [[0]], width: 1, closed: false, color: '#000' }] },
    { primitives: [{ kind: 'polyline', points: [[0, 0]], width: -1, closed: false, color: '#000' }] },
    { primitives: [{ kind: 'polyline', points: [[0, 0]], width: 1, closed: 0, color: '#000' }] },
    { primitives: [], label: { ...op.drawing!.label, fontSize: 4097 } },
    { primitives: [], label: { ...op.drawing!.label, italic: 'true' } },
  ];
  for (const drawing of invalid) {
    assert.equal(formDrawingReplayRejection({ ...op, drawing } as LayerFormObjectOp), 'invalidGeometry');
  }
});
