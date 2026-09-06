import test from 'node:test';
import assert from 'node:assert/strict';

import { VirtualScroll } from '../src/view/virtual-scroll.ts';
import type { PageArrangement } from '../src/view/page-arrangement.ts';
import type { PageMovementDirection } from '../src/view/page-movement.ts';

type Page = { width: number; height: number };

function linearVisible(
  scroll: VirtualScroll,
  scrollY: number,
  viewportHeight: number,
  scrollX = 0,
  viewportWidth = 0,
): number[] {
  const vpBottom = scrollY + viewportHeight;
  const vpRight = viewportWidth > 0 ? scrollX + viewportWidth : Infinity;
  const visible: number[] = [];
  for (let pageIdx = 0; pageIdx < scroll.pageCount; pageIdx++) {
    const top = scroll.getPageOffset(pageIdx);
    const left = scroll.getPageLeftResolved(pageIdx, scroll.getTotalWidth());
    if (
      top < vpBottom
      && top + scroll.getPageHeight(pageIdx) > scrollY
      && left < vpRight
      && left + scroll.getPageWidth(pageIdx) > scrollX
    ) visible.push(pageIdx);
  }
  return visible;
}

function linearPrefetch(scroll: VirtualScroll, visible: readonly number[]): number[] {
  if (visible.length === 0) return [];
  const prefetch = new Set(visible);
  if (scroll.isHorizontalMode()) {
    if (visible[0] > 0) prefetch.add(visible[0] - 1);
    if (visible[visible.length - 1] + 1 < scroll.pageCount) {
      prefetch.add(visible[visible.length - 1] + 1);
    }
  } else {
    const offsets = [...new Set(
      Array.from({ length: scroll.pageCount }, (_, pageIdx) => scroll.getPageOffset(pageIdx)),
    )];
    const firstRow = offsets.indexOf(scroll.getPageOffset(visible[0]));
    const lastRow = offsets.indexOf(scroll.getPageOffset(visible[visible.length - 1]));
    for (const row of [firstRow - 1, lastRow + 1]) {
      if (row < 0 || row >= offsets.length) continue;
      for (let pageIdx = 0; pageIdx < scroll.pageCount; pageIdx++) {
        if (scroll.getPageOffset(pageIdx) === offsets[row]) prefetch.add(pageIdx);
      }
    }
  }
  return [...prefetch].sort((a, b) => a - b);
}

function linearPageAtY(scroll: VirtualScroll, docY: number): number {
  if (scroll.isHorizontalMode()) return 0;
  for (let pageIdx = scroll.pageCount - 1; pageIdx >= 0; pageIdx--) {
    if (docY >= scroll.getPageOffset(pageIdx)) return pageIdx;
  }
  return 0;
}

function linearPageAtPoint(scroll: VirtualScroll, docX: number, docY: number): number {
  if (scroll.isHorizontalMode()) {
    let bestIdx = 0;
    let bestDist = Infinity;
    for (let pageIdx = 0; pageIdx < scroll.pageCount; pageIdx++) {
      const left = scroll.getPageLeft(pageIdx);
      const right = left + scroll.getPageWidth(pageIdx);
      if (docX >= left && docX <= right) return pageIdx;
      const dist = docX < left ? left - docX : docX - right;
      if (dist < bestDist) { bestDist = dist; bestIdx = pageIdx; }
    }
    return bestIdx;
  }
  const rowLast = linearPageAtY(scroll, docY);
  if (!scroll.isGridMode()) return rowLast;
  const rowOffset = scroll.getPageOffset(rowLast);
  let rowFirst = rowLast;
  while (rowFirst > 0 && scroll.getPageOffset(rowFirst - 1) === rowOffset) rowFirst--;
  let bestIdx = rowFirst;
  let bestDist = Infinity;
  for (let pageIdx = rowFirst; pageIdx <= rowLast; pageIdx++) {
    const left = scroll.getPageLeft(pageIdx);
    const right = left + scroll.getPageWidth(pageIdx);
    if (docX >= left && docX <= right) return pageIdx;
    const dist = docX < left ? left - docX : docX > right ? docX - right : 0;
    if (dist < bestDist) { bestDist = dist; bestIdx = pageIdx; }
  }
  return bestIdx;
}

const mixedPages: Page[] = Array.from({ length: 13 }, (_, pageIdx) => ({
  width: 480 + (pageIdx % 3) * 70,
  height: 650 + (pageIdx % 4) * 90,
}));

const layouts: {
  name: string;
  arrangement: PageArrangement;
  movement?: PageMovementDirection;
}[] = [
  { name: 'single', arrangement: { kind: 'single' } },
  { name: 'double', arrangement: { kind: 'double' } },
  { name: 'facing', arrangement: { kind: 'facing' } },
  { name: 'multiple', arrangement: { kind: 'multiple', columns: 4, rows: 3 } },
  { name: 'auto', arrangement: { kind: 'auto' } },
  { name: 'horizontal', arrangement: { kind: 'single' }, movement: 'horizontal' },
];

test('모든 쪽 배치 snapshot은 기존 선형 AABB·prefetch 결과와 같다', () => {
  const viewports = [
    { y: -40, h: 500, x: -20, w: 900 },
    { y: 300, h: 720, x: 0, w: 1100 },
    { y: 1200, h: 0, x: 420, w: 680 },
    { y: 1700, h: 900, x: 900, w: 0 },
  ];
  for (const layout of layouts) {
    const scroll = new VirtualScroll(10);
    scroll.setPageDimensions(
      mixedPages as never,
      0.72,
      1200,
      layout.arrangement,
      layout.movement ?? 'vertical',
      720,
    );
    for (const viewport of viewports) {
      const expectedVisible = linearVisible(
        scroll,
        viewport.y,
        viewport.h,
        viewport.x,
        viewport.w,
      );
      const snapshot = scroll.getVisibilitySnapshot(
        viewport.y,
        viewport.h,
        viewport.x,
        viewport.w,
      );
      assert.deepEqual(snapshot.visiblePages, expectedVisible, `${layout.name}: visible`);
      assert.deepEqual(
        snapshot.prefetchPages,
        linearPrefetch(scroll, expectedVisible),
        `${layout.name}: prefetch`,
      );
    }
  }
});

test('동일 key는 한 불변 snapshot을 재사용하고 geometry 변경은 같은 좌표도 무효화한다', () => {
  const scroll = new VirtualScroll(10);
  scroll.setPageDimensions(mixedPages as never, 0.5, 1200, { kind: 'double' });
  const first = scroll.getVisibilitySnapshot(400, 700, 0, 1200);
  const second = scroll.getVisibilitySnapshot(400, 700, 0, 1200);
  assert.equal(second, first);
  assert.ok(Object.isFrozen(first));
  assert.ok(Object.isFrozen(first.visiblePages));
  assert.ok(Object.isFrozen(first.queryStats));

  const legacy = scroll.getVisiblePages(400, 700, 0, 1200);
  legacy.length = 0;
  assert.deepEqual(scroll.getVisiblePages(400, 700, 0, 1200), first.visiblePages);

  scroll.setPageDimensions(mixedPages as never, 0.75, 1200, { kind: 'double' });
  const changed = scroll.getVisibilitySnapshot(400, 700, 0, 1200);
  assert.notEqual(changed, first);
  assert.ok(changed.geometryRevision > first.geometryRevision);
  assert.deepEqual(changed.visiblePages, linearVisible(scroll, 400, 700, 0, 1200));
});

test('큰 세로·가로 문서는 전체 페이지 대신 교차 후보만 조사한다', () => {
  const pages = Array.from({ length: 500 }, () => ({ width: 200, height: 300 })) as never;
  const vertical = new VirtualScroll(10);
  vertical.setPageDimensions(pages, 1, 900, { kind: 'multiple', columns: 4, rows: 2 });
  const verticalSnapshot = vertical.getVisibilitySnapshot(12_000, 700, 0, 900);
  assert.deepEqual(
    verticalSnapshot.visiblePages,
    linearVisible(vertical, 12_000, 700, 0, 900),
  );
  assert.ok(verticalSnapshot.queryStats.pagesExamined <= 16);
  assert.ok(verticalSnapshot.queryStats.rowsExamined <= 4);

  const horizontal = new VirtualScroll(10);
  horizontal.setPageDimensions(pages, 1, 900, { kind: 'single' }, 'horizontal', 700);
  const horizontalSnapshot = horizontal.getVisibilitySnapshot(0, 700, 40_000, 900);
  assert.deepEqual(
    horizontalSnapshot.visiblePages,
    linearVisible(horizontal, 0, 700, 40_000, 900),
  );
  assert.ok(horizontalSnapshot.queryStats.pagesExamined <= 6);
  assert.equal(horizontalSnapshot.queryStats.rowsExamined, 1);
});

test('행/X hit test는 행 마지막·첫 쪽과 gap의 왼쪽 tie fallback을 보존한다', () => {
  const scroll = new VirtualScroll(10);
  scroll.setPageDimensions(
    Array.from({ length: 6 }, () => ({ width: 200, height: 300 })) as never,
    1,
    700,
    { kind: 'multiple', columns: 3, rows: 2 },
  );
  const y = scroll.getPageOffset(3) + 100;
  assert.equal(scroll.getPageAtY(y), 5);
  assert.equal(scroll.getRowFirstPageAtY(y), 3);
  assert.equal(scroll.getPageAtY(-1), 0, '첫 행 위 좌표는 기존처럼 0쪽으로 fallback');
  assert.equal(scroll.getPageAtPoint(scroll.getPageLeft(2), -1), 0, '첫 행 위에서는 X로 다른 쪽을 고르지 않는다');
  const leftRight = scroll.getPageLeft(3) + scroll.getPageWidth(3);
  const rightLeft = scroll.getPageLeft(4);
  assert.equal(scroll.getPageAtPoint((leftRight + rightLeft) / 2, y), 3);

  const horizontal = new VirtualScroll(10);
  horizontal.setPageDimensions(
    Array.from({ length: 4 }, () => ({ width: 200, height: 300 })) as never,
    1,
    500,
    { kind: 'single' },
    'horizontal',
    400,
  );
  const horizontalGapCenter = (
    horizontal.getPageLeft(1) + horizontal.getPageWidth(1) + horizontal.getPageLeft(2)
  ) / 2;
  assert.equal(horizontal.getPageAtPoint(horizontalGapCenter, 200), 1);
});

test('행/X hit index는 배치별 기존 선형 fallback 결과를 보존한다', () => {
  const coordinates = [-Infinity, -1, 0, 8, 250, 650, 1200, 3000, Infinity, NaN];
  for (const layout of layouts) {
    const scroll = new VirtualScroll(10);
    scroll.setPageDimensions(
      mixedPages as never,
      0.72,
      1200,
      layout.arrangement,
      layout.movement ?? 'vertical',
      720,
    );
    for (const y of coordinates) {
      assert.equal(scroll.getPageAtY(y), linearPageAtY(scroll, y), `${layout.name}: y=${y}`);
      for (const x of coordinates) {
        assert.equal(
          scroll.getPageAtPoint(x, y),
          linearPageAtPoint(scroll, x, y),
          `${layout.name}: x=${x}, y=${y}`,
        );
      }
    }
  }
});

test('reset은 이전 snapshot과 geometry를 즉시 빈 문서 경계로 바꾼다', () => {
  const scroll = new VirtualScroll(10);
  scroll.setPageDimensions(mixedPages as never, 0.5, 1200, { kind: 'double' });
  const previous = scroll.getVisibilitySnapshot(0, 720, 0, 1200);
  assert.ok(previous.visiblePages.length > 0);

  scroll.reset();
  const empty = scroll.getVisibilitySnapshot(0, 720, 0, 1200);
  assert.notEqual(empty, previous);
  assert.ok(empty.geometryRevision > previous.geometryRevision);
  assert.deepEqual(empty.visiblePages, []);
  assert.deepEqual(empty.prefetchPages, []);
  assert.equal(scroll.pageCount, 0);
  assert.equal(scroll.getTotalHeight(), 0);
  assert.equal(scroll.getTotalWidth(), 0);
});
