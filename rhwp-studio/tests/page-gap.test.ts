import test from 'node:test';
import assert from 'node:assert/strict';

import { resolvePageGap } from '../src/view/page-gap.ts';

test('10%에서도 페이지 경계를 구분할 최소 화면 간격을 보장한다', () => {
  assert.equal(resolvePageGap(0.1), 6);
  assert.equal(resolvePageGap(0.5), 6);
});

test('100% 기존 간격을 보존하고 고배율에서는 연속적으로 확장한다', () => {
  assert.equal(resolvePageGap(1), 10);
  assert.ok(Math.abs(resolvePageGap(2.22) - 22.2) < 1e-9);
  assert.equal(resolvePageGap(5), 50);
});

test('사용자 기준 간격과 잘못된 입력을 안전하게 정규화한다', () => {
  assert.equal(resolvePageGap(2, 12), 24);
  assert.equal(resolvePageGap(Number.NaN), 10);
  assert.equal(resolvePageGap(-1), 6);
  assert.equal(resolvePageGap(1, -5), 6);
});
