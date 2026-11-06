import type { Canvas, CanvasKit, Font, Paint, Typeface } from 'canvaskit-wasm';
import type { LayerEquationLayoutBox } from '../../core/types.ts';

type PositionedGlyphRun = Extract<LayerEquationLayoutBox['kind'], { type: 'hancomGlyphs' }>;

interface PositionedGlyphContext {
  canvasKit: CanvasKit;
  typefaces: readonly Typeface[];
  maxTextLength: number;
  makeFillPaint: (color: string) => Paint;
}

/** 측정한 scalar별 전진을 유지하며 선언된 프로그램의 실제 coverage로 재생한다. */
export function drawPositionedEquationGlyphs(
  context: PositionedGlyphContext, canvas: Canvas, run: PositionedGlyphRun,
  x: number, baseline: number, fontSize: number, color: string,
): boolean {
  const glyphs = Array.from(run.glyphs);
  const shears = run.shears ?? glyphs.map(() => 0);
  if (glyphs.length > context.maxTextLength || glyphs.length !== run.positions.length
    || glyphs.length !== shears.length || !shears.every(Number.isFinite)
    || !run.positions.every(Number.isFinite) || !Number.isFinite(run.baselineShift)
    || ![x, baseline, fontSize].every(Number.isFinite) || fontSize <= 0) return false;
  const fonts = new Map<Typeface, Font>();
  let paint: Paint | undefined;
  try {
    // 모든 문자를 준비하기 전에는 앞부분도 그리지 않는다.
    const selected: Font[] = [];
    for (const glyph of glyphs) {
      let covered: Font | undefined;
      for (const typeface of context.typefaces) {
        let font = fonts.get(typeface);
        if (!font) {
          font = new context.canvasKit.Font(typeface, fontSize);
          fonts.set(typeface, font);
          font.setHinting(context.canvasKit.FontHinting.None);
          font.setSubpixel(true);
        }
        const ids = font.getGlyphIDs(glyph, 1);
        if (ids?.length === 1 && ids[0] !== 0) { covered = font; break; }
      }
      if (!covered) return false;
      selected.push(covered);
    }
    paint = context.makeFillPaint(color);
    glyphs.forEach((glyph, index) => {
      canvas.save();
      try {
        canvas.translate(x + run.positions[index], baseline + run.baselineShift);
        canvas.concat([1, -shears[index], 0, 0, 1, 0, 0, 0, 1]);
        canvas.drawText(glyph, 0, 0, paint!, selected[index]);
      } finally { canvas.restore(); }
    });
    return true;
  } finally {
    for (const font of fonts.values()) font.delete();
    paint?.delete();
  }
}
