// 원 paint op의 원문·주소·전진을 바꾸지 않는 글리프 재생 뷰.
interface Candidate {start: number; end: number; glyph: string; fallbackGlyph?: string}
interface FontCoverage {
  getGlyphIDs(text: string): Uint16Array;
  getGlyphBounds?(glyphs: Uint16Array): Float32Array;
}
export function compatibilityGlyphView(text: readonly string[], positions: readonly number[],
  candidates: readonly Candidate[], font: FontCoverage): {text: string; points: string[]; positions: number[]; sourceIndices: number[]} | null {
  if (positions.length !== text.length + 1 || candidates.length > text.length) return null;
  let previous = 0;
  for (const c of candidates) {
    const valid = (glyph: string) => {
      const cp = glyph.codePointAt(0) ?? 0;
      return [...glyph].length === 1 && ((cp >= 0xe000 && cp <= 0xf8ff) || (cp >= 0xac00 && cp <= 0xd7a3)
        || (cp >= 0x1100 && cp <= 0x11ff) || (cp >= 0xa960 && cp <= 0xa97f) || (cp >= 0xd7b0 && cp <= 0xd7ff));
    };
    if (!Number.isInteger(c.start) || !Number.isInteger(c.end) || c.start < previous
      || c.start >= c.end || c.end > text.length || !valid(c.glyph)
      || (c.fallbackGlyph !== undefined && !valid(c.fallbackGlyph))) return null;
    previous = c.end;
  }
  const byStart = new Map(candidates.map(c => [c.start, c]));
  const points: string[] = [], replayPositions: number[] = [], sourceIndices: number[] = [];
  for (let at = 0; at < text.length;) {
    const c = byStart.get(at);
    const selected = c ? [c.glyph, ...(c.fallbackGlyph ? [c.fallbackGlyph] : [])].find(glyph => {
      const ids = font.getGlyphIDs(glyph);
      const bounds = ids.length === 1 && ids[0] ? font.getGlyphBounds?.(ids) : undefined;
      return bounds && bounds[2] > bounds[0] && bounds[3] > bounds[1];
    }) : undefined;
    replayPositions.push(positions[at]);
    if (c && selected) {sourceIndices.push(points.length); points.push(selected); at = c.end;}
    else {points.push(text[at]); at++;}
  }
  replayPositions.push(positions[text.length]);
  return {text: points.join(''), points, positions: replayPositions, sourceIndices};
}
