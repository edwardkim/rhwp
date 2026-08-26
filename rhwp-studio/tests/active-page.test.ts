import test from 'node:test';
import assert from 'node:assert/strict';
import { resolveActivePage } from '../src/view/active-page.ts';

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
