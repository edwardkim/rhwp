import type { LocalFontRecord } from '../../core/local-fonts.ts';

/** 이름 추정은 SFNT의 물리 face 스타일이 없을 때만 사용한다. */
export function localFontFaceStyle(record: LocalFontRecord): { weight: number; italic: boolean } {
  const name = `${record.fullName} ${record.postscriptName} ${record.style}`.toLowerCase();
  return {
    weight: record.weightClass ?? (/(?:bold|black|heavy|볼드)/.test(name) ? 700
      : /(?:light|thin)/.test(name) ? 300 : /(?:medium|메디움)/.test(name) ? 500 : 400),
    italic: record.italic ?? /italic|oblique|slanted|kursiv|이탤릭/.test(name),
  };
}

/** 같은 스타일이면 공통 resolver가 선택한 face를 유지한다. 캐시 삽입 순서로 윤곽을 바꾸지 않는다. */
export function closestLocalFontStyle<T extends { record?: LocalFontRecord | null }>(
  preferred: T | null,
  candidates: Iterable<T>,
  bold: boolean,
  italic: boolean,
): T | null {
  const score = (candidate: T): number => {
    if (!candidate.record) return Number.POSITIVE_INFINITY;
    const physical = localFontFaceStyle(candidate.record);
    return (physical.italic === italic ? 0 : 2000)
      + Math.abs(physical.weight - (bold ? 700 : 400));
  };
  let selected = preferred;
  let best = preferred ? score(preferred) : Number.POSITIVE_INFINITY;
  for (const candidate of candidates) {
    const distance = score(candidate);
    if (distance < best) { selected = candidate; best = distance; }
  }
  return selected;
}
