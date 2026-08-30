import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

import {
  uniqueRowIndices,
  VirtualScroll,
} from '../src/view/virtual-scroll.ts';

function pages(n: number, width = 800, height = 1000) {
  return Array.from({ length: n }, () => ({ width, height })) as never;
}

test('행 인덱스 이진 탐색은 100쪽 그리드에서 전체 페이지를 선형 스캔하지 않는다', () => {
  const scroll = new VirtualScroll(10);
  scroll.setPageDimensions(
    pages(100),
    0.4,
    4000,
    { kind: 'multiple', columns: 4, rows: 2 },
  );
  assert.equal(scroll.getColumns(), 4);
  assert.equal(scroll.getRowCount(), 25);

  const rowHeight = scroll.getPageHeight(0) + scroll.getPageGap();
  const snapshot = scroll.getVisibilitySnapshot(rowHeight * 10, 200, 0, 4000);

  assert.deepEqual(snapshot.visiblePages, [40, 41, 42, 43]);
  assert.deepEqual(snapshot.visibleRows, [10]);
  assert.ok(snapshot.probedRows < 20, `행 이진 탐색이어야 한다 (실제 ${snapshot.probedRows})`);
  assert.ok(snapshot.probedPages < 20, `보이는 행의 페이지만 조사해야 한다 (실제 ${snapshot.probedPages})`);
  assert.equal(snapshot.computed, true);
});

test('같은 뷰포트 키의 getVisiblePages·getPrefetchPages는 스냅샷을 재사용한다', () => {
  const scroll = new VirtualScroll(10);
  scroll.setPageDimensions(pages(40), 0.4, 4000, { kind: 'multiple', columns: 4, rows: 2 });

  const first = scroll.getVisibilitySnapshot(0, 200, 0, 4000);
  const visible = scroll.getVisiblePages(0, 200, 0, 4000);
  const prefetch = scroll.getPrefetchPages(0, 200, 0, 4000);
  const second = scroll.getVisibilitySnapshot(0, 200, 0, 4000);

  assert.deepEqual(visible, [0, 1, 2, 3]);
  assert.deepEqual(visible, first.visiblePages);
  assert.deepEqual(prefetch, first.prefetchPages);
  assert.equal(second.computed, false);
  assert.equal(second.probedPages, first.probedPages);
  assert.deepEqual(prefetch, [0, 1, 2, 3, 4, 5, 6, 7]);
});

test('스크롤 방향 작업 집합은 진행 쪽 prefetch를 앞에 둔다', () => {
  const scroll = new VirtualScroll(10);
  scroll.setPageDimensions(pages(12), 0.4, 4000, { kind: 'multiple', columns: 4, rows: 2 });
  const snapshot = scroll.getVisibilitySnapshot(0, 200, 0, 4000);
  assert.deepEqual(snapshot.visiblePages, [0, 1, 2, 3]);

  const down = scroll.getWorkSet(snapshot, 1);
  const up = scroll.getWorkSet(snapshot, -1);
  assert.deepEqual(down.visible, [0, 1, 2, 3]);
  assert.deepEqual(down.prefetch, [4, 5, 6, 7]);
  assert.deepEqual(up.prefetch, [4, 5, 6, 7]);
});

test('uniqueRowIndices는 그리드 한 행의 열 중복을 한 번으로 접는다', () => {
  assert.deepEqual(uniqueRowIndices([2, 2, 2, 2, 3, 3]), [2, 3]);
});

test('getPageAtY 행 이진 탐색은 행의 마지막 쪽을 유지한다', () => {
  const scroll = new VirtualScroll(10);
  scroll.setPageDimensions(pages(9), 0.4, 4000, { kind: 'multiple', columns: 3, rows: 2 });
  const y = scroll.getPageOffset(3);
  assert.equal(scroll.getPageAtY(y), 5);
  assert.equal(scroll.getRowFirstPageAtY(y), 3);
});

test('CanvasView는 가시성 스냅샷과 행 단위 세로 눈금자를 쓴다', () => {
  const view = readFileSync(new URL('../src/view/canvas-view.ts', import.meta.url), 'utf8');
  const ruler = readFileSync(new URL('../src/view/ruler.ts', import.meta.url), 'utf8');
  assert.match(view, /getVisibilitySnapshot\(/);
  assert.match(view, /pageSurfaceLru/);
  assert.match(view, /pageRenderScheduler/);
  assert.match(view, /syncVisibleRenderBudget/);
  assert.doesNotMatch(view, /prefetchSet\.has\(pageIdx\)/);
  assert.match(view, /for \(const pageIdx of visiblePages\)/);
  assert.match(view, /this\.schedulePrefetchPages\(prefetchPages\.filter/);
  assert.match(view, /requestIdleCallback\(run, \{ timeout: 1000 \}\)/);
  assert.match(view, /cancelPendingPrefetch\(\)/);
  assert.match(ruler, /uniqueRowIndices/);
  assert.match(ruler, /peekVisibilitySnapshot/);
});
