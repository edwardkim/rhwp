import test from 'node:test';
import assert from 'node:assert/strict';
import { VirtualScroll } from '../src/view/virtual-scroll.ts';
import { calculateAnchoredScroll } from '../src/view/zoom-anchor.ts';

const pages = [{ width: 800, height: 1000 }] as never;

test('horizontal pan coordinates center the page when content overflows', () => {
  const viewportWidth = 900;
  const scroll = new VirtualScroll();

  scroll.setPageDimensions(pages, 1.25, viewportWidth);
  const overflowLeft = scroll.getPageLeft(0);
  const overflowCenter = scroll.getCenteredScrollLeft(viewportWidth);
  assert.equal(
    overflowLeft - overflowCenter,
    (viewportWidth - scroll.getPageWidth(0)) / 2,
  );
});

// [#3591] 팬 정책: pan = (콘텐츠폭 <= 창폭) ? 0 : clamp(창폭 × 0.25, 80, 240)
test('no pan space when the content fits inside the viewport', () => {
  const viewportWidth = 900;
  const scroll = new VirtualScroll();

  // zoom 0.5 → 페이지 400 + 여백 40 = 440 < 900: 팬 불필요
  scroll.setPageDimensions(pages, 0.5, viewportWidth);
  assert.equal(scroll.getTotalWidth(), 440, '콘텐츠가 창에 들어가면 totalWidth 는 base 그대로');
  assert.equal(scroll.getPageLeft(0), -1, '단일 열은 CSS 중앙 정렬(-1)을 유지');
  assert.equal(scroll.getCenteredScrollLeft(viewportWidth), 0, '가로 스크롤 없음');
});

test('pan space is clamped so large screens do not grow the scroll area', () => {
  const scroll = new VirtualScroll();
  const panOf = (zoom: number, viewportWidth: number) => {
    scroll.setPageDimensions(pages, zoom, viewportWidth);
    return (scroll.getTotalWidth() - (800 * zoom + 40)) / 2;
  };

  // 상한: 창 1000 × 0.25 = 250 → 240 으로 클램프 (콘텐츠 1640 > 창)
  assert.equal(panOf(2, 1000), 240);
  // 비율 구간: 창 900 × 0.25 = 225
  assert.equal(panOf(4, 900), 225);
  // 비율 구간: 창 600 × 0.25 = 150
  assert.equal(panOf(6, 600), 150);
  // 하한: 창 200 × 0.25 = 50 → 80 으로 클램프
  assert.equal(panOf(2, 200), 80);

  // 큰 화면에서도 팬은 상한을 넘지 않는다 — 콘텐츠가 창보다 넓은 조건을 유지하려면
  // 줌을 키워야 한다(zoom 6 → 4840 > 3840).
  assert.equal(panOf(6, 3840), 240);
  assert.equal(panOf(6, 1920), 240);
  // 4K 최대화라도 스크롤 영역은 문서 폭 + 480 으로 수렴한다.
  scroll.setPageDimensions(pages, 6, 3840);
  assert.equal(scroll.getTotalWidth(), 800 * 6 + 40 + 480);
});

test('grid layout keeps its own centering when no pan space is applied', () => {
  const scroll = new VirtualScroll();
  const gridPages = [
    { width: 400, height: 500 },
    { width: 400, height: 500 },
  ] as never;

  // zoom 0.25 → 그리드 모드. 두 페이지가 한 행에 들어가고 콘텐츠는 창 안에 있다.
  scroll.setPageDimensions(gridPages, 0.25, 900);
  assert.ok(scroll.isGridMode());
  assert.equal(scroll.getTotalWidth(), 900, '팬 없음: totalWidth 는 창 폭(그리드 base)');

  const left0 = scroll.getPageLeft(0);
  const left1 = scroll.getPageLeft(1);
  assert.ok(left0 >= 0 && left1 > left0, '그리드는 명시 left 를 유지');
  assert.equal(
    left1 - left0,
    scroll.getPageWidth(0) + 10,
    '열 간격은 페이지 폭 + gap(10)',
  );
});

test('pointer anchor remains representable across the viewport-width boundary', () => {
  const viewportWidth = 900;
  const anchor = { x: 0.35, y: 0.75 };
  const scroll = new VirtualScroll();

  scroll.setPageDimensions(pages, 0.54, viewportWidth);
  const oldBox = {
    left: scroll.getPageLeft(0),
    top: scroll.getPageOffset(0),
    width: scroll.getPageWidth(0),
    height: scroll.getPageHeight(0),
  };
  const viewport = {
    width: viewportWidth,
    height: 650,
    scrollLeft: scroll.getCenteredScrollLeft(viewportWidth),
    scrollTop: 0,
  };

  scroll.setPageDimensions(pages, 1.2, viewportWidth);
  const newBox = {
    left: scroll.getPageLeft(0),
    top: scroll.getPageOffset(0),
    width: scroll.getPageWidth(0),
    height: scroll.getPageHeight(0),
  };
  const forward = calculateAnchoredScroll(oldBox, newBox, viewport, anchor);
  assert.ok(forward.scrollLeft >= 0);
  // [#3591] 팬 여백이 얇아지면 앵커 계산이 스크롤 범위를 넘을 수 있다. 순수 함수는
  // 앵커 보존값을 그대로 돌려주고, 범위 제한은 호출부(CanvasView.clampScrollLeft)와
  // 브라우저의 scrollLeft 대입이 담당한다. 여기서는 왕복 가역성만 계약으로 고정한다.
  const reverse = calculateAnchoredScroll(
    newBox,
    oldBox,
    { ...viewport, ...forward },
    anchor,
  );
  assert.ok(Math.abs(reverse.scrollLeft - viewport.scrollLeft) < 1e-9);
});

// [#3591] 그리드는 layoutGrid 가 중앙을 잡으므로 팬을 주지 않는다.
// 그리드 첫 진입(zoom 0.5)에서만 팬이 붙어 스크롤 여지가 생기고 문서가 밀리던 회귀 가드.
test('grid mode never receives pan space at any zoom', () => {
  const scroll = new VirtualScroll();
  const gridPages = Array.from({ length: 5 }, () => ({ width: 794, height: 1123 })) as never;
  const viewportWidth = 1229;

  for (const zoom of [0.5, 0.45, 0.35, 0.3, 0.25]) {
    scroll.setPageDimensions(gridPages, zoom, viewportWidth);
    assert.ok(scroll.isGridMode(), `zoom ${zoom} 은 그리드 모드여야 한다`);
    const overflow = scroll.getTotalWidth() - viewportWidth;
    assert.ok(
      overflow <= 10,
      `zoom ${zoom}: 그리드에 팬이 붙어 가로 스크롤 여지가 생겼다 (초과 ${overflow}px)`,
    );
  }
});
