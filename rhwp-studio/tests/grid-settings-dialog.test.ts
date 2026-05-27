import test from 'node:test';
import assert from 'node:assert/strict';

import { convertGridOffsetForOrigin } from '../src/view/grid-settings.ts';
import type { GridOffsetMm, GridOrigin } from '../src/view/grid-settings.ts';

const sample16Bases: Record<GridOrigin, GridOffsetMm> = {
  page: { x: 15, y: 20.01 },
  paper: { x: 0, y: 0 },
};

test('격자 기준 전환은 한컴처럼 종이 절대 원점을 보존한다', () => {
  assert.deepEqual(
    convertGridOffsetForOrigin({ x: 15, y: 24 }, 'paper', 'page', sample16Bases),
    { x: 0, y: 3.99 },
  );

  assert.deepEqual(
    convertGridOffsetForOrigin({ x: 0, y: 3.99 }, 'page', 'paper', sample16Bases),
    { x: 15, y: 24 },
  );
});

test('일반 문서의 쪽 0,0 기준은 종이 기준 본문 시작 위치로 환산된다', () => {
  const examBases: Record<GridOrigin, GridOffsetMm> = {
    page: { x: 9, y: 24 },
    paper: { x: 0, y: 0 },
  };

  assert.deepEqual(
    convertGridOffsetForOrigin({ x: 0, y: 0 }, 'page', 'paper', examBases),
    { x: 9, y: 24 },
  );
});
