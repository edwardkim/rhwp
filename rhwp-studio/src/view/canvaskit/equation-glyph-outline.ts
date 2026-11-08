import type { Canvas, CanvasKit, Paint } from 'canvaskit-wasm';
import type { LayerFontGlyphGeometry } from '@/core/types';

/** 수식 측정 결과의 일반 윤곽·전진과 같은 fill+stroke·장평을 소비한다. */
export function drawEquationGlyphOutline(
  canvasKit: CanvasKit,
  canvas: Canvas,
  geometry: LayerFontGlyphGeometry,
  x: number,
  y: number,
  color: string,
  makeFillPaint: (color: string) => Paint,
  makeStrokePaint: (color: string, width: number) => Paint,
): boolean {
  const scaleX = geometry.scaleX ?? 1;
  if (!geometry.pathData || geometry.pathData.length > 65536
    || ![x, y, geometry.strokeWidth, scaleX].every(Number.isFinite)
    || geometry.strokeWidth < 0 || scaleX <= 0) return false;
  const path = canvasKit.Path.MakeFromSVGString(geometry.pathData);
  if (!path) return false;
  let fill: Paint | null = null;
  let stroke: Paint | null = null;
  try {
    fill = makeFillPaint(color);
    if (geometry.strokeWidth > 0) {
      stroke = makeStrokePaint(color, geometry.strokeWidth);
      // 한컴의 em 획을 공용 shape helper의 화면 최소 굵기로 늘리지 않는다.
      stroke.setStrokeWidth(geometry.strokeWidth);
      stroke.setStrokeCap(canvasKit.StrokeCap.Butt);
      stroke.setStrokeJoin(canvasKit.StrokeJoin.Miter);
      stroke.setStrokeMiter(10);
    }
    canvas.save();
    try {
      canvas.translate(x, y);
      canvas.scale(scaleX, 1);
      if (geometry.fillAndStroke !== false || !stroke) canvas.drawPath(path, fill);
      if (stroke) canvas.drawPath(path, stroke);
    } finally { canvas.restore(); }
    return true;
  } finally {
    stroke?.delete();
    fill?.delete();
    path.delete();
  }
}
