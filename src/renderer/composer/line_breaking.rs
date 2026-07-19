//! 줄 나눔 엔진 (Line Breaking Engine)
//!
//! 문단 텍스트를 토큰화하고 줄 나눔을 수행한다.
//! 한글 어절/글자, 영어 단어/하이픈, CJK 개별 분할을 지원한다.

use super::{find_active_char_shape, is_lang_neutral};
use crate::model::control::Control;
use crate::model::paragraph::{CharShapeRef, LineSeg, Paragraph};
use crate::model::style::LineSpacingType;
use crate::renderer::layout::{
    estimate_text_width_unrounded, estimate_text_width_unrounded_scoped, is_cjk_char,
    resolved_to_text_style,
};
use crate::renderer::px_to_hwpunit;
use crate::renderer::style_resolver::{detect_lang_category, ResolvedStyleSet};

/// 줄 나눔 토큰
#[derive(Debug, Clone)]
pub(crate) enum BreakToken {
    /// 분할 불가 텍스트 조각 (어절/단어/글자)
    /// char_widths: 글자별 px 폭 (char_level_break용, 단일 글자 토큰은 비어있음)
    Text {
        start_idx: usize,
        end_idx: usize,
        width: f64,
        max_font_size: f64,
        char_widths: Vec<f64>,
    },
    /// 공백 (줄 바꿈 가능 지점, 줄 끝에서 흡수)
    Space {
        idx: usize,
        width: f64,
        max_font_size: f64,
    },
    /// 탭 (줄 바꿈 가능 지점, 폭은 줄 위치에 따라 동적)
    Tab { idx: usize, max_font_size: f64 },
    /// 강제 줄 바꿈 (\n)
    LineBreak { idx: usize },
}

/// 줄 채움 결과
#[derive(Debug)]
struct LineBreakResult {
    start_idx: usize,
    end_idx: usize, // exclusive
    max_font_size: f64,
    has_line_break: bool, // 강제 줄 바꿈 여부
}

/// 줄 머리 금칙: 줄 시작에 올 수 없는 문자
pub(crate) fn is_line_start_forbidden(ch: char) -> bool {
    matches!(
        ch,
        ')' | ']'
            | '}'
            | ','
            | '.'
            | '!'
            | '?'
            | ';'
            | ':'
            | '\''
            | '"'
            | '\u{3001}'
            | '\u{3002}'
            | '\u{2026}'
            | '\u{00B7}'
            | '\u{2015}'
            | '\u{30FC}'
            | '\u{300B}'
            | '\u{300D}'
            | '\u{300F}'
            | '\u{3011}'
            | '\u{FF09}'
            | '\u{FF5D}'
            | '\u{3015}'
            | '\u{3009}'
            | '\u{FF1E}'
            | '\u{226B}'
            | '\u{FF3D}'
            | '\u{FE5E}'
            | '\u{301E}'
            | '\u{2019}'
            | '\u{201D}'
            | '\u{FF0C}'
            | '\u{FF0E}'
            | '\u{FF01}'
            | '\u{FF1F}'
            | '\u{FF1B}'
            | '\u{FF1A}'
            | '%'
            | '\u{2030}'
            | '\u{2103}'
            | '\u{00B0}'
            | '\u{FF05}'
    )
}

/// 줄 꼬리 금칙: 줄 끝에 올 수 없는 문자
pub(crate) fn is_line_end_forbidden(ch: char) -> bool {
    matches!(
        ch,
        '(' | '['
            | '{'
            | '\''
            | '"'
            | '\u{300A}'
            | '\u{300C}'
            | '\u{300E}'
            | '\u{3010}'
            | '\u{FF08}'
            | '\u{FF5B}'
            | '\u{3014}'
            | '\u{3008}'
            | '\u{FF1C}'
            | '\u{226A}'
            | '\u{FF3B}'
            | '\u{301D}'
            | '\u{2018}'
            | '\u{201C}'
            | '$'
            | '\u{20A9}'
            | '\u{00A3}'
            | '\u{20AC}'
            | '\u{00A5}'
            | '\u{FF04}'
            | '\u{FFE5}'
    )
}

/// 한글 음절/자모 여부 (옛한글 확장 자모 포함)
fn is_hangul(ch: char) -> bool {
    ('\u{AC00}'..='\u{D7A3}').contains(&ch)       // 한글 음절
        || ('\u{1100}'..='\u{11FF}').contains(&ch) // 한글 자모
        || ('\u{3130}'..='\u{318F}').contains(&ch) // 한글 호환 자모 (ㆍ U+318D 포함)
        || ('\u{A960}'..='\u{A97F}').contains(&ch) // 한글 자모 확장-A (옛한글 초성)
        || ('\u{D7B0}'..='\u{D7FF}').contains(&ch) // 한글 자모 확장-B (옛한글 중/종성)
}

/// 라틴 문자 여부 (영문+숫자)
fn is_latin(ch: char) -> bool {
    let lang = detect_lang_category(ch);
    lang == 1 // English/Latin
}

/// CJK 문자 여부 (한자/일본어 — 개별 분할 대상)
fn is_cjk_ideograph(ch: char) -> bool {
    let lang = detect_lang_category(ch);
    lang == 2 || lang == 3 // Chinese or Japanese
}

/// [#22xx P] 각 글자의 maximal no-space run 이 순수 ASCII(비한글/비CJK)인지.
///
/// 함초롬바탕 ASCII 의 half-em(0.50em) 반각 셀 fit 은 이 값이 true 인 run 에만
/// 적용한다. run 경계는 공백/탭/개행이며, run 안에 한글/CJK 한자·일본어가 하나
/// 라도 있으면 그 run 전체 글자를 false 로 표시(natural HCR hmtx 로 fit).
/// per-glyph-INDEX 로 계산하므로 토큰 폭·char_widths 가 동일 스코프를 공유한다.
///   · 라틴/숫자/구두점만 있는 run → true  (re-03/04: 85 글자/줄 반각 셀)
///   · 한글 포함 run              → false (re-05 라틴 natural, re-06 구두점 natural)
fn compute_ascii_run_mask(text_chars: &[char]) -> Vec<bool> {
    let n = text_chars.len();
    let mut mask = vec![true; n];
    let mut i = 0;
    while i < n {
        let c = text_chars[i];
        if c == ' ' || c == '\t' || c == '\n' {
            mask[i] = true; // 공백류는 스코프 무관(측정도 half-em 대상 아님)
            i += 1;
            continue;
        }
        // no-space run [start, j) 스캔
        let start = i;
        let mut j = i;
        let mut all_ascii = true;
        while j < n {
            let cj = text_chars[j];
            if cj == ' ' || cj == '\t' || cj == '\n' {
                break;
            }
            // 한글/CJK 한자·일본어가 하나라도 있으면 run 은 non-ASCII.
            if is_hangul(cj) || is_cjk_ideograph(cj) {
                all_ascii = false;
            }
            j += 1;
        }
        for m in mask.iter_mut().take(j).skip(start) {
            *m = all_ascii;
        }
        i = j;
    }
    mask
}

/// 문단 텍스트를 줄 나눔 토큰으로 분할한다.
pub(crate) fn tokenize_paragraph(
    text_chars: &[char],
    char_offsets: &[u32],
    char_shapes: &[CharShapeRef],
    styles: &ResolvedStyleSet,
    english_break_unit: u8,
    korean_break_unit: u8,
) -> Vec<BreakToken> {
    let text_len = text_chars.len();
    if text_len == 0 {
        return Vec::new();
    }

    // [#22xx P] no-space run 별 "전부 ASCII" 판정을 글자 인덱스 단위로 1 회 계산.
    // 이후 모든 단일-글자 fit 측정(토큰 폭·char_widths)이 동일 스코프를 공유해
    // half-em(re-03/04) vs natural(re-05/06) 을 일관되게 고른다.
    let ascii_run_mask = compute_ascii_run_mask(text_chars);

    let mut tokens = Vec::new();
    let mut i = 0;
    let mut current_lang: usize = 0;

    while i < text_len {
        let ch = text_chars[i];

        // 강제 줄 바꿈
        if ch == '\n' {
            tokens.push(BreakToken::LineBreak { idx: i });
            i += 1;
            continue;
        }

        // 탭
        if ch == '\t' {
            let utf16_pos = if i < char_offsets.len() {
                char_offsets[i]
            } else {
                i as u32
            };
            let style_id = find_active_char_shape(char_shapes, utf16_pos);
            let ts = resolved_to_text_style(styles, style_id, current_lang);
            let font_size = if ts.font_size > 0.0 {
                ts.font_size
            } else {
                12.0
            };
            tokens.push(BreakToken::Tab {
                idx: i,
                max_font_size: font_size,
            });
            i += 1;
            continue;
        }

        // 공백 (줄 바꿈 지점) — NonBreakingSpace(\u{00A0})는 제외
        if ch == ' ' {
            let utf16_pos = if i < char_offsets.len() {
                char_offsets[i]
            } else {
                i as u32
            };
            let style_id = find_active_char_shape(char_shapes, utf16_pos);
            let ts = resolved_to_text_style(styles, style_id, current_lang);
            let font_size = if ts.font_size > 0.0 {
                ts.font_size
            } else {
                12.0
            };
            let w = estimate_text_width_unrounded(" ", &ts);
            tokens.push(BreakToken::Space {
                idx: i,
                width: w,
                max_font_size: font_size,
            });
            i += 1;
            continue;
        }

        // 한글 어절 또는 글자.
        // [#2185] bit7=1(KEEP_WORD)이 **글자 단위**, bit7=0(BREAK_WORD)이
        // 어절 단위 — 스키마 명목과 반대 (한컴 통제 실측 3중 확증: #2169
        // kbu 사다리, 80168 r10, #2185 giant-cell LINE_SEG [0,44,84,122]
        // 보존 대조). 종전 == 1 어절 분기는 역해석 (0da18bbc 회귀).
        if is_hangul(ch) {
            if korean_break_unit == 0 {
                // 어절 모드: 연속 한글 + 후행 금칙 문자를 하나의 토큰으로
                let start = i;
                let mut max_fs = 0.0f64;
                let mut token_text = String::new();
                let mut token_lang = current_lang;

                while i < text_len {
                    let c = text_chars[i];
                    if c == ' ' || c == '\n' || c == '\t' {
                        break;
                    }
                    // 한글이 아니고 라틴이면 다른 토큰으로 분리
                    if !is_hangul(c) && is_latin(c) {
                        break;
                    }
                    // CJK 한자/일본어는 개별 토큰
                    if is_cjk_ideograph(c) {
                        break;
                    }

                    let utf16_pos = if i < char_offsets.len() {
                        char_offsets[i]
                    } else {
                        i as u32
                    };
                    let style_id = find_active_char_shape(char_shapes, utf16_pos);
                    let lang = if is_lang_neutral(c) {
                        token_lang
                    } else {
                        let detected = detect_lang_category(c);
                        token_lang = detected;
                        current_lang = detected;
                        detected
                    };
                    let ts = resolved_to_text_style(styles, style_id, lang);
                    let fs = if ts.font_size > 0.0 {
                        ts.font_size
                    } else {
                        12.0
                    };
                    if fs > max_fs {
                        max_fs = fs;
                    }
                    token_text.push(c);
                    i += 1;
                }

                // 후행 금칙 문자 (줄 머리 금칙) 흡수
                while i < text_len
                    && is_line_start_forbidden(text_chars[i])
                    && text_chars[i] != '\n'
                    && text_chars[i] != '\t'
                {
                    let c = text_chars[i];
                    let utf16_pos = if i < char_offsets.len() {
                        char_offsets[i]
                    } else {
                        i as u32
                    };
                    let style_id = find_active_char_shape(char_shapes, utf16_pos);
                    let lang = if is_lang_neutral(c) {
                        current_lang
                    } else {
                        let detected = detect_lang_category(c);
                        current_lang = detected;
                        detected
                    };
                    let ts = resolved_to_text_style(styles, style_id, lang);
                    let fs = if ts.font_size > 0.0 {
                        ts.font_size
                    } else {
                        12.0
                    };
                    if fs > max_fs {
                        max_fs = fs;
                    }
                    token_text.push(c);
                    i += 1;
                }

                if !token_text.is_empty() {
                    // [#22xx P] 토큰 폭·per-glyph 폭 모두 run 스코프(ascii_run_mask)로.
                    let width = measure_token_width_scoped(
                        start,
                        i,
                        text_chars,
                        char_offsets,
                        char_shapes,
                        styles,
                        &ascii_run_mask,
                    );
                    // per-glyph 폭 수집 (누적-핏 엔진 + char_level_break 용)
                    let cw: Vec<f64> = (start..i)
                        .map(|ci| {
                            let c = text_chars[ci];
                            let u16p = if ci < char_offsets.len() {
                                char_offsets[ci]
                            } else {
                                ci as u32
                            };
                            let sid = find_active_char_shape(char_shapes, u16p);
                            let lang = if is_lang_neutral(c) {
                                token_lang
                            } else {
                                detect_lang_category(c)
                            };
                            let ts = resolved_to_text_style(styles, sid, lang);
                            estimate_text_width_unrounded_scoped(&c.to_string(), &ts, ascii_run_mask[ci])
                        })
                        .collect();
                    tokens.push(BreakToken::Text {
                        start_idx: start,
                        end_idx: i,
                        width,
                        max_font_size: max_fs,
                        char_widths: cw,
                    });
                }
                continue;
            } else {
                // 글자 모드: 한글 개별 분할.
                // [#22xx P] 한글→라틴 바인딩: 한글 바로 뒤(i+1)가 라틴이면 단독
                // 토큰으로 내보내지 않고 뒤따르는 라틴 어절과 묶어 [한글][라틴-단어]
                // 한 개의 분할불가 Text 토큰으로 만든다(re-05: "글English" 가 한
                // 토큰 → 줄끝에서 분리 금지). 한글 뒤가 한글이면 종전대로 단독 토큰
                // (한글↔한글 브레이크 자유). 바인딩은 korean_break_unit==1(글자 모드)
                // + "한글 바로 뒤 라틴" 에만 발화 — 순수 한글/순수 라틴 문단 무영향.
                // 폰트 게이트: 바탕/돋움/맑은 등 타 폰트의 한영 무공백 혼합
                // (re-eng-mixed)은 한컴이 바인딩 없이 저장하므로, 함초롬바탕
                // (HCR Batang) 문서에만 바인딩을 발화시켜 오발화 회귀를 막는다.
                let bind_font_is_hcr = {
                    let u16p = if i < char_offsets.len() {
                        char_offsets[i]
                    } else {
                        i as u32
                    };
                    let sid = find_active_char_shape(char_shapes, u16p);
                    let ts = resolved_to_text_style(styles, sid, detect_lang_category(ch));
                    let fam = ts.font_family.split(',').next().unwrap_or("").trim();
                    matches!(fam, "함초롬바탕" | "HCR Batang")
                };
                let bind_to_latin =
                    bind_font_is_hcr && i + 1 < text_len && is_latin(text_chars[i + 1]);

                if bind_to_latin {
                    // 선두 한글 + 후속 연속 라틴을 하나의 토큰으로 수집.
                    let start = i;
                    let mut max_fs = 0.0f64;

                    // 선두 한글 1 글자.
                    {
                        let utf16_pos = if i < char_offsets.len() {
                            char_offsets[i]
                        } else {
                            i as u32
                        };
                        let style_id = find_active_char_shape(char_shapes, utf16_pos);
                        current_lang = detect_lang_category(ch);
                        let ts = resolved_to_text_style(styles, style_id, current_lang);
                        let fs = if ts.font_size > 0.0 { ts.font_size } else { 12.0 };
                        if fs > max_fs {
                            max_fs = fs;
                        }
                        i += 1;
                    }

                    // 후속 연속 라틴(언어중립 포함) 흡수 — 공백/개행/탭/한글/CJK 에서 정지.
                    // 언어중립 문자는 current_lang(직전 확정 언어)을 이어받아 분류를
                    // 오염시키지 않는다(re-05 라틴 run 유지).
                    while i < text_len {
                        let c = text_chars[i];
                        if c == ' ' || c == '\n' || c == '\t' {
                            break;
                        }
                        if !is_latin(c) && !is_lang_neutral(c) {
                            break;
                        }
                        let utf16_pos = if i < char_offsets.len() {
                            char_offsets[i]
                        } else {
                            i as u32
                        };
                        let style_id = find_active_char_shape(char_shapes, utf16_pos);
                        let lang = if is_lang_neutral(c) {
                            current_lang
                        } else {
                            current_lang = 1; // English
                            1
                        };
                        let ts = resolved_to_text_style(styles, style_id, lang);
                        let fs = if ts.font_size > 0.0 { ts.font_size } else { 12.0 };
                        if fs > max_fs {
                            max_fs = fs;
                        }
                        i += 1;
                    }

                    // 토큰 폭·per-glyph 폭 (run 스코프: 한글 섞인 run 이므로 natural).
                    let width = measure_token_width_scoped(
                        start,
                        i,
                        text_chars,
                        char_offsets,
                        char_shapes,
                        styles,
                        &ascii_run_mask,
                    );
                    let cw: Vec<f64> = (start..i)
                        .map(|ci| {
                            let c = text_chars[ci];
                            let u16p = if ci < char_offsets.len() {
                                char_offsets[ci]
                            } else {
                                ci as u32
                            };
                            let sid = find_active_char_shape(char_shapes, u16p);
                            // 언어중립 글자는 선두 한글 언어(0) 기준 — run 오염 방지.
                            let lang = if is_lang_neutral(c) {
                                detect_lang_category(text_chars[start])
                            } else {
                                detect_lang_category(c)
                            };
                            let ts = resolved_to_text_style(styles, sid, lang);
                            estimate_text_width_unrounded_scoped(&c.to_string(), &ts, ascii_run_mask[ci])
                        })
                        .collect();
                    tokens.push(BreakToken::Text {
                        start_idx: start,
                        end_idx: i,
                        width,
                        max_font_size: max_fs,
                        char_widths: cw,
                    });
                    continue;
                } else {
                    // 종전 경로: 한글 단독 분할 토큰.
                    let utf16_pos = if i < char_offsets.len() {
                        char_offsets[i]
                    } else {
                        i as u32
                    };
                    let style_id = find_active_char_shape(char_shapes, utf16_pos);
                    current_lang = detect_lang_category(ch);
                    let ts = resolved_to_text_style(styles, style_id, current_lang);
                    let fs = if ts.font_size > 0.0 {
                        ts.font_size
                    } else {
                        12.0
                    };
                    // 한글 섞인 run → natural(970). (전부-ASCII 아님 → mask=false)
                    let w = estimate_text_width_unrounded_scoped(
                        &ch.to_string(),
                        &ts,
                        ascii_run_mask[i],
                    );
                    tokens.push(BreakToken::Text {
                        start_idx: i,
                        end_idx: i + 1,
                        width: w,
                        max_font_size: fs,
                        char_widths: vec![],
                    });
                    i += 1;
                    continue;
                }
            }
        }

        // 라틴 단어 또는 글자
        if is_latin(ch) {
            if english_break_unit == 0 || english_break_unit == 1 {
                // 단어/하이픈 모드: 연속 라틴 문자를 하나의 토큰으로
                let start = i;
                let mut max_fs = 0.0f64;
                let mut token_text = String::new();

                while i < text_len {
                    let c = text_chars[i];
                    if c == ' ' || c == '\n' || c == '\t' {
                        break;
                    }
                    if !is_latin(c) && !is_lang_neutral(c) {
                        break;
                    }
                    // 하이픈 모드: 하이픈에서 분할 (하이픈 포함 후 분리)
                    if english_break_unit == 1 && c == '-' && !token_text.is_empty() {
                        let utf16_pos = if i < char_offsets.len() {
                            char_offsets[i]
                        } else {
                            i as u32
                        };
                        let style_id = find_active_char_shape(char_shapes, utf16_pos);
                        let lang = 1usize; // English
                        let ts = resolved_to_text_style(styles, style_id, lang);
                        let fs = if ts.font_size > 0.0 {
                            ts.font_size
                        } else {
                            12.0
                        };
                        if fs > max_fs {
                            max_fs = fs;
                        }
                        token_text.push(c);
                        i += 1;
                        break; // 하이픈 뒤에서 분할
                    }

                    let utf16_pos = if i < char_offsets.len() {
                        char_offsets[i]
                    } else {
                        i as u32
                    };
                    let style_id = find_active_char_shape(char_shapes, utf16_pos);
                    let lang = if is_lang_neutral(c) {
                        current_lang
                    } else {
                        current_lang = 1; // English
                        1
                    };
                    let ts = resolved_to_text_style(styles, style_id, lang);
                    let fs = if ts.font_size > 0.0 {
                        ts.font_size
                    } else {
                        12.0
                    };
                    if fs > max_fs {
                        max_fs = fs;
                    }
                    token_text.push(c);
                    i += 1;
                }

                if !token_text.is_empty() {
                    // [#22xx P] 토큰 폭·per-glyph 폭 모두 run 스코프(ascii_run_mask)로.
                    let width = measure_token_width_scoped(
                        start,
                        i,
                        text_chars,
                        char_offsets,
                        char_shapes,
                        styles,
                        &ascii_run_mask,
                    );
                    // 개별 글자 폭 수집 (char_level_break용)
                    let cw: Vec<f64> = (start..i)
                        .map(|ci| {
                            let c = text_chars[ci];
                            let u16p = if ci < char_offsets.len() {
                                char_offsets[ci]
                            } else {
                                ci as u32
                            };
                            let sid = find_active_char_shape(char_shapes, u16p);
                            let lang = if is_lang_neutral(c) { current_lang } else { 1 };
                            let ts = resolved_to_text_style(styles, sid, lang);
                            estimate_text_width_unrounded_scoped(&c.to_string(), &ts, ascii_run_mask[ci])
                        })
                        .collect();
                    tokens.push(BreakToken::Text {
                        start_idx: start,
                        end_idx: i,
                        width,
                        max_font_size: max_fs,
                        char_widths: cw,
                    });
                }
                continue;
            } else {
                // 글자 모드
                let utf16_pos = if i < char_offsets.len() {
                    char_offsets[i]
                } else {
                    i as u32
                };
                let style_id = find_active_char_shape(char_shapes, utf16_pos);
                current_lang = 1;
                let ts = resolved_to_text_style(styles, style_id, current_lang);
                let fs = if ts.font_size > 0.0 {
                    ts.font_size
                } else {
                    12.0
                };
                let w = estimate_text_width_unrounded_scoped(&ch.to_string(), &ts, ascii_run_mask[i]);
                tokens.push(BreakToken::Text {
                    start_idx: i,
                    end_idx: i + 1,
                    width: w,
                    max_font_size: fs,
                    char_widths: vec![],
                });
                i += 1;
                continue;
            }
        }

        // CJK 한자/일본어: 항상 개별 토큰
        if is_cjk_ideograph(ch) {
            let utf16_pos = if i < char_offsets.len() {
                char_offsets[i]
            } else {
                i as u32
            };
            let style_id = find_active_char_shape(char_shapes, utf16_pos);
            current_lang = detect_lang_category(ch);
            let ts = resolved_to_text_style(styles, style_id, current_lang);
            let fs = if ts.font_size > 0.0 {
                ts.font_size
            } else {
                12.0
            };
            let w = estimate_text_width_unrounded_scoped(&ch.to_string(), &ts, ascii_run_mask[i]);
            tokens.push(BreakToken::Text {
                start_idx: i,
                end_idx: i + 1,
                width: w,
                max_font_size: fs,
                char_widths: vec![],
            });
            i += 1;
            continue;
        }

        // 기타 문자 (기호, NonBreakingSpace 등): 개별 Text 토큰
        {
            let utf16_pos = if i < char_offsets.len() {
                char_offsets[i]
            } else {
                i as u32
            };
            let style_id = find_active_char_shape(char_shapes, utf16_pos);
            let lang = if is_lang_neutral(ch) {
                current_lang
            } else {
                let detected = detect_lang_category(ch);
                current_lang = detected;
                detected
            };
            let ts = resolved_to_text_style(styles, style_id, lang);
            let fs = if ts.font_size > 0.0 {
                ts.font_size
            } else {
                12.0
            };
            let w = estimate_text_width_unrounded_scoped(&ch.to_string(), &ts, ascii_run_mask[i]);
            tokens.push(BreakToken::Text {
                start_idx: i,
                end_idx: i + 1,
                width: w,
                max_font_size: fs,
                char_widths: vec![],
            });
            i += 1;
        }
    }

    tokens
}

/// [#22xx P] 토큰 [start, end) 의 폭을 글자별 언어 인식 + run 스코프로 합산한다.
///
/// 기존 measure_token_width 대체(문자열 기반 → 인덱스 범위 기반). 각 글자의
/// ascii_run_mask[idx] 로 half-em(전부-ASCII run) vs natural(한글 섞인 run) 을
/// 골라 char_widths 수집과 **동일 스코프**를 공유한다. 선두 글자 언어를 중립
/// 문자 기본값으로 사용하는 것(default_lang 세맨틱)은 종전과 동일.
fn measure_token_width_scoped(
    start: usize,
    end: usize,
    text_chars: &[char],
    char_offsets: &[u32],
    char_shapes: &[CharShapeRef],
    styles: &ResolvedStyleSet,
    ascii_run_mask: &[bool],
) -> f64 {
    let mut total = 0.0;
    // 선두 글자 언어를 중립 문자 기본값으로 사용(기존 default_lang 세맨틱: 0).
    let mut current_lang = if start < text_chars.len() {
        let c0 = text_chars[start];
        if is_lang_neutral(c0) {
            0
        } else {
            detect_lang_category(c0)
        }
    } else {
        0
    };
    for idx in start..end {
        let ch = text_chars[idx];
        let utf16_pos = if idx < char_offsets.len() {
            char_offsets[idx]
        } else {
            idx as u32
        };
        let style_id = find_active_char_shape(char_shapes, utf16_pos);
        let lang = if is_lang_neutral(ch) {
            current_lang
        } else {
            let detected = detect_lang_category(ch);
            current_lang = detected;
            detected
        };
        let ts = resolved_to_text_style(styles, style_id, lang);
        total += estimate_text_width_unrounded_scoped(&ch.to_string(), &ts, ascii_run_mask[idx]);
    }
    total
}

/// px를 HWPUNIT(i32)로 변환 (내림, DPI=96 기준: px * 75)
#[inline]
fn to_hwp(px: f64) -> i32 {
    (px * 75.0) as i32
}

fn condense_space_savings_hwp(space_width_hwp: i32, condense_min_space: u8) -> i32 {
    if condense_min_space == 0 || space_width_hwp <= 0 {
        return 0;
    }
    let shrink_percent = condense_min_space.min(75) as i32;
    space_width_hwp * shrink_percent / 100
}

fn condensed_line_width_hwp(width_hwp: i32, space_savings_hwp: i32) -> i32 {
    width_hwp - space_savings_hwp
}

fn condense_fit_can_pull_next_token(
    current_width_hwp: i32,
    current_space_savings_hwp: i32,
    effective_width_hwp: i32,
    max_font_size: f64,
) -> bool {
    let current_condensed_width =
        condensed_line_width_hwp(current_width_hwp, current_space_savings_hwp);
    let remaining_hwp = effective_width_hwp - current_condensed_width;
    // Hancom uses condense to rescue a line that still has a meaningful
    // natural gap, but it does not pull the next word into an already tight
    // line. The p03 PDF preface is sensitive to that distinction.
    let min_remaining_hwp = to_hwp((max_font_size * 2.5).max(20.0));
    remaining_hwp >= min_remaining_hwp
}

/// 토큰을 줄에 배치하는 한컴 네이티브 누적-펜 종결 엔진.
///
/// §4.2: 라인폭을 라인 시작점(origin) 기준 **정수 HWPUNIT 펜**으로 누적한다
/// (`cumulative[]` = `Σ to_hwp(advance)`). 각 토큰의 advance 를 `to_hwp` 로
/// **토큰마다** 절삭해 정수 누적한다 — 이는 한컴 실측(§4.2 정수 누적 펜)이자
/// 검증된 65/82 baseline 의 greedy 누적(`lw += to_hwp(width)`)과 동일하다.
/// 종전 연속 px 펜(`to_hwp(Σpx)`)은 탭 문단(lseg-05-tab)에서 정지점이 어긋나
/// 회귀했으므로 정수 누적으로 되돌린다.
///
/// §4.8: 자연폭은 이미 HWPUNIT 정수이므로 `to_hwp(pen)` 재양자화 없이 그대로
/// 정확 `≤` (tolerance 없음) 로 fit 판정한다.
///
/// condense(공백 압축)는 종전 greedy 와 **정수 바이트 동일**하게 유지한다:
/// `line_space_savings` 를 공백마다 `condense_space_savings_hwp(to_hwp(width), …)`
/// 로 정수 누적하고, 압축 라인폭·인입 게이트(`condense_fit_can_pull_next_token`)를
/// greedy 그대로 쓴다.
fn fill_lines(
    tokens: &[BreakToken],
    text_chars: &[char],
    available_width_px: f64,
    indent_px: f64,
    default_tab_width: f64,
    korean_break_unit: u8,
    condense_min_space: u8,
    // [Task #624] treat_as_char 인라인 개체의 (anchor char idx, 폭 HWPUNIT). anchor
    // 순 정렬. 개체는 char 를 갖지 않아 토큰이 없으므로 그 폭을 pen 에 예약해
    // 개체 앞에서 줄바꿈되게 한다. 인라인 개체 없는 문단은 빈 slice → 동작 불변.
    inline_obj_widths_hwp: &[(usize, i32)],
) -> Vec<LineBreakResult> {
    if tokens.is_empty() {
        return vec![LineBreakResult {
            start_idx: 0,
            end_idx: 0,
            max_font_size: 0.0,
            has_line_break: false,
        }];
    }

    let tab_w_px = if default_tab_width > 0.0 {
        default_tab_width
    } else {
        48.0
    };

    // ── 라인-로컬 상태 ──────────────────────────────────────────────
    // pen: 라인 origin 기준 정수 HWPUNIT 펜 위치. §4.2 cumulative[] =
    //      Σ to_hwp(advance) — 토큰마다 to_hwp 절삭 후 정수 누적.
    // line_space_savings: 압축 가능 공백 폭(HWPUNIT 정수) — greedy 와 동일.
    let mut results = Vec::new();
    let mut line_start_idx = 0usize;
    let mut pen = 0i32; // §4.2 cumulative[] (라인-로컬 HWPUNIT)
    // [Task #624] 현재 라인에서 이미 폭을 예약한 개체 anchor 상한(중복 예약 방지).
    let mut reserved_upto = 0usize;
    let mut line_space_savings = 0i32; // greedy 와 정수 동일
    let mut line_max_fs = 0.0f64;
    let mut is_first_line = true;

    // 마지막 줄바꿈 가능 지점(공백/탭/단일글자 경계)의 스냅샷.
    let mut has_break: bool = false;
    let mut break_char_idx: usize = 0;
    let mut fs_at_break = 0.0f64;

    // 라인 origin 기준 가용폭(HWPUNIT). 첫 줄 들여쓰기(양수)·내어쓰기(음수) 반영.
    let eff_w = |first: bool| -> i32 {
        if indent_px > 0.0 {
            if first {
                to_hwp((available_width_px - indent_px).max(1.0))
            } else {
                to_hwp(available_width_px)
            }
        } else if indent_px < 0.0 {
            if first {
                to_hwp(available_width_px)
            } else {
                to_hwp((available_width_px + indent_px).max(1.0))
            }
        } else {
            to_hwp(available_width_px)
        }
    };

    // 라인 커밋 헬퍼: [line_start_idx, end) 를 한 줄로 확정.
    macro_rules! commit {
        ($end:expr, $fs:expr, $brk:expr) => {{
            results.push(LineBreakResult {
                start_idx: line_start_idx,
                end_idx: $end,
                max_font_size: $fs,
                has_line_break: $brk,
            });
        }};
    }

    for (ti, token) in tokens.iter().enumerate() {
        match token {
            // ── 강제 줄바꿈(\n) ──────────────────────────────────────
            BreakToken::LineBreak { idx } => {
                commit!(*idx + 1, line_max_fs, true);
                line_start_idx = *idx + 1;
                reserved_upto = line_start_idx; // [Task #624]
                pen = 0;
                line_space_savings = 0;
                line_max_fs = 0.0;
                is_first_line = false;
                has_break = false;
            }

            // ── 탭: 다음 탭 정지점까지 펜 전진(동적 폭) ───────────────
            BreakToken::Tab { idx, max_font_size } => {
                if *max_font_size > line_max_fs {
                    line_max_fs = *max_font_size;
                }
                // §4.2: 탭 정지점은 정수 HWPUNIT 펜을 px 로 환산해 산출한다
                // (한컴과 동일). 펜이 이미 정수 HWPUNIT 이므로 /75.0 만 하면 된다.
                let pen_px = pen as f64 / 75.0;
                let next_tab_px = ((pen_px / tab_w_px).floor() + 1.0) * tab_w_px;

                // 탭 정지점이 가용폭을 넘고, 라인에 이미 내용이 있으면 종결.
                if to_hwp(next_tab_px) > eff_w(is_first_line) && line_start_idx < *idx {
                    if has_break {
                        // 마지막 브레이크 지점에서 종결, 잔여를 다음 줄로 이월.
                        // 선행 공백은 흡수(줄 시작에서 제외) — Text 경로와 대칭.
                        commit!(break_char_idx, fs_at_break, false);
                        let mut next_start = break_char_idx;
                        while next_start < text_chars.len() && text_chars[next_start] == ' ' {
                            next_start += 1;
                        }
                        line_start_idx = next_start;
                        reserved_upto = line_start_idx; // [Task #624]
                        let (carry_pen, carry_saving) =
                            carryover(tokens, ti, next_start, condense_min_space);
                        pen = carry_pen;
                        line_space_savings = carry_saving;
                    } else {
                        // 브레이크 지점 없음 → 탭 직전에서 강제 종결.
                        commit!(*idx, line_max_fs, false);
                        line_start_idx = *idx;
                        reserved_upto = line_start_idx; // [Task #624]
                        pen = 0;
                        line_space_savings = 0;
                        line_max_fs = *max_font_size;
                    }
                    is_first_line = false;
                    has_break = false;
                    // 새 줄에서 탭 정지점 재계산(정수 펜 → px 환산).
                    let pen_px = pen as f64 / 75.0;
                    pen = to_hwp(((pen_px / tab_w_px).floor() + 1.0) * tab_w_px);
                } else {
                    // 탭은 줄바꿈 가능 지점 — 스냅샷 후 펜 전진.
                    has_break = true;
                    break_char_idx = *idx;
                    fs_at_break = line_max_fs;
                    pen = to_hwp(next_tab_px);
                }
            }

            // ── 공백: 줄바꿈 가능 지점(줄 끝에서 흡수) ─────────────────
            BreakToken::Space {
                idx,
                width,
                max_font_size,
            } => {
                if *max_font_size > line_max_fs {
                    line_max_fs = *max_font_size;
                }
                // 공백은 '들어가기 전'이 브레이크 지점 — 폭 반영 전에 스냅샷.
                has_break = true;
                break_char_idx = *idx;
                fs_at_break = line_max_fs;
                // §4.2: 자연폭은 to_hwp 절삭 후 정수 HWPUNIT 로 누적.
                let space_hwp = to_hwp(*width);
                pen += space_hwp;
                // §4.8 condense: 공백 압축 가능분은 greedy 와 동일하게 공백마다
                // to_hwp 절삭 후 정수 누적(바이트 동일).
                line_space_savings += condense_space_savings_hwp(space_hwp, condense_min_space);
            }

            // ── 텍스트 조각(어절/단어/단일글자) ──────────────────────
            BreakToken::Text {
                start_idx,
                end_idx,
                width,
                max_font_size,
                ref char_widths,
            } => {
                if *max_font_size > line_max_fs {
                    line_max_fs = *max_font_size;
                }

                // [Task #624] 이 토큰 시작(start_idx) 이전에 놓인 인라인 개체의 폭을
                // pen 에 예약한다. 개체는 char 를 갖지 않아(gap-anchored) 자연폭에
                // 안 잡히므로, 개체 폭만큼 라인을 미리 채워 개체 앞에서 넘치게 한다.
                for &(anchor, w_hwp) in inline_obj_widths_hwp {
                    if anchor > line_start_idx && anchor <= *start_idx && anchor >= reserved_upto {
                        pen += w_hwp;
                    }
                }
                reserved_upto = reserved_upto.max(*start_idx + 1);

                let eff = eff_w(is_first_line);
                // §4.2: 이 토큰의 advance 를 to_hwp 로 절삭해 정수 펜에 더한 후보.
                //       cand 자체가 라인 origin 기준 자연폭(HWPUNIT)이다.
                let cand = pen + to_hwp(*width);
                let cand_condensed = condensed_line_width_hwp(cand, line_space_savings);

                // (1) 단일글자 CJK/한글 토큰: 글자 경계 자체가 줄바꿈 가능 지점.
                //     이 글자를 포함해도 압축 후 fit 이면 브레이크 지점을 '이 글자
                //     다음'으로 전진(§4.9 CJK 경량 경로 — 위치별 endpoint 커밋).
                if *end_idx - *start_idx == 1 && *start_idx > line_start_idx {
                    let c = text_chars[*start_idx];
                    let allow_break = if is_hangul(c) {
                        korean_break_unit == 1 // [#2185] bit7=1 = 글자 단위
                    } else {
                        is_cjk_ideograph(c)
                    };
                    if allow_break && cand_condensed <= eff {
                        has_break = true;
                        break_char_idx = *end_idx; // 이 글자 포함
                        fs_at_break = line_max_fs;
                    }
                }

                // (2) fit 판정 — greedy 와 동일한 condense 게이트 세맨틱을 정확 재현.
                //     · needs_condense_to_fit: 자연폭은 넘치되 압축하면 들어감
                //     · 이때만 인입 게이트(condense_fit_can_pull_next_token)를 본다.
                //       (한컴은 이미 빡빡한 줄엔 다음 어절을 끌어들이지 않음 — p03
                //        PDF 서문 민감; 종전 greedy 실측과 동일)
                let needs_condense_to_fit = cand > eff && cand_condensed <= eff;
                let condense_pull_allowed = !needs_condense_to_fit
                    || condense_fit_can_pull_next_token(
                        pen, // 현재 라인폭(HWPUNIT 정수)
                        line_space_savings,
                        eff,
                        *max_font_size,
                    );
                if cand_condensed <= eff && condense_pull_allowed {
                    pen = cand; // 들어감
                    continue;
                }

                // (3) 넘침 — 라인 종결이 필요하다.
                if *start_idx > line_start_idx && has_break {
                    // 3a. 마지막 브레이크 지점에서 종결. 새 줄 origin 기준으로
                    //     이월분을 정수 HWPUNIT 로 재-누적하고 선행 공백을 흡수.
                    commit!(break_char_idx, fs_at_break, false);
                    let mut next_start = break_char_idx;
                    while next_start < text_chars.len() && text_chars[next_start] == ' ' {
                        next_start += 1;
                    }
                    line_start_idx = next_start;
                    reserved_upto = line_start_idx; // [Task #624]
                    let (carry_pen, carry_saving) =
                        carryover(tokens, ti, next_start, condense_min_space);
                    pen = carry_pen + to_hwp(*width);
                    line_space_savings = carry_saving;
                    line_max_fs = *max_font_size;
                    is_first_line = false;
                    has_break = false;
                    continue;
                }

                // 3b. 브레이크 지점 없음(라인 첫 토큰이 홀로 넘침) → 글자 단위
                //     폴백(§4.9 워드 경로 finalizer). 펜이 이미 HWPUNIT 정수이므로
                //     그대로 넘기고 토큰 내 per-glyph 폭으로 라인을 쪼갠다.
                let cw_hwp: Vec<i32> = char_widths.iter().map(|w| to_hwp(*w)).collect();
                let (parts, remaining_w, remaining_fs) = char_level_break_hwp(
                    text_chars,
                    *start_idx,
                    *end_idx,
                    &mut line_start_idx,
                    pen, // 현재 라인-로컬 펜(HWPUNIT 정수)
                    line_max_fs,
                    eff_w(is_first_line),
                    eff_w(false),
                    is_first_line,
                    &cw_hwp,
                );
                for r in parts {
                    results.push(r);
                    is_first_line = false;
                }
                // 폴백 핸드오프: char_level_break 는 HWPUNIT 정수 도메인이므로
                // 잔여 폭(정수 HWPUNIT)을 그대로 펜에 되돌린다(왕복 절삭 무추가).
                pen = remaining_w;
                line_space_savings = 0;
                line_max_fs = remaining_fs;
                has_break = false;
                reserved_upto = line_start_idx; // [Task #624]
            }
        }
    }

    // 마지막 줄 커밋.
    let last_end = tokens
        .last()
        .map(|t| match t {
            BreakToken::Text { end_idx, .. } => *end_idx,
            BreakToken::Space { idx, .. }
            | BreakToken::Tab { idx, .. }
            | BreakToken::LineBreak { idx } => *idx + 1,
        })
        .unwrap_or(text_chars.len());

    if line_start_idx <= last_end {
        commit!(last_end, line_max_fs, false);
    }

    if results.is_empty() {
        results.push(LineBreakResult {
            start_idx: 0,
            end_idx: text_chars.len(),
            max_font_size: 0.0,
            has_line_break: false,
        });
    }

    // 줄 꼬리 금칙(line-end kinsoku): 한컴 endpoint 후퇴로 정합.
    apply_line_end_kinsoku(&mut results, text_chars);

    results
}

/// 라인 이월(carryover) — 새 줄 origin([new_line_start, ·)) 기준으로 아직 처리
/// 안 된 선행 토큰들의 누적 펜(HWPUNIT 정수 자연폭)과 압축 가능 공백(HWPUNIT
/// 정수)을 되살린다. 종전 greedy 의 `recalc_width_hwp`+`recalc_space_savings_hwp`
/// 를 하나로 융합 — §4.2 정수 누적 펜(`Σ to_hwp(advance)`)과 정확히 일치한다.
fn carryover(
    tokens: &[BreakToken],
    current_token_idx: usize,
    new_line_start: usize,
    condense_min_space: u8,
) -> (i32, i32) {
    let mut pen = 0i32;
    let mut saving = 0i32;
    for t in &tokens[..current_token_idx] {
        match t {
            BreakToken::Text {
                start_idx, width, ..
            } if *start_idx >= new_line_start => {
                pen += to_hwp(*width);
            }
            BreakToken::Space { idx, width, .. } if *idx >= new_line_start => {
                let space_hwp = to_hwp(*width);
                pen += space_hwp;
                saving += condense_space_savings_hwp(space_hwp, condense_min_space);
            }
            _ => {}
        }
    }
    (pen, saving)
}

/// 줄 꼬리 금칙(line-end kinsoku): 줄이 여는 괄호/통화기호 등 줄 끝에 올 수
/// 없는 문자로 끝나면 그 클러스터를 다음 줄로 후퇴시킨다(한컴 endpoint 후퇴,
/// 최대 2 클러스터). 여는/닫는 따옴표('/")는 줄머리·줄꼬리 금칙에 겹치므로
/// 오검출 방지 위해 제외한다.
fn apply_line_end_kinsoku(results: &mut [LineBreakResult], text_chars: &[char]) {
    let n = results.len();
    if n < 2 {
        return;
    }
    for k in 0..n - 1 {
        if results[k].has_line_break {
            continue; // 강제 줄바꿈(\n)으로 끝나는 줄은 후퇴 대상 아님
        }
        let start = results[k].start_idx;
        let orig_end = results[k].end_idx;
        let mut end = orig_end;
        let mut retreats = 0;
        while retreats < 2 && end > start + 1 {
            let last = text_chars[end - 1];
            if is_line_end_forbidden(last) && !is_line_start_forbidden(last) {
                end -= 1;
                retreats += 1;
            } else {
                break;
            }
        }
        if end != orig_end {
            results[k].end_idx = end;
            if results[k + 1].start_idx > end {
                results[k + 1].start_idx = end; // 후퇴 클러스터를 다음 줄 시작으로
            }
        }
    }
}

/// 긴 단어 폴백: 글자 단위 분할 (HWPUNIT)
/// char_widths_hwp: 토큰 내 각 글자의 HWPUNIT 폭 (None이면 휴리스틱)
fn char_level_break_hwp(
    text_chars: &[char],
    token_start: usize,
    token_end: usize,
    line_start_idx: &mut usize,
    mut lw: i32,
    mut line_max_fs: f64,
    first_line_w: i32,
    normal_w: i32,
    mut is_first_line: bool,
    char_widths_hwp: &[i32], // 토큰 내 글자별 HWPUNIT 폭
) -> (Vec<LineBreakResult>, i32, f64) {
    let mut results = Vec::new();
    let mut current_w = if is_first_line {
        first_line_w
    } else {
        normal_w
    };

    for ci in token_start..token_end {
        let rel_idx = ci - token_start;
        let char_w = if rel_idx < char_widths_hwp.len() {
            char_widths_hwp[rel_idx]
        } else {
            let ch = text_chars[ci];
            let char_w_px = if is_cjk_char(ch) {
                line_max_fs.max(12.0)
            } else {
                line_max_fs.max(12.0) * 0.5
            };
            to_hwp(char_w_px)
        };

        if lw + char_w > current_w && ci > *line_start_idx {
            results.push(LineBreakResult {
                start_idx: *line_start_idx,
                end_idx: ci,
                max_font_size: line_max_fs,
                has_line_break: false,
            });
            *line_start_idx = ci;
            lw = char_w;
            is_first_line = false;
            current_w = normal_w;
        } else {
            lw += char_w;
        }
    }

    (results, lw, line_max_fs)
}

/// 문단의 line_segs를 텍스트 내용과 컬럼 너비에 맞게 재계산한다.
///
/// 텍스트 편집(삽입/삭제) 후 호출하여 줄 바꿈을 재배치한다.
/// `available_width_px`는 문단 여백을 제외한 사용 가능 너비(px)이다.
fn inline_control_line_height_hwp(para: &Paragraph) -> Option<i32> {
    para.controls
        .iter()
        .filter_map(|ctrl| match ctrl {
            Control::Picture(pic) if pic.common.treat_as_char => Some(pic.common.height as i32),
            Control::Shape(shape) if shape.common().treat_as_char => {
                let common_h = shape.common().height as i32;
                let current_h = shape.shape_attr().current_height as i32;
                Some(common_h.max(current_h))
            }
            Control::Table(table) if table.common.treat_as_char => Some(table.common.height as i32),
            Control::Equation(eq) if eq.common.treat_as_char => Some(eq.common.height as i32),
            Control::Form(form) => Some(form.height as i32),
            _ => None,
        })
        .filter(|height| *height > 0)
        .max()
}

pub(crate) fn inline_control_size_hwp(ctrl: &Control) -> Option<(i32, i32)> {
    let (width, height) = match ctrl {
        Control::Picture(pic) if pic.common.treat_as_char => {
            (pic.common.width as i32, pic.common.height as i32)
        }
        Control::Shape(shape) if shape.common().treat_as_char => {
            let common = shape.common();
            let shape_attr = shape.shape_attr();
            (
                (common.width as i32).max(shape_attr.current_width as i32),
                (common.height as i32).max(shape_attr.current_height as i32),
            )
        }
        Control::Table(table) if table.common.treat_as_char => {
            let width = table.get_column_widths().iter().sum::<u32>() as i32;
            (width, table.common.height as i32)
        }
        Control::Equation(eq) if eq.common.treat_as_char => {
            (eq.common.width as i32, eq.common.height as i32)
        }
        Control::Form(form) => (form.width as i32, form.height as i32),
        _ => return None,
    };

    if width > 0 && height > 0 {
        Some((width, height))
    } else {
        None
    }
}

fn apply_inline_control_line_height(
    seg: &mut LineSeg,
    height_hwp: i32,
    adjust_baseline_to_object_bottom: bool,
) {
    if height_hwp > seg.line_height {
        if adjust_baseline_to_object_bottom {
            // Native HwpApp keeps the ordinary-text height/baseline metrics
            // separate from the all-unit maximum, then computes:
            //   object_max + text_height - text_baseline
            // This puts the inline object's bottom on the text baseline while
            // retaining the text descent below it.
            let text_height = seg.line_height.max(0);
            let text_baseline = seg.baseline_distance.clamp(0, text_height);
            let text_descent = text_height - text_baseline;
            seg.line_height = height_hwp.saturating_add(text_descent);
            seg.text_height = text_height;
            seg.baseline_distance = height_hwp;
        } else {
            seg.line_height = height_hwp;
            seg.text_height = height_hwp;
            seg.baseline_distance = (height_hwp as f64 * 0.85).round() as i32;
        }
    }
}

pub(crate) fn reflow_line_segs(
    para: &mut Paragraph,
    available_width_px: f64,
    styles: &ResolvedStyleSet,
    dpi: f64,
) {
    // rhwp 는 line_segs 를 **항상** 여기서 스스로 계산한다. 저장된 line_segs 는
    // 레이아웃 소스가 아니라 파서 보존·검증(round-trip) 전용이므로 dimension·tag·
    // vpos 를 저장 seg 에서 상속하지 않는다. 절대 vpos 는 recalculate_section_vpos
    // 가 문단 간 누적으로 결정한다 (여기서는 0 으로 시작).
    let seg_width_hwp = px_to_hwpunit(available_width_px, dpi);

    // ParaPr의 줄간격 설정 (합성 LineSeg에서 line_spacing 계산에 사용)
    let para_style = styles.para_styles.get(para.para_shape_id as usize);
    let ls_type = para_style
        .map(|s| s.line_spacing_type)
        .unwrap_or(LineSpacingType::Percent);
    let ls_value = para_style.map(|s| s.line_spacing).unwrap_or(160.0);

    // 줄별 max_font_size에 따라 line_height/text_height/baseline_distance를 계산
    // 한컴은 줄마다 최대 폰트 크기에 맞게 다른 치수를 사용
    let make_line_seg = |utf16_start: u32, max_font_size: f64| -> LineSeg {
        let fs = if max_font_size > 0.0 {
            max_font_size
        } else {
            12.0
        };
        let line_height_hwp = font_size_to_line_height(fs, dpi);
        let text_height_hwp = line_height_hwp;
        let baseline_distance_hwp = (line_height_hwp as f64 * 0.85) as i32;
        let line_spacing_hwp = compute_line_spacing_hwp(ls_type, ls_value, line_height_hwp, dpi);
        // reflow 는 언제나 자기 계산 결과이므로 tag 는 단일 세그먼트 라인으로
        // 고정한다. 구현속성(TAG_IMPLEMENTATION_PROPERTY) 표식은 더 이상 부여하지
        // 않는다 — 저장 seg 와의 구분(합성 vs 실측)이 레이아웃 판단에서 제거됐다.
        LineSeg {
            text_start: utf16_start,
            line_height: line_height_hwp,
            text_height: text_height_hwp,
            baseline_distance: baseline_distance_hwp,
            line_spacing: line_spacing_hwp,
            segment_width: seg_width_hwp,
            tag: LineSeg::TAG_SINGLE_SEGMENT_LINE,
            ..Default::default()
        }
    };

    if para.text.is_empty() {
        let inline_sizes = para
            .controls
            .iter()
            .filter_map(inline_control_size_hwp)
            .collect::<Vec<_>>();
        if !inline_sizes.is_empty() {
            let max_line_width = seg_width_hwp.max(1);
            let mut line_specs: Vec<(usize, i32, i32)> = Vec::new();
            let mut line_start = 0usize;
            let mut line_width = 0i32;
            let mut line_height = 0i32;

            for (idx, (ctrl_width, ctrl_height)) in inline_sizes.iter().copied().enumerate() {
                if line_width > 0 && line_width + ctrl_width > max_line_width {
                    line_specs.push((line_start, line_width, line_height));
                    line_start = idx;
                    line_width = 0;
                    line_height = 0;
                }
                line_width += ctrl_width;
                line_height = line_height.max(ctrl_height);
            }
            line_specs.push((line_start, line_width, line_height));

            // 인라인 개체 줄박스: 치수는 make_line_seg(폰트/줄간격)와 개체 높이에서
            // 계산한다. 저장 template 을 참조하지 않는다 (레이아웃 소스 금지).
            let mut new_line_segs = Vec::with_capacity(line_specs.len());
            for (start_pos, _line_width, height_hwp) in line_specs.into_iter() {
                let mut seg = make_line_seg(start_pos as u32, 0.0);
                apply_inline_control_line_height(
                    &mut seg,
                    height_hwp,
                    styles.adjust_baseline_of_object_to_bottom,
                );
                new_line_segs.push(seg);
            }

            // vpos 는 문단 내 0 원점부터 누적(절대 vpos 는 recalculate_section_vpos).
            let mut vpos = 0;
            for seg in &mut new_line_segs {
                seg.vertical_pos = vpos;
                vpos += seg.line_height + seg.line_spacing;
            }
            para.line_segs = new_line_segs;
        } else {
            // 빈 문단도 활성 글자 모양의 크기로 줄을 만든다. 저장 seg 치수/vpos 를
            // 상속하지 않는다 — 절대 vpos 는 recalculate_section_vpos 가 결정한다.
            let font_size = para
                .char_shapes
                .first()
                .and_then(|char_shape| styles.char_styles.get(char_shape.char_shape_id as usize))
                .map(|style| style.font_size)
                .unwrap_or(12.0);
            let mut seg = make_line_seg(0, font_size);
            if let Some(height_hwp) = inline_control_line_height_hwp(para) {
                apply_inline_control_line_height(
                    &mut seg,
                    height_hwp,
                    styles.adjust_baseline_of_object_to_bottom,
                );
            }
            para.line_segs = vec![seg];
        }
        return;
    }

    let text_chars: Vec<char> = para.text.chars().collect();
    let text_len = text_chars.len();

    // 문단 스타일에서 들여쓰기 및 줄 나눔 설정 조회
    let para_style = styles.para_styles.get(para.para_shape_id as usize);
    let indent_px = para_style.map(|s| s.indent).unwrap_or(0.0);
    let english_break_unit = para_style.map(|s| s.english_break_unit).unwrap_or(0);
    let korean_break_unit = para_style.map(|s| s.korean_break_unit).unwrap_or(0);
    let condense_min_space = para_style.map(|s| s.condense_min_space).unwrap_or(0);
    let tab_width = para_style.map(|s| s.default_tab_width).unwrap_or(0.0);

    // 토큰화 → 줄 채움 → LineSeg 생성
    let tokens = tokenize_paragraph(
        &text_chars,
        &para.char_offsets,
        &para.char_shapes,
        styles,
        english_break_unit,
        korean_break_unit,
    );
    // [Task #624] treat_as_char 인라인 개체의 (anchor char idx, 폭 HWPUNIT) 목록.
    // 개체는 텍스트 char 가 없어 tokenize 에 안 잡히므로 fill_lines 가 그 폭을
    // 예약해 개체 앞에서 줄바꿈하도록 한다. inline_control_size_hwp 는 TAC 개체만
    // Some → 인라인 개체 없는 문단은 빈 목록 → 동작 불변.
    let inline_obj_widths_hwp: Vec<(usize, i32)> = {
        // recompose_for_cell_width 와 동일한 anchor 함수를 써야 reflow 와 셀 재래핑의
        // 줄바꿈 위치가 일치한다 (control_text_positions 와 결과가 다르면 reflow 가
        // 개체를 엉뚱한 줄에 두고 boost → 셀 재래핑이 그 boost 를 상속).
        let positions = crate::document_core::find_control_text_positions(para);
        let mut v: Vec<(usize, i32)> = para
            .controls
            .iter()
            .enumerate()
            .filter_map(|(i, c)| Some((*positions.get(i)?, inline_control_size_hwp(c)?.0)))
            .collect();
        v.sort_by_key(|(p, _)| *p);
        v
    };
    let line_breaks = fill_lines(
        &tokens,
        &text_chars,
        available_width_px,
        indent_px,
        tab_width,
        korean_break_unit,
        condense_min_space,
        &inline_obj_widths_hwp,
    );
    let mut new_line_segs: Vec<LineSeg> = Vec::new();
    for lb in &line_breaks {
        let utf16_start = if new_line_segs.is_empty() {
            0 // 첫 번째 줄의 text_start는 항상 0 (문단 시작)
        } else if lb.start_idx < para.char_offsets.len() {
            para.char_offsets[lb.start_idx]
        } else if !para.char_offsets.is_empty() {
            // start_idx가 텍스트 끝을 넘을 때: 마지막 문자 다음 UTF-16 위치
            let last_idx = para.char_offsets.len() - 1;
            let last_char_utf16_len = para
                .text
                .chars()
                .nth(last_idx)
                .map(|c| c.len_utf16() as u32)
                .unwrap_or(1);
            para.char_offsets[last_idx] + last_char_utf16_len
        } else {
            lb.start_idx as u32
        };
        let fs = if lb.max_font_size > 0.0 {
            lb.max_font_size
        } else {
            12.0
        };
        new_line_segs.push(make_line_seg(utf16_start as u32, fs));
    }

    if new_line_segs.is_empty() {
        new_line_segs.push(make_line_seg(0, 12.0));
    }

    // TAC 개체 높이 반영. 한컴 저장 구조(diag_table_height_ground_truth 실증):
    //  - 블록형 TAC 표(전폭, is_tac_table_inline_in_para=false): 표는 **자기 줄**
    //    (lh=표높이)을 갖고, 실제 본문 텍스트가 있으면 그 다음 줄(ls[1]+)에 온다.
    //    공백만 있는 문단은 표줄 하나로 흡수(추가 줄 없음 — B-A 회귀의 원인 제거).
    //  - 인라인 TAC 개체(수식·작은 그림 등): 종전대로 그 줄의 높이만 확대.
    {
        let block_table_h = para.controls.iter().find_map(|ctrl| match ctrl {
            Control::Table(t)
                if t.common.treat_as_char
                    && t.common.height > 0
                    && !crate::renderer::height_measurer::is_tac_table_inline_in_para(
                        t,
                        seg_width_hwp,
                        para,
                    ) =>
            {
                Some(t.common.height as i32)
            }
            _ => None,
        });
        if let Some(h) = block_table_h {
            if para.text.trim().is_empty() {
                // 공백만 → 표줄 하나로 흡수.
                if let Some(seg) = new_line_segs.first_mut() {
                    apply_inline_control_line_height(
                        seg,
                        h,
                        styles.adjust_baseline_of_object_to_bottom,
                    );
                }
            } else {
                // 실제 본문 → 표줄(ts=0, lh=표높이)을 맨 앞에 삽입, 본문줄 유지.
                // 본문 첫 줄의 text_start 를 첫 본문 글자 오프셋으로 재-각인.
                let mut table_seg = make_line_seg(0, 0.0);
                apply_inline_control_line_height(
                    &mut table_seg,
                    h,
                    styles.adjust_baseline_of_object_to_bottom,
                );
                if let Some(first_body) = para.char_offsets.first().copied() {
                    if let Some(seg0) = new_line_segs.first_mut() {
                        seg0.text_start = first_body;
                    }
                }
                new_line_segs.insert(0, table_seg);
            }
        } else if let Some(height_hwp) = inline_control_line_height_hwp(para) {
            if let Some(seg) = new_line_segs.first_mut() {
                apply_inline_control_line_height(
                    seg,
                    height_hwp,
                    styles.adjust_baseline_of_object_to_bottom,
                );
            }
        }
    }

    // vertical_pos 누적 계산 (각 줄의 문단 내 Y 오프셋). 문단 내 원점은 0 —
    // 절대 vpos 는 recalculate_section_vpos 가 문단 간 누적으로 채운다.
    let mut vpos = 0;
    for i in 0..new_line_segs.len() {
        new_line_segs[i].vertical_pos = vpos;
        vpos += new_line_segs[i].line_height + new_line_segs[i].line_spacing;
    }

    para.line_segs = new_line_segs;
}

/// 구역 내 문단들의 vertical_pos를 순차적으로 재계산한다.
///
/// `start_para`부터 구역 끝까지 각 문단의 vpos를 이전 문단의 vpos_end 기준으로 재계산.
/// 표 등 특수 문단의 line_height는 보존하고 vpos만 갱신한다.
///
/// [Task #2299] 저장 vpos 리셋(단/쪽 경계 인코딩) 보존: 편집발 재계산이 구역 전체를
/// 선형 누적 좌표로 이어붙이면 다단 zone 의 단-상대 리셋(급감)이 소멸해
/// typeset(#321/#470/#702)·pagination 의 단/쪽 진행 신호가 무력화된다
/// (shortcut.hwp 앞문단 편집 시 col=[0,1]→[0], 7→9쪽). 현재 문단의 저장 first 가
/// 직전 문단의 "이동 전(저장)" end 보다 감소하면 경계 인코딩으로 보고 delta=0 으로
/// 보존한다. 저장 좌표는 밴드 내 정상 흐름에서 단조 증가하므로 감소 감지에 임계가
/// 필요 없다.
///
/// 좌표 갱신은 경계 성격별로 셋으로 나뉜다.
///
/// - **리셋 경계**: delta=0 보존.
/// - **변조 인접 경계**(현재 문단이 편집 대상 `start_para` 이거나 신규
///   문단(`ignore_reset_range`)이거나, 직전 문단이 그중 하나): 직전 이동 후 end 에
///   문단 여백 gap(spacing_after + spacing_before, 셀 recalc `boundary_gaps` 동일
///   산식)을 더해 다시 잇는다. reflow/신규 생성으로 저장 gap 이 소실된 경계라
///   스타일에서 재유도한다. gap 없는 abutment 는 문단 간격을 압축해 near-top
///   리셋(#1086/#1921)의 `prev_vpos_end > 60000` 임계를 무너뜨렸다
///   (SO-SUEOP.hwpx 46→44).
/// - **미변조 연속 경계**: 직전 문단의 delta 를 그대로 캐리해 저장(또는 로드 합성
///   #927) 문단 간격을 정확히 보존한다. 스타일 gap 재유도는 저장 gap 과의
///   오차(px 왕복 절삭 ±1HU, 스타일-저장 불일치)를 밴드 전체에 누적시키고 로드
///   합성 gap-less 체인과도 어긋나므로 쓰지 않는다. delta==0 이면 순수 no-op.
///
/// 리셋 감지는 저장 좌표끼리의 비교여야 한다. 직전 문단이 변조 대상이면 그 end 는
/// 저장 좌표가 아니므로(성장 편집이 다음 문단을 가짜 리셋으로 동결시키고,
/// placeholder 는 기준을 붕괴시킨다) reflow 가 보존하는 **first** 로 비교한다.
/// 미변조 경계는 end 기준을 유지한다(연속 0-first 밴드 감지에 필요).
///
/// placeholder 저지선 2종: ① split/insert/paste 가 방금 만든 신규 문단의 vpos=0 은
/// 경계 인코딩이 아니다 — 보존하면 문단마다 가짜 쪽나눔이 생긴다
/// (test_page_boundary_with_incremental_spacing_increase 핀). 호출자가 신규 구간을
/// `ignore_reset_range` 로 지정하면 보존 없이 흐름에 연결한다(셀 경로
/// `recalculate_cell_paragraph_vpos` 의 ignore_reset_at 과 동일 취지, 다중 삽입을
/// 위해 범위형). ② lineseg 부재였다가 on-demand reflow(#177/#927)로 합성된
/// seg(TAG_IMPLEMENTATION_PROPERTY, #1811)도 보존하지 않는다.
///
/// 줄 전진량은 로드 경로(document.rs 의 vpos 체인)와 동일하게 TAC 호스트
/// 줄(lh>th)을 th 기준으로 센다 — lh 기준이면 인라인 개체 호스트의 end 가 저장
/// 후속 first 를 넘어서 가짜 리셋을 만든다.
pub(crate) fn recalculate_section_vpos(
    paragraphs: &mut [Paragraph],
    start_para: usize,
    ignore_reset_range: Option<std::ops::Range<usize>>,
    start_stored_end: Option<i32>,
    styles: &ResolvedStyleSet,
    dpi: f64,
    is_hwp3_variant: bool,
) {
    if paragraphs.is_empty() || start_para >= paragraphs.len() {
        return;
    }

    // 문단 경계 gap (HWPUNIT) = 앞 문단 spacing_after + 뒤 문단 spacing_before.
    // recalculate_cell_paragraph_vpos 의 boundary_gaps 와 동일 산식.
    let boundary_gap = |prev: &Paragraph, curr: &Paragraph| -> i32 {
        let spacing_after = styles
            .para_styles
            .get(prev.para_shape_id as usize)
            .map(|style| style.spacing_after)
            .unwrap_or(0.0);
        let spacing_before = styles
            .para_styles
            .get(curr.para_shape_id as usize)
            .map(|style| style.spacing_before)
            .unwrap_or(0.0);
        let spacing_before =
            crate::renderer::hwp3_variant_flow_spacing_before(spacing_before, is_hwp3_variant);
        px_to_hwpunit(spacing_after + spacing_before, dpi)
    };

    // 줄 전진량 — 로드 경로와 동일한 TAC th-관례. saturating: 조작 파일의 극단
    // spacing/좌표로 i32 가 넘치지 않게 한다 (release wasm 은 overflow-check 가
    // 없어 무음 랩 → 전 문단 오판으로 이어진다).
    let seg_advance = |ls: &LineSeg| -> i32 {
        let height = if ls.line_height > ls.text_height && ls.text_height > 0 {
            ls.text_height
        } else {
            ls.line_height
        };
        height.saturating_add(ls.line_spacing)
    };
    let seg_end = |p: &Paragraph| -> Option<i32> {
        p.line_segs
            .last()
            .map(|ls| ls.vertical_pos.saturating_add(seg_advance(ls)))
    };
    let is_ignored = |pi: usize| {
        ignore_reset_range
            .as_ref()
            .is_some_and(|range| range.contains(&pi))
    };

    // 직전 문단(마지막 비어있지 않은 lineseg 보유 문단) 인덱스.
    // start_para 이전 문단들은 이 호출에서 이동하지 않으므로 현재 좌표가 곧 저장 좌표다.
    let mut prev_idx: Option<usize> = paragraphs[..start_para]
        .iter()
        .rposition(|p| !p.line_segs.is_empty());
    let mut next_vpos = match prev_idx {
        Some(pp) => seg_end(&paragraphs[pp]).unwrap_or(0),
        // 첫 문단: 기존 vpos 유지
        None => paragraphs[start_para]
            .line_segs
            .first()
            .map(|ls| ls.vertical_pos)
            .unwrap_or(0),
    };
    // 리셋 감지 기준 — 직전 문단의 "이동 전(저장)" first/end.
    let mut orig_prev_first: Option<i32> = prev_idx
        .and_then(|pp| paragraphs[pp].line_segs.first())
        .map(|ls| ls.vertical_pos);
    let mut orig_prev_end: Option<i32> = prev_idx.and_then(|pp| seg_end(&paragraphs[pp]));
    // 직전 문단이 이번 편집의 변조 대상이었는가 + 직전 문단에 적용된 delta.
    let mut prev_modified = false;
    let mut prev_delta: i32 = 0;

    for pi in start_para..paragraphs.len() {
        if paragraphs[pi].line_segs.is_empty() {
            continue;
        }

        let para_modified = pi == start_para || is_ignored(pi);
        let current_start = paragraphs[pi].line_segs[0].vertical_pos;
        let is_original_lineseg =
            paragraphs[pi].line_segs[0].tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0;

        // 리셋 감지: 신규 문단(placeholder)·합성 seg 는 제외. 기준은 직전 문단의
        // "저장" 좌표여야 한다 — 직전이 편집 문단(start_para)이면 reflow 로 end 가
        // 이미 변조됐으므로 호출자가 캡처해 준 reflow 이전 저장 end 를 쓰고(성장
        // 편집의 가짜 리셋과 저장-겹침 문서의 정당한 리셋을 모두 정확히 판별),
        // 없으면 reflow 가 보존하는 first 로 보수적으로 비교한다. 신규 문단이
        // 직전이면 placeholder 라 first(=0) 기준. 미변조 경계는 end 기준을
        // 유지한다(연속 0-first 밴드 감지에 필요).
        let prev_stored_bound = if prev_idx == Some(start_para) && !is_ignored(start_para) {
            start_stored_end.or(orig_prev_first)
        } else if prev_modified {
            orig_prev_first
        } else {
            orig_prev_end
        };
        // [회귀 수정] 편집으로 방금 reflow 된 문단(pi == start_para)은 그 vertical_pos
        // 가 문단-로컬 원점 0 으로 재계산돼 있어(reflow 계약), 이를 직전 문단의 저장
        // end 와 비교하면 항상 0 < end 라 가짜 리셋으로 오판된다 → 편집 문단이 흐름에
        // 누적되지 못하고 매번 페이지 상단(vpos 0)에 고정 → 문단마다 새 페이지(줄간격
        // 증가가 페이지 수에 무반영). 편집 문단은 명시적 쪽/단 나누기(column_type)로만
        // 페이지 경계를 가지므로, 저장 vpos 리셋 판정에서 제외하고 항상 흐름에 잇는다.
        // (후속 문단 pi > start_para 의 리셋 보존은 그대로 유지 — Task #2299 불변.)
        let is_reset = is_original_lineseg
            && !is_ignored(pi)
            && pi != start_para
            && prev_stored_bound.is_some_and(|bound| current_start < bound);

        let delta = if is_reset {
            // 단/쪽 리셋 경계 — 저장 좌표 유지.
            0
        } else if para_modified || prev_modified {
            // 변조 인접 경계 — 이동 후 흐름에 스타일 여백 gap 으로 다시 잇는다.
            let gap = prev_idx
                .map(|pp| boundary_gap(&paragraphs[pp], &paragraphs[pi]))
                .unwrap_or(0);
            next_vpos.saturating_add(gap) - current_start
        } else {
            // 미변조 연속 경계 — 직전 delta 캐리로 기존 간격을 정확히 보존.
            prev_delta
        };

        // 다음 문단의 리셋 감지 기준은 "이동 전(저장)" first/end 로 기록한다.
        let orig_first = current_start;
        let orig_end = seg_end(&paragraphs[pi]);

        if delta != 0 {
            // 모든 LineSeg의 vpos를 delta만큼 이동
            for seg in &mut paragraphs[pi].line_segs {
                seg.vertical_pos = seg.vertical_pos.saturating_add(delta);
            }
        }

        // 다음 문단의 시작 vpos 계산 (이동 후 end = 저장 end + delta)
        if let Some(end) = orig_end {
            next_vpos = end.saturating_add(delta);
        }
        orig_prev_first = Some(orig_first);
        orig_prev_end = orig_end;
        prev_modified = para_modified;
        prev_delta = delta;
        prev_idx = Some(pi);
    }
}

/// [Task #2299] 문단의 흐름 end (마지막 LineSeg 의 vpos + 전진량, TAC th-관례).
/// 편집 호출자가 reflow 이전에 캡처해 `recalculate_section_vpos` 의
/// `start_stored_end` 로 전달하기 위한 헬퍼 — reflow 가 end 를 덮은 뒤에는 저장
/// 좌표를 복원할 수 없다.
pub(crate) fn paragraph_flow_end(para: &Paragraph) -> Option<i32> {
    para.line_segs.last().map(|ls| {
        let height = if ls.line_height > ls.text_height && ls.text_height > 0 {
            ls.text_height
        } else {
            ls.line_height
        };
        ls.vertical_pos
            .saturating_add(height.saturating_add(ls.line_spacing))
    })
}

/// font_size(px)를 LineSeg의 line_height(HWPUNIT)로 변환한다.
/// HWP의 LineSeg.line_height = 폰트 크기 (HWPUNIT).
/// 실증 데이터: 10pt → lh=1000, 12pt → lh=1200, 25pt → lh=2500
fn font_size_to_line_height(font_size_px: f64, dpi: f64) -> i32 {
    px_to_hwpunit(font_size_px, dpi)
}

/// ParaPr의 줄간격 설정으로부터 LineSeg.line_spacing(HWPUNIT)을 계산한다.
///
/// line_spacing = 현재 줄 하단 → 다음 줄 상단 사이의 추가 간격.
/// Y advance = line_height + line_spacing.
fn compute_line_spacing_hwp(
    ls_type: LineSpacingType,
    ls_value: f64,
    line_height_hwp: i32,
    dpi: f64,
) -> i32 {
    match ls_type {
        LineSpacingType::Percent => {
            // ls_value = 비율값 (예: 160 = 160%)
            // 전체 줄 피치 = line_height * percent / 100
            // line_spacing = 전체 줄 피치 - line_height
            // [#2279] sub-100% 퍼센트는 음수 gap(압축)으로 존중 — 한글은
            // line=60% 를 advance 13.6px(=lh×0.6)로 렌더한다 (36398700 pi20
            // 한글 재저장 anchor 1020HU 실측). 종전 .max(0) 클램프는 fresh
            // 합성을 lh 그대로(+9px/문단) 팽창시켰다.
            // ls_value<=0 은 결손 데이터(속성 미지정 파싱 0) — 음수 적용 금지.
            if ls_value > 0.0 {
                (line_height_hwp as f64 * (ls_value - 100.0) / 100.0) as i32
            } else {
                0
            }
        }
        LineSpacingType::Fixed => {
            // ls_value = 고정 줄 피치 (px, resolver가 HWPUNIT→px 변환 완료)
            // line_spacing = 고정값 - line_height
            let fixed_hwp = px_to_hwpunit(ls_value, dpi);
            (fixed_hwp - line_height_hwp).max(0)
        }
        LineSpacingType::SpaceOnly => {
            // ls_value = 줄 사이 추가 간격만 (px)
            px_to_hwpunit(ls_value, dpi)
        }
        LineSpacingType::Minimum => {
            // 최소값: 콘텐츠가 최소값보다 크면 추가 간격 없음
            let min_hwp = px_to_hwpunit(ls_value, dpi);
            (min_hwp - line_height_hwp).max(0)
        }
    }
}

#[cfg(test)]
mod compatibility_tests {
    use super::*;

    #[test]
    fn adjust_baseline_of_object_to_bottom_retains_text_descent() {
        let mut seg = LineSeg {
            line_height: 1200,
            text_height: 1200,
            baseline_distance: 1020,
            ..Default::default()
        };

        apply_inline_control_line_height(&mut seg, 2000, true);

        assert_eq!(seg.line_height, 2180);
        assert_eq!(seg.text_height, 1200);
        assert_eq!(seg.baseline_distance, 2000);
    }

    #[test]
    fn default_inline_object_metric_keeps_existing_behavior() {
        let mut seg = LineSeg {
            line_height: 1200,
            text_height: 1200,
            baseline_distance: 1020,
            ..Default::default()
        };

        apply_inline_control_line_height(&mut seg, 2000, false);

        assert_eq!(seg.line_height, 2000);
        assert_eq!(seg.text_height, 2000);
        assert_eq!(seg.baseline_distance, 1700);
    }
}
