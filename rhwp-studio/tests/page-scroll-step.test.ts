import test from 'node:test';
import assert from 'node:assert/strict';
import { VirtualScroll } from '../src/view/virtual-scroll.ts';
import { scrollByPageStep } from '../src/view/page-scroll.ts';
import type { ViewportManager } from '../src/view/viewport-manager.ts';

// PgUp/PgDn 은 화면을 쪽 단위로 옮긴다. 이 테스트가 그 계약을 DOM 없이 고정한다.
//
// 종전에는 이 계산이 input-handler-keyboard.ts 의 switch case 안에만 있어서,
//   - 머리말/각주·개체 선택 같은 모드별 조기 반환이 키를 삼키면 무동작이었고
//   - 편집기 textarea 밖(툴바 버튼·서식 콤보)으로 포커스가 나가면 아무도 처리하지 않았다.
// 계산을 이 순수 모듈로 옮겨 편집기 경로와 전역 폴백이 같은 규칙을 쓰게 했다.
//
// 그리고 "다음 쪽 머리로 뛴다" 는 단순 규칙은 쪽이 화면보다 클 때(100% 이상 확대)
// 그 사이를 한 번도 보여주지 않고 건너뛰었다. 지금 규칙은 `min(다음 쪽 경계, 화면 하나)`
// 이라, 착지점은 여전히 쪽 경계지만 지나친 내용이 남지 않는다.

const VIEWPORT_HEIGHT = 700;
const PAGE_HEIGHT = 1000;
const GAP = 10;

function pages(n: number, width = 800, height = PAGE_HEIGHT) {
  return Array.from({ length: n }, () => ({ width, height })) as never;
}

/** setScrollTop/getScrollY 만 쓰는 ViewportManager 대역. */
function fakeViewport(height = VIEWPORT_HEIGHT) {
  let scrollY = 0;
  let scrollX = 0;
  return {
    getScrollY: () => scrollY,
    getScrollX: () => scrollX,
    getViewportSize: () => ({ width: 1000, height }),
    setScrollTop: (y: number) => { scrollY = y; },
    setScrollLeft: (x: number) => { scrollX = x; },
  } as unknown as ViewportManager & { getScrollY(): number; getScrollX(): number };
}

function fakeHorizontalViewport(width: number, height = VIEWPORT_HEIGHT) {
  let scrollY = 0;
  let scrollX = 0;
  return {
    getScrollY: () => scrollY,
    getScrollX: () => scrollX,
    getViewportSize: () => ({ width, height }),
    setScrollTop: (y: number) => { scrollY = y; },
    setScrollLeft: (x: number) => { scrollX = x; },
  } as unknown as ViewportManager & { getScrollY(): number; getScrollX(): number };
}

/** 단일 컬럼(zoom 1.0). 쪽 높이 1000 > 뷰포트 700 이라 쪽이 화면보다 크다. */
function singleColumn(pageCount: number) {
  const vs = new VirtualScroll(GAP);
  vs.setPageDimensions(pages(pageCount), 1.0, 1200);
  assert.equal(vs.isGridMode(), false, '전제: 단일 컬럼이어야 함');
  return vs;
}

/** 쪽이 화면 안에 들어오는 배치(쪽 맞춤에 해당). */
function pageFitsViewport(pageCount: number) {
  const vs = new VirtualScroll(GAP);
  vs.setPageDimensions(pages(pageCount, 800, 600), 1.0, 1200);
  assert.equal(vs.isGridMode(), false, '전제: 단일 컬럼이어야 함');
  return vs;
}

/** 행 위쪽 여백이 시작되는 문서 Y — 화면을 여기 맞추면 그 쪽 머리부터 보인다. */
const rowTop = (vs: VirtualScroll, page: number) => vs.getPageOffset(page) - GAP;
const pageLeft = (vs: VirtualScroll, page: number) => (
  vs.getPageLeftResolved(page, vs.getTotalWidth()) - GAP
);

test('쪽이 화면 안에 들어오면 한 번에 다음 쪽 머리로 간다', () => {
  const vs = pageFitsViewport(5);
  const vm = fakeViewport();

  assert.equal(scrollByPageStep(vs, vm, 1).moved, true);
  assert.equal(vm.getScrollY(), rowTop(vs, 1));

  assert.equal(scrollByPageStep(vs, vm, 1).moved, true);
  assert.equal(vm.getScrollY(), rowTop(vs, 2));

  assert.equal(scrollByPageStep(vs, vm, -1).moved, true);
  assert.equal(vm.getScrollY(), rowTop(vs, 1));
});

test('쪽이 화면보다 크면 화면 하나씩 밟되 쪽 머리에 정확히 착지한다', () => {
  const vs = singleColumn(5);
  const vm = fakeViewport();

  // 1) 첫 걸음은 화면 하나 — 다음 쪽 머리(1010)까지 한 번에 뛰면 그 사이 310px 을
  //    한 번도 보여주지 않고 건너뛴다.
  assert.equal(scrollByPageStep(vs, vm, 1).moved, true);
  assert.equal(vm.getScrollY(), VIEWPORT_HEIGHT);

  // 2) 두 번째 걸음은 화면 하나(1400)가 아니라 쪽 경계(1010)에서 멈춘다.
  assert.equal(scrollByPageStep(vs, vm, 1).moved, true);
  assert.equal(vm.getScrollY(), rowTop(vs, 1), '쪽 머리 정렬이 어긋나지 않는다');

  assert.equal(scrollByPageStep(vs, vm, 1).moved, true);
  assert.equal(vm.getScrollY(), rowTop(vs, 1) + VIEWPORT_HEIGHT);

  assert.equal(scrollByPageStep(vs, vm, 1).moved, true);
  assert.equal(vm.getScrollY(), rowTop(vs, 2), '다음 쪽 머리에도 정확히 착지한다');
});

test('PgDn 은 어떤 배치에서도 한 화면보다 많이 건너뛰지 않는다', () => {
  for (const vs of [singleColumn(6), pageFitsViewport(6)]) {
    const vm = fakeViewport();
    let previous = vm.getScrollY();
    for (let i = 0; i < 30; i++) {
      if (!scrollByPageStep(vs, vm, 1).moved) break;
      const advanced = vm.getScrollY() - previous;
      assert.ok(advanced > 0, '아래로 움직여야 한다');
      assert.ok(
        advanced <= VIEWPORT_HEIGHT + 0.5,
        `한 걸음이 화면(${VIEWPORT_HEIGHT})을 넘으면 그만큼이 안 보인 채 지나간다: ${advanced}`,
      );
      previous = vm.getScrollY();
    }
  }
});

test('PgDn 을 계속 누르면 모든 쪽 머리를 하나도 빠짐없이 지난다', () => {
  const vs = singleColumn(4);
  const vm = fakeViewport();
  const visited = new Set<number>([vm.getScrollY()]);
  for (let i = 0; i < 30 && scrollByPageStep(vs, vm, 1).moved; i++) {
    visited.add(vm.getScrollY());
  }
  for (let page = 0; page < vs.pageCount; page++) {
    assert.ok(visited.has(rowTop(vs, page)), `${page + 1}쪽 머리에 착지한 적이 있어야 한다`);
  }
});

test('PgUp 도 한 화면 이내로 올라가며 모든 쪽 머리를 지난다', () => {
  // PgDn 이 쪽 경계에 붙고 PgUp 도 쪽 경계에 붙으므로, 올라가는 경로가 내려온 경로와
  // 같은 좌표를 밟지는 않는다(쪽 머리에서 한 화면 위는 내려올 때의 직전 위치가 아니다).
  // 보장하는 것은 방향과 무관한 두 성질이다 — 건너뛰지 않고, 쪽 머리에 정확히 선다.
  const vs = singleColumn(4);
  const vm = fakeViewport();
  while (scrollByPageStep(vs, vm, 1).moved) { /* 문서 끝까지 */ }

  const visited = new Set<number>([vm.getScrollY()]);
  let previous = vm.getScrollY();
  for (let i = 0; i < 30 && scrollByPageStep(vs, vm, -1).moved; i++) {
    const climbed = previous - vm.getScrollY();
    assert.ok(climbed > 0, '위로 움직여야 한다');
    assert.ok(
      climbed <= VIEWPORT_HEIGHT + 0.5,
      `한 걸음이 화면(${VIEWPORT_HEIGHT})을 넘으면 그만큼이 안 보인 채 지나간다: ${climbed}`,
    );
    visited.add(vm.getScrollY());
    previous = vm.getScrollY();
  }

  assert.equal(vm.getScrollY(), 0, '문서 맨 위까지 되돌아온다');
  for (let page = 0; page < vs.pageCount; page++) {
    assert.ok(visited.has(rowTop(vs, page)), `${page + 1}쪽 머리에 착지한 적이 있어야 한다`);
  }
});

test('문서 끝에서는 마지막 쪽 아래까지 붙이고 멈춘다', () => {
  const vs = singleColumn(3);
  const vm = fakeViewport();
  const bottom = vs.getTotalHeight() - VIEWPORT_HEIGHT;
  for (let i = 0; i < 30 && scrollByPageStep(vs, vm, 1).moved; i++) { /* 끝까지 */ }
  assert.equal(vm.getScrollY(), bottom, '마지막 쪽의 안 보인 아래쪽까지 보여준다');
  assert.deepEqual(scrollByPageStep(vs, vm, 1), {
    moved: false,
    deltaX: 0,
    deltaY: 0,
  });
});

test('문서 처음에서는 맨 위까지만 간다', () => {
  const vs = singleColumn(3);
  const vm = fakeViewport();
  scrollByPageStep(vs, vm, 1);
  assert.equal(scrollByPageStep(vs, vm, -1).moved, true);
  assert.equal(vm.getScrollY(), 0, '첫 쪽 위쪽 여백까지 되돌아온다');
  assert.deepEqual(scrollByPageStep(vs, vm, -1), {
    moved: false,
    deltaX: 0,
    deltaY: 0,
  });
});

test('그리드 모드는 쪽이 아니라 행 단위로 움직인다 (#2560)', () => {
  const vs = new VirtualScroll(GAP);
  vs.setPageDimensions(pages(12), 0.4, 1000);
  assert.ok(vs.isGridMode() && vs.pagesPerRow > 1, '전제: 다중 열 그리드');
  const vm = fakeViewport();

  assert.equal(scrollByPageStep(vs, vm, 1).moved, true);
  assert.equal(vm.getScrollY(), rowTop(vs, vs.pagesPerRow));
});

test('맞쪽 PageDown은 첫 빈 슬롯과 마지막 단독 행을 포함한 모든 실제 행을 지난다', () => {
  const vs = new VirtualScroll(GAP);
  vs.setPageDimensions(pages(6, 800, 600), 0.5, 1200, { kind: 'facing' });
  const vm = fakeViewport(200);
  const rowStarts = vs.getRowStartPages();
  assert.deepEqual(rowStarts, [0, 1, 3, 5], '1 / 2-3 / 4-5 / 6쪽 행');

  const visited = new Set<number>([vm.getScrollY()]);
  for (let i = 0; i < 30 && scrollByPageStep(vs, vm, 1).moved; i++) {
    visited.add(vm.getScrollY());
  }
  for (const page of rowStarts) {
    assert.ok(visited.has(rowTop(vs, page)), `${page + 1}쪽이 시작하는 행을 지나야 한다`);
  }
});

test('세로 이동 delta는 실제 Y 스크롤 변화량이다 — 캐럿을 같은 화면 자리에 붙이는 근거', () => {
  const vs = singleColumn(5);
  const vm = fakeViewport();
  const before = vm.getScrollY();
  const result = scrollByPageStep(vs, vm, 1);
  assert.equal(result.deltaX, 0);
  assert.equal(result.deltaY, vm.getScrollY() - before);

  const beforeUp = vm.getScrollY();
  const up = scrollByPageStep(vs, vm, -1);
  assert.equal(up.deltaX, 0);
  assert.equal(up.deltaY, vm.getScrollY() - beforeUp);
  assert.ok(up.deltaY < 0, 'PgUp 의 deltaY 는 음수다');
});

test('가로 이동은 X축 페이지 경계와 화면 폭을 따라 이동한다', () => {
  const viewportWidth = 600;
  const vs = new VirtualScroll(GAP);
  vs.setPageDimensions(
    pages(3, 800, 1000),
    1,
    viewportWidth,
    { kind: 'single' },
    'horizontal',
    VIEWPORT_HEIGHT,
  );
  assert.equal(vs.isHorizontalMode(), true);

  const vm = fakeHorizontalViewport(viewportWidth);
  const visited = new Set<number>([vm.getScrollX()]);
  let previous = vm.getScrollX();
  for (let i = 0; i < 20; i++) {
    const result = scrollByPageStep(vs, vm, 1);
    if (!result.moved) break;
    assert.equal(result.deltaY, 0);
    assert.equal(result.deltaX, vm.getScrollX() - previous);
    assert.ok(result.deltaX > 0 && result.deltaX <= viewportWidth + 0.5);
    visited.add(vm.getScrollX());
    previous = vm.getScrollX();
  }

  for (let page = 0; page < vs.pageCount; page++) {
    assert.ok(visited.has(pageLeft(vs, page)), `${page + 1}쪽 왼쪽 경계를 지나야 한다`);
  }
  assert.equal(vm.getScrollX(), vs.getTotalWidth() - viewportWidth);
  assert.deepEqual(scrollByPageStep(vs, vm, 1), {
    moved: false,
    deltaX: 0,
    deltaY: 0,
  });
});

test('가로 PageUp은 한 화면 이내로 이동하며 모든 페이지 왼쪽 경계를 지난다', () => {
  const viewportWidth = 600;
  const vs = new VirtualScroll(GAP);
  vs.setPageDimensions(
    pages(4, 800, 1000),
    1,
    viewportWidth,
    { kind: 'single' },
    'horizontal',
    VIEWPORT_HEIGHT,
  );
  const vm = fakeHorizontalViewport(viewportWidth);
  while (scrollByPageStep(vs, vm, 1).moved) { /* 문서 오른쪽 끝까지 */ }

  const visited = new Set<number>([vm.getScrollX()]);
  let previous = vm.getScrollX();
  for (let i = 0; i < 30; i++) {
    const result = scrollByPageStep(vs, vm, -1);
    if (!result.moved) break;
    assert.equal(result.deltaY, 0);
    assert.ok(result.deltaX < 0 && -result.deltaX <= viewportWidth + 0.5);
    visited.add(vm.getScrollX());
    previous = vm.getScrollX();
  }

  assert.equal(vm.getScrollX(), 0);
  for (let page = 0; page < vs.pageCount; page++) {
    assert.ok(visited.has(pageLeft(vs, page)), `${page + 1}쪽 왼쪽 경계를 지나야 한다`);
  }
  assert.ok(previous >= 0);
});

test('문서가 없으면 아무 것도 하지 않는다', () => {
  const vs = new VirtualScroll(GAP);
  const vm = fakeViewport();
  assert.deepEqual(scrollByPageStep(vs, vm, 1), {
    moved: false,
    deltaX: 0,
    deltaY: 0,
  });
  assert.equal(vm.getScrollY(), 0);
});
