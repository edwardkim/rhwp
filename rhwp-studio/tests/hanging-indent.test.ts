import test from 'node:test';
import assert from 'node:assert/strict';

import {
  computeHangingIndentProps,
  computeHangingIndentPx,
} from '../src/engine/hanging-indent.ts';

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

test('내어쓰기는 여백과 첫 줄 들여쓰기를 함께 옮긴다', () => {
  // 여백 0인 문단: 둘째 줄부터 거리만큼 들어가고, 첫 줄은 제자리(합이 0)
  assert.deepEqual(computeHangingIndentProps(0, 0, 400), { marginLeft: 400, indent: -400 });
});

test('내어쓰기는 이미 들어간 문단의 첫 줄 자리를 지킨다', () => {
  // 여백 200 · 첫 줄 들여쓰기 100 → 첫 줄은 300 에서 시작한다.
  // 거리 400 을 주면 둘째 줄은 700, 첫 줄은 그대로 300(=700-400)이어야 한다.
  const props = computeHangingIndentProps(200, 100, 400);
  assert.deepEqual(props, { marginLeft: 700, indent: -400 });
  assert.equal(props.marginLeft + props.indent, 300);
});

test('내어쓰기를 두 번 눌러도 첫 줄 자리는 그대로다', () => {
  const first = computeHangingIndentProps(0, 0, 400);
  const second = computeHangingIndentProps(first.marginLeft, first.indent, 250);
  assert.equal(first.marginLeft + first.indent, 0);
  assert.equal(second.marginLeft + second.indent, 0);
  assert.equal(second.marginLeft, 250);
});

test('거리가 0이면 여백을 건드리지 않는다', () => {
  assert.deepEqual(computeHangingIndentProps(200, 0, 0), { marginLeft: 200, indent: -0 });
});
