import test from 'node:test';
import assert from 'node:assert/strict';

import { computeHangingIndentPx } from '../src/engine/hanging-indent.ts';
import { paragraphLengthFromPt, paragraphLengthFromPx } from '../src/core/paragraph-spacing.ts';

test('절대 pt 길이는 가장 가까운 정수 HWPUNIT을 먼저 고르고 단위 비트를 보존한다', () => {
  for (const [pt, raw] of [[0, 0], [0.0049, 0], [0.005, 2], [0.1, 20],
    [1.0049999999999997, 200], [1.005, 202], [1.01, 202], [1.015, 204], [4.567, 914], [1e-7, 0]]) {
    assert.equal(paragraphLengthFromPt(pt), raw);
    assert.equal(paragraphLengthFromPt(-pt), raw === 0 ? 0 : -raw);
  }
});

test('커서와 첫 줄 사이의 거리를 내어쓰기로 저장할 때 0.5 HWPUNIT 경계를 보존한다', () => {
  assert.equal(paragraphLengthFromPx(computeHangingIndentPx(117.5, 100)), 2626);
  assert.equal(paragraphLengthFromPx(-17.5), -2626);
  assert.equal(paragraphLengthFromPx(17.49), 2624);
  assert.equal(paragraphLengthFromPx(0), 0);
});

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
