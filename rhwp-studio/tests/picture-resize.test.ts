import test from 'node:test';
import assert from 'node:assert/strict';

import {
  arrowResizeDelta,
  computeArrowResize,
  MIN_SIZE_HWP,
  type ArrowKey,
} from '../src/engine/picture-resize.ts';

// 격자 1mm 기준 step (이동과 동일: mm → HWPUNIT)
const STEP = Math.round(1 * 7200 / 25.4); // 283

test('arrowResizeDelta는 →/←를 가로 증감, ↓/↑를 세로 증감으로 매핑한다', () => {
  assert.deepEqual(arrowResizeDelta('ArrowRight', STEP), { deltaW: STEP, deltaH: 0 });
  assert.deepEqual(arrowResizeDelta('ArrowLeft', STEP), { deltaW: -STEP, deltaH: 0 });
  assert.deepEqual(arrowResizeDelta('ArrowDown', STEP), { deltaW: 0, deltaH: STEP });
  assert.deepEqual(arrowResizeDelta('ArrowUp', STEP), { deltaW: 0, deltaH: -STEP });
});

test('computeArrowResize는 한 축만 step 만큼 변경한 before/after를 돌려준다', () => {
  const r = computeArrowResize('ArrowRight', 5776, 6592, STEP);
  assert.ok(r);
  assert.deepEqual(r.before, { width: 5776, height: 6592 });
  assert.deepEqual(r.after, { width: 5776 + STEP, height: 6592 });

  const r2 = computeArrowResize('ArrowUp', 5776, 6592, STEP);
  assert.ok(r2);
  assert.deepEqual(r2.after, { width: 5776, height: 6592 - STEP });
});

test('computeArrowResize는 최소 크기(MIN_SIZE_HWP) 아래로 줄이지 않는다', () => {
  // 최소 크기 직전: 클램프되어 MIN_SIZE_HWP에 멈춘다
  const r = computeArrowResize('ArrowLeft', MIN_SIZE_HWP + 10, 1000, STEP);
  assert.ok(r);
  assert.equal(r.after.width, MIN_SIZE_HWP);
  assert.equal(r.after.height, 1000);

  // 이미 최소 크기: 변화 없음 → null (불필요한 Undo 기록 방지)
  assert.equal(computeArrowResize('ArrowLeft', MIN_SIZE_HWP, 1000, STEP), null);
  assert.equal(computeArrowResize('ArrowUp', 1000, MIN_SIZE_HWP, STEP), null);
});

test('computeArrowResize는 비정상 입력에 null을 돌려준다', () => {
  assert.equal(computeArrowResize('ArrowRight', Number.NaN, 1000, STEP), null);
  assert.equal(computeArrowResize('ArrowRight', 1000, Number.POSITIVE_INFINITY, STEP), null);
  assert.equal(computeArrowResize('ArrowRight', 1000, 1000, 0), null);
  assert.equal(computeArrowResize('ArrowRight', 1000, 1000, -5), null);
  // 런타임 비검증 key — arrowResizeDelta 안전망이 무변화로 흡수해 null
  assert.equal(computeArrowResize('PageUp' as ArrowKey, 1000, 1000, STEP), null);
});

test('computeArrowResize는 비변경 축의 비정수 값을 보정하지 않는다', () => {
  // 가로만 변경: 세로의 소수값은 그대로 (의도치 않은 라운딩/Undo 기록 방지)
  const r = computeArrowResize('ArrowRight', 1000, 1234.5, STEP);
  assert.ok(r);
  assert.equal(r.after.width, 1000 + STEP);
  assert.equal(r.after.height, 1234.5);

  // 세로만 변경: 가로의 소수값은 그대로
  const r2 = computeArrowResize('ArrowDown', 999.25, 1000, STEP);
  assert.ok(r2);
  assert.equal(r2.after.width, 999.25);
  assert.equal(r2.after.height, 1000 + STEP);
});

test('확대는 모든 방향에서 격자 step 정수 단위로 일관된다', () => {
  const keys: ArrowKey[] = ['ArrowUp', 'ArrowDown', 'ArrowLeft', 'ArrowRight'];
  for (const key of keys) {
    const r = computeArrowResize(key, 10000, 10000, STEP);
    assert.ok(r, `${key} 변화 없음`);
    const dw = Math.abs(r.after.width - 10000);
    const dh = Math.abs(r.after.height - 10000);
    assert.deepEqual([dw, dh].sort((a, b) => a - b), [0, STEP], `${key} 증감 폭`);
  }
});
