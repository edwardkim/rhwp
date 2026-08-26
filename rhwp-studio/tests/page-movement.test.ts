import test from 'node:test';
import assert from 'node:assert/strict';

import {
  normalizePageMovementSettings,
  resolvePageViewSettings,
} from '../src/view/page-movement.ts';

test('쪽 이동은 세로 방향과 가로 휠 사용을 기본값으로 복원한다', () => {
  assert.deepEqual(normalizePageMovementSettings(undefined), {
    direction: 'vertical',
    wheelHorizontal: true,
  });
  assert.deepEqual(normalizePageMovementSettings({
    direction: 'unknown',
    wheelHorizontal: 'yes',
  }), {
    direction: 'vertical',
    wheelHorizontal: true,
  });
});

test('가로 방향은 한컴 계약에 따라 쪽 모양을 한 쪽으로 강제한다', () => {
  assert.deepEqual(resolvePageViewSettings(
    { kind: 'multiple', columns: 4, rows: 2 },
    { direction: 'horizontal', wheelHorizontal: false },
  ), {
    arrangement: { kind: 'single' },
    movement: { direction: 'horizontal', wheelHorizontal: false },
  });
});

test('세로 방향은 선택한 쪽 모양을 그대로 유지한다', () => {
  assert.deepEqual(resolvePageViewSettings(
    { kind: 'double' },
    { direction: 'vertical', wheelHorizontal: true },
  ), {
    arrangement: { kind: 'double' },
    movement: { direction: 'vertical', wheelHorizontal: true },
  });
});
