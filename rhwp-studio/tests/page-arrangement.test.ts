import test from 'node:test';
import assert from 'node:assert/strict';

import { normalizePageArrangement } from '../src/view/page-arrangement.ts';

test('저장값이 없거나 잘못된 쪽 배치는 자동으로 복원된다', () => {
  assert.deepEqual(normalizePageArrangement(undefined), { kind: 'auto' });
  assert.deepEqual(normalizePageArrangement(null), { kind: 'auto' });
  assert.deepEqual(normalizePageArrangement({ kind: 'unknown' }), { kind: 'auto' });
});

test('고정 쪽 배치 종류는 판별값만 보존한다', () => {
  for (const kind of ['auto', 'single', 'double', 'facing'] as const) {
    assert.deepEqual(normalizePageArrangement({ kind }), { kind });
  }
});

test('여러 쪽 가로·세로 값은 각각 1~8 정수로 정규화된다', () => {
  assert.deepEqual(
    normalizePageArrangement({ kind: 'multiple', columns: 0, rows: 12 }),
    { kind: 'multiple', columns: 1, rows: 8 },
  );
  assert.deepEqual(
    normalizePageArrangement({ kind: 'multiple', columns: 3.6, rows: '2' }),
    { kind: 'multiple', columns: 4, rows: 2 },
  );
});
