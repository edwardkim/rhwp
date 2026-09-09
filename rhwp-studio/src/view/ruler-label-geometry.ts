/**
 * 가운데 정렬된 눈금자 라벨이 용지 범위 안에 온전히 들어오는지 판정한다.
 *
 * 용지 끝 tick과 경계선은 그대로 그리되, 끝점에 중심을 둔 숫자가 용지 밖으로
 * 넘치는 경우 라벨만 숨기기 위한 순수 좌표 계약이다.
 */
export function isRulerLabelInsidePage(
  centerX: number,
  labelWidth: number,
  pageLeft: number,
  pageWidth: number,
): boolean {
  if (![centerX, labelWidth, pageLeft, pageWidth].every(Number.isFinite)) return false;
  if (labelWidth < 0 || pageWidth < 0) return false;

  const halfLabelWidth = labelWidth / 2;
  const pageRight = pageLeft + pageWidth;
  return centerX - halfLabelWidth >= pageLeft
    && centerX + halfLabelWidth <= pageRight;
}
