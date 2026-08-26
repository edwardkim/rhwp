import test from 'node:test';
import assert from 'node:assert/strict';

import { currentPageLabel, formatPageIndicator } from '../src/view/page-indicator.ts';

test('문서 쪽번호가 있으면 그 숫자를 보여준다 (새 번호로 시작 반영)', () => {
  // 앞 2쪽 뒤에 1쪽부터 다시 시작하는 문서: 세 번째 물리 쪽의 문서 쪽번호는 1이다.
  assert.equal(
    formatPageIndicator({ pageIndex: 2, totalPages: 33, documentPageNumber: 1 }),
    '1 / 33 쪽',
  );
  assert.equal(
    formatPageIndicator({ pageIndex: 3, totalPages: 33, documentPageNumber: 2 }),
    '2 / 33 쪽',
  );
});

test('전체 쪽수는 물리 쪽수를 유지한다', () => {
  // 한글과 같은 규칙 — 현재 쪽만 문서 번호이고 분모는 실제 쪽 수다.
  assert.match(
    formatPageIndicator({ pageIndex: 32, totalPages: 33, documentPageNumber: 31 }),
    /\/ 33 쪽$/,
  );
});

test('새 번호가 없는 문서는 지금까지와 같은 숫자를 보여준다', () => {
  for (let i = 0; i < 5; i++) {
    assert.equal(
      formatPageIndicator({ pageIndex: i, totalPages: 5, documentPageNumber: i + 1 }),
      `${i + 1} / 5 쪽`,
    );
  }
});

test('문서 쪽번호를 모르면 물리 순번으로 물러난다', () => {
  for (const unknown of [undefined, null, 0, -3, Number.NaN, Number.POSITIVE_INFINITY]) {
    assert.equal(
      formatPageIndicator({ pageIndex: 4, totalPages: 10, documentPageNumber: unknown as number }),
      '5 / 10 쪽',
      `모르는 값(${String(unknown)})은 물리 순번으로 물러나야 한다`,
    );
  }
});

test('현재 쪽 번호는 정수로 자른다', () => {
  assert.equal(currentPageLabel({ pageIndex: 2, totalPages: 9, documentPageNumber: 1.9 }), 1);
});
