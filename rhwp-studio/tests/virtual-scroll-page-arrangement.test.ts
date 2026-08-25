import test from 'node:test';
import assert from 'node:assert/strict';

import { VirtualScroll } from '../src/view/virtual-scroll.ts';

function pages(n: number, width = 800, height = 1000) {
  return Array.from({ length: n }, () => ({ width, height })) as never;
}

test('자동은 기존 50% 임계값과 뷰포트 최대 열 계산을 보존한다', () => {
  const scroll = new VirtualScroll(10);
  scroll.setPageDimensions(pages(6), 0.4, 2000, { kind: 'auto' });
  assert.equal(scroll.getColumns(), 6);
  assert.equal(scroll.getPageOffset(0), scroll.getPageOffset(5));

  scroll.setPageDimensions(pages(6), 0.51, 2000, { kind: 'auto' });
  assert.equal(scroll.getColumns(), 1);
  assert.notEqual(scroll.getPageOffset(0), scroll.getPageOffset(1));
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
  const secondSlotLeft = firstSlotLeft + scroll.getPageWidth(0) + 10;
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
