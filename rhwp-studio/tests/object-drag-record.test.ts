import test from 'node:test';
import assert from 'node:assert/strict';

import {
  computeRotationRecord,
  computeLineEndpointRecord,
  type LineEndpoints,
} from '../src/engine/object-drag-record.ts';

// [Task #2759] 개체 드래그 종료 시 "기록할 before/after 결정" 순수 로직 단위 테스트.
// 회전 핸들 드래그·직선 끝점 드래그가 undo 가능하려면 이 결정이 정확해야 한다
// (변화 없으면 null → 무의미한 Undo 항목·redo 스택 무효화 방지).

test('computeRotationRecord 는 각이 바뀌면 origAngle→finalAngle before/after 를 준다', () => {
  const r = computeRotationRecord(0, 45);
  assert.ok(r);
  assert.deepEqual(r.before, { rotationAngle: 0 });
  assert.deepEqual(r.after, { rotationAngle: 45 });
});

test('computeRotationRecord 는 각이 같으면 null (드래그 없음 = 미기록)', () => {
  assert.equal(computeRotationRecord(90, 90), null);
  assert.equal(computeRotationRecord(0, 0), null);
});

test('computeRotationRecord 는 음수각·정규화된 값을 그대로 보존한다', () => {
  const r = computeRotationRecord(30, -15);
  assert.ok(r);
  assert.deepEqual(r.before, { rotationAngle: 30 });
  assert.deepEqual(r.after, { rotationAngle: -15 });
});

test('computeRotationRecord 는 비정상 입력(NaN/Infinity)에 null 을 준다', () => {
  assert.equal(computeRotationRecord(Number.NaN, 45), null);
  assert.equal(computeRotationRecord(0, Number.POSITIVE_INFINITY), null);
});

const A: LineEndpoints = { sx: 1000, sy: 2000, ex: 3000, ey: 4000 };

test('computeLineEndpointRecord 는 끝점이 바뀌면 before/after 를 복사해 준다', () => {
  const after: LineEndpoints = { sx: 1500, sy: 2000, ex: 3000, ey: 4000 };
  const r = computeLineEndpointRecord(A, after);
  assert.ok(r);
  assert.deepEqual(r.before, A);
  assert.deepEqual(r.after, after);
  // 방어적 복사 — 원본 변형이 기록에 새지 않아야 한다.
  assert.notEqual(r.before, A);
});

test('computeLineEndpointRecord 는 좌표가 동일하면 null (클릭만 = 미기록)', () => {
  assert.equal(computeLineEndpointRecord(A, { ...A }), null);
});

test('computeLineEndpointRecord 는 어느 한 좌표만 달라도 기록 대상이다', () => {
  assert.ok(computeLineEndpointRecord(A, { ...A, ey: 4001 }));
  assert.ok(computeLineEndpointRecord(A, { ...A, sx: 999 }));
});

test('computeLineEndpointRecord 는 비정상 좌표(NaN)에 null 을 준다', () => {
  assert.equal(computeLineEndpointRecord(A, { ...A, sx: Number.NaN }), null);
  assert.equal(computeLineEndpointRecord({ ...A, ey: Number.NaN }, A), null);
});
