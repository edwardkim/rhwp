import test from 'node:test';
import assert from 'node:assert/strict';

import {
  clearObjectEditingPage,
  summarizeObjectSelection,
} from '../src/engine/object-selection-page.ts';

test('다중 개체 렌더 페이지는 기존 마지막 bbox를 보존하고 편집 focus는 첫 bbox를 쓴다', () => {
  const summary = summarizeObjectSelection([
    { pageIndex: 4, x: 10, y: 20, w: 30, h: 40 },
    { pageIndex: 5, x: 5, y: 15, w: 60, h: 10 },
  ]);

  assert.deepEqual(summary, {
    renderPageIndex: 5,
    editingPageIndex: 4,
    x: 5,
    y: 15,
    width: 60,
    height: 45,
  });
  assert.equal(summarizeObjectSelection([]), null);
});

test('개체 선택 렌더를 지우면 stale 편집 페이지도 null로 해제한다', () => {
  const events: Array<[string, number | null]> = [];
  clearObjectEditingPage({
    emit(event, pageIndex) {
      events.push([event, pageIndex]);
    },
  });

  assert.deepEqual(events, [['editing-page-changed', null]]);
});
