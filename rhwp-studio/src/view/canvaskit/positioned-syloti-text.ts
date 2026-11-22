import type { CanvasKit, FontMgr, Paragraph } from 'canvaskit-wasm';
import { textRequiresComplexShaping } from './nominal-text-replay';

/** The layer tree owns the displayed dotted circle and the next run's advance. */
export function isSylotiShapingRun(text: string, points: readonly string[]): boolean {
  return /(?=\p{Script=Syloti_Nagri})\p{M}/u.test(text)
    && points.every(ch => /\p{Script=Syloti_Nagri}|[\u200c\u200d]/u.test(ch)
      || (!textRequiresComplexShaping(ch)
        && /\p{Script=Common}|\p{Script=Latin}/u.test(ch)
        && !/\p{Extended_Pictographic}/u.test(ch)));
}

export interface PositionedGlyphs {
  glyphs: Uint16Array;
  positions: Float32Array;
}

export function shapePositionedSyloti(
  kit: CanvasKit, manager: FontMgr, points: readonly string[], origins: readonly number[],
  fontSize: number, ratio: number, bold: boolean, italic: boolean, maxWidth: number, maxGlyphs: number,
): PositionedGlyphs | null {
  if (!manager.countFamilies()) return null;
  // SkParagraph reports glyph clusters in UTF-8, caret queries in UTF-16,
  // and the paint stream supplies one origin per displayed scalar.
  const utf8ToUtf16 = new Map<number, number>();
  const utf16ToScalar = new Map<number, number>();
  let byte = 0;
  let unit = 0;
  for (const [index, point] of points.entries()) {
    utf8ToUtf16.set(byte, unit);
    utf16ToScalar.set(unit, index);
    const code = point.codePointAt(0)!;
    byte += code <= 0x7f ? 1 : code <= 0x7ff ? 2 : code <= 0xffff ? 3 : 4;
    unit += point.length;
  }
  const style = new kit.ParagraphStyle({
    maxLines: 1, textStyle: { fontFamilies: [manager.getFamilyName(0)], fontSize,
      fontStyle: { weight: bold ? kit.FontWeight.Bold : kit.FontWeight.Normal,
        slant: italic ? kit.FontSlant.Italic : kit.FontSlant.Upright } },
  });
  const builder = kit.ParagraphBuilder.Make(style, manager);
  let paragraph: Paragraph | null = null;
  try {
    builder.addText(points.join(''));
    paragraph = builder.build();
    paragraph.layout(maxWidth);
    if (paragraph.didExceedMaxLines() || paragraph.unresolvedCodepoints().length) return null;
    const lines = paragraph.getShapedLines();
    if (lines.length !== 1) return null;
    const line = lines[0];
    const count = line.runs.reduce((n, run) => n + run.glyphs.length, 0);
    if (!count || count > maxGlyphs) return null;
    const glyphs = new Uint16Array(count);
    const positions = new Float32Array(2 * count);
    let index = 0;
    for (const run of line.runs) {
      for (const [i, glyph] of run.glyphs.entries()) {
        const utf16 = utf8ToUtf16.get(run.offsets[i]);
        const cluster = utf16 === undefined ? null : paragraph.getGlyphInfoAt(utf16);
        if (!glyph || !cluster) return null;
        const start = utf16ToScalar.get(cluster.graphemeClusterTextRange.start);
        if (start === undefined) return null;
        const x = origins[start] + (run.positions[i * 2] - cluster.graphemeLayoutBounds[0]) * ratio;
        const y = run.positions[i * 2 + 1] - line.baseline;
        if (!Number.isFinite(x) || !Number.isFinite(y)) return null;
        glyphs[index] = glyph;
        positions[index * 2] = x;
        positions[index * 2 + 1] = y;
        index++;
      }
    }
    return { glyphs, positions };
  } finally {
    paragraph?.delete();
    builder.delete();
  }
}
