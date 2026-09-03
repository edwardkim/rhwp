import test from 'node:test';
import assert from 'node:assert/strict';

import {
  resolveAutoPageColumns,
  VirtualScroll,
} from '../src/view/virtual-scroll.ts';

function pages(n: number, width = 800, height = 1000) {
  return Array.from({ length: n }, () => ({ width, height })) as never;
}

test('자동은 절대 배율 gate 없이 표시 geometry로 열 수를 계산한다', () => {
  const scroll = new VirtualScroll(10);
  scroll.setPageDimensions(pages(6), 0.4, 2000, { kind: 'auto' });
  assert.equal(scroll.getColumns(), 6);
  assert.equal(scroll.getPageOffset(0), scroll.getPageOffset(5));

  scroll.setPageDimensions(pages(6), 0.51, 2000, { kind: 'auto' });
  assert.equal(scroll.getColumns(), 4);
  assert.equal(scroll.getPageOffset(0), scroll.getPageOffset(3));
  assert.notEqual(scroll.getPageOffset(0), scroll.getPageOffset(4));
});

test('자동은 50% 위에서도 두 쪽 폭이면 2열을 선택하고 50% 전후에 1열을 끼우지 않는다', () => {
  const scroll = new VirtualScroll(10);
  for (const zoom of [0.51, 0.5, 0.49]) {
    scroll.setPageDimensions(pages(6), zoom, 900, { kind: 'auto' });
    assert.equal(scroll.getColumns(), 2, `zoom ${zoom}`);
    assert.equal(scroll.getPageOffset(0), scroll.getPageOffset(1));
  }
});

test('자동 열 후보는 실제 페이지 수를 넘지 않고 실제 점유 묶음을 중앙 정렬한다', () => {
  const viewportWidth = 2000;
  for (const zoom of [0.27, 0.17]) {
    const scroll = new VirtualScroll(10);
    scroll.setPageDimensions(pages(3), zoom, viewportWidth, { kind: 'auto' });

    assert.equal(scroll.getColumns(), 3, `zoom ${zoom}`);
    const groupLeft = scroll.getPageLeft(0);
    const groupRight = scroll.getPageLeft(2) + scroll.getPageWidth(2);
    assert.ok(
      Math.abs((groupLeft + groupRight) / 2 - viewportWidth / 2) <= 1,
      `zoom ${zoom}: 실제 3쪽 묶음이 viewport 중앙이어야 한다`,
    );
  }
});

test('자동 열 후보는 잘못된 geometry에 1열로 수렴하고 열 경계에 hysteresis를 적용한다', () => {
  assert.equal(resolveAutoPageColumns({
    pageCount: 3,
    viewportWidth: 0,
    displayedPageWidth: 400,
    pageGap: 6,
  }), 1);
  assert.equal(resolveAutoPageColumns({
    pageCount: 3,
    viewportWidth: 1000,
    displayedPageWidth: 0,
    pageGap: 6,
  }), 1);

  const boundary = 2 * 400 + 6;
  assert.equal(resolveAutoPageColumns({
    pageCount: 3,
    viewportWidth: boundary + 4,
    displayedPageWidth: 400,
    pageGap: 6,
    committedColumns: 1,
  }), 1, '증가 경계 +8px 안에서는 기존 1열 commit 유지');
  assert.equal(resolveAutoPageColumns({
    pageCount: 3,
    viewportWidth: boundary + 8,
    displayedPageWidth: 400,
    pageGap: 6,
    committedColumns: 1,
  }), 2, '증가 경계 +8px에서 2열 commit 허용');
  assert.equal(resolveAutoPageColumns({
    pageCount: 3,
    viewportWidth: boundary - 4,
    displayedPageWidth: 400,
    pageGap: 6,
    committedColumns: 2,
  }), 2, '감소 경계 -8px 안에서는 기존 2열 commit 유지');
  assert.equal(resolveAutoPageColumns({
    pageCount: 3,
    viewportWidth: boundary - 9,
    displayedPageWidth: 400,
    pageGap: 6,
    committedColumns: 2,
  }), 1, '감소 경계 -8px를 벗어나면 1열 commit');
});

test('같은 VirtualScroll의 자동 열 commit은 연속 zoom/resize 경계 입력에서 왕복하지 않는다', () => {
  const scroll = new VirtualScroll(10);
  const setViewport = (viewportWidth: number): number => {
    scroll.setPageDimensions(pages(3, 400, 600), 1, viewportWidth, { kind: 'auto' });
    return scroll.getColumns();
  };

  assert.equal(setViewport(800), 1);
  assert.equal(setViewport(814), 1, '증가 dead band 안에서는 1열 유지');
  assert.equal(setViewport(806), 1, '경계를 되짚어도 1열 유지');
  assert.equal(setViewport(818), 2, '증가 dead band를 지난 뒤에만 2열 commit');
  assert.equal(setViewport(806), 2, '감소 dead band 안에서는 2열 유지');
  assert.equal(setViewport(801), 1, '감소 dead band를 벗어나면 1열 commit');
});

test('명시 배치와 문서 교체는 이전 자동 열 commit을 다음 자동 계산에 넘기지 않는다', () => {
  const scroll = new VirtualScroll(10);
  const samplePages = pages(3, 400, 600);

  scroll.setPageDimensions(samplePages, 1, 800, { kind: 'auto' });
  assert.equal(scroll.getColumns(), 1);

  scroll.setPageDimensions(samplePages, 1, 814, { kind: 'double' });
  scroll.setPageDimensions(samplePages, 1, 814, { kind: 'auto' });
  assert.equal(scroll.getColumns(), 2, '명시 배치를 거치면 자동 배치는 현재 geometry에서 다시 시작');

  scroll.setPageDimensions(samplePages, 1, 806, { kind: 'auto' });
  assert.equal(scroll.getColumns(), 2, '감소 dead band 안에서는 기존 2열 commit 유지');
  scroll.resetAutoColumnCommit();
  scroll.setPageDimensions(samplePages, 1, 806, { kind: 'auto' });
  assert.equal(scroll.getColumns(), 1, '문서 교체 reset 뒤에는 현재 geometry 후보를 즉시 commit');
});

test('한 쪽은 낮은 배율에서도 한 행 한 쪽과 중앙 정렬을 유지한다', () => {
  const scroll = new VirtualScroll(10);
  scroll.setPageDimensions(pages(3), 0.25, 1200, { kind: 'single' });

  assert.equal(scroll.getColumns(), 1);
  assert.equal(scroll.getPageLeft(0), -1);
  assert.notEqual(scroll.getPageOffset(0), scroll.getPageOffset(1));
});

test('두 쪽은 1-2, 3-4를 같은 행에 놓는다', () => {
  const scroll = new VirtualScroll(10);
  scroll.setPageDimensions(pages(5), 0.6, 1200, { kind: 'double' });

  assert.equal(scroll.getColumns(), 2);
  assert.equal(scroll.getPageOffset(0), scroll.getPageOffset(1));
  assert.equal(scroll.getPageOffset(2), scroll.getPageOffset(3));
  assert.ok(scroll.getPageOffset(2) > scroll.getPageOffset(0));
  assert.ok(scroll.getPageLeft(0) < scroll.getPageLeft(1));
});

test('맞쪽은 첫 홀수 쪽을 오른쪽에 두고 이후 짝수·홀수를 좌우에 놓는다', () => {
  const scroll = new VirtualScroll(10);
  scroll.setPageDimensions(pages(5), 0.5, 1200, { kind: 'facing' });

  assert.equal(scroll.getColumns(), 2);
  assert.ok(scroll.getPageLeft(0) > 1200 / 2, '1쪽은 첫 행의 오른쪽 슬롯');
  assert.ok(scroll.getPageOffset(1) > scroll.getPageOffset(0));
  assert.equal(scroll.getPageOffset(1), scroll.getPageOffset(2));
  assert.ok(scroll.getPageLeft(1) < scroll.getPageLeft(2), '2쪽 왼쪽, 3쪽 오른쪽');
});

test('여러 쪽은 지정한 열 수를 줌과 무관하게 유지한다', () => {
  const scroll = new VirtualScroll(10);
  scroll.setPageDimensions(
    pages(10),
    0.8,
    1600,
    { kind: 'multiple', columns: 4, rows: 2 },
  );

  assert.equal(scroll.getColumns(), 4);
  assert.equal(scroll.getPageOffset(0), scroll.getPageOffset(3));
  assert.ok(scroll.getPageOffset(4) > scroll.getPageOffset(3));
});

test('크기가 다른 페이지는 동일 슬롯 안에서 가운데 정렬된다', () => {
  const scroll = new VirtualScroll(10);
  const mixed = [
    { width: 800, height: 1000 },
    { width: 600, height: 900 },
  ] as never;
  scroll.setPageDimensions(mixed, 0.5, 1200, { kind: 'double' });

  const firstSlotLeft = scroll.getPageLeft(0);
  const secondSlotLeft = firstSlotLeft + scroll.getPageWidth(0) + scroll.getPageGap();
  assert.equal(
    scroll.getPageLeft(1),
    secondSlotLeft + (scroll.getPageWidth(0) - scroll.getPageWidth(1)) / 2,
  );
});

test('맞쪽 첫 행의 프리페치는 다음 실제 행 전체를 포함한다', () => {
  const scroll = new VirtualScroll(10);
  scroll.setPageDimensions(pages(6), 0.5, 1200, { kind: 'facing' });

  const prefetched = scroll.getPrefetchPages(scroll.getPageOffset(0), 100);
  assert.deepEqual(prefetched, [0, 1, 2]);
});

test('가로 쪽 이동은 한 쪽씩 한 행에 놓고 뷰포트 높이에서 세로 중앙 정렬한다', () => {
  const scroll = new VirtualScroll(10);
  scroll.setPageDimensions(
    pages(4, 200, 300),
    1,
    500,
    { kind: 'single' },
    'horizontal',
    400,
  );

  assert.equal(scroll.isHorizontalMode(), true);
  assert.equal(scroll.getPageLeft(0), 10);
  assert.equal(scroll.getPageLeft(1), 220);
  assert.equal(scroll.getPageOffset(0), 50);
  assert.equal(scroll.getPageOffset(3), 50);
  assert.equal(scroll.getTotalWidth(), 850);
  assert.equal(scroll.getTotalHeight(), 400);
  assert.equal(scroll.getPageAtPoint(230, 200), 1);
});

test('가로 쪽 이동은 가로 가시 범위만 렌더하고 양옆 한 쪽만 프리페치한다', () => {
  const scroll = new VirtualScroll(10);
  scroll.setPageDimensions(
    pages(8, 200, 300),
    1,
    400,
    { kind: 'single' },
    'horizontal',
    400,
  );

  assert.deepEqual(scroll.getVisiblePages(0, 400, 0, 200), [0]);
  assert.deepEqual(scroll.getPrefetchPages(0, 400, 0, 200), [0, 1]);
  assert.deepEqual(scroll.getVisiblePages(0, 400, 430, 200), [2]);
  assert.deepEqual(scroll.getPrefetchPages(0, 400, 430, 200), [1, 2, 3]);
});

test('저배율 페이지 간격은 모든 배치에서 같은 CSS px 하한을 사용한다', () => {
  const layouts = [
    { arrangement: { kind: 'single' } as const, firstNextRow: 1 },
    { arrangement: { kind: 'double' } as const, firstNextRow: 2 },
    { arrangement: { kind: 'facing' } as const, firstNextRow: 1 },
    { arrangement: { kind: 'multiple', columns: 3, rows: 2 } as const, firstNextRow: 3 },
  ];

  for (const { arrangement, firstNextRow } of layouts) {
    const scroll = new VirtualScroll(10);
    scroll.setPageDimensions(pages(8, 200, 300), 0.1, 1000, arrangement);
    assert.equal(scroll.getPageGap(), 6, `${arrangement.kind}의 최소 gap`);
    assert.equal(
      scroll.getPageOffset(firstNextRow) - scroll.getPageOffset(0),
      300 * 0.1 + 6,
      `${arrangement.kind}의 행 간격`,
    );
  }

  const horizontal = new VirtualScroll(10);
  horizontal.setPageDimensions(
    pages(3, 200, 300),
    0.1,
    1000,
    { kind: 'single' },
    'horizontal',
    500,
  );
  assert.equal(horizontal.getPageGap(), 6);
  assert.equal(
    horizontal.getPageLeft(1) - horizontal.getPageLeft(0),
    200 * 0.1 + 6,
  );
});

test('고배율 페이지 간격은 모든 배치 좌표에 배율 비례값으로 반영된다', () => {
  const scroll = new VirtualScroll(10);
  scroll.setPageDimensions(pages(4, 200, 300), 2, 1200, { kind: 'double' });
  assert.equal(scroll.getPageGap(), 20);
  assert.equal(scroll.getPageLeft(1) - scroll.getPageLeft(0), 200 * 2 + 20);
  assert.equal(scroll.getPageOffset(2) - scroll.getPageOffset(0), 300 * 2 + 20);
});
