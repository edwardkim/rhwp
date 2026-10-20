import type { Canvas, CanvasKit, Font, Paint, Typeface } from 'canvaskit-wasm';

interface EquationTextFonts {
  canvasKit: CanvasKit;
  typefaces: readonly (Typeface | null)[];
  maxTextLength: number;
  makeFillPaint: (color: string) => Paint;
  onTypefaceSelected?: (typeface: Typeface) => void;
}

/** 논리 예약 폭과 글리프의 자연 표시 폭을 구분해 SVG와 같은 정렬을 한다. */
export function drawEquationTextWithFonts(
  fonts: EquationTextFonts,
  canvas: Canvas,
  text: string,
  x: number,
  baselineY: number,
  fontSize: number,
  color: string,
  italic: boolean,
  bold: boolean,
  targetWidth: number,
  centered: boolean,
): boolean {
  if (!text || text.length > fonts.maxTextLength
    || ![x, baselineY, fontSize, targetWidth].every(Number.isFinite)) return false;

  let font: Font | null = null;
  let paint: Paint | null = null;
  try {
    for (const typeface of fonts.typefaces) {
      if (!typeface) continue;
      const candidate = new fonts.canvasKit.Font(typeface, Math.max(1, fontSize));
      const glyphIds = candidate.getGlyphIDs(text, Array.from(text).length);
      if (glyphIds && glyphIds.length > 0 && glyphIds.every(id => id !== 0)) {
        font = candidate;
        fonts.onTypefaceSelected?.(typeface);
        break;
      }
      candidate.delete();
    }
    if (!font) return false;
    paint = fonts.makeFillPaint(color);
    const glyphIds = font.getGlyphIDs(text, Array.from(text).length);
    if (!glyphIds || glyphIds.some(id => id === 0)) return false;
    const naturalWidth = (font.getGlyphWidths(glyphIds) ?? [])
      .reduce((sum, width) => sum + width, 0);
    const adjustable = font as Font & {
      setEmbolden?: (enabled: boolean) => void;
      setSkewX?: (skew: number) => void;
    };
    adjustable.setEmbolden?.(bold);
    adjustable.setSkewX?.(italic ? -0.2 : 0);
    const drawX = centered ? x + (targetWidth - naturalWidth) / 2 : x;
    canvas.drawText(text, drawX, baselineY, paint, font);
    return true;
  } finally {
    font?.delete?.();
    paint?.delete?.();
  }
}
