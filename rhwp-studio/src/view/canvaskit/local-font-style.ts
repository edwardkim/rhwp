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
