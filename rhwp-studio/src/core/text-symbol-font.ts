/** 기존 필수 웹 글꼴의 전각 BMP 기호를 자동 이모지보다 먼저 소비한다. */
export const TEXT_SYMBOL_FONT_FAMILY = '함초롬바탕';
/** Symbols2의 미수록 BMP 기호는 기존 본문 공급과 마지막 고정폭 후보에서 찾는다. */
export const TEXT_SYMBOL_FONT_FAMILIES = ['Noto Sans Symbols2', TEXT_SYMBOL_FONT_FAMILY, 'D2Coding'] as const;
