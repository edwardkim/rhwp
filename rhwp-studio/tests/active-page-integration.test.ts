import test from 'node:test';
import assert from 'node:assert/strict';

import { resolveActivePage } from '../src/view/active-page.ts';
import { VirtualScroll } from '../src/view/virtual-scroll.ts';

function pages(n: number, width = 800, height = 1000) {
  return Array.from({ length: n }, () => ({ width, height })) as never;
}

test('여러 쪽 배치는 X/Y가 실제로 겹치는 페이지만 활성 후보로 사용한다', () => {
  const scroll = new VirtualScroll(10);
  scroll.setPageDimensions(
    pages(3),
    1,
    1000,
    { kind: 'multiple', columns: 3, rows: 1 },
  );

  const leftVisible = scroll.getVisiblePages(0, 800, 0, 1000);
  const leftCenterPage = scroll.getPageAtPoint(500, 400);
  assert.deepEqual(leftVisible, [0, 1]);
  assert.deepEqual(resolveActivePage({
    pageCount: scroll.pageCount,
    visiblePages: leftVisible,
    editingPageIndex: 2,
    viewportPageIndex: leftCenterPage,
  }), { pageIndex: 0, source: 'viewport' });

  const rightVisible = scroll.getVisiblePages(0, 800, 1620, 1000);
  const rightCenterPage = scroll.getPageAtPoint(2120, 400);
  assert.deepEqual(rightVisible, [2]);
  assert.deepEqual(resolveActivePage({
    pageCount: scroll.pageCount,
    visiblePages: rightVisible,
    editingPageIndex: 2,
    viewportPageIndex: rightCenterPage,
  }), { pageIndex: 2, source: 'editing' });
});

test('단일 열은 X 가시성 보강 뒤에도 현재 페이지 판정을 유지한다', () => {
  const scroll = new VirtualScroll(10);
  scroll.setPageDimensions(pages(3), 1, 1000, { kind: 'single' });

  const visible = scroll.getVisiblePages(1010, 800, 0, 1000);
  const centerPage = scroll.getPageAtPoint(500, 1410);
  assert.deepEqual(visible, [1]);
  assert.equal(centerPage, 1);
  assert.deepEqual(resolveActivePage({
    pageCount: scroll.pageCount,
    visiblePages: visible,
    editingPageIndex: null,
    viewportPageIndex: centerPage,
  }), { pageIndex: 1, source: 'viewport' });
});
