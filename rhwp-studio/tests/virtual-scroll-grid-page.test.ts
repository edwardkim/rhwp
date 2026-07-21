import test from 'node:test';
import assert from 'node:assert/strict';
import { VirtualScroll } from '../src/view/virtual-scroll.ts';

// [#2560] 그리드 모드에서 "현재 쪽" 판정 가드.
//
// layoutGrid 는 한 행의 모든 쪽에 같은 offset(rowTop)을 넣는다. 뒤에서부터
// 스캔하는 getPageAtY 는 그래서 행의 *마지막* 쪽을 돌려준다 — getPageAtPoint 가
// X 로 좁히기 위한 스캔 끝점으로 쓰는 의도된 의미다.
//
// 문제는 "현재 쪽" 이 필요한 소비처가 그걸 직접 쓴 것이었다:
//   - 상태바(canvas-view.ts) → 3열이면 첫 행에서 "3 / N" 으로 표시
//   - PageUp(input-handler-keyboard.ts) → 행의 마지막 쪽에서 -1 하면 같은 행에
//     머물러 offset 이 동일 → 스크롤이 전혀 움직이지 않음(PageUp 무동작)
//
// getRowFirstPageAtY 와 pagesPerRow 로 두 소비처를 고쳤고, 이 테스트가 그 계약을
// 고정한다. VirtualScroll 은 타입만 import 하는 순수 모듈이라 DOM 없이 검증된다.

/** width×height 페이지 n개. */
function pages(n: number, width = 800, height = 1000) {
  return Array.from({ length: n }, () => ({ width, height })) as never;
}

/** 3열 그리드가 되도록 충분히 넓은 뷰포트. zoom 0.4 < 0.5 임계값. */
const ZOOM = 0.4;
const VIEWPORT = 2000;

test('그리드 모드에서 getPageAtY 는 행의 마지막 쪽을 준다(의도된 의미)', () => {
  const vs = new VirtualScroll(10);
  vs.setPageDimensions(pages(6), ZOOM, VIEWPORT);
  assert.ok(vs.isGridMode(), '전제: 그리드 모드여야 함');
  assert.ok(vs.pagesPerRow > 1, `전제: 다중 열이어야 함 (실제 ${vs.pagesPerRow})`);

  const firstRowY = vs.getPageOffset(0);
  assert.equal(
    vs.getPageAtY(firstRowY),
    vs.pagesPerRow - 1,
    'getPageAtPoint 의 스캔 끝점으로 쓰이므로 행의 마지막 쪽이어야 함',
  );
});

test('getRowFirstPageAtY 는 행의 첫 쪽을 준다 — 상태바가 1쪽을 표시할 수 있어야 함', () => {
  const vs = new VirtualScroll(10);
  vs.setPageDimensions(pages(6), ZOOM, VIEWPORT);

  const firstRowY = vs.getPageOffset(0);
  assert.equal(
    vs.getRowFirstPageAtY(firstRowY),
    0,
    '첫 행의 현재 쪽은 0 이어야 한다(종전엔 행의 마지막 쪽이라 1쪽을 표시할 수 없었다)',
  );
});

test('PageUp 이 이전 행으로 실제로 이동한다 — 행 단위 스텝', () => {
  const vs = new VirtualScroll(10);
  vs.setPageDimensions(pages(9), ZOOM, VIEWPORT);
  const step = vs.pagesPerRow;

  // 둘째 행 위에 커서가 있다고 가정.
  const secondRowY = vs.getPageOffset(step);
  const current = vs.getRowFirstPageAtY(secondRowY);
  assert.equal(current, step, '둘째 행의 첫 쪽이어야 함');

  const target = Math.max(0, current - step);
  assert.notEqual(
    vs.getPageOffset(target),
    vs.getPageOffset(current),
    'PageUp 목표는 offset 이 달라야 스크롤이 움직인다 — 종전 ±1 은 같은 행이라 무동작이었다',
  );
});

test('단일 컬럼 모드에서는 종전 동작과 동일하다', () => {
  const vs = new VirtualScroll(10);
  // viewportWidth=0 → 그리드 모드 아님
  vs.setPageDimensions(pages(4), 1.0, 0);
  assert.ok(!vs.isGridMode(), '전제: 단일 컬럼');
  assert.equal(vs.pagesPerRow, 1, '단일 컬럼은 행당 1쪽');

  const y = vs.getPageOffset(2);
  assert.equal(
    vs.getRowFirstPageAtY(y),
    vs.getPageAtY(y),
    '단일 컬럼에서는 두 진입점이 동치여야 한다(회귀 없음)',
  );
});
