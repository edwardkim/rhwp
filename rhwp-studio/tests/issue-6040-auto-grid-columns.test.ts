import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';

import { resolvePageGap } from '../src/view/page-gap.ts';
import {
  autoGridColumnCandidate,
  commitAutoGridColumns,
  VirtualScroll,
} from '../src/view/virtual-scroll.ts';

function pages(n: number, width = 800, height = 1000) {
  return Array.from({ length: n }, () => ({ width, height })) as never;
}

function occupiedCenter(scroll: VirtualScroll, pageCount: number): number {
  const first = scroll.getPageLeft(0);
  const last = scroll.getPageLeft(pageCount - 1) + scroll.getPageWidth(pageCount - 1);
  return (first + last) / 2;
}

test('자동 열 후보는 50% gate 없이 폭으로 계산하고 페이지 수를 넘지 않는다', () => {
  const gap = resolvePageGap(0.27, 10);
  const pw = 800 * 0.27;
  // 넓은 뷰포트: 폭만 보면 8열 이상이지만 3쪽 문서는 3열.
  const candidate = autoGridColumnCandidate(2000, pw, gap, 3);
  assert.equal(candidate, 3);

  const twoFit = autoGridColumnCandidate(1000, 800 * 0.51, resolvePageGap(0.51, 10), 6);
  assert.equal(twoFit, 2, '두 쪽만 들어가는 폭에서는 2열 (51%여도 1열이 아님)');
});

test('줌과 리사이즈가 같은 입력에서 같은 자동 열 후보를 낸다', () => {
  const zoom = 0.4;
  const viewport = 1600;
  const pageWidth = 800;
  const gap = resolvePageGap(zoom, 10);
  const fromZoom = autoGridColumnCandidate(viewport, pageWidth * zoom, gap, 9);
  const fromResize = autoGridColumnCandidate(viewport, pageWidth * zoom, gap, 9);
  assert.equal(fromZoom, fromResize);
  assert.ok(fromZoom >= 2);
});

test('3쪽 문서는 27%·17%에서 빈 열 없이 편집 영역 가운데에 온다', () => {
  const scroll = new VirtualScroll(10);
  for (const zoom of [0.27, 0.17]) {
    scroll.setPageDimensions(pages(3), zoom, 2000, { kind: 'auto' });
    assert.equal(scroll.getColumns(), 3, `${zoom} 열 수는 페이지 수`);
    assert.equal(scroll.getPageOffset(0), scroll.getPageOffset(2));
    const center = occupiedCenter(scroll, 3);
    assert.ok(
      Math.abs(center - 1000) <= 1,
      `${zoom} 점유 묶음 중심 ${center} 이 뷰포트 중심 1000 에서 1px 이내`,
    );
  }
});

test('두 쪽만 들어가는 폭에서는 50%를 넘어도 2열이다', () => {
  const scroll = new VirtualScroll(10);
  const viewport = 1000;
  scroll.setPageDimensions(pages(6), 0.51, viewport, { kind: 'auto' });
  assert.equal(scroll.getColumns(), 2);
  assert.equal(scroll.getPageOffset(0), scroll.getPageOffset(1));
  assert.notEqual(scroll.getPageOffset(0), scroll.getPageOffset(2));
});

test('핀치 hold 열 수는 후보가 바뀌어도 토폴로지를 유지한다', () => {
  const scroll = new VirtualScroll(10);
  scroll.setPageDimensions(pages(6), 1, 900, { kind: 'auto' });
  assert.equal(scroll.getColumns(), 1);
  const held = scroll.getLayoutTopologyKey();
  scroll.setPageDimensions(pages(6), 0.27, 2000, { kind: 'auto' }, 'vertical', 0, 1);
  assert.equal(scroll.getColumns(), 1, '핀치 중에는 1열 snapshot 유지');
  assert.equal(scroll.getLayoutTopologyKey(), held);
  scroll.setPageDimensions(pages(6), 0.27, 2000, { kind: 'auto' });
  assert.ok(scroll.getColumns() > 1, '정착 후 후보를 commit');
});

test('히스테리시스는 같은 임계값 주변에서 열 수를 왕복하지 않는다', () => {
  const pw = 400;
  const gap = 6;
  const pageCount = 6;
  const threeWidth = 3 * pw + 2 * gap;
  const committed = commitAutoGridColumns(3, 2, threeWidth + 20, pw, gap, pageCount);
  assert.equal(committed, 3);
  const stillThree = commitAutoGridColumns(2, 3, threeWidth - 4, pw, gap, pageCount);
  assert.equal(stillThree, 3, '4px 부족은 아직 3열 유지');
  const drop = commitAutoGridColumns(2, 3, threeWidth - 20, pw, gap, pageCount);
  assert.equal(drop, 2);
});

test('CanvasView 줌 정착은 전량 Canvas 해제를 하지 않고 핀치 열을 고정한다', () => {
  const source = readFileSync(
    fileURLToPath(new URL('../src/view/canvas-view.ts', import.meta.url)),
    'utf8',
  );
  const start = source.search(/\n  (?:private )?onZoomChanged\(/);
  const rest = source.slice(start + 1);
  const relEnd = rest.search(/\n  (?:private )?updateRenderedPageZoomPreview\(/);
  const method = source.slice(start, start + 1 + relEnd);
  assert.match(method, /pinchHoldColumns/);
  assert.match(method, /rerenderVisiblePagesAtCurrentZoom/);
  assert.doesNotMatch(method, /releaseAllRenderedPages\(\)/);
  assert.match(source, /this\.pinchHoldColumns,/);
});
