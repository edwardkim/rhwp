export const DEFAULT_PAGE_GAP_AT_100_PERCENT = 10;
export const MIN_PAGE_GAP_CSS_PX = 6;

/** 문서 좌표와 분리된 화면용 페이지 간격을 계산한다. */
export function resolvePageGap(
  zoom: number,
  gapAt100Percent = DEFAULT_PAGE_GAP_AT_100_PERCENT,
): number {
  const safeZoom = Number.isFinite(zoom) ? Math.max(0, zoom) : 1;
  const safeBaseGap = Number.isFinite(gapAt100Percent)
    ? Math.max(0, gapAt100Percent)
    : DEFAULT_PAGE_GAP_AT_100_PERCENT;
  return Math.max(MIN_PAGE_GAP_CSS_PX, safeBaseGap * safeZoom);
}

