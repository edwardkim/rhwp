// Rust와 같은 동봉 em 메트릭을 소비한다. 화면 px 핀이나 사유 글리프는 포함하지 않는다.
import metrics from '../../../../assets/fonts/boxed-digit-metrics.json' with { type: 'json' };

export function singleDigitGeometry(number: number, fontSize: number,
  sourceInkBounds?: readonly number[]) {
  const digit = metrics.digits[number];
  if (!digit) return null;
  const [left, bottom, right, top] = digit.source;
  const [targetLeft, targetBottom, targetRight, targetTop] = digit.target;
  const unit = fontSize / metrics.unitsPerEm;
  const scaleX = (targetRight - targetLeft) / (right - left);
  const scaleY = (targetTop - targetBottom) / (top - bottom);
  // 설치 face를 공급하지 못한 경로는 선택한 대체 face 자신의 잉크 경계를 쓴다.
  // CanvasKit의 bounds는 baseline 기준 [left, top, right, bottom] px다.
  const actual = sourceInkBounds;
  const hasActualBounds = actual?.length === 4 && actual.every(Number.isFinite)
    && actual[2] > actual[0] && actual[3] > actual[1];
  const actualScaleX = hasActualBounds
    ? (targetRight - targetLeft) * unit / (actual[2] - actual[0]) : scaleX;
  const actualScaleY = hasActualBounds
    ? (targetTop - targetBottom) * unit / (actual[3] - actual[1]) : scaleY;
  return {
    face: metrics.numberFace,
    frameX: metrics.frame[0] * unit,
    frameY: metrics.frame[1] * unit,
    frameWidth: metrics.frame[2] * unit,
    frameHeight: metrics.frame[3] * unit,
    strokeWidth: metrics.frame[4] * unit,
    scaleX: actualScaleX, scaleY: actualScaleY,
    numberX: hasActualBounds
      ? targetLeft * unit - actual[0] * actualScaleX : (targetLeft - left * scaleX) * unit,
    numberY: hasActualBounds
      ? -targetBottom * unit - actual[3] * actualScaleY : (-targetBottom + bottom * scaleY) * unit,
  };
}
