// Rust와 같은 동봉 em 메트릭을 소비한다. 화면 px 핀이나 사유 글리프는 포함하지 않는다.
import { enclosedPuaMetrics as metrics, type EnclosedPua } from '../../core/enclosed-pua.ts';

export function enclosedPuaGeometry(glyph: EnclosedPua, fontSize: number,
  sourceInkBounds?: readonly number[]) {
  const ellipse = glyph.kind === 'circledSeal';
  const frame = ellipse ? metrics.circledSeal.frame : metrics.boxedDigit.frame;
  const inner = ellipse ? metrics.circledSeal : metrics.boxedDigit.digits[glyph.number];
  if (!inner) return null;
  const [left, bottom, right, top] = inner.source;
  const [targetLeft, targetBottom, targetRight, targetTop] = inner.target;
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
    face: metrics.innerFace,
    // true면 frame 상자에 내접하는 타원 테두리, 아니면 사각 테두리다.
    ellipse,
    frameX: frame[0] * unit,
    frameY: frame[1] * unit,
    frameWidth: frame[2] * unit,
    frameHeight: frame[3] * unit,
    strokeWidth: frame[4] * unit,
    scaleX: actualScaleX, scaleY: actualScaleY,
    innerX: hasActualBounds
      ? targetLeft * unit - actual[0] * actualScaleX : (targetLeft - left * scaleX) * unit,
    innerY: hasActualBounds
      ? -targetBottom * unit - actual[3] * actualScaleY : (-targetBottom + bottom * scaleY) * unit,
  };
}
