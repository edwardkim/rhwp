// 일반 glyph replay의 문자 범위 계약. 조합/형식 문자와 복합 script는 계속 shaping 경로로 보낸다.
const COMPLEX_SHAPING_UNICODE_CATEGORY = /[\p{M}\p{Cf}]/u;

export function textRequiresComplexShaping(text: string): boolean {
  for (const character of text) {
    const codePoint = character.codePointAt(0) ?? 0;
    const isOldHangul = (codePoint >= 0x1100 && codePoint <= 0x11ff)
      || (codePoint >= 0xa960 && codePoint <= 0xa97f)
      || (codePoint >= 0xd7b0 && codePoint <= 0xd7ff);
    const isBoxedPua = codePoint >= 0xf02b1 && codePoint <= 0xf02c4;
    if (isOldHangul || isBoxedPua) continue;

    const nominalGlyphReplayIsSafe = codePoint <= 0x02ff
      || (codePoint >= 0x0370 && codePoint <= 0x058f)
      || (codePoint >= 0x1e00 && codePoint <= 0x1fff)
      || (codePoint >= 0x2000 && codePoint <= 0x2fff)
      || (codePoint >= 0x2e80 && codePoint <= 0xd7af)
      || (codePoint >= 0xe000 && codePoint <= 0xf8ff)
      || (codePoint >= 0xf900 && codePoint <= 0xfb06)
      || (codePoint >= 0xfe10 && codePoint <= 0xfe1f)
      || (codePoint >= 0xfe30 && codePoint <= 0xfe6f)
      || (codePoint >= 0xff00 && codePoint <= 0xffef)
      || (codePoint >= 0x1d400 && codePoint <= 0x1d7ff)
      // 두 그림자 원형 기호는 결합·조인 없는 일반 글리프다. VS/ZWJ는 아래 M/Cf 검사로 보류한다.
      || (codePoint >= 0x1f53e && codePoint <= 0x1f53f)
      || (codePoint >= 0x20000 && codePoint <= 0x323af);
    if (!nominalGlyphReplayIsSafe || COMPLEX_SHAPING_UNICODE_CATEGORY.test(character)) {
      return true;
    }
  }
  return false;
}
