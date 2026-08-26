import test from 'node:test';
import assert from 'node:assert/strict';
import {
  hasRulerEditingContext,
  resolveActivePage,
  resolveRulerPageIndex,
} from '../src/view/active-page.ts';

test('보이는 편집 페이지가 뷰포트 중심 페이지보다 우선한다', () => {
  assert.deepEqual(resolveActivePage({
    pageCount: 6,
    visiblePages: [1, 2, 3],
    editingPageIndex: 3,
    viewportPageIndex: 2,
  }), { pageIndex: 3, source: 'editing' });
});

test('편집 페이지가 화면 밖이면 보이는 뷰포트 페이지로 전환한다', () => {
  assert.deepEqual(resolveActivePage({
    pageCount: 6,
    visiblePages: [3, 4],
    editingPageIndex: 1,
    viewportPageIndex: 4,
  }), { pageIndex: 4, source: 'viewport' });
});

test('뷰포트 기준점이 빈 슬롯이나 범위 밖이면 첫 실제 가시 페이지를 쓴다', () => {
  assert.deepEqual(resolveActivePage({
    pageCount: 6,
    visiblePages: [-1, 2, 3, 8],
    editingPageIndex: null,
    viewportPageIndex: -1,
  }), { pageIndex: 2, source: 'viewport' });
});

test('가시 페이지가 없거나 문서가 비었으면 활성 페이지도 없다', () => {
  assert.equal(resolveActivePage({
    pageCount: 0,
    visiblePages: [],
    editingPageIndex: null,
    viewportPageIndex: null,
  }), null);
  assert.equal(resolveActivePage({
    pageCount: 3,
    visiblePages: [],
    editingPageIndex: 1,
    viewportPageIndex: 1,
  }), null);
});

test('0번 페이지도 유효한 편집 페이지로 보존한다', () => {
  assert.deepEqual(resolveActivePage({
    pageCount: 3,
    visiblePages: [0, 1],
    editingPageIndex: 0,
    viewportPageIndex: 1,
  }), { pageIndex: 0, source: 'editing' });
});

test('눈금자는 순수 스크롤로 활성 페이지가 바뀌어도 마지막 편집 focus를 유지한다', () => {
  assert.equal(resolveRulerPageIndex({
    documentPageCount: 6,
    layoutPageCount: 6,
    focusedPageIndex: 1,
    activePageIndex: 4,
  }), 1);
});

test('편집 focus가 아직 없을 때만 활성 뷰포트 페이지로 눈금자를 초기화한다', () => {
  assert.equal(resolveRulerPageIndex({
    documentPageCount: 6,
    layoutPageCount: 6,
    focusedPageIndex: null,
    activePageIndex: 4,
  }), 4);
  assert.equal(resolveRulerPageIndex({
    documentPageCount: 6,
    layoutPageCount: 6,
    focusedPageIndex: 8,
    activePageIndex: 2,
  }), 2);
});

test('눈금자는 문서와 확정 레이아웃에 모두 존재하는 페이지만 사용한다', () => {
  assert.equal(resolveRulerPageIndex({
    documentPageCount: 6,
    layoutPageCount: 3,
    focusedPageIndex: 4,
    activePageIndex: 2,
  }), 2);
  assert.equal(resolveRulerPageIndex({
    documentPageCount: 2,
    layoutPageCount: 5,
    focusedPageIndex: 3,
    activePageIndex: 1,
  }), 1);
});

test('눈금자의 편집 문맥은 viewport 활성 쪽이 바뀌어도 마지막 focus에 남는다', () => {
  assert.equal(hasRulerEditingContext(1, 1), true);
  assert.equal(hasRulerEditingContext(4, 1), false);
  assert.equal(hasRulerEditingContext(1, null), false);
});

test('focus와 활성 페이지가 모두 무효하면 눈금자 대상도 없다', () => {
  assert.equal(resolveRulerPageIndex({
    documentPageCount: 0,
    layoutPageCount: 0,
    focusedPageIndex: 0,
    activePageIndex: 0,
  }), null);
  assert.equal(resolveRulerPageIndex({
    documentPageCount: 3,
    layoutPageCount: 3,
    focusedPageIndex: null,
    activePageIndex: null,
  }), null);
});
