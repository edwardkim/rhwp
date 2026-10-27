import type { LayerTextStyle } from '../../core/types.ts';

/** Rust가 확정한 첨자 기하를 재생한다. 새 계약의 배율을 여기서 다시 계산하지 않는다. */
export function scriptDrawMetrics(
  style: LayerTextStyle,
  baseFontSize: number,
): { fontSize: number; baselineShift: number } {
  if (style.scriptMetrics !== undefined) {
    const { fontSize, baselineShift } = style.scriptMetrics;
    if ((!style.superscript && !style.subscript)
      || !Number.isFinite(fontSize) || fontSize < 0
      || !Number.isFinite(baselineShift)) {
      throw new Error('공급된 첨자 기하가 올바르지 않습니다.');
    }
    return { fontSize, baselineShift };
  }
  // 이전 transport의 기존 계약이다. 새 producer는 해소한 값을 위 경계로 공급한다.
  if (style.superscript) {
    return { fontSize: baseFontSize * 0.7, baselineShift: -baseFontSize * 0.3 };
  }
  if (style.subscript) {
    return { fontSize: baseFontSize * 0.7, baselineShift: baseFontSize * 0.15 };
  }
  return { fontSize: baseFontSize, baselineShift: 0 };
}
