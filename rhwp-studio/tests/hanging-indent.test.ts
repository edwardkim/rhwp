import test from 'node:test';
import assert from 'node:assert/strict';

import { computeHangingIndentPx } from '../src/engine/hanging-indent.ts';

test('Shift+Tab 내어쓰기는 첫 줄 시작점을 기준으로 계산한다', () => {
  assert.equal(computeHangingIndentPx(140, 100), 40);
});

test('Shift+Tab 내어쓰기는 첫 줄보다 왼쪽인 커서를 0으로 보정한다', () => {
  assert.equal(computeHangingIndentPx(80, 100), 0);
});

test('Shift+Tab 내어쓰기는 비정상 좌표를 0으로 처리한다', () => {
  assert.equal(computeHangingIndentPx(Number.NaN, 100), 0);
  assert.equal(computeHangingIndentPx(140, Number.POSITIVE_INFINITY), 0);
});
