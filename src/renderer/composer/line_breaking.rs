//! 줄 나눔 엔진 (Line Breaking Engine)
//!
//! 문단 텍스트를 토큰화하고 줄 나눔을 수행한다.
//! 한글 어절/글자, 영어 단어/하이픈, CJK 개별 분할을 지원한다.

use super::{find_active_char_shape, is_lang_neutral};
use crate::model::control::{Control, CTRL_CHAR_CODE_UNITS};
use crate::model::paragraph::{CharShapeRef, ColumnBreakType, LineSeg, Paragraph};
use crate::model::style::LineSpacingType;
use crate::renderer::layout::{
    estimate_text_width, estimate_text_width_unrounded, hancom_regenerated_space_width,
    is_cjk_char, resolved_to_text_style,
};
use crate::renderer::layout_frame::{FrameRowMetrics, LayoutFrame, RowSegment};
use crate::renderer::px_to_hwpunit;
use crate::renderer::style_resolver::{detect_lang_category, ResolvedStyleSet};
use std::ops::Range;

/// A complete, source-independent projection of one supported Picture wrap
/// band. The document layer owns the one-shot publication of every paragraph
/// in this range.
#[derive(Debug, Clone)]
pub(crate) struct PictureBandLayout {
    pub(crate) paragraph_range: Range<usize>,
    pub(crate) line_segs: Vec<Vec<LineSeg>>,
}

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

/// 글자처럼 취급되는 인라인 제어문의 문단 내 위치와 물리 크기.
///
/// HWP `PARA_TEXT`에는 수식·그림 본문이 보이지 않는 8 UTF-16 단위 제어문자로
/// 들어가므로, visible text만 토큰화하면 제어문이 차지한 폭이 사라진다. 재조판은
/// 그 폭과 높이를 별도로 들고 줄나눔과 line box에 반영해야 한다 (#3211).
#[derive(Debug, Clone, Copy)]
struct FlowInlineControl {
    char_position: usize,
    width_hwp: i32,
    height_hwp: i32,
    /// Equation supplies an object-owned baseline for the physical row. Other
    /// inline objects keep the text metrics already selected by the caller.
    baseline_distance_hwp: Option<i32>,
}

/// 줄 채움 결과
#[derive(Debug, Clone, PartialEq)]
struct LineBreakResult {
    start_idx: usize,
    end_idx: usize, // exclusive
    max_font_size: f64,
    has_line_break: bool, // 강제 줄 바꿈 여부
}

/// Why one carved interval stopped receiving text.
///
/// A false `has_line_break` alone is ambiguous: a segment can stop because it
/// reached its interval, or because it finished the paragraph. Frame layout
/// needs the distinction to decide whether the next horizontal interval is
/// part of the same physical row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FillTermination {
    IntervalFull,
    ForcedBreak,
    ParagraphEnd,
}

#[derive(Debug, Clone, PartialEq)]
struct FilledInterval {
    line: LineBreakResult,
    termination: FillTermination,
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

/// 문단 텍스트를 줄 나눔 토큰으로 분할한다.
pub(crate) fn tokenize_paragraph(
    text_chars: &[char],
    char_offsets: &[u32],
    char_shapes: &[CharShapeRef],
    styles: &ResolvedStyleSet,
    english_break_unit: u8,
    korean_break_unit: u8,
) -> Vec<BreakToken> {
    tokenize_paragraph_with_regenerated_space_metric(
        text_chars,
        char_offsets,
        char_shapes,
        styles,
        english_break_unit,
        korean_break_unit,
        false,
        &[],
    )
}

/// `regenerated_line_space_metric`은 한컴이 폭 변경 뒤 다시 저장하는 공백 규칙을
/// 쓴다. 일반 HWP/HWPX tokenization은 저장 LINE_SEG 호환성을 위해 끈다.
fn tokenize_paragraph_with_regenerated_space_metric(
    text_chars: &[char],
    char_offsets: &[u32],
    char_shapes: &[CharShapeRef],
    styles: &ResolvedStyleSet,
    english_break_unit: u8,
    korean_break_unit: u8,
    regenerated_line_space_metric: bool,
    inline_controls: &[FlowInlineControl],
) -> Vec<BreakToken> {
    let text_len = text_chars.len();
    if text_len == 0 {
        return Vec::new();
    }

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
            let w = if regenerated_line_space_metric {
                hancom_regenerated_space_width(&ts)
                    .unwrap_or_else(|| estimate_text_width_unrounded(" ", &ts))
            } else {
                estimate_text_width_unrounded(" ", &ts)
            } + inline_width_px_at(inline_controls, i);
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
                    let width = measure_token_width(
                        &token_text,
                        start,
                        char_offsets,
                        char_shapes,
                        styles,
                        current_lang,
                        inline_controls,
                    );
                    let char_widths = if has_inline_control_in_range(inline_controls, start, i) {
                        (start..i)
                            .map(|ci| {
                                measure_char_width(
                                    text_chars[ci],
                                    ci,
                                    char_offsets,
                                    char_shapes,
                                    styles,
                                    current_lang,
                                    inline_controls,
                                )
                            })
                            .collect()
                    } else {
                        Vec::new()
                    };
                    tokens.push(BreakToken::Text {
                        start_idx: start,
                        end_idx: i,
                        width,
                        max_font_size: max_fs,
                        char_widths,
                    });
                }
                continue;
            } else {
                // 글자 모드: 한글 개별 분할
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
                let w = estimate_text_width_unrounded(&ch.to_string(), &ts)
                    + inline_width_px_at(inline_controls, i);
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
                    let width = measure_token_width(
                        &token_text,
                        start,
                        char_offsets,
                        char_shapes,
                        styles,
                        current_lang,
                        inline_controls,
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
                            estimate_text_width_unrounded(&c.to_string(), &ts)
                                + inline_width_px_at(inline_controls, ci)
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
                let w = estimate_text_width_unrounded(&ch.to_string(), &ts)
                    + inline_width_px_at(inline_controls, i);
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
            let w = estimate_text_width_unrounded(&ch.to_string(), &ts)
                + inline_width_px_at(inline_controls, i);
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
            let w = estimate_text_width_unrounded(&ch.to_string(), &ts)
                + inline_width_px_at(inline_controls, i);
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

/// 토큰 텍스트의 폭을 글자별 언어 인식 측정으로 합산한다.
fn measure_token_width(
    text: &str,
    start_char_idx: usize,
    char_offsets: &[u32],
    char_shapes: &[CharShapeRef],
    styles: &ResolvedStyleSet,
    default_lang: usize,
    inline_controls: &[FlowInlineControl],
) -> f64 {
    let mut total = 0.0;
    let mut current_lang = default_lang;
    for (offset, ch) in text.chars().enumerate() {
        let idx = start_char_idx + offset;
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
        total += estimate_text_width_unrounded(&ch.to_string(), &ts)
            + inline_width_px_at(inline_controls, idx);
    }
    total
}

fn measure_char_width(
    ch: char,
    char_idx: usize,
    char_offsets: &[u32],
    char_shapes: &[CharShapeRef],
    styles: &ResolvedStyleSet,
    default_lang: usize,
    inline_controls: &[FlowInlineControl],
) -> f64 {
    let utf16_pos = char_offsets
        .get(char_idx)
        .copied()
        .unwrap_or(char_idx as u32);
    let style_id = find_active_char_shape(char_shapes, utf16_pos);
    let lang = if is_lang_neutral(ch) {
        default_lang
    } else {
        detect_lang_category(ch)
    };
    let style = resolved_to_text_style(styles, style_id, lang);
    estimate_text_width_unrounded(&ch.to_string(), &style)
        + inline_width_px_at(inline_controls, char_idx)
}

fn inline_width_px_at(inline_controls: &[FlowInlineControl], char_idx: usize) -> f64 {
    inline_controls
        .iter()
        .filter(|control| control.char_position == char_idx)
        .map(|control| control.width_hwp as f64 / 75.0)
        .sum()
}

fn has_inline_control_in_range(
    inline_controls: &[FlowInlineControl],
    start: usize,
    end: usize,
) -> bool {
    inline_controls
        .iter()
        .any(|control| (start..end).contains(&control.char_position))
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

// 한컴은 HWPUNIT 정수 양자화 시 미세한 반올림 차이를 허용한다.
// 15 HU 이내의 초과는 줄에 포함한다.
const LINE_BREAK_TOLERANCE: i32 = 15;

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

fn text_token_fits_line_hwp(
    current_width_hwp: i32,
    token_width_hwp: i32,
    space_savings_hwp: i32,
    effective_width_hwp: i32,
    max_font_size: f64,
) -> bool {
    let natural_candidate = current_width_hwp + token_width_hwp;
    let condensed_candidate = condensed_line_width_hwp(natural_candidate, space_savings_hwp);
    let needs_condense_to_fit = natural_candidate > effective_width_hwp + LINE_BREAK_TOLERANCE
        && condensed_candidate <= effective_width_hwp + LINE_BREAK_TOLERANCE;
    let condense_pull_allowed = !needs_condense_to_fit
        || condense_fit_can_pull_next_token(
            current_width_hwp,
            space_savings_hwp,
            effective_width_hwp,
            max_font_size,
        );

    condensed_candidate <= effective_width_hwp + LINE_BREAK_TOLERANCE && condense_pull_allowed
}

/// Greedy line-fill continuation.
///
/// A visible-text boundary does not always identify the next token: a long
/// `Text` token can continue after an emitted row. Keep the complete state so
/// callers can fill one interval at a time without rediscovering a boundary.
#[derive(Debug, Clone)]
struct FillCursor {
    token_index: usize,
    fallback_char_idx: Option<usize>,
    initial_start_idx: usize,
    line_start_idx: usize,
    lw: i32,
    line_space_savings: i32,
    line_max_fs: f64,
    is_first_line: bool,
    last_break_token_idx: Option<usize>,
    last_break_char_idx: usize,
    width_at_last_break: i32,
    space_savings_at_last_break: i32,
    fs_at_last_break: f64,
    finished: bool,
    emitted_any: bool,
}

impl FillCursor {
    fn new(initial_start_idx: usize, initial_is_first_line: bool) -> Self {
        Self {
            token_index: 0,
            fallback_char_idx: None,
            initial_start_idx,
            line_start_idx: initial_start_idx,
            lw: 0,
            line_space_savings: 0,
            line_max_fs: 0.0,
            is_first_line: initial_is_first_line,
            last_break_token_idx: None,
            last_break_char_idx: 0,
            width_at_last_break: 0,
            space_savings_at_last_break: 0,
            fs_at_last_break: 0.0,
            finished: false,
            emitted_any: false,
        }
    }
}

/// Fill all scalar intervals through the resumable greedy continuation.
fn fill_lines(
    tokens: &[BreakToken],
    text_chars: &[char],
    available_width_px: f64,
    indent_px: f64,
    default_tab_width: f64,
    korean_break_unit: u8,
    condense_min_space: u8,
    initial_start_idx: usize,
    initial_is_first_line: bool,
) -> Vec<LineBreakResult> {
    let mut cursor = FillCursor::new(initial_start_idx, initial_is_first_line);
    let mut results = Vec::new();

    while let Some(interval) = fill_one_interval(
        tokens,
        text_chars,
        available_width_px,
        indent_px,
        default_tab_width,
        korean_break_unit,
        condense_min_space,
        &mut cursor,
    ) {
        results.push(interval.line);
    }

    results
}

/// Fill at most one logical row and retain the greedy continuation state.
fn fill_one_interval(
    tokens: &[BreakToken],
    text_chars: &[char],
    available_width_px: f64,
    indent_px: f64,
    default_tab_width: f64,
    korean_break_unit: u8,
    condense_min_space: u8,
    cursor: &mut FillCursor,
) -> Option<FilledInterval> {
    if cursor.finished {
        return None;
    }

    if tokens.is_empty() {
        cursor.finished = true;
        cursor.emitted_any = true;
        return Some(FilledInterval {
            line: LineBreakResult {
                start_idx: cursor.initial_start_idx,
                end_idx: cursor.initial_start_idx,
                max_font_size: 0.0,
                has_line_break: false,
            },
            termination: FillTermination::ParagraphEnd,
        });
    }

    let tab_w_px = if default_tab_width > 0.0 {
        default_tab_width
    } else {
        48.0
    };
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

    loop {
        if cursor.token_index >= tokens.len() {
            cursor.finished = true;
            let last_end = tokens
                .last()
                .map(|token| match token {
                    BreakToken::Text { end_idx, .. } => *end_idx,
                    BreakToken::Space { idx, .. }
                    | BreakToken::Tab { idx, .. }
                    | BreakToken::LineBreak { idx } => *idx + 1,
                })
                .unwrap_or(text_chars.len());

            if cursor.line_start_idx <= last_end {
                cursor.emitted_any = true;
                return Some(FilledInterval {
                    line: LineBreakResult {
                        start_idx: cursor.line_start_idx,
                        end_idx: last_end,
                        max_font_size: cursor.line_max_fs,
                        has_line_break: false,
                    },
                    termination: FillTermination::ParagraphEnd,
                });
            }

            if !cursor.emitted_any {
                cursor.emitted_any = true;
                return Some(FilledInterval {
                    line: LineBreakResult {
                        start_idx: cursor.initial_start_idx,
                        end_idx: text_chars.len(),
                        max_font_size: 0.0,
                        has_line_break: false,
                    },
                    termination: FillTermination::ParagraphEnd,
                });
            }
            return None;
        }

        let ti = cursor.token_index;
        match &tokens[ti] {
            BreakToken::LineBreak { idx } => {
                let result = LineBreakResult {
                    start_idx: cursor.line_start_idx,
                    end_idx: *idx + 1,
                    max_font_size: cursor.line_max_fs,
                    has_line_break: true,
                };
                cursor.line_start_idx = *idx + 1;
                cursor.lw = 0;
                cursor.line_space_savings = 0;
                cursor.line_max_fs = 0.0;
                cursor.is_first_line = false;
                cursor.last_break_token_idx = None;
                cursor.token_index += 1;
                cursor.emitted_any = true;
                return Some(FilledInterval {
                    line: result,
                    termination: FillTermination::ForcedBreak,
                });
            }
            BreakToken::Tab { idx, max_font_size } => {
                // 탭 계산은 px로 수행 후 HWPUNIT 변환 (정밀도 유지)
                let lw_px = cursor.lw as f64 / 75.0;
                let next_tab_px = ((lw_px / tab_w_px).floor() + 1.0) * tab_w_px;
                let next_tab_hwp = to_hwp(next_tab_px);
                if *max_font_size > cursor.line_max_fs {
                    cursor.line_max_fs = *max_font_size;
                }

                if next_tab_hwp > eff_w(cursor.is_first_line) && cursor.line_start_idx < *idx {
                    let result = if cursor.last_break_token_idx.is_some() {
                        let result = LineBreakResult {
                            start_idx: cursor.line_start_idx,
                            end_idx: cursor.last_break_char_idx,
                            max_font_size: cursor.fs_at_last_break,
                            has_line_break: false,
                        };
                        cursor.line_start_idx = cursor.last_break_char_idx;
                        cursor.lw -= cursor.width_at_last_break;
                        cursor.line_space_savings -= cursor.space_savings_at_last_break;
                        result
                    } else {
                        let result = LineBreakResult {
                            start_idx: cursor.line_start_idx,
                            end_idx: *idx,
                            max_font_size: cursor.line_max_fs,
                            has_line_break: false,
                        };
                        cursor.line_start_idx = *idx;
                        cursor.lw = 0;
                        cursor.line_space_savings = 0;
                        cursor.line_max_fs = *max_font_size;
                        result
                    };
                    cursor.is_first_line = false;
                    cursor.last_break_token_idx = None;
                    let lw_px2 = cursor.lw as f64 / 75.0;
                    let next_tab2 = ((lw_px2 / tab_w_px).floor() + 1.0) * tab_w_px;
                    cursor.lw = to_hwp(next_tab2);
                    cursor.token_index += 1;
                    cursor.emitted_any = true;
                    return Some(FilledInterval {
                        line: result,
                        termination: FillTermination::IntervalFull,
                    });
                }

                cursor.last_break_token_idx = Some(ti);
                cursor.last_break_char_idx = *idx;
                cursor.width_at_last_break = cursor.lw;
                cursor.space_savings_at_last_break = cursor.line_space_savings;
                cursor.fs_at_last_break = cursor.line_max_fs;
                cursor.lw = next_tab_hwp;
                cursor.token_index += 1;
            }
            BreakToken::Space {
                idx,
                width,
                max_font_size,
            } => {
                if *max_font_size > cursor.line_max_fs {
                    cursor.line_max_fs = *max_font_size;
                }
                cursor.last_break_token_idx = Some(ti);
                cursor.last_break_char_idx = *idx;
                cursor.width_at_last_break = cursor.lw;
                cursor.space_savings_at_last_break = cursor.line_space_savings;
                cursor.fs_at_last_break = cursor.line_max_fs;
                let space_hwp = to_hwp(*width);
                cursor.lw += space_hwp;
                cursor.line_space_savings +=
                    condense_space_savings_hwp(space_hwp, condense_min_space);
                cursor.token_index += 1;
            }
            BreakToken::Text {
                start_idx,
                end_idx,
                width,
                max_font_size,
                char_widths,
            } => {
                if let Some(next_char_idx) = cursor.fallback_char_idx {
                    debug_assert!(*start_idx <= next_char_idx && next_char_idx <= *end_idx);
                    let mut ci = next_char_idx;
                    while ci < *end_idx {
                        let rel_idx = ci - *start_idx;
                        let char_w = char_widths
                            .get(rel_idx)
                            .map(|width| to_hwp(*width))
                            .unwrap_or_else(|| {
                                let ch = text_chars[ci];
                                let char_w_px = if is_cjk_char(ch) {
                                    cursor.line_max_fs.max(12.0)
                                } else {
                                    cursor.line_max_fs.max(12.0) * 0.5
                                };
                                to_hwp(char_w_px)
                            });
                        let current_width = eff_w(cursor.is_first_line);
                        if cursor.lw + char_w > current_width && ci > cursor.line_start_idx {
                            let result = LineBreakResult {
                                start_idx: cursor.line_start_idx,
                                end_idx: ci,
                                max_font_size: cursor.line_max_fs,
                                has_line_break: false,
                            };
                            cursor.line_start_idx = ci;
                            cursor.lw = char_w;
                            cursor.is_first_line = false;
                            cursor.fallback_char_idx = Some(ci + 1);
                            cursor.emitted_any = true;
                            return Some(FilledInterval {
                                line: result,
                                termination: FillTermination::IntervalFull,
                            });
                        }
                        cursor.lw += char_w;
                        ci += 1;
                    }
                    cursor.fallback_char_idx = None;
                    cursor.token_index += 1;
                    continue;
                }

                if *max_font_size > cursor.line_max_fs {
                    cursor.line_max_fs = *max_font_size;
                }

                let w_hwp = to_hwp(*width);

                // 단일 문자 CJK/한글 토큰의 줄바꿈 가능 지점 처리
                // 이 글자를 포함한 후 break point 갱신 (end_idx 사용)
                // → 초과 시 이 글자까지 L0에 포함하고 다음 토큰부터 다음 줄
                if *end_idx - *start_idx == 1 && *start_idx > cursor.line_start_idx {
                    let c = text_chars[*start_idx];
                    let allow_break = if is_hangul(c) {
                        // [#2185] bit7=1 = 글자 단위 break 허용 (위 주석 참조)
                        korean_break_unit == 1
                    } else {
                        is_cjk_ideograph(c)
                    };
                    let candidate_w = cursor.lw + w_hwp;
                    // 이 글자가 줄에 들어가는 경우에만 break point 갱신
                    if allow_break
                        && condensed_line_width_hwp(candidate_w, cursor.line_space_savings)
                            <= eff_w(cursor.is_first_line) + LINE_BREAK_TOLERANCE
                    {
                        cursor.last_break_token_idx = Some(ti);
                        cursor.last_break_char_idx = *end_idx; // 이 글자 다음 (이 글자 포함)
                        cursor.width_at_last_break = candidate_w; // 이 글자 폭 포함
                        cursor.space_savings_at_last_break = cursor.line_space_savings;
                        cursor.fs_at_last_break = cursor.line_max_fs;
                    }
                }
                let effective_width = eff_w(cursor.is_first_line);
                if !text_token_fits_line_hwp(
                    cursor.lw,
                    w_hwp,
                    cursor.line_space_savings,
                    effective_width,
                    *max_font_size,
                ) {
                    if *start_idx > cursor.line_start_idx {
                        if let Some(break_token_idx) = cursor.last_break_token_idx {
                            let result = LineBreakResult {
                                start_idx: cursor.line_start_idx,
                                end_idx: cursor.last_break_char_idx,
                                max_font_size: cursor.fs_at_last_break,
                                has_line_break: false,
                            };
                            let mut next_start = cursor.last_break_char_idx;
                            while next_start < text_chars.len() && text_chars[next_start] == ' ' {
                                next_start += 1;
                            }
                            cursor.line_start_idx = next_start;
                            cursor.lw = recalc_width_hwp(tokens, ti, next_start);
                            cursor.line_space_savings = recalc_space_savings_hwp(
                                tokens,
                                ti,
                                next_start,
                                condense_min_space,
                            );
                            cursor.line_max_fs = *max_font_size;
                            cursor.is_first_line = false;
                            cursor.last_break_token_idx = None;

                            // 현재 단일 CJK/한글 토큰 자체가 break point였던 기존 경로는
                            // 이미 위 결과에 포함됐으므로 동작을 바꾸지 않는다.
                            if break_token_idx == ti {
                                cursor.lw += w_hwp;
                                cursor.token_index += 1;
                                cursor.emitted_any = true;
                                return Some(FilledInterval {
                                    line: result,
                                    termination: FillTermination::IntervalFull,
                                });
                            }

                            // [#3822] 이전 break 뒤로 옮긴 현재 토큰이 새 줄에도
                            // 들어가는지 다시 확인한다. 종전에는 토큰 전체 폭을 무조건
                            // 더하고 continue하여, 긴 영문·숫자 토큰의 글자 단위 fallback을
                            // 건너뛰었다.
                            if text_token_fits_line_hwp(
                                cursor.lw,
                                w_hwp,
                                cursor.line_space_savings,
                                eff_w(false),
                                *max_font_size,
                            ) {
                                cursor.lw += w_hwp;
                                cursor.token_index += 1;
                                cursor.emitted_any = true;
                                return Some(FilledInterval {
                                    line: result,
                                    termination: FillTermination::IntervalFull,
                                });
                            }

                            cursor.line_space_savings = 0;
                            cursor.last_break_token_idx = None;
                            cursor.fallback_char_idx = Some(*start_idx);
                            cursor.emitted_any = true;
                            return Some(FilledInterval {
                                line: result,
                                termination: FillTermination::IntervalFull,
                            });
                        }
                    }

                    // 토큰에 저장된 개별 글자 폭을 HWPUNIT로 변환
                    cursor.line_space_savings = 0;
                    cursor.last_break_token_idx = None;
                    cursor.fallback_char_idx = Some(*start_idx);
                    continue;
                }

                cursor.lw += w_hwp;
                cursor.token_index += 1;
            }
        }
    }
}

/// Frozen scalar implementation used only to prove cursor equivalence.
#[cfg(test)]
fn fill_lines_before_cursor(
    tokens: &[BreakToken],
    text_chars: &[char],
    available_width_px: f64,
    indent_px: f64,
    default_tab_width: f64,
    korean_break_unit: u8,
    condense_min_space: u8,
    initial_start_idx: usize,
    initial_is_first_line: bool,
) -> Vec<LineBreakResult> {
    if tokens.is_empty() {
        return vec![LineBreakResult {
            start_idx: initial_start_idx,
            end_idx: initial_start_idx,
            max_font_size: 0.0,
            has_line_break: false,
        }];
    }

    let tab_w_hwp = to_hwp(if default_tab_width > 0.0 {
        default_tab_width
    } else {
        48.0
    });
    let tab_w_px = if default_tab_width > 0.0 {
        default_tab_width
    } else {
        48.0
    };
    let mut results = Vec::new();
    let mut line_start_idx = initial_start_idx;
    let mut lw = 0i32; // HWPUNIT 정수 누적
    let mut line_space_savings = 0i32;
    let mut line_max_fs = 0.0f64;
    let mut is_first_line = initial_is_first_line;

    let mut last_break_token_idx: Option<usize> = None;
    let mut last_break_char_idx: usize = 0;
    let mut width_at_last_break = 0i32;
    let mut space_savings_at_last_break = 0i32;
    let mut fs_at_last_break = 0.0f64;

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

    for (ti, token) in tokens.iter().enumerate() {
        match token {
            BreakToken::LineBreak { idx } => {
                results.push(LineBreakResult {
                    start_idx: line_start_idx,
                    end_idx: *idx + 1,
                    max_font_size: line_max_fs,
                    has_line_break: true,
                });
                line_start_idx = *idx + 1;
                lw = 0;
                line_space_savings = 0;
                line_max_fs = 0.0;
                is_first_line = false;
                last_break_token_idx = None;
            }
            BreakToken::Tab { idx, max_font_size } => {
                // 탭 계산은 px로 수행 후 HWPUNIT 변환 (정밀도 유지)
                let lw_px = lw as f64 / 75.0;
                let next_tab_px = ((lw_px / tab_w_px).floor() + 1.0) * tab_w_px;
                let next_tab_hwp = to_hwp(next_tab_px);
                if *max_font_size > line_max_fs {
                    line_max_fs = *max_font_size;
                }

                if next_tab_hwp > eff_w(is_first_line) && line_start_idx < *idx {
                    if let Some(_) = last_break_token_idx {
                        results.push(LineBreakResult {
                            start_idx: line_start_idx,
                            end_idx: last_break_char_idx,
                            max_font_size: fs_at_last_break,
                            has_line_break: false,
                        });
                        line_start_idx = last_break_char_idx;
                        lw = lw - width_at_last_break;
                        line_space_savings -= space_savings_at_last_break;
                    } else {
                        results.push(LineBreakResult {
                            start_idx: line_start_idx,
                            end_idx: *idx,
                            max_font_size: line_max_fs,
                            has_line_break: false,
                        });
                        line_start_idx = *idx;
                        lw = 0;
                        line_space_savings = 0;
                        line_max_fs = *max_font_size;
                    }
                    is_first_line = false;
                    last_break_token_idx = None;
                    let lw_px2 = lw as f64 / 75.0;
                    let next_tab2 = ((lw_px2 / tab_w_px).floor() + 1.0) * tab_w_px;
                    lw = to_hwp(next_tab2);
                } else {
                    last_break_token_idx = Some(ti);
                    last_break_char_idx = *idx;
                    width_at_last_break = lw;
                    space_savings_at_last_break = line_space_savings;
                    fs_at_last_break = line_max_fs;
                    lw = next_tab_hwp;
                }
            }
            BreakToken::Space {
                idx,
                width,
                max_font_size,
            } => {
                if *max_font_size > line_max_fs {
                    line_max_fs = *max_font_size;
                }
                last_break_token_idx = Some(ti);
                last_break_char_idx = *idx;
                width_at_last_break = lw;
                space_savings_at_last_break = line_space_savings;
                fs_at_last_break = line_max_fs;
                let space_hwp = to_hwp(*width);
                lw += space_hwp;
                line_space_savings += condense_space_savings_hwp(space_hwp, condense_min_space);
            }
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

                let w_hwp = to_hwp(*width);

                // 단일 문자 CJK/한글 토큰의 줄바꿈 가능 지점 처리
                // 이 글자를 포함한 후 break point 갱신 (end_idx 사용)
                // → 초과 시 이 글자까지 L0에 포함하고 다음 토큰부터 다음 줄
                if *end_idx - *start_idx == 1 && *start_idx > line_start_idx {
                    let c = text_chars[*start_idx];
                    let allow_break = if is_hangul(c) {
                        // [#2185] bit7=1 = 글자 단위 break 허용 (위 주석 참조)
                        korean_break_unit == 1
                    } else {
                        is_cjk_ideograph(c)
                    };
                    let candidate_w = lw + w_hwp;
                    // 이 글자가 줄에 들어가는 경우에만 break point 갱신
                    if allow_break
                        && condensed_line_width_hwp(candidate_w, line_space_savings)
                            <= eff_w(is_first_line) + LINE_BREAK_TOLERANCE
                    {
                        last_break_token_idx = Some(ti);
                        last_break_char_idx = *end_idx; // 이 글자 다음 (이 글자 포함)
                        width_at_last_break = candidate_w; // 이 글자 폭 포함
                        space_savings_at_last_break = line_space_savings;
                        fs_at_last_break = line_max_fs;
                    }
                }
                let effective_width = eff_w(is_first_line);
                if !text_token_fits_line_hwp(
                    lw,
                    w_hwp,
                    line_space_savings,
                    effective_width,
                    *max_font_size,
                ) {
                    if *start_idx > line_start_idx {
                        if let Some(break_token_idx) = last_break_token_idx {
                            results.push(LineBreakResult {
                                start_idx: line_start_idx,
                                end_idx: last_break_char_idx,
                                max_font_size: fs_at_last_break,
                                has_line_break: false,
                            });
                            let mut next_start = last_break_char_idx;
                            while next_start < text_chars.len() && text_chars[next_start] == ' ' {
                                next_start += 1;
                            }
                            line_start_idx = next_start;
                            lw = recalc_width_hwp(tokens, ti, next_start);
                            line_space_savings = recalc_space_savings_hwp(
                                tokens,
                                ti,
                                next_start,
                                condense_min_space,
                            );
                            line_max_fs = *max_font_size;
                            is_first_line = false;
                            last_break_token_idx = None;

                            // 현재 단일 CJK/한글 토큰 자체가 break point였던 기존 경로는
                            // 이미 위 결과에 포함됐으므로 동작을 바꾸지 않는다.
                            if break_token_idx == ti {
                                lw += w_hwp;
                                continue;
                            }

                            // [#3822] 이전 break 뒤로 옮긴 현재 토큰이 새 줄에도
                            // 들어가는지 다시 확인한다. 종전에는 토큰 전체 폭을 무조건
                            // 더하고 continue하여, 긴 영문·숫자 토큰의 글자 단위 fallback을
                            // 건너뛰었다.
                            if text_token_fits_line_hwp(
                                lw,
                                w_hwp,
                                line_space_savings,
                                eff_w(false),
                                *max_font_size,
                            ) {
                                lw += w_hwp;
                                continue;
                            }
                        }
                    }
                    // 토큰에 저장된 개별 글자 폭을 HWPUNIT로 변환
                    let cw_hwp: Vec<i32> = char_widths.iter().map(|w| to_hwp(*w)).collect();
                    let (results_part, remaining_w, remaining_fs) = char_level_break_hwp(
                        text_chars,
                        *start_idx,
                        *end_idx,
                        &mut line_start_idx,
                        lw,
                        line_max_fs,
                        eff_w(is_first_line),
                        eff_w(false),
                        is_first_line,
                        &cw_hwp,
                    );
                    for r in results_part {
                        results.push(r);
                        is_first_line = false;
                    }
                    lw = remaining_w;
                    line_space_savings = 0;
                    line_max_fs = remaining_fs;
                    last_break_token_idx = None;
                    continue;
                } else {
                    lw += w_hwp;
                }
            }
        }
    }

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
        results.push(LineBreakResult {
            start_idx: line_start_idx,
            end_idx: last_end,
            max_font_size: line_max_fs,
            has_line_break: false,
        });
    }

    if results.is_empty() {
        results.push(LineBreakResult {
            start_idx: initial_start_idx,
            end_idx: text_chars.len(),
            max_font_size: 0.0,
            has_line_break: false,
        });
    }

    results
}

/// 줄 바꿈 지점 이후 토큰의 누적 폭 재계산 (HWPUNIT)
fn recalc_width_hwp(tokens: &[BreakToken], current_token_idx: usize, new_line_start: usize) -> i32 {
    let mut w = 0i32;
    for t in &tokens[..current_token_idx] {
        match t {
            BreakToken::Text {
                start_idx, width, ..
            } if *start_idx >= new_line_start => {
                w += to_hwp(*width);
            }
            BreakToken::Space { idx, width, .. } if *idx >= new_line_start => {
                w += to_hwp(*width);
            }
            _ => {}
        }
    }
    w
}

/// 줄 바꿈 지점 이후 공백 압축 가능 폭 재계산 (HWPUNIT)
fn recalc_space_savings_hwp(
    tokens: &[BreakToken],
    current_token_idx: usize,
    new_line_start: usize,
    condense_min_space: u8,
) -> i32 {
    let mut w = 0i32;
    for t in &tokens[..current_token_idx] {
        match t {
            BreakToken::Space {
                idx,
                width,
                max_font_size,
            } if *idx >= new_line_start => {
                let space_hwp = to_hwp(*width);
                w += condense_space_savings_hwp(space_hwp, condense_min_space);
            }
            _ => {}
        }
    }
    w
}

/// 긴 단어 폴백: 글자 단위 분할 (HWPUNIT)
/// char_widths_hwp: 토큰 내 각 글자의 HWPUNIT 폭 (None이면 휴리스틱)
#[cfg(test)]
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

fn inline_control_line_height_hwp(para: &Paragraph) -> Option<i32> {
    para.controls
        .iter()
        .filter_map(|ctrl| match ctrl {
            Control::Picture(pic) if pic.common.treat_as_char => Some(pic.common.height as i32),
            Control::Shape(shape) if shape.common().treat_as_char => Some(shape.flow_height_hu()),
            Control::Table(table) if table.common.treat_as_char => Some(table.common.height as i32),
            Control::Equation(eq) if eq.common.treat_as_char => Some(eq.common.height as i32),
            Control::Form(form) => Some(form.height as i32),
            _ => None,
        })
        .filter(|height| *height > 0)
        .max()
}

fn inline_control_size_hwp(ctrl: &Control) -> Option<(i32, i32)> {
    let (width, height) = match ctrl {
        Control::Picture(pic) if pic.common.treat_as_char => {
            (pic.common.width as i32, pic.common.height as i32)
        }
        Control::Shape(shape) if shape.common().treat_as_char => (
            (shape.common().width as i32).max(shape.shape_attr().current_width as i32),
            shape.flow_height_hu(),
        ),
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

fn flow_inline_controls(para: &Paragraph) -> Vec<FlowInlineControl> {
    let text_len = para.text.chars().count();
    para.controls
        .iter()
        .zip(para.control_text_positions())
        .filter_map(|(control, char_position)| {
            // 글자처럼 취급되는 표는 renderer가 control 위치를 기준으로 별도
            // TextRun/Table 경계를 만든다. 보이지 않는 PARA_TEXT 위치의 다음
            // 글자 폭에 표 전체 폭을 더하면 HML의 `abc + table + efg`처럼
            // 기존 경계를 잃는다. #3211의 HWP oracle은 수식·그림 계열의
            // 재조판 폭을 대상으로 하므로 표는 기존 control 배치 경로에 둔다.
            if matches!(control, Control::Table(_)) {
                return None;
            }
            let (width_hwp, height_hwp) = inline_control_size_hwp(control)?;
            let baseline_distance_hwp = match control {
                Control::Equation(equation) if equation.baseline > 0 => Some(
                    height_hwp
                        .saturating_mul(i32::from(equation.baseline))
                        .saturating_div(100),
                ),
                _ => None,
            };
            (char_position < text_len).then_some(FlowInlineControl {
                char_position,
                width_hwp,
                height_hwp,
                baseline_distance_hwp,
            })
        })
        .collect()
}

/// The picture-band frame intentionally admits only its floating host and the
/// already-supported treat-as-character Equation flow. Other controls have
/// their own layout owners and must leave this transaction untouched.
fn supports_picture_band_frame_controls(para: &Paragraph) -> bool {
    let mut non_tac_pictures = 0usize;
    for control in &para.controls {
        match control {
            Control::Picture(picture) if !picture.common.treat_as_char => {
                non_tac_pictures += 1;
            }
            Control::Equation(equation) if equation.common.treat_as_char => {}
            _ => return false,
        }
    }
    non_tac_pictures <= 1
}

/// 본문 뒤 남은 폭에 놓이지 않는 inline control은 별도 physical line을 가진다.
///
/// 종전 cell reflow는 text token만 줄바꿈한 뒤 첫 LineSeg를 표 높이만큼 키웠다.
/// 그 결과 분할로 폭이 좁아진 셀에서 `text + inline object`가 한 줄로 합쳐졌다.
/// 한컴은 control 위치부터 object 전용 LineSeg를 만들어 다음 physical line으로 보낸다
/// (#4138: 1×2 split 뒤 nested table/picture host). control 자체가 셀 폭을 넘거나,
/// control 앞의 실제 text 폭과 합쳐 현재 줄의 폭을 넘는 경우만 대상으로 한다.
/// 같은 줄에 들어가는 작은 object와 복수 control 문단의 기존 reflow는 건드리지 않는다.
fn inline_control_requires_own_line(
    para: &Paragraph,
    text_chars: &[char],
    line_breaks: &[LineBreakResult],
    available_width_px: f64,
    indent_px: f64,
    reflow_is_first_line: bool,
    styles: &ResolvedStyleSet,
) -> Option<(usize, i32)> {
    let text_len = para.text.chars().count();
    let positions = para.control_text_positions();
    let mut candidates = para
        .controls
        .iter()
        .zip(positions)
        .filter_map(|(control, position)| {
            let (width, height) = inline_control_size_hwp(control)?;
            (position > 0 && position <= text_len).then_some((position, width, height))
        });
    let (position, control_width, height) = candidates.next()?;
    // 여러 inline control은 일반 placement가 순서를 보존해야 하므로 이 좁은
    // single-control 계약 밖이다.
    if candidates.next().is_some() {
        return None;
    }

    // 같은 text offset에서 새 줄이 시작되면 control은 그 줄의 선두에 놓인다.
    // 그렇지 않으면 control 직전 text가 실제로 속한 줄을 선택한다. 마지막 글자
    // 뒤의 control은 마지막 text line에 속한다.
    let (line_idx, line) = line_breaks
        .iter()
        .enumerate()
        .find(|(_, line)| line.start_idx == position)
        .or_else(|| {
            line_breaks
                .iter()
                .enumerate()
                .rfind(|(_, line)| line.start_idx < position && position <= line.end_idx)
        })?;
    let is_first_line = reflow_is_first_line && line_idx == 0;
    let available_hwp = if indent_px > 0.0 && is_first_line {
        to_hwp((available_width_px - indent_px).max(1.0))
    } else if indent_px < 0.0 && !is_first_line {
        to_hwp((available_width_px + indent_px).max(1.0))
    } else {
        to_hwp(available_width_px)
    };
    let prefix: String = text_chars[line.start_idx..position].iter().collect();
    let prefix_width = to_hwp(measure_token_width(
        &prefix,
        line.start_idx,
        &para.char_offsets,
        &para.char_shapes,
        styles,
        0,
        &[],
    ));

    (control_width > available_hwp + LINE_BREAK_TOLERANCE
        || prefix_width + control_width > available_hwp + LINE_BREAK_TOLERANCE)
        .then_some((position, height))
}

fn char_index_to_utf16_offset(para: &Paragraph, char_index: usize) -> u32 {
    if let Some(offset) = para.char_offsets.get(char_index) {
        return *offset;
    }

    // char_offsets에는 visible text 앞의 control stream gap도 반영된다. 따라서
    // 끝의 빈 physical line(예: trailing Shift+Enter)을 단순 text 길이로 매핑하면
    // SectionDef/ColumnDef가 앞선 문단에서 21이어야 할 start가 5로 되돌아간다.
    // 마지막 visible char의 실제 stream offset을 기준으로 종단을 계산한다.
    para.char_offsets
        .last()
        .zip(para.text.chars().last())
        .map(|(offset, ch)| *offset + ch.len_utf16() as u32)
        .unwrap_or_else(|| {
            // 합성 문단처럼 char_offsets가 비어 있으면 char_index(Unicode scalar
            // index)를 UTF-16 code-unit 위치로 직접 환산한다. 단순 `as u32`는
            // 보충 평면 문자를 1 unit으로 세어 후행 줄의 start를 당긴다.
            para.text
                .chars()
                .take(char_index)
                .map(|ch| ch.len_utf16() as u32)
                .sum()
        })
}

fn apply_inline_control_line_height(seg: &mut LineSeg, height_hwp: i32) {
    if height_hwp > seg.line_height {
        seg.line_height = height_hwp;
        seg.text_height = height_hwp;
        seg.baseline_distance = (height_hwp as f64 * 0.85).round() as i32;
    }
}

fn apply_inline_control_frame_height(metrics: &mut FrameRowMetrics, height_hwp: i32) {
    if height_hwp > metrics.line_height {
        metrics.line_height = height_hwp;
        metrics.text_height = height_hwp;
        metrics.baseline_distance = (height_hwp as f64 * 0.85).round() as i32;
    }
}

fn frame_metrics_for_line(
    max_font_size: f64,
    fallback_font_size: f64,
    line_spacing_type: LineSpacingType,
    line_spacing_value: f64,
    dpi: f64,
) -> FrameRowMetrics {
    let font_size = if max_font_size > 0.0 {
        max_font_size
    } else {
        fallback_font_size
    };
    let line_height = font_size_to_line_height(font_size, dpi).max(1);
    FrameRowMetrics {
        vertical_pos: 0,
        line_height,
        text_height: line_height,
        baseline_distance: (line_height as f64 * 0.85) as i32,
        line_spacing: compute_line_spacing_hwp(
            line_spacing_type,
            line_spacing_value,
            line_height,
            dpi,
        ),
    }
}

/// Lay out the small scalar/Picture-band paragraph subset through a
/// caller-owned physical frame.
///
/// Every interval returned by one carve belongs to the same physical row. The
/// cursor continues from left to right and the frame does not advance until
/// that complete row has been committed.
pub(crate) fn layout_paragraph_in_frame(
    para: &Paragraph,
    frame: &mut LayoutFrame,
    styles: &ResolvedStyleSet,
    dpi: f64,
) -> Option<Vec<LineSeg>> {
    if !supports_picture_band_frame_controls(para) {
        return None;
    }

    let text_chars = para.text.chars().collect::<Vec<_>>();
    let para_style = styles.para_styles.get(para.para_shape_id as usize);
    let indent_px = para_style.map(|style| style.indent).unwrap_or(0.0);
    let english_break_unit = para_style
        .map(|style| style.english_break_unit)
        .unwrap_or(0);
    let korean_break_unit = para_style.map(|style| style.korean_break_unit).unwrap_or(0);
    let condense_min_space = para_style
        .map(|style| style.condense_min_space)
        .unwrap_or(0);
    let default_tab_width = para_style
        .map(|style| style.default_tab_width)
        .unwrap_or(0.0);
    let line_spacing_type = para_style
        .map(|style| style.line_spacing_type)
        .unwrap_or(LineSpacingType::Percent);
    let line_spacing_value = para_style.map(|style| style.line_spacing).unwrap_or(160.0);
    // Keep Equation width and height ownership with the current scalar
    // `FlowInlineControl` path. A non-TAC Picture deliberately contributes no
    // inline token: it is represented by the caller's exclusion instead.
    let inline_controls = flow_inline_controls(para);
    let tokens = tokenize_paragraph_with_regenerated_space_metric(
        &text_chars,
        &para.char_offsets,
        &para.char_shapes,
        styles,
        english_break_unit,
        korean_break_unit,
        false,
        &inline_controls,
    );
    let fallback_font_size = if para.text.is_empty() {
        para.char_shapes
            .first()
            .and_then(|char_shape| styles.char_styles.get(char_shape.char_shape_id as usize))
            .map(|style| style.font_size)
            .unwrap_or(12.0)
    } else {
        12.0
    };
    // This matches the scalar path's terminal-control behavior: controls not
    // admitted to `FlowInlineControl` because they sit after the last visible
    // character enlarge the first line box without inventing a second width
    // accounting path.
    let terminal_inline_metrics = inline_controls
        .is_empty()
        .then(|| {
            let height_hwp = inline_control_line_height_hwp(para)?;
            let baseline_distance_hwp = para
                .controls
                .iter()
                .filter_map(|control| match control {
                    Control::Equation(equation)
                        if equation.common.treat_as_char
                            && equation.common.height as i32 == height_hwp
                            && equation.baseline > 0 =>
                    {
                        Some(
                            height_hwp
                                .saturating_mul(i32::from(equation.baseline))
                                .saturating_div(100),
                        )
                    }
                    _ => None,
                })
                .max();
            Some((height_hwp, baseline_distance_hwp))
        })
        .flatten();
    let source_tag = para
        .line_segs
        .first()
        .map(|segment| segment.tag)
        .unwrap_or(LineSeg::TAG_IMPLEMENTATION_PROPERTY | LineSeg::TAG_SINGLE_SEGMENT_LINE);
    let first_row = frame.row_count();
    let frame_checkpoint = frame.clone();
    let mut cursor = FillCursor::new(0, true);

    let result = (|| {
        while !cursor.finished {
            let row_frame_checkpoint = frame.clone();
            let cursor_checkpoint = cursor.clone();
            let mut candidate_height = frame_metrics_for_line(
                fallback_font_size,
                fallback_font_size,
                line_spacing_type,
                line_spacing_value,
                dpi,
            )
            .line_height;
            const MAX_ROW_HEIGHT_TRIALS: usize = 8;
            let mut attempted_trials = Vec::with_capacity(MAX_ROW_HEIGHT_TRIALS);

            loop {
                frame.restore_checkpoint(row_frame_checkpoint.clone());
                cursor = cursor_checkpoint.clone();
                let intervals = frame.carve(candidate_height).to_vec();
                if intervals.is_empty()
                    || intervals
                        .iter()
                        .any(|interval| interval.start >= interval.end)
                {
                    return None;
                }
                let trial = (frame.top, candidate_height, intervals.clone());
                if attempted_trials.contains(&trial)
                    || attempted_trials.len() == MAX_ROW_HEIGHT_TRIALS
                {
                    return None;
                }
                attempted_trials.push(trial);

                let mut segments = Vec::with_capacity(intervals.len());
                let mut maximum_font_size = 0.0f64;
                let mut inline_metrics = (frame.row_count() == first_row)
                    .then_some(terminal_inline_metrics)
                    .flatten();
                let mut row_terminated = false;
                for interval in intervals {
                    let available_width_px = crate::renderer::hwpunit_to_px(
                        interval.end.saturating_sub(interval.start),
                        dpi,
                    );
                    let filled = fill_one_interval(
                        &tokens,
                        &text_chars,
                        available_width_px,
                        indent_px,
                        default_tab_width,
                        korean_break_unit,
                        condense_min_space,
                        &mut cursor,
                    )?;
                    let line = &filled.line;
                    maximum_font_size = maximum_font_size.max(line.max_font_size);
                    for control in inline_controls.iter().filter(|control| {
                        (line.start_idx..line.end_idx).contains(&control.char_position)
                            || (line.end_idx == text_chars.len()
                                && control.char_position == text_chars.len())
                    }) {
                        inline_metrics = match inline_metrics {
                            Some((height_hwp, baseline_distance_hwp))
                                if height_hwp > control.height_hwp =>
                            {
                                Some((height_hwp, baseline_distance_hwp))
                            }
                            Some((height_hwp, baseline_distance_hwp))
                                if height_hwp == control.height_hwp =>
                            {
                                Some((
                                    height_hwp,
                                    baseline_distance_hwp.max(control.baseline_distance_hwp),
                                ))
                            }
                            _ => Some((control.height_hwp, control.baseline_distance_hwp)),
                        };
                    }
                    let text_start = if frame.row_count() == first_row && segments.is_empty() {
                        0
                    } else {
                        char_index_to_utf16_offset(para, line.start_idx)
                    };
                    let text_end = char_index_to_utf16_offset(para, line.end_idx).max(text_start);
                    segments.push(RowSegment::new(text_start..text_end, interval, source_tag));

                    if filled.termination != FillTermination::IntervalFull {
                        row_terminated = true;
                        break;
                    }
                }

                let mut metrics = frame_metrics_for_line(
                    maximum_font_size,
                    fallback_font_size,
                    line_spacing_type,
                    line_spacing_value,
                    dpi,
                );
                if let Some((height_hwp, baseline_distance_hwp)) = inline_metrics {
                    let inline_owns_row_height = height_hwp > metrics.line_height;
                    apply_inline_control_frame_height(&mut metrics, height_hwp);
                    if inline_owns_row_height {
                        if let Some(baseline_distance_hwp) = baseline_distance_hwp {
                            metrics.baseline_distance = baseline_distance_hwp;
                        }
                    }
                }
                if metrics.line_height != candidate_height {
                    candidate_height = metrics.line_height;
                    continue;
                }

                if row_terminated && segments.len() < frame.current_intervals.len() {
                    let text_start = segments
                        .last()
                        .map(|segment| segment.text_range.end)
                        .unwrap_or(0);
                    for interval in frame.current_intervals[segments.len()..].iter().cloned() {
                        segments.push(RowSegment::new(
                            text_start..text_start,
                            interval,
                            source_tag | LineSeg::TAG_EMPTY_SEGMENT,
                        ));
                    }
                }

                frame.commit_carved_row(metrics, segments)?;
                break;
            }
        }
        Some(frame.project_line_segs_since(first_row))
    })();

    if result.is_none() {
        frame.restore_checkpoint(frame_checkpoint);
    }
    result
}

/// This mirrors `float_placement::horizontal_range`'s `HorzRelTo::Para`
/// rule: the host's left paragraph margin shifts the object reference, but
/// the right text margin does not shrink it.
fn picture_band_paragraph_reference(
    column_horizontal: &Range<i32>,
    host_margin_left: i32,
) -> Option<Range<i32>> {
    let start = column_horizontal.start.saturating_add(host_margin_left);
    (start < column_horizontal.end).then_some(start..column_horizontal.end)
}

/// Lay out one proven non-TAC Picture/Square band without reading stored
/// `LineSeg` geometry. One `LayoutFrame` remains live from the Picture's
/// source anchor through the first full-width paragraph boundary.
///
/// This is intentionally a fail-closed transaction. It accepts one host
/// Picture, treats only TAC Equations as inline flow, and rejects any
/// paragraph boundary that would require another layout owner.
pub(crate) fn layout_picture_band(
    paragraphs: &[Paragraph],
    host_index: usize,
    column_width_hwp: i32,
    styles: &ResolvedStyleSet,
    dpi: f64,
) -> Option<PictureBandLayout> {
    let host = paragraphs.get(host_index)?;
    let column_horizontal = 0..column_width_hwp;
    let margins_for = |paragraph: &Paragraph| {
        let style = styles.para_styles.get(paragraph.para_shape_id as usize);
        (
            px_to_hwpunit(style.map(|value| value.margin_left).unwrap_or(0.0), dpi),
            px_to_hwpunit(style.map(|value| value.margin_right).unwrap_or(0.0), dpi),
        )
    };
    let horizontal_for = |paragraph: &Paragraph| {
        let (margin_left, margin_right) = margins_for(paragraph);
        let start = column_horizontal.start.saturating_add(margin_left);
        let end = column_horizontal.end.saturating_sub(margin_right);
        (start < end).then_some(start..end)
    };
    let host_horizontal = horizontal_for(host)?;
    let (host_margin_left, _) = margins_for(host);
    let host_paragraph_horizontal =
        picture_band_paragraph_reference(&column_horizontal, host_margin_left)?;

    let mut host_pictures = host
        .controls
        .iter()
        .enumerate()
        .filter_map(|(index, control)| match control {
            Control::Picture(picture) if !picture.common.treat_as_char => {
                Some((index, picture.as_ref()))
            }
            _ => None,
        });
    let (picture_control_index, picture) = host_pictures.next()?;
    if host_pictures.next().is_some() {
        return None;
    }

    // A paragraph-relative Picture starts at its control's raw UTF-16 stream
    // position, not necessarily at the first visible character. Lay out a
    // clean, full-width host first so the anchor row has no stored-LineSeg
    // dependency.
    let picture_raw_start = host
        .control_utf16_positions()
        .get(picture_control_index)
        .copied()?;
    let mut anchor_input = host.clone();
    anchor_input.line_segs.clear();
    let mut anchor_frame = LayoutFrame::new(host_horizontal.clone(), 0, Vec::new());
    let anchor_rows = layout_paragraph_in_frame(&anchor_input, &mut anchor_frame, styles, dpi)?;
    let anchor_top = anchor_rows
        .iter()
        .rfind(|row| row.text_start <= picture_raw_start)
        .map(|row| row.vertical_pos)?;

    let exclusion = crate::renderer::float_placement::resolve_picture_exclusion(
        picture,
        column_horizontal.clone(),
        host_paragraph_horizontal,
        anchor_top,
    )?;
    let exclusion_end = exclusion.vertical.end;
    let mut frame = LayoutFrame::new(host_horizontal.clone(), 0, vec![exclusion]);
    let mut line_segs = Vec::new();

    for (paragraph_index, paragraph) in paragraphs.iter().enumerate().skip(host_index) {
        if frame.top >= exclusion_end {
            break;
        }

        let paragraph_style = styles.para_styles.get(paragraph.para_shape_id as usize);
        if paragraph.column_type != ColumnBreakType::None
            || paragraph_style.is_some_and(|style| {
                style.spacing_before.abs() > f64::EPSILON
                    || style.spacing_after.abs() > f64::EPSILON
                    || style.page_break_before
            })
            || horizontal_for(paragraph)? != host_horizontal
            || (paragraph_index != host_index
                && paragraph.controls.iter().any(|control| {
                    matches!(control, Control::Picture(picture) if !picture.common.treat_as_char)
                }))
        {
            return None;
        }

        let mut input = paragraph.clone();
        // Clones carry only source content. A failed band leaves every cached
        // LineSeg exactly as it was; only the finished projection is published
        // by the document owner.
        input.line_segs.clear();
        let paragraph_lines = layout_paragraph_in_frame(&input, &mut frame, styles, dpi)?;
        line_segs.push(paragraph_lines);
    }

    (!line_segs.is_empty() && frame.top >= exclusion_end).then_some(PictureBandLayout {
        paragraph_range: host_index..host_index + line_segs.len(),
        line_segs,
    })
}

/// 문단의 line_segs를 텍스트 내용과 컬럼 너비에 맞게 재계산한다.
///
/// 텍스트 편집(삽입/삭제) 후 호출하여 줄 바꿈을 재배치한다.
/// `available_width_px`는 문단 여백을 제외한 사용 가능 너비(px)이다.
pub(crate) fn reflow_line_segs(
    para: &mut Paragraph,
    available_width_px: f64,
    styles: &ResolvedStyleSet,
    dpi: f64,
) {
    let _ = reflow_line_segs_impl(para, available_width_px, styles, dpi, None, false);
}

/// 셀 분할로 저장 폭이 stale해진 문단을 다시 조판한다.
///
/// 한컴은 좁아진 셀에서만 본문 뒤의 inline control을 별도 source line으로 저장한다.
/// 이 규칙을 일반 reflow에 적용하면 원본 문서의 이미 권위적인 control host line까지
/// 분리되어 pagination이 달라진다 (#4138/#2424). 호출자는 split 직후 stale-cell
/// 복구 경로로 한정한다.
pub(crate) fn reflow_line_segs_after_cell_split(
    para: &mut Paragraph,
    available_width_px: f64,
    styles: &ResolvedStyleSet,
    dpi: f64,
) {
    let _ = reflow_line_segs_impl(para, available_width_px, styles, dpi, None, true);
}

/// 저장 LINE_SEG가 유효한 셀 텍스트 편집은 수정된 줄 이전의 경계를 그대로 둔다.
///
/// 한컴은 중간 줄의 짧은 edit에서 문단 전체를 다시 나누지 않는다. prefix 경계를 다시
/// 계산하면 뒤 줄의 가용 폭이 인위적으로 커져 실제 HWP 저장본과 다른 다음 줄 전환을 만들 수
/// 있다. 단, prefix가 유효한 token 경계일 때만 보존하며, 합성 문단·첫 줄 edit·inline control은
/// 기존 full reflow로 안전하게 폴백한다.
pub(crate) fn reflow_line_segs_after_cell_text_edit(
    para: &mut Paragraph,
    available_width_px: f64,
    styles: &ResolvedStyleSet,
    dpi: f64,
    edit_char_offset: usize,
) -> bool {
    reflow_line_segs_impl(
        para,
        available_width_px,
        styles,
        dpi,
        Some(edit_char_offset),
        false,
    )
}

fn reflow_line_segs_impl(
    para: &mut Paragraph,
    available_width_px: f64,
    styles: &ResolvedStyleSet,
    dpi: f64,
    preserve_prefix_for_edit: Option<usize>,
    split_stale_cell_reflow: bool,
) -> bool {
    // [#4149] 셀 편집의 단일 관문(reflow_cell_paragraph[_by_path])과 서식 적용
    // (formatting.rs) 이 모두 여기로 수렴한다 — 단일줄 과밀 memo 무효화.
    para.invalidate_single_line_overflow_memo();
    // [#4677] 줄을 다시 계산하면 이전에 붙여 둔 조판 전용 보강 줄은 사라진다 — 표식을
    // 남겨 두면 실제 줄을 저장에서 잘라 내게 된다.
    para.layout_only_fill_lines = 0;
    // 기존 LineSeg에서 dimension 값 보존 (원본 HWP 호환성 유지)
    let seg_width_hwp = px_to_hwpunit(available_width_px, dpi);
    let orig = para.line_segs.first().cloned();

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
        // [Task #1811] 원본 linesegarray 부재(orig=None) 시 합성 seg 에 구현속성
        // 태그를 부여 — vpos 보정 등에서 실제 저장 증거와 구분한다 (컨버터의
        // 합성 lineseg flags=0x8000_0000 관례와 정합).
        let orig_tag = orig
            .as_ref()
            .map(|ls| ls.tag)
            .unwrap_or(LineSeg::TAG_SINGLE_SEGMENT_LINE | LineSeg::TAG_IMPLEMENTATION_PROPERTY);
        LineSeg {
            text_start: utf16_start,
            line_height: line_height_hwp,
            text_height: text_height_hwp,
            baseline_distance: baseline_distance_hwp,
            line_spacing: line_spacing_hwp,
            segment_width: seg_width_hwp,
            tag: if orig_tag != 0 {
                orig_tag
            } else {
                LineSeg::TAG_SINGLE_SEGMENT_LINE
            },
            ..Default::default()
        }
    };

    if para.text.is_empty() {
        // [#4677] 각 인라인 개체의 **UTF-16 오프셋**을 함께 들고 다닌다. lineseg 의
        // `text_start` 는 PARA_TEXT 안의 코드유닛 위치이고 확장 제어문자 하나가 8 유닛을
        // 차지하므로, 컨트롤 인덱스를 그대로 쓰면 둘째 줄이 첫 제어문자 블록 한가운데(=1)를
        // 가리킨다. 한글 2022 는 그런 문서를 열 때 본문을 통째로 버리고 빈 1쪽으로 연다
        // (10k 전수 스윕의 x2h 본문 소실군 — 저장본은 rhwp 재파싱만 통과하는 함정).
        let inline_sizes = para
            .controls
            .iter()
            .scan(0u32, |utf16_pos, ctrl| {
                let start = *utf16_pos;
                if ctrl.occupies_ctrl_char_slot() {
                    *utf16_pos += CTRL_CHAR_CODE_UNITS;
                }
                Some((start, ctrl))
            })
            .filter_map(|(start, ctrl)| inline_control_size_hwp(ctrl).map(|size| (start, size)))
            .collect::<Vec<_>>();
        if !inline_sizes.is_empty() {
            let max_line_width = seg_width_hwp.max(1);
            let mut line_specs: Vec<(u32, i32, i32)> = Vec::new();
            let mut line_start = 0u32;
            let mut line_width = 0i32;
            let mut line_height = 0i32;

            for (utf16_start, (ctrl_width, ctrl_height)) in inline_sizes.iter().copied() {
                if line_width > 0 && line_width + ctrl_width > max_line_width {
                    line_specs.push((line_start, line_width, line_height));
                    line_start = utf16_start;
                    line_width = 0;
                    line_height = 0;
                }
                line_width += ctrl_width;
                line_height = line_height.max(ctrl_height);
            }
            line_specs.push((line_start, line_width, line_height));

            let orig_line_segs = para.line_segs.clone();
            let mut new_line_segs = Vec::with_capacity(line_specs.len());
            for (line_idx, (start_pos, _line_width, height_hwp)) in
                line_specs.into_iter().enumerate()
            {
                let mut seg = make_line_seg(start_pos, 0.0);
                if let Some(template) = orig_line_segs
                    .get(line_idx)
                    .or_else(|| orig_line_segs.first())
                {
                    seg.line_spacing = template.line_spacing;
                    seg.segment_width = if template.segment_width > 0 {
                        template.segment_width
                    } else {
                        seg_width_hwp
                    };
                    seg.tag = if template.tag != 0 {
                        template.tag
                    } else {
                        seg.tag
                    };
                }
                apply_inline_control_line_height(&mut seg, height_hwp);
                new_line_segs.push(seg);
            }

            let mut vpos = orig.as_ref().map(|ls| ls.vertical_pos).unwrap_or(0);
            for seg in &mut new_line_segs {
                seg.vertical_pos = vpos;
                vpos += seg.line_height.saturating_add(seg.line_spacing);
            }
            para.line_segs = new_line_segs;
        } else {
            // 빈 문단도 활성 글자 모양의 크기로 줄을 만든다. 앞 문단 LINE_SEG의
            // 치수를 복사하면 TAC 그림 높이까지 상속되므로 vpos 원점만 보존한다.
            let font_size = para
                .char_shapes
                .first()
                .and_then(|char_shape| styles.char_styles.get(char_shape.char_shape_id as usize))
                .map(|style| style.font_size)
                .unwrap_or(12.0);
            let mut seg = make_line_seg(0, font_size);
            if let Some(template) = orig.as_ref() {
                seg.vertical_pos = template.vertical_pos;
            }
            if let Some(height_hwp) = inline_control_line_height_hwp(para) {
                apply_inline_control_line_height(&mut seg, height_hwp);
            }
            para.line_segs = vec![seg];
        }
        return false;
    }

    let text_chars: Vec<char> = para.text.chars().collect();
    let text_len = text_chars.len();
    let inline_controls = flow_inline_controls(para);

    // 문단 스타일에서 들여쓰기 및 줄 나눔 설정 조회
    let para_style = styles.para_styles.get(para.para_shape_id as usize);
    let indent_px = para_style.map(|s| s.indent).unwrap_or(0.0);
    let english_break_unit = para_style.map(|s| s.english_break_unit).unwrap_or(0);
    let korean_break_unit = para_style.map(|s| s.korean_break_unit).unwrap_or(0);
    let condense_min_space = para_style.map(|s| s.condense_min_space).unwrap_or(0);
    let tab_width = para_style.map(|s| s.default_tab_width).unwrap_or(0.0);

    // 토큰화 → 줄 채움 → LineSeg 생성
    let tokens = tokenize_paragraph_with_regenerated_space_metric(
        &text_chars,
        &para.char_offsets,
        &para.char_shapes,
        styles,
        english_break_unit,
        korean_break_unit,
        split_stale_cell_reflow,
        &inline_controls,
    );
    // 저장 LINE_SEG 기반 incremental edit는 앞선 줄을 유지한다. LINE_SEG start가 현재
    // char_offsets와 token 경계 모두에 정확히 대응할 때만 suffix reflow를 허용한다.
    // 그렇지 않으면 (HWPX 합성 boundary, inline control, token 내부 boundary 등) full
    // reflow가 보수적인 경로다.
    let original_line_segs = para.line_segs.clone();
    let token_start_idx = |token: &BreakToken| match token {
        BreakToken::Text { start_idx, .. } => *start_idx,
        BreakToken::Space { idx, .. }
        | BreakToken::Tab { idx, .. }
        | BreakToken::LineBreak { idx } => *idx,
    };
    let mut preserved_prefix = Vec::new();
    let mut reflow_start_idx = 0usize;
    let mut reflow_is_first_line = true;
    let mut token_start = 0usize;
    // `DocumentCore::new_empty()`의 기본 source_format도 Hwp이므로 형식만으로는
    // 합성 test/new-document LineSeg를 native 저장 경계로 오인할 수 없다. 실제 HWP
    // LINE_SEG가 가진 line-height와, 0에서 시작해 엄격히 증가하는 start가 모두
    // 있어야 prefix를 권위 경계로 채택한다. 범위 삭제는 삭제된 여러 줄을 같은
    // start로 접을 수 있으므로 duplicate/역행 경계는 full reflow가 안전하다.
    let has_valid_orig = original_line_segs
        .iter()
        .all(|seg| seg.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0);
    let authoritative_line_seg_prefix = has_valid_orig
        && original_line_segs
            .first()
            .is_some_and(|seg| seg.text_start == 0)
        && original_line_segs
            .windows(2)
            .all(|pair| pair[0].text_start < pair[1].text_start);
    if para.controls.is_empty() && authoritative_line_seg_prefix {
        if let Some(edit_char_offset) = preserve_prefix_for_edit {
            // Delete-at-end는 삭제 뒤 `char_offsets`에 caret 위치가 없지만, 텍스트
            // UTF-16 끝은 정확한 token boundary다. 삭제된 마지막 글자가 있던 줄의
            // 앞줄부터 다시 채워야 5→4 shrink도 표현할 수 있다.
            let edit_is_document_end = edit_char_offset == text_len;
            let edit_utf16 = para
                .char_offsets
                .get(edit_char_offset)
                .copied()
                .or_else(|| edit_is_document_end.then(|| para.text.encode_utf16().count() as u32));
            let affected_line = edit_utf16.and_then(|offset| {
                let line = original_line_segs
                    .iter()
                    .rposition(|seg| seg.text_start <= offset)?;
                if edit_is_document_end && original_line_segs[line].text_start < offset {
                    // 삭제 대상이 들어 있던 마지막 줄도 다시 채워야 직전 줄에
                    // 합쳐질 수 있다. line=0이면 prefix 없이 full reflow한다.
                    line.checked_sub(1)
                } else {
                    Some(line)
                }
            });
            if let Some(affected_line) = affected_line.filter(|line| *line > 0) {
                let reflow_utf16 = original_line_segs[affected_line].text_start;
                let reflow_char_idx = para
                    .char_offsets
                    .iter()
                    .position(|offset| *offset == reflow_utf16);
                let suffix_token_start = reflow_char_idx.and_then(|char_idx| {
                    tokens
                        .iter()
                        .position(|token| token_start_idx(token) == char_idx)
                        .map(|token_idx| (char_idx, token_idx))
                });
                if let Some((char_idx, token_idx)) = suffix_token_start {
                    preserved_prefix = original_line_segs[..affected_line].to_vec();
                    reflow_start_idx = char_idx;
                    reflow_is_first_line = false;
                    token_start = token_idx;
                }
            }
        }
    }

    // The frame owns physical-row recurrence only for an ordinary scalar
    // reflow. Stored-prefix edits, split-cell recovery, empty paragraphs, and
    // inline controls retain their established specialized paths below.
    let frame_eligible = !split_stale_cell_reflow
        && preserve_prefix_for_edit.is_none()
        && !para.text.is_empty()
        && para.controls.is_empty()
        && preserved_prefix.is_empty();
    if frame_eligible {
        let mut frame = LayoutFrame::new(
            0..seg_width_hwp,
            orig.as_ref().map(|line| line.vertical_pos).unwrap_or(0),
            Vec::new(),
        );
        if let Some(projected) = layout_paragraph_in_frame(para, &mut frame, styles, dpi) {
            para.line_segs = projected;
            return false;
        }
    }

    let line_breaks = fill_lines(
        &tokens[token_start..],
        &text_chars,
        available_width_px,
        indent_px,
        tab_width,
        korean_break_unit,
        condense_min_space,
        reflow_start_idx,
        reflow_is_first_line,
    );
    let forced_inline_line = split_stale_cell_reflow
        .then(|| {
            inline_control_requires_own_line(
                para,
                &text_chars,
                &line_breaks,
                available_width_px,
                indent_px,
                reflow_is_first_line,
                styles,
            )
        })
        .flatten();
    let preserved_prefix_len = preserved_prefix.len();
    let mut new_line_segs: Vec<LineSeg> = preserved_prefix;
    for (line_idx, lb) in line_breaks.iter().enumerate() {
        let utf16_start = if new_line_segs.is_empty() {
            0 // 첫 번째 줄의 text_start는 항상 0 (문단 시작)
        } else {
            char_index_to_utf16_offset(para, lb.start_idx)
        };
        let fs = if lb.max_font_size > 0.0 {
            lb.max_font_size
        } else {
            12.0
        };
        let mut text_seg = make_line_seg(utf16_start, fs);
        if forced_inline_line.is_some_and(|(position, _)| position == lb.start_idx) {
            let (_, height_hwp) = forced_inline_line.expect("checked inline control");
            apply_inline_control_line_height(&mut text_seg, height_hwp);
        }
        if let Some(height_hwp) = inline_controls
            .iter()
            .filter(|control| {
                (lb.start_idx..lb.end_idx).contains(&control.char_position)
                    || (lb.end_idx == text_len && control.char_position == text_len)
            })
            .map(|control| control.height_hwp)
            .max()
        {
            apply_inline_control_line_height(&mut text_seg, height_hwp);
        }
        new_line_segs.push(text_seg);

        // control이 text line 한가운데/끝에 있으면 먼저 text prefix를 확정하고,
        // control offset에서 다음 LineSeg를 삽입한다. 단순히 vector 끝에 붙이면
        // 중간 nested table 뒤의 text가 control보다 앞에서 그려진다.
        let control_after_text = forced_inline_line.is_some_and(|(position, _)| {
            position > lb.start_idx
                && (position < lb.end_idx
                    || (position == lb.end_idx && line_idx + 1 == line_breaks.len()))
        });
        if control_after_text {
            let (position, height_hwp) = forced_inline_line.expect("checked inline control");
            let mut control_seg = make_line_seg(char_index_to_utf16_offset(para, position), fs);
            apply_inline_control_line_height(&mut control_seg, height_hwp);
            new_line_segs.push(control_seg);
        }
    }

    if new_line_segs.is_empty() {
        new_line_segs.push(make_line_seg(0, 12.0));
    }

    if forced_inline_line.is_none() && inline_controls.is_empty() {
        if let Some(height_hwp) = inline_control_line_height_hwp(para) {
            // 기존 인라인 TAC 개체는 해당 문단의 최초 line box에 남긴다.
            if let Some(seg) = new_line_segs.first_mut() {
                apply_inline_control_line_height(seg, height_hwp);
            }
        }
    }

    // vertical_pos 누적 계산 (각 줄의 문단 내 Y 오프셋)
    // 원본 첫 LineSeg의 vertical_pos를 보존하여 vpos 체계 연속성 유지
    // (layout.rs의 vpos 보정이 문단 간 vpos 연속성을 가정하므로)
    let mut vpos = if preserved_prefix_len > 0 {
        let last = &new_line_segs[preserved_prefix_len - 1];
        last.vertical_pos
            .saturating_add(last.line_height)
            .saturating_add(last.line_spacing)
    } else {
        orig.as_ref().map(|ls| ls.vertical_pos).unwrap_or(0)
    };
    for i in preserved_prefix_len..new_line_segs.len() {
        new_line_segs[i].vertical_pos = vpos;
        vpos += new_line_segs[i].line_height + new_line_segs[i].line_spacing;
    }

    para.line_segs = new_line_segs;
    preserved_prefix_len > 0
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
        let is_reset = is_original_lineseg
            && !is_ignored(pi)
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
mod fill_cursor_tests {
    use super::*;

    fn collect_one_interval_at_a_time(
        tokens: &[BreakToken],
        text_chars: &[char],
        available_width_px: f64,
        indent_px: f64,
        default_tab_width: f64,
        korean_break_unit: u8,
        condense_min_space: u8,
        initial_start_idx: usize,
        initial_is_first_line: bool,
    ) -> Vec<LineBreakResult> {
        let mut cursor = FillCursor::new(initial_start_idx, initial_is_first_line);
        let mut results = Vec::new();
        while let Some(result) = fill_one_interval(
            tokens,
            text_chars,
            available_width_px,
            indent_px,
            default_tab_width,
            korean_break_unit,
            condense_min_space,
            &mut cursor,
        ) {
            results.push(result.line);
        }
        results
    }

    fn assert_cursor_matches_frozen_scalar(
        tokens: &[BreakToken],
        text_chars: &[char],
        available_width_px: f64,
        indent_px: f64,
        default_tab_width: f64,
        korean_break_unit: u8,
        condense_min_space: u8,
        initial_start_idx: usize,
        initial_is_first_line: bool,
    ) -> Vec<LineBreakResult> {
        let frozen = fill_lines_before_cursor(
            tokens,
            text_chars,
            available_width_px,
            indent_px,
            default_tab_width,
            korean_break_unit,
            condense_min_space,
            initial_start_idx,
            initial_is_first_line,
        );
        let scalar = fill_lines(
            tokens,
            text_chars,
            available_width_px,
            indent_px,
            default_tab_width,
            korean_break_unit,
            condense_min_space,
            initial_start_idx,
            initial_is_first_line,
        );
        let resumed = collect_one_interval_at_a_time(
            tokens,
            text_chars,
            available_width_px,
            indent_px,
            default_tab_width,
            korean_break_unit,
            condense_min_space,
            initial_start_idx,
            initial_is_first_line,
        );

        assert_eq!(scalar, frozen);
        assert_eq!(resumed, frozen);
        frozen
    }

    #[test]
    fn cursor_resumes_a_long_text_token_at_each_interval() {
        let text_chars = "abcdefghij".chars().collect::<Vec<_>>();
        let tokens = vec![BreakToken::Text {
            start_idx: 0,
            end_idx: text_chars.len(),
            width: 100.0,
            max_font_size: 12.0,
            char_widths: vec![10.0; text_chars.len()],
        }];

        let results = assert_cursor_matches_frozen_scalar(
            &tokens,
            &text_chars,
            25.0,
            0.0,
            48.0,
            0,
            0,
            0,
            true,
        );

        assert_eq!(
            results
                .iter()
                .map(|result| (result.start_idx, result.end_idx, result.has_line_break))
                .collect::<Vec<_>>(),
            vec![
                (0, 2, false),
                (2, 4, false),
                (4, 6, false),
                (6, 8, false),
                (8, 10, false),
            ]
        );
    }

    #[test]
    fn cursor_preserves_scalar_space_tab_and_forced_break_results() {
        let text_chars = "ab c\td\nxy".chars().collect::<Vec<_>>();
        let tokens = vec![
            BreakToken::Text {
                start_idx: 0,
                end_idx: 2,
                width: 20.0,
                max_font_size: 12.0,
                char_widths: vec![10.0, 10.0],
            },
            BreakToken::Space {
                idx: 2,
                width: 5.0,
                max_font_size: 12.0,
            },
            BreakToken::Text {
                start_idx: 3,
                end_idx: 4,
                width: 10.0,
                max_font_size: 12.0,
                char_widths: vec![10.0],
            },
            BreakToken::Tab {
                idx: 4,
                max_font_size: 12.0,
            },
            BreakToken::Text {
                start_idx: 5,
                end_idx: 6,
                width: 10.0,
                max_font_size: 12.0,
                char_widths: vec![10.0],
            },
            BreakToken::LineBreak { idx: 6 },
            BreakToken::Text {
                start_idx: 7,
                end_idx: 9,
                width: 20.0,
                max_font_size: 12.0,
                char_widths: vec![10.0, 10.0],
            },
        ];

        assert_cursor_matches_frozen_scalar(&tokens, &text_chars, 24.0, 0.0, 48.0, 0, 0, 0, true);
    }
}

#[cfg(test)]
mod frame_reflow_tests {
    use super::*;
    use crate::renderer::layout_frame::{FrameExclusion, FrameExclusionPolicy};
    use crate::renderer::style_resolver::{ResolvedCharStyle, ResolvedParaStyle};

    fn styles(font_sizes: &[f64]) -> ResolvedStyleSet {
        ResolvedStyleSet {
            char_styles: font_sizes
                .iter()
                .map(|font_size| ResolvedCharStyle {
                    font_size: *font_size,
                    ratio: 1.0,
                    ..Default::default()
                })
                .collect(),
            para_styles: vec![ResolvedParaStyle::default()],
            ..Default::default()
        }
    }

    fn paragraph(text: &str, char_shapes: Vec<CharShapeRef>) -> Paragraph {
        Paragraph {
            text: text.to_string(),
            char_offsets: text
                .chars()
                .scan(0u32, |offset, character| {
                    let current = *offset;
                    *offset += character.len_utf16() as u32;
                    Some(current)
                })
                .collect(),
            char_count: text.encode_utf16().count() as u32 + 1,
            char_shapes,
            ..Default::default()
        }
    }

    fn shared_metrics(lines: &[LineSeg]) -> Vec<(i32, i32, i32, i32, i32)> {
        lines
            .iter()
            .map(|line| {
                (
                    line.vertical_pos,
                    line.line_height,
                    line.text_height,
                    line.baseline_distance,
                    line.line_spacing,
                )
            })
            .collect()
    }

    fn line_fields(lines: &[LineSeg]) -> Vec<(u32, i32, i32, i32, i32, i32, i32, i32, u32)> {
        lines
            .iter()
            .map(|line| {
                (
                    line.text_start,
                    line.vertical_pos,
                    line.line_height,
                    line.text_height,
                    line.baseline_distance,
                    line.line_spacing,
                    line.column_start,
                    line.segment_width,
                    line.tag,
                )
            })
            .collect()
    }

    fn frozen_scalar_projection(
        para: &Paragraph,
        available_width_px: f64,
        styles: &ResolvedStyleSet,
        dpi: f64,
    ) -> Vec<LineSeg> {
        let text_chars = para.text.chars().collect::<Vec<_>>();
        let style = styles.para_styles.get(para.para_shape_id as usize);
        let indent_px = style.map(|value| value.indent).unwrap_or(0.0);
        let english_break_unit = style.map(|value| value.english_break_unit).unwrap_or(0);
        let korean_break_unit = style.map(|value| value.korean_break_unit).unwrap_or(0);
        let condense_min_space = style.map(|value| value.condense_min_space).unwrap_or(0);
        let default_tab_width = style.map(|value| value.default_tab_width).unwrap_or(0.0);
        let line_spacing_type = style
            .map(|value| value.line_spacing_type)
            .unwrap_or(LineSpacingType::Percent);
        let line_spacing_value = style.map(|value| value.line_spacing).unwrap_or(160.0);
        let tokens = tokenize_paragraph_with_regenerated_space_metric(
            &text_chars,
            &para.char_offsets,
            &para.char_shapes,
            styles,
            english_break_unit,
            korean_break_unit,
            false,
            &[],
        );
        let line_breaks = fill_lines_before_cursor(
            &tokens,
            &text_chars,
            available_width_px,
            indent_px,
            default_tab_width,
            korean_break_unit,
            condense_min_space,
            0,
            true,
        );
        let segment_width = px_to_hwpunit(available_width_px, dpi);
        let source_tag = para
            .line_segs
            .first()
            .map(|line| line.tag)
            .unwrap_or(LineSeg::TAG_SINGLE_SEGMENT_LINE | LineSeg::TAG_IMPLEMENTATION_PROPERTY);
        let mut vertical_pos = para
            .line_segs
            .first()
            .map(|line| line.vertical_pos)
            .unwrap_or(0);

        line_breaks
            .iter()
            .enumerate()
            .map(|(index, line)| {
                let font_size = if line.max_font_size > 0.0 {
                    line.max_font_size
                } else {
                    12.0
                };
                let line_height = font_size_to_line_height(font_size, dpi);
                let line_spacing = compute_line_spacing_hwp(
                    line_spacing_type,
                    line_spacing_value,
                    line_height,
                    dpi,
                );
                let projected = LineSeg {
                    text_start: if index == 0 {
                        0
                    } else {
                        char_index_to_utf16_offset(para, line.start_idx)
                    },
                    vertical_pos,
                    line_height,
                    text_height: line_height,
                    baseline_distance: (line_height as f64 * 0.85) as i32,
                    line_spacing,
                    column_start: 0,
                    segment_width,
                    tag: if source_tag == 0 {
                        LineSeg::TAG_SINGLE_SEGMENT_LINE
                    } else {
                        source_tag
                    },
                };
                vertical_pos += line_height + line_spacing;
                projected
            })
            .collect()
    }

    #[test]
    fn frame_reflow_projects_two_intervals_as_one_physical_row() {
        let styles = styles(&[12.0]);
        let para = paragraph(
            "abcdef ghijkl",
            vec![CharShapeRef {
                start_pos: 0,
                char_shape_id: 0,
            }],
        );
        let mut frame = LayoutFrame::new(
            0..9_000,
            100,
            vec![FrameExclusion {
                horizontal: 3_000..5_000,
                vertical: 0..10_000,
                policy: FrameExclusionPolicy::BothSides,
            }],
        );

        let lines = layout_paragraph_in_frame(&para, &mut frame, &styles, 96.0)
            .expect("two usable intervals accept scalar text");

        assert_eq!(lines.len(), 2);
        assert_eq!(
            lines.iter().map(|line| line.text_start).collect::<Vec<_>>(),
            vec![0, 7]
        );
        assert_eq!(
            lines
                .iter()
                .map(|line| (line.column_start, line.segment_width))
                .collect::<Vec<_>>(),
            vec![(0, 3_000), (5_000, 4_000)]
        );
        assert_eq!(shared_metrics(&lines), vec![(100, 900, 900, 765, 540); 2]);
        assert!(lines[0].is_first_segment());
        assert!(!lines[0].is_last_segment());
        assert!(!lines[1].is_first_segment());
        assert!(lines[1].is_last_segment());
        assert_eq!(frame.row_count(), 1);
        assert_eq!(frame.top, 1_540);
    }

    #[test]
    fn frame_reflow_retries_a_taller_row_without_consuming_the_cursor() {
        let styles = styles(&[12.0, 20.0]);
        let para = paragraph(
            "abcdef ghijk",
            vec![
                CharShapeRef {
                    start_pos: 0,
                    char_shape_id: 0,
                },
                CharShapeRef {
                    start_pos: 7,
                    char_shape_id: 1,
                },
            ],
        );
        let mut frame = LayoutFrame::new(
            0..9_000,
            0,
            vec![FrameExclusion {
                horizontal: 4_000..5_000,
                vertical: 1_000..5_000,
                policy: FrameExclusionPolicy::BothSides,
            }],
        );

        let lines = layout_paragraph_in_frame(&para, &mut frame, &styles, 96.0)
            .expect("the taller retry restores the first interval's cursor");

        // The 12px trial has one full-width interval below the exclusion. The
        // 20px row reaches it, so retrying from the same cursor must produce
        // the two carved segments rather than an exhausted paragraph.
        assert_eq!(lines.len(), 2);
        assert_eq!(
            lines.iter().map(|line| line.text_start).collect::<Vec<_>>(),
            vec![0, 7]
        );
        assert_eq!(
            lines
                .iter()
                .map(|line| (line.column_start, line.segment_width))
                .collect::<Vec<_>>(),
            vec![(0, 4_000), (5_000, 4_000)]
        );
        assert_eq!(
            shared_metrics(&lines),
            vec![(0, 1_500, 1_500, 1_275, 900); 2]
        );
        assert_eq!(frame.row_count(), 1);
        assert_eq!(frame.top, 2_400);
    }

    #[test]
    fn eligible_scalar_reflow_projects_the_frozen_scalar_oracle() {
        let styles = styles(&[12.0]);
        let mut para = paragraph(
            "alpha beta gamma delta epsilon",
            vec![CharShapeRef {
                start_pos: 0,
                char_shape_id: 0,
            }],
        );
        para.line_segs = vec![LineSeg {
            vertical_pos: 321,
            tag: LineSeg::TAG_SINGLE_SEGMENT_LINE | LineSeg::TAG_IMPLEMENTATION_PROPERTY,
            ..Default::default()
        }];
        let expected = frozen_scalar_projection(&para, 50.0, &styles, 96.0);
        assert!(expected.len() > 1, "fixture must exercise row recurrence");

        reflow_line_segs(&mut para, 50.0, &styles, 96.0);

        assert_eq!(line_fields(&para.line_segs), line_fields(&expected));
        assert!(para
            .line_segs
            .iter()
            .all(|line| line.segment_width == 3_750 && line.column_start == 0));
        assert_eq!(para.line_segs[0].vertical_pos, 321);
    }

    #[test]
    fn picture_band_uses_para_reference_not_text_frame_for_right_aligned_picture() {
        use crate::model::image::Picture;
        use crate::model::shape::{HorzAlign, HorzRelTo, TextFlow, TextWrap, VertAlign, VertRelTo};

        const DPI: f64 = 96.0;
        const COLUMN_WIDTH: i32 = 15_000;
        const MARGIN_LEFT: i32 = 1_500;
        const MARGIN_RIGHT: i32 = 3_000;
        const PICTURE_WIDTH: u32 = 3_000;

        let mut styles = styles(&[12.0]);
        styles.para_styles[0].margin_left = crate::renderer::hwpunit_to_px(MARGIN_LEFT, DPI);
        styles.para_styles[0].margin_right = crate::renderer::hwpunit_to_px(MARGIN_RIGHT, DPI);
        let text_frame = MARGIN_LEFT..COLUMN_WIDTH - MARGIN_RIGHT;
        let paragraph_reference = picture_band_paragraph_reference(&(0..COLUMN_WIDTH), MARGIN_LEFT)
            .expect("host left margin leaves a usable Paragraph reference");
        assert_eq!(paragraph_reference, MARGIN_LEFT..COLUMN_WIDTH);
        assert_ne!(text_frame, paragraph_reference);

        let picture = Picture {
            common: crate::model::shape::CommonObjAttr {
                width: PICTURE_WIDTH,
                height: 600,
                text_wrap: TextWrap::Square,
                text_flow: TextFlow::BothSides,
                vert_rel_to: VertRelTo::Para,
                vert_align: VertAlign::Top,
                horz_rel_to: HorzRelTo::Para,
                horz_align: HorzAlign::Right,
                ..Default::default()
            },
            ..Default::default()
        };
        let expected_exclusion = crate::renderer::float_placement::resolve_picture_exclusion(
            &picture,
            0..COLUMN_WIDTH,
            paragraph_reference.clone(),
            0,
        )
        .expect("supported Paragraph-relative Picture");
        assert_eq!(expected_exclusion.horizontal, 12_000..15_000);

        let mut host = paragraph(
            "x",
            vec![CharShapeRef {
                start_pos: 0,
                char_shape_id: 0,
            }],
        );
        host.controls.push(Control::Picture(Box::new(picture)));

        let band = layout_picture_band(&[host], 0, COLUMN_WIDTH, &styles, DPI)
            .expect("the one-row Paragraph-relative Picture band");

        assert_eq!(band.paragraph_range, 0..1);
        assert_eq!(band.line_segs[0].len(), 1);
        assert_eq!(band.line_segs[0][0].column_start, text_frame.start);
        assert_eq!(
            band.line_segs[0][0].segment_width,
            text_frame.end - text_frame.start,
            "the right-aligned Para exclusion begins at the text frame's end"
        );
    }

    #[test]
    fn real_p325_picture_band_matches_the_stored_seven_paragraph_geometry() {
        use crate::model::page::ColumnDef;
        use crate::renderer::page_layout::PageLayoutInfo;

        const DPI: f64 = 96.0;
        let bytes = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("samples/3-09월_교육_통합_2022.hwp"),
        )
        .expect("p325 corpus fixture");
        let document = crate::parse_document(&bytes).expect("parse p325 corpus fixture");
        let section = &document.sections[0];
        let column_def = section.paragraphs[..=325]
            .iter()
            .flat_map(|paragraph| &paragraph.controls)
            .filter_map(|control| match control {
                Control::ColumnDef(column) => Some(column.clone()),
                _ => None,
            })
            .next_back()
            .unwrap_or_else(ColumnDef::default);
        let page_layout =
            PageLayoutInfo::from_page_def(&section.section_def.page_def, &column_def, DPI);
        let column_width = crate::renderer::px_to_hwpunit(page_layout.column_areas[0].width, DPI);
        let styles = crate::renderer::style_resolver::resolve_styles(&document.doc_info, DPI);

        let band = layout_picture_band(&section.paragraphs, 325, column_width, &styles, DPI)
            .expect("one Picture + trailing TAC Equation p325 band");

        assert_eq!(band.paragraph_range, 325..332);
        assert_eq!(band.line_segs.len(), 7);
        for (paragraph_index, generated) in band.paragraph_range.clone().zip(&band.line_segs) {
            let stored = &section.paragraphs[paragraph_index].line_segs;
            assert_eq!(generated.len(), 1, "p{paragraph_index}");
            assert_eq!(
                generated[0].text_start, stored[0].text_start,
                "p{paragraph_index}"
            );
            assert_eq!(
                generated[0].column_start, stored[0].column_start,
                "p{paragraph_index}"
            );
            assert_eq!(
                generated[0].segment_width, stored[0].segment_width,
                "p{paragraph_index}"
            );
            assert!(
                generated[0].line_height.abs_diff(stored[0].line_height) <= 1,
                "p{paragraph_index}"
            );
            assert!(
                generated[0].text_height.abs_diff(stored[0].text_height) <= 1,
                "p{paragraph_index}"
            );
            assert!(
                generated[0]
                    .baseline_distance
                    .abs_diff(stored[0].baseline_distance)
                    <= 1,
                "p{paragraph_index}: generated={} stored={}",
                generated[0].baseline_distance,
                stored[0].baseline_distance,
            );
            assert!(
                generated[0].line_spacing.abs_diff(stored[0].line_spacing) <= 3,
                "p{paragraph_index}"
            );
        }
        assert!(
            band.line_segs[0][0].line_height > 900,
            "the host's trailing TAC Equation must enlarge the retried first row"
        );
        assert_eq!(
            band.line_segs[0][0].baseline_distance,
            section.paragraphs[325].line_segs[0].baseline_distance,
            "p325 retains the TAC Equation's object-owned baseline"
        );
        assert_ne!(
            band.line_segs.last().expect("band tail")[0].segment_width,
            section.paragraphs[332].line_segs[0].segment_width,
            "p332 is the first full-width paragraph after the exclusion"
        );
    }

    #[test]
    fn picture_band_rejects_a_truncated_p325_before_any_projection() {
        use crate::model::page::ColumnDef;
        use crate::renderer::page_layout::PageLayoutInfo;

        const DPI: f64 = 96.0;
        let bytes = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("samples/3-09월_교육_통합_2022.hwp"),
        )
        .expect("p325 corpus fixture");
        let document = crate::parse_document(&bytes).expect("parse p325 corpus fixture");
        let section = &document.sections[0];
        let column_def = section.paragraphs[..=325]
            .iter()
            .flat_map(|paragraph| &paragraph.controls)
            .filter_map(|control| match control {
                Control::ColumnDef(column) => Some(column.clone()),
                _ => None,
            })
            .next_back()
            .unwrap_or_else(ColumnDef::default);
        let page_layout =
            PageLayoutInfo::from_page_def(&section.section_def.page_def, &column_def, DPI);
        let column_width = crate::renderer::px_to_hwpunit(page_layout.column_areas[0].width, DPI);
        let styles = crate::renderer::style_resolver::resolve_styles(&document.doc_info, DPI);

        assert!(
            layout_picture_band(&section.paragraphs[325..329], 0, column_width, &styles, DPI)
                .is_none(),
            "a subset ending before the exclusion clears cannot be published"
        );
    }

    #[test]
    fn pic2_two_picture_host_is_explicitly_outside_the_one_picture_band_contract() {
        use crate::model::page::ColumnDef;
        use crate::renderer::page_layout::PageLayoutInfo;

        const DPI: f64 = 96.0;
        let bytes = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/pic2.hwp"),
        )
        .expect("pic2 fixture");
        let document = crate::parse_document(&bytes).expect("parse pic2 fixture");
        let section = &document.sections[0];
        let column_def = section.paragraphs[..=0]
            .iter()
            .flat_map(|paragraph| &paragraph.controls)
            .filter_map(|control| match control {
                Control::ColumnDef(column) => Some(column.clone()),
                _ => None,
            })
            .next_back()
            .unwrap_or_else(ColumnDef::default);
        let page_layout =
            PageLayoutInfo::from_page_def(&section.section_def.page_def, &column_def, DPI);
        let column_width = crate::renderer::px_to_hwpunit(page_layout.column_areas[0].width, DPI);
        let styles = crate::renderer::style_resolver::resolve_styles(&document.doc_info, DPI);

        assert_eq!(
            section.paragraphs[0]
                .controls
                .iter()
                .filter(|control| {
                    matches!(control, Control::Picture(picture) if !picture.common.treat_as_char)
                })
                .count(),
            2,
            "fixture premise: pic2's first paragraph has two floating pictures"
        );
        assert!(
            layout_picture_band(&section.paragraphs, 0, column_width, &styles, DPI).is_none(),
            "two floating pictures are deliberately not a one-picture band"
        );
    }
}

#[cfg(test)]
mod utf16_offset_tests {
    use super::*;

    #[test]
    fn trailing_physical_line_preserves_control_stream_end_offset() {
        let mut para = Paragraph {
            text: "가\n".to_string(),
            // visible text 앞에 16 UTF-16 unit의 control stream gap이 있다.
            char_offsets: vec![16, 17],
            ..Default::default()
        };

        reflow_line_segs(&mut para, 500.0, &ResolvedStyleSet::default(), 96.0);

        assert_eq!(para.line_segs.len(), 2);
        assert_eq!(para.line_segs[1].text_start, 18);
    }

    #[test]
    fn missing_char_offsets_count_supplementary_unicode_as_utf16_units() {
        let para = Paragraph {
            text: "😀\n".to_string(),
            ..Default::default()
        };

        assert_eq!(char_index_to_utf16_offset(&para, 2), 3);
    }
}
