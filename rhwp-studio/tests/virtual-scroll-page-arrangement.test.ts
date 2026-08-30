import test from 'node:test';
import assert from 'node:assert/strict';

import { VirtualScroll } from '../src/view/virtual-scroll.ts';

function pages(n: number, width = 800, height = 1000) {
  return Array.from({ length: n }, () => ({ width, height })) as never;
}

test('자동은 50% gate 없이 뷰포트에 들어가는 열 수를 쓴다', () => {
  const scroll = new VirtualScroll(10);
  scroll.setPageDimensions(pages(6), 0.4, 2000, { kind: 'auto' });
  assert.equal(scroll.getColumns(), 6);
  assert.equal(scroll.getPageOffset(0), scroll.getPageOffset(5));

  // 0.51 에서도 폭이 4열을 담으면 1열이 아니라 4열이다 (#6040).
  scroll.setPageDimensions(pages(6), 0.51, 2000, { kind: 'auto' });
  assert.equal(scroll.getColumns(), 4);
  assert.equal(scroll.getPageOffset(0), scroll.getPageOffset(3));
  assert.notEqual(scroll.getPageOffset(0), scroll.getPageOffset(4));
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
