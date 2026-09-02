import type { VirtualScroll } from './virtual-scroll';
import type { ViewportManager } from './viewport-manager';

/** -1 = PgUp(위/왼쪽), 1 = PgDn(아래/오른쪽) */
export type PageScrollDirection = -1 | 1;

export interface PageScrollResult {
  /** 실제로 스크롤이 일어났으면 true */
  moved: boolean;
  /** 실제 X축 스크롤 변화량(px, 오른쪽 이동이 +). */
  deltaX: number;
  /** 실제 Y축 스크롤 변화량(px, 아래 이동이 +). */
  deltaY: number;
}

const NO_MOVE: PageScrollResult = { moved: false, deltaX: 0, deltaY: 0 };

/** 부동소수 오차로 "같은 자리에 또 스크롤" 이 나지 않게 하는 하한. */
const EPSILON = 0.5;

/**
 * 행 `pageIdx` 의 위쪽 여백이 시작되는 문서 Y — 화면을 여기에 맞추면 그 쪽의 위
 * 여백부터 보인다. 그리드 모드에서는 한 행의 쪽들이 같은 offset 을 가지므로 행 값이다.
 */
function rowTop(virtualScroll: VirtualScroll, pageIdx: number): number {
  return virtualScroll.getPageOffset(pageIdx) - virtualScroll.gap;
}

/** 스크롤 가능한 최대 위치. 이 아래로는 문서가 없다. */
function maxScrollTop(virtualScroll: VirtualScroll, viewportHeight: number): number {
  return Math.max(0, virtualScroll.getTotalHeight() - viewportHeight);
}

/** 가로 이동에서 `pageIdx` 쪽의 왼쪽 여백이 시작되는 문서 X. */
function pageLeft(virtualScroll: VirtualScroll, pageIdx: number, totalWidth: number): number {
  return virtualScroll.getPageLeftResolved(pageIdx, totalWidth)
    - virtualScroll.gap;
}

/** `scrollY` 바로 아래의 행 경계. 더 없으면 문서 끝. */
function nextRowBoundary(
  virtualScroll: VirtualScroll,
  scrollY: number,
  rowStarts: readonly number[],
  limit: number,
): number {
  for (const page of rowStarts) {
    const top = rowTop(virtualScroll, page);
    if (top > scrollY + EPSILON) return top;
  }
  return limit;
}

/** `scrollY` 바로 위의 행 경계. 더 없으면 문서 처음. */
function prevRowBoundary(
  virtualScroll: VirtualScroll,
  scrollY: number,
  rowStarts: readonly number[],
): number {
  for (let row = rowStarts.length - 1; row >= 0; row--) {
    const page = rowStarts[row];
    const top = rowTop(virtualScroll, page);
    if (top < scrollY - EPSILON) return top;
  }
  return 0;
}

/** `scrollX` 바로 오른쪽의 페이지 경계. 더 없으면 문서 오른쪽 끝. */
function nextPageBoundary(
  virtualScroll: VirtualScroll,
  scrollX: number,
  limit: number,
  totalWidth: number,
): number {
  for (let pageIdx = 0; pageIdx < virtualScroll.pageCount; pageIdx++) {
    const left = pageLeft(virtualScroll, pageIdx, totalWidth);
    if (left > scrollX + EPSILON) return left;
  }
  return limit;
}

/** `scrollX` 바로 왼쪽의 페이지 경계. 더 없으면 문서 왼쪽 끝. */
function prevPageBoundary(
  virtualScroll: VirtualScroll,
  scrollX: number,
  totalWidth: number,
): number {
  for (let pageIdx = virtualScroll.pageCount - 1; pageIdx >= 0; pageIdx--) {
    const left = pageLeft(virtualScroll, pageIdx, totalWidth);
    if (left < scrollX - EPSILON) return left;
  }
  return 0;
}

/**
 * PgUp/PgDn — 쪽 이동 방향에 맞춰 화면을 쪽 단위로 옮긴다.
 *
 * 목표는 **쪽 경계에 정확히 붙되 지나친 내용이 없게** 하는 것이다. 한 번에 가는 거리는
 * `min(다음 쪽 경계, 화면 하나)` 다.
 *
 * - 쪽이 화면 안에 들어오면(쪽 맞춤·그리드) 경계가 화면보다 가까우니 한 번에 다음 쪽으로
 *   넘어간다 — 종전과 같은 쪽 단위 이동이다.
 * - 쪽이 화면보다 크면(100% 이상 확대) 종전에는 다음 쪽 머리로 뛰어 그 사이를 한 번도
 *   보여주지 않고 건너뛰었다. 이제 화면 하나씩 밟아 내려가되 **착지점은 항상 쪽 경계**라,
 *   몇 번을 눌러도 쪽 머리 정렬이 어긋나지 않는다.
 *
 * 그리드 모드에서는 `VirtualScroll`이 제공하는 실제 행 시작 목록으로 센다. 맞쪽처럼 첫 행이나
 * 마지막 행에 빈 슬롯이 있어도 페이지 인덱스 산술로 행을 추정하지 않는다. 가로 이동은 같은
 * 규칙을 실제 페이지 왼쪽 경계와 뷰포트 너비에 적용한다.
 */
export function scrollByPageStep(
  virtualScroll: VirtualScroll,
  viewportManager: ViewportManager,
  direction: PageScrollDirection,
): PageScrollResult {
  if (virtualScroll.pageCount <= 0) return NO_MOVE;

  if (virtualScroll.isHorizontalMode()) {
    const scrollX = viewportManager.getScrollX();
    const viewportWidth = viewportManager.getViewportSize().width;
    const totalWidth = virtualScroll.getTotalWidth();
    const limit = Math.max(0, totalWidth - viewportWidth);
    const target = direction > 0
      ? Math.min(
        nextPageBoundary(virtualScroll, scrollX, limit, totalWidth),
        scrollX + viewportWidth,
      )
      : Math.max(
        prevPageBoundary(virtualScroll, scrollX, totalWidth),
        scrollX - viewportWidth,
      );

    const clamped = Math.max(0, Math.min(target, limit));
    if (Math.abs(clamped - scrollX) < EPSILON) return NO_MOVE;

    viewportManager.setScrollLeft(clamped);
    const deltaX = viewportManager.getScrollX() - scrollX;
    if (Math.abs(deltaX) < EPSILON) return NO_MOVE;
    return { moved: true, deltaX, deltaY: 0 };
  }

  const scrollY = viewportManager.getScrollY();
  const viewportHeight = viewportManager.getViewportSize().height;
  const limit = maxScrollTop(virtualScroll, viewportHeight);
  const rowStarts = virtualScroll.getRowStartPages();

  const target = direction > 0
    ? Math.min(nextRowBoundary(virtualScroll, scrollY, rowStarts, limit), scrollY + viewportHeight)
    : Math.max(prevRowBoundary(virtualScroll, scrollY, rowStarts), scrollY - viewportHeight);

  const clamped = Math.max(0, Math.min(target, limit));
  if (Math.abs(clamped - scrollY) < EPSILON) return NO_MOVE;

  viewportManager.setScrollTop(clamped);
  // 브라우저가 자체 clamp 를 할 수 있으니 실제 반영된 값으로 delta 를 낸다 —
  // 캐럿을 같은 화면 위치에 붙여 두려면 이 값이 정확해야 한다.
  const deltaY = viewportManager.getScrollY() - scrollY;
  if (Math.abs(deltaY) < EPSILON) return NO_MOVE;
  return { moved: true, deltaX: 0, deltaY };
}
