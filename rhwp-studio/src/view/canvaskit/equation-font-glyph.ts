import type { Canvas, CanvasKit, Paint, Typeface } from 'canvaskit-wasm';
import { drawEquationTextWithFonts } from './equation-text-replay.ts';

interface DeclaredFontGlyphContext {
  canvasKit: CanvasKit;
  maxTextLength: number;
  lookupTypeface: (family: string) => Typeface | null;
  makeFillPaint: (color: string) => Paint;
  onFallback: (requestedFamily: string, resolvedFamily: string) => void;
}

/** SVG FontGlyph와 같은 선언 순서·자연 표시 폭·기준선으로 재생한다. */
export function drawDeclaredEquationFontGlyph(
  context: DeclaredFontGlyphContext,
  canvas: Canvas,
  text: string,
  fontFamily: string,
  x: number,
  baseline: number,
  fontSize: number,
  color: string,
  reservedWidth: number,
): boolean {
  const families = fontFamily.split(',').map(value => value.trim().replace(/^(['"])(.*)\1$/, '$2'));
  const candidates = families.flatMap(family => {
    const typeface = context.lookupTypeface(family);
    return typeface ? [{ family, typeface }] : [];
  });
  // 준비된 선언 후보만 사용한다. 요청 목록 밖의 기본/기호 글꼴은 추가하지 않는다.
  return drawEquationTextWithFonts({
    canvasKit: context.canvasKit,
    typefaces: candidates.map(candidate => candidate.typeface),
    maxTextLength: context.maxTextLength,
    makeFillPaint: context.makeFillPaint,
    onTypefaceSelected(typeface) {
      const selected = candidates.find(candidate => candidate.typeface === typeface);
      if (selected && selected.family !== families[0]) {
        context.onFallback(families[0], selected.family);
      }
    },
  }, canvas, text, x, baseline, fontSize, color, false, false, reservedWidth, false);
}
