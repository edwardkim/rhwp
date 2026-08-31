//! 줄 나눔 엔진 (Line Breaking Engine)
//!
//! 문단 텍스트를 토큰화하고 줄 나눔을 수행한다.
//! 한글 어절/글자, 영어 단어/하이픈, CJK 개별 분할을 지원한다.

use super::{find_active_char_shape, is_lang_neutral, ComposedParagraph};
use crate::model::control::{Control, CTRL_CHAR_CODE_UNITS};
use crate::model::paragraph::{CharShapeRef, ColumnBreakType, LineSeg, Paragraph};
use crate::model::style::LineSpacingType;
use crate::renderer::layout::{
    estimate_text_width, estimate_text_width_unrounded, hancom_regenerated_space_width,
    is_cjk_char, resolved_letter_spacing, resolved_to_text_style,
};
use crate::renderer::layout_frame::{FrameRowMetrics, LayoutFrame, ParagraphBox, RowSegment};
use crate::renderer::style_resolver::{detect_lang_category, ResolvedStyleSet};
use crate::renderer::{hwpunit_to_px, px_to_hwpunit};
use std::ops::Range;

struct PreparedParagraphKerning {
    context: std::sync::Arc<crate::renderer::kerning::KerningMeasurementContext>,
    scalar_styles: Vec<crate::renderer::kerning::KerningParagraphScalarStyle>,
    hard_boundaries: Vec<bool>,
    measurement: std::sync::Arc<crate::renderer::kerning::KerningParagraphMeasurement>,
}

struct PreparedParagraphProjection {
    base_positions: Vec<f64>,
    kerning_scalar_styles: Vec<crate::renderer::kerning::KerningParagraphScalarStyle>,
    shaping_scalar_styles:
        Vec<crate::renderer::shaping_paragraph::HorizontalShapingParagraphScalarStyle>,
    hard_boundaries: Vec<bool>,
}

/// A complete, source-independent projection of one supported Picture wrap
/// band. The document layer owns the one-shot publication of every paragraph
/// in this range.
#[derive(Debug, Clone)]
pub(crate) struct PictureBandLayout {
    pub(crate) paragraph_range: Range<usize>,
    pub(crate) line_segs: Vec<Vec<LineSeg>>,
}

/// How the frame resolved a paragraph's stored rows.
///
/// Stored rows are a cache of a prior frame computation. Reuse is therefore
/// allowed only after the current frame reproduces their physical-row key:
/// interval count plus exact `column_start`/`segment_width` for every slot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum StoredRowResolution {
    /// The current frame admitted every stored physical row exactly.
    ///
    /// This is the right answer rather than a placeholder, and the reason is
    /// worth stating because "the frame owns both routes" makes it sound wrong.
    /// The frame is licensed to write two lanes, and on this arm neither one
    /// has anything to say:
    ///
    /// - **Geometry** (`column_start`, `segment_width`). Publishing is a no-op
    ///   by construction: exact equality is the admission test.
    /// - **Row metrics** (`line_height`, `baseline_distance`, `line_spacing`).
    ///   Deliberately not published: §1.4.1's accept-arm write-back is not
    ///   implemented. [`LayoutFrame::try_admit_stored_rows`] records the
    ///   measurement that settled it — publishing them took the suite from
    ///   5983 passed / 2 failed to 5979 / 6.
    ///
    /// So the stored partition may stand only after the frame has checked it.
    Stored,
    /// The stored rows were stale or their physical-row key did not match the
    /// current frame, so strict reflow carved replacement rows. The rows live in
    /// the frame; take them with `project_line_segs()`.
    Reflowed,
}

/// What a caller may conclude from an exact stored-row geometry miss.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StoredRowMissPolicy {
    /// The caller knows its Frame inputs are current and may publish fresh rows.
    Reflow,
    /// Imported geometry is not derivable from this Frame. A dirty text/style
    /// partition still reflows; a clean geometry miss stays with its source owner.
    UnmodelledUnlessStale,
}

/// 줄 나눔 토큰
#[derive(Debug, Clone)]
pub(crate) enum BreakToken {
    /// 분할 불가 텍스트 조각 (어절/단어/글자)
    /// char_widths: 글자별 px 폭 (char_level_break용, 단일 글자 토큰은 비어있음)
    Text {
        start_idx: usize,
        end_idx: usize,
        /// K0 scalar pen width. Dynamic space/tab accounting and boundary
        /// pair correction are applied against this invariant base.
        base_width: f64,
        /// Owned paragraph-position token total. K0에서는 base_width와 같다.
        width: f64,
        max_font_size: f64,
        base_char_widths: Vec<f64>,
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
        SpaceMetric::Stored,
        &[],
    )
}

/// 공백 토큰의 advance 를 어느 규칙으로 재는가.
///
/// 종전에는 `bool` 이었고, 그래서 세 번째 규칙이 필요해졌을 때 표현할 자리가 없었다.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum SpaceMetric {
    /// 글꼴 고유 U+0020 advance. 저장 `LINE_SEG` 와의 호환이 걸려 있는 기본값이다.
    Stored,
    /// 한컴이 폭 변경 뒤 다시 저장할 때 쓰는 공백 폭 (legacy bullet 계열).
    HancomRegenerated,
    /// [#3128] 들여쓴 셀 문단의 반각 칸.
    ///
    /// 한컴은 이 계급에서 글꼴 고유 U+0020 폭이 반각보다 넓어도 선행 들여쓰기와
    /// 재조판된 내부 공백을 **0.5em 칸**으로 잰다. 그 규칙이 없으면 프레임이 셀
    /// 안에서 한컴보다 넓게 재고, 줄이 밀려 셀이 쪽 밖으로 자란다.
    HalfCell,
}

impl SpaceMetric {
    /// 이 규칙에서 공백 하나의 advance.
    fn space_advance(self, style: &crate::renderer::TextStyle) -> f64 {
        match self {
            Self::Stored => estimate_text_width_unrounded(" ", style),
            Self::HancomRegenerated => hancom_regenerated_space_width(style)
                .unwrap_or_else(|| estimate_text_width_unrounded(" ", style)),
            Self::HalfCell => super::regenerated_half_space_width(style),
        }
    }
}

/// `space_metric` 은 공백 advance 규칙이다. 일반 HWP/HWPX tokenization 은 저장
/// `LINE_SEG` 호환성을 위해 [`SpaceMetric::Stored`] 를 쓴다.
fn tokenize_paragraph_with_regenerated_space_metric(
    text_chars: &[char],
    char_offsets: &[u32],
    char_shapes: &[CharShapeRef],
    styles: &ResolvedStyleSet,
    english_break_unit: u8,
    korean_break_unit: u8,
    space_metric: SpaceMetric,
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
            let w = space_metric.space_advance(&ts) + inline_width_px_at(inline_controls, i);
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
                        base_width: width,
                        width,
                        max_font_size: max_fs,
                        base_char_widths: char_widths.clone(),
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
                    base_width: w,
                    width: w,
                    max_font_size: fs,
                    base_char_widths: vec![],
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
                        base_width: width,
                        width,
                        max_font_size: max_fs,
                        base_char_widths: cw.clone(),
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
                    base_width: w,
                    width: w,
                    max_font_size: fs,
                    base_char_widths: vec![],
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
                base_width: w,
                width: w,
                max_font_size: fs,
                base_char_widths: vec![],
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
                base_width: w,
                width: w,
                max_font_size: fs,
                base_char_widths: vec![],
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

/// 기존 scalar tokenization과 같은 문자별 base pen을 만든 뒤, 그 입력 전체를
/// transaction-shared paragraph measurement cache에 건넨다.
fn prepare_paragraph_projection(
    para: &Paragraph,
    text_chars: &[char],
    styles: &ResolvedStyleSet,
    space_metric: SpaceMetric,
    inline_controls: &[FlowInlineControl],
) -> Option<PreparedParagraphProjection> {
    if text_chars.is_empty() {
        return None;
    }
    let mut current_lang = 0usize;
    let mut base_positions = Vec::with_capacity(text_chars.len().saturating_add(1));
    let mut kerning_scalar_styles = Vec::with_capacity(text_chars.len());
    let mut shaping_scalar_styles = Vec::with_capacity(text_chars.len());
    let mut hard_boundaries = vec![false; text_chars.len().saturating_add(1)];
    let mut pen = 0.0f64;
    base_positions.push(pen);

    for (index, character) in text_chars.iter().copied().enumerate() {
        let language_index = if is_lang_neutral(character) {
            current_lang
        } else {
            let detected = detect_lang_category(character);
            current_lang = detected;
            detected
        };
        let utf16_pos = para
            .char_offsets
            .get(index)
            .copied()
            .unwrap_or(index as u32);
        let char_shape_id = find_active_char_shape(&para.char_shapes, utf16_pos);
        let text_style = resolved_to_text_style(styles, char_shape_id, language_index);
        let base_font_size = if text_style.font_size > 0.0 {
            text_style.font_size
        } else {
            12.0
        };
        let effective_font_size_px = if text_style.superscript || text_style.subscript {
            base_font_size * crate::renderer::SCRIPT_FONT_SCALE
        } else {
            base_font_size
        };
        let width_ratio = if text_style.ratio > 0.0 {
            text_style.ratio
        } else {
            1.0
        };
        let slot = crate::renderer::kerning::ExactFontSlot::new(char_shape_id, language_index);
        kerning_scalar_styles.push(crate::renderer::kerning::KerningParagraphScalarStyle {
            slot,
            requested: text_style.kerning,
            effective_font_size_px,
            width_ratio,
        });
        shaping_scalar_styles.push(
            crate::renderer::shaping_paragraph::HorizontalShapingParagraphScalarStyle {
                slot,
                effective_font_size_px,
                width_ratio,
                letter_spacing_px: text_style.letter_spacing,
                kerning: text_style.kerning,
                bold: text_style.bold,
                italic: text_style.italic,
                superscript: text_style.superscript,
                subscript: text_style.subscript,
            },
        );

        let scalar_width = match character {
            '\t' | '\n' | '\r' => 0.0,
            ' ' => space_metric.space_advance(&text_style),
            _ => estimate_text_width_unrounded(&character.to_string(), &text_style),
        } + inline_width_px_at(inline_controls, index);
        pen += scalar_width.max(0.0);
        base_positions.push(pen);
    }

    for control in inline_controls {
        let boundary = control.char_position.min(text_chars.len());
        hard_boundaries[boundary] = true;
        if boundary < text_chars.len() {
            hard_boundaries[boundary + 1] = true;
        }
    }
    Some(PreparedParagraphProjection {
        base_positions,
        kerning_scalar_styles,
        shaping_scalar_styles,
        hard_boundaries,
    })
}

fn prepare_paragraph_kerning(
    para: &Paragraph,
    text_chars: &[char],
    styles: &ResolvedStyleSet,
    space_metric: SpaceMetric,
    inline_controls: &[FlowInlineControl],
) -> Option<PreparedParagraphKerning> {
    let paragraph_requests_kerning = if para.char_shapes.is_empty() {
        styles
            .char_styles
            .first()
            .is_some_and(|style| style.kerning)
    } else {
        para.char_shapes.iter().any(|shape| {
            styles
                .char_styles
                .get(shape.char_shape_id as usize)
                .is_some_and(|style| style.kerning)
        })
    };
    if !paragraph_requests_kerning {
        return None;
    }
    let context = styles.kerning_measurement_context.as_ref()?.clone();
    let projection =
        prepare_paragraph_projection(para, text_chars, styles, space_metric, inline_controls)?;
    if !projection
        .kerning_scalar_styles
        .iter()
        .any(|style| style.requested)
    {
        return None;
    }
    let measurement = context.paragraph_measurement(
        &para.text,
        projection.base_positions,
        &projection.kerning_scalar_styles,
        &projection.hard_boundaries,
    );
    if measurement.disposition
        != crate::renderer::kerning::KerningParagraphMeasurementDisposition::PairAdjusted
    {
        return None;
    }

    Some(PreparedParagraphKerning {
        context,
        scalar_styles: projection.kerning_scalar_styles,
        hard_boundaries: projection.hard_boundaries,
        measurement,
    })
}

fn has_rtl_or_bidi_control(text: &str) -> bool {
    text.chars().any(|character| {
        matches!(
            character as u32,
            0x0590..=0x08FF
                | 0xFB1D..=0xFDFF
                | 0xFE70..=0xFEFF
                | 0x10800..=0x10FFF
                | 0x1E800..=0x1EEFF
                | 0x200E..=0x200F
                | 0x202A..=0x202E
                | 0x2066..=0x2069
        )
    })
}

/// Run Q2-C only at the opt-in composition boundary and retain the exact Arc
/// returned by the transaction.  Every malformed or unsupported input maps to
/// `None`; the legacy `ComposedParagraph` fields are never rewritten here.
pub(super) fn compose_horizontal_shaping_handoff(
    para: &Paragraph,
    composed: &ComposedParagraph,
    styles: &ResolvedStyleSet,
) -> Option<std::sync::Arc<crate::renderer::shaping_paragraph::HorizontalShapingLineOutcome>> {
    let kerning_context = styles.kerning_measurement_context.as_ref()?;
    let shaping_context = styles.horizontal_shaping_context.as_ref()?;
    if kerning_context.registry_generation() != shaping_context.registry_generation() {
        return None;
    }
    let q2_candidate =
        crate::renderer::shaping_paragraph::is_bounded_horizontal_shaping_candidate_text(
            &para.text,
        );
    if !q2_candidate && shaping_context.instance_request_count() == 0 {
        return None;
    }

    let text_chars = para.text.chars().collect::<Vec<_>>();
    let inline_controls = flow_inline_controls(para);
    let projection = prepare_paragraph_projection(
        para,
        &text_chars,
        styles,
        SpaceMetric::Stored,
        &inline_controls,
    )?;
    let kerning_measurement = kerning_context.paragraph_measurement(
        &para.text,
        projection.base_positions.clone(),
        &projection.kerning_scalar_styles,
        &projection.hard_boundaries,
    );
    let (fallback_positions, fallback_owner) = if kerning_measurement.disposition
        == crate::renderer::kerning::KerningParagraphMeasurementDisposition::PairAdjusted
    {
        (
            kerning_measurement.positions(),
            crate::renderer::shaping_paragraph::HorizontalShapingFallbackOwner::W9K1,
        )
    } else {
        (
            projection.base_positions.as_slice(),
            crate::renderer::shaping_paragraph::HorizontalShapingFallbackOwner::ExistingK0,
        )
    };

    let code_point_count = text_chars.len();
    let mut candidate_boundaries = composed
        .lines
        .iter()
        .map(|line| line.char_start)
        .chain(std::iter::once(code_point_count))
        .collect::<Vec<_>>();
    candidate_boundaries.sort_unstable();
    candidate_boundaries.dedup();
    if candidate_boundaries
        .iter()
        .any(|index| *index > code_point_count)
    {
        return None;
    }
    let available_widths_px = composed
        .lines
        .iter()
        .map(|line| hwpunit_to_px(line.segment_width, 96.0))
        .collect::<Vec<_>>();
    if available_widths_px.is_empty()
        || available_widths_px
            .iter()
            .any(|width| !width.is_finite() || *width <= 0.0)
    {
        return None;
    }

    let composed_text = composed
        .lines
        .iter()
        .flat_map(|line| line.runs.iter())
        .map(|run| run.text.as_str())
        .collect::<String>();
    let has_char_overlap = composed
        .lines
        .iter()
        .flat_map(|line| line.runs.iter())
        .any(|run| run.char_overlap.is_some());
    let para_style = styles.para_styles.get(para.para_shape_id as usize);
    let paragraph = crate::renderer::shaping_paragraph::HorizontalShapingParagraphRequest {
        attempt_id_base: 1,
        text: &para.text,
        fallback_positions,
        scalar_styles: &projection.shaping_scalar_styles,
        hard_boundaries: &projection.hard_boundaries,
        fallback_owner,
        model_text_matches_shaping_text: composed_text == para.text,
        horizontal_ltr_bidi0: !has_rtl_or_bidi_control(&para.text),
        condense_min_space: para_style
            .map(|style| style.condense_min_space)
            .unwrap_or(0),
        has_inline_controls: !para.controls.is_empty(),
        has_tabs: para.text.contains('\t'),
        has_rotation: false,
        has_char_overlap,
    };
    let request = crate::renderer::shaping_paragraph::HorizontalShapingLineRequest {
        paragraph,
        candidate_boundaries: &candidate_boundaries,
        available_widths_px: &available_widths_px,
    };
    let outcome = if q2_candidate {
        let mut transaction = shaping_context.transaction();
        std::sync::Arc::new(
            crate::renderer::shaping_paragraph::run_horizontal_shaping_line_transaction(
                &mut transaction,
                &request,
            ),
        )
    } else {
        let line = composed.lines.first()?;
        let run = line.runs.first()?;
        if composed.lines.len() != 1
            || line.runs.len() != 1
            || line.has_line_break
            || line.char_start != 0
            || run.text != para.text
            || run.display_text.is_some()
        {
            return None;
        }
        let slot = projection.shaping_scalar_styles.first()?.slot;
        let mut transaction = shaping_context.explicit_instance_transaction(slot).ok()?;
        // An explicit default remains distinct in the request registry and
        // shaping cache, but must not widen the product activation surface.
        // Returning to the existing composer here preserves Q2 pixels and
        // geometry; only a canonical non-default instance may publish the
        // Q3-E portable pair.
        if transaction.is_default_instance() {
            return None;
        }
        if !crate::renderer::shaping_paragraph::is_bounded_explicit_instance_candidate_text(
            &para.text,
        ) {
            return None;
        }
        std::sync::Arc::new(
            crate::renderer::shaping_paragraph::run_bounded_explicit_instance_line_transaction(
                &mut transaction,
                &request,
            ),
        )
    };
    crate::renderer::shaping_composition::retain_qualified_horizontal_shaping_outcome(outcome)
}

fn apply_paragraph_kerning_to_tokens(
    tokens: &mut [BreakToken],
    measurement: &crate::renderer::kerning::KerningParagraphMeasurement,
) -> Option<()> {
    for token in tokens {
        let BreakToken::Text {
            start_idx,
            end_idx,
            width,
            char_widths,
            ..
        } = token
        else {
            continue;
        };
        *width = measurement.range_width(*start_idx, *end_idx)?;
        *char_widths = (*start_idx..*end_idx)
            .map(|index| measurement.range_width(index, index + 1))
            .collect::<Option<Vec<_>>>()?;
    }
    Some(())
}

/// px를 HWPUNIT(i32)로 변환 (내림, DPI=96 기준: px * 75)
///
/// `pub` 인 이유는 `tests/cases/issue_5678_fit_test_letter_spacing_trim.rs` 가 fit 판정 계약을
/// 밖에서 구속하기 때문이지, rhwp 의 API 라는 뜻이 아니다. `#[doc(hidden)]` 이 그 사실을 적는다.
#[doc(hidden)]
#[inline]
pub fn to_hwp(px: f64) -> i32 {
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
#[doc(hidden)]
pub const LINE_BREAK_TOLERANCE: i32 = 15;

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

/// Letter spacing is excluded from the fit test and included in the pen.
///
/// The pen already carries every earlier glyph's letter spacing. The fit
/// comparison omits only the candidate token's trailing letter space—one per
/// candidate, not one per character.
///
/// Dropping it from every character instead — the reading a summary of this
/// divergence invites — is arithmetically the single-shape blindness of
/// `compose_lines`' NO_LS fallback, which measures a paragraph with
/// `char_shapes[0]`'s spacing throughout. Measured, it reproduced the retired
/// cell wrap character for character against Hancom's own PDF and cost 12 tests
/// across six unrelated fixture families. At this stage the suite is unchanged.
///
/// Negative spacing is legal, so the sign is not fixed. For non-negative 자간
/// the candidate becomes narrower; under compressed 자간 the omission makes
/// the candidate wider and the fit stricter (`76076_regulatory_analysis` runs
/// `-0.16…-1.76` px).
/// Forced to 0 under an active character grid, which is inert here: every
/// corpus section has `char_grid == 0`.
#[doc(hidden)]
pub fn fit_test_letter_spacing_trim_hwp(letter_spacing_px: &[f64], token_end_idx: usize) -> i32 {
    if token_end_idx == 0 {
        return 0;
    }
    letter_spacing_px
        .get(token_end_idx - 1)
        .map(|spacing| to_hwp(*spacing))
        .unwrap_or(0)
}

/// Resolved per-character letter spacing in px, indexed by character position.
///
/// The fill needs the candidate glyph's own spacing at the moment it tests a
/// token, and the token carries only a total width, so it is resolved once for
/// the paragraph rather than re-derived per test.
fn resolved_letter_spacing_px(
    text_chars: &[char],
    char_offsets: &[u32],
    char_shapes: &[CharShapeRef],
    styles: &ResolvedStyleSet,
) -> Vec<f64> {
    let mut lang = 0usize;
    text_chars
        .iter()
        .enumerate()
        .map(|(idx, ch)| {
            if !is_lang_neutral(*ch) {
                lang = detect_lang_category(*ch);
            }
            let utf16_pos = char_offsets.get(idx).copied().unwrap_or(idx as u32);
            let style_id = find_active_char_shape(char_shapes, utf16_pos);
            // [#5678] `TextStyle` 을 통째로 만들지 않고 자간만 읽는다.
            resolved_letter_spacing(styles, style_id, lang)
        })
        .collect()
}

/// fit test 에 쓰는 후보 폭. 펜이 전진하는 폭과 **다른 값**이다.
///
/// [#5678] 종전에는 두 값이 다 `i32` 라 호출부가 아무거나 넘길 수 있었고, 실제로
/// 한 자리만 자간을 뺀 값을 넘겼다. 자간이 0 인 문단에서는 두 값이 같아 어떤 테스트도
/// 차이를 잡지 못했다. 이제 원시 정수는 이 함수에 들어가지 못하고, 호출부는 생성자
/// 이름으로 어느 쪽인지 밝혀야 한다.
///
/// `pub` 이지만 내부 필드는 private 이다 — 원시 정수가 생성자를 우회하지 못한다는 위 계약이
/// 밖에서도 그대로 선다. `#[doc(hidden)]` 은 이것이 rhwp 의 API 가 아님을 적는다.
#[doc(hidden)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FitWidthHwp(i32);

impl FitWidthHwp {
    /// fit 판정 폭을 HWPUNIT 으로 읽는다. 시험이 값을 확인하는 유일한 통로다.
    #[inline]
    pub fn hwp(self) -> i32 {
        self.0
    }

    /// 자간 보정 없이 fit 판정 폭을 만든다 (자간이 0 인 문단·대조군용).
    #[inline]
    pub fn untrimmed(token_width_hwp: i32) -> Self {
        Self(token_width_hwp)
    }

    /// 후보 토큰의 마지막 글자 뒤 자간을 뺀 폭. 실사용 fill 이 쓰는 값이다.
    ///
    /// 줄 끝에 오는 글자의 뒤 자간은 그려지지 않으므로 들어가는지 따질 때 빼고 잰다.
    /// 펜은 전체 폭만큼 전진한다.
    pub fn trimmed(token_width_hwp: i32, letter_spacing_px: &[f64], token_end_idx: usize) -> Self {
        Self(token_width_hwp - fit_test_letter_spacing_trim_hwp(letter_spacing_px, token_end_idx))
    }

    /// 커닝 경계쌍 보정을 fit 판정 폭에 더한다 (#4439 커닝 세션과의 병합점).
    /// 펜 전진 폭에는 더하지 않는다 — fit 판정 전용 축이다.
    pub fn with_pair_adjustment(self, adjustment_hwp: i32) -> Self {
        Self(self.0 + adjustment_hwp)
    }
}

#[doc(hidden)]
pub fn text_token_fits_line_hwp(
    current_width_hwp: i32,
    token_width: FitWidthHwp,
    space_savings_hwp: i32,
    effective_width_hwp: i32,
    max_font_size: f64,
) -> bool {
    let natural_candidate = current_width_hwp + token_width.0;
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
    letter_spacing_px: &[f64],
    initial_start_idx: usize,
    initial_is_first_line: bool,
    mut kerning: Option<&mut crate::renderer::kerning::KerningParagraphBreakSession<'_, '_, '_>>,
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
        letter_spacing_px,
        &mut cursor,
        kerning.as_deref_mut(),
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
    letter_spacing_px: &[f64],
    cursor: &mut FillCursor,
    mut kerning: Option<&mut crate::renderer::kerning::KerningParagraphBreakSession<'_, '_, '_>>,
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
                base_width,
                max_font_size,
                base_char_widths,
                ..
            } => {
                if let Some(next_char_idx) = cursor.fallback_char_idx {
                    debug_assert!(*start_idx <= next_char_idx && next_char_idx <= *end_idx);
                    let mut ci = next_char_idx;
                    while ci < *end_idx {
                        let rel_idx = ci - *start_idx;
                        let char_w = base_char_widths
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
                        let candidate_base = cursor.lw + char_w;
                        let candidate_width = if let Some(session) = kerning.as_deref_mut() {
                            candidate_base
                                + to_hwp(
                                    session
                                        .boundary_pair_adjustment(cursor.line_start_idx, ci + 1)?,
                                )
                        } else {
                            candidate_base
                        };
                        if candidate_width > current_width && ci > cursor.line_start_idx {
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

                let w_hwp = to_hwp(*base_width);
                let effective_width = eff_w(cursor.is_first_line);
                // The pen keeps the full width; only the comparison drops the
                // candidate's trailing letter space.
                let w_hwp_fit = FitWidthHwp::trimmed(w_hwp, letter_spacing_px, *end_idx);
                let pair_adjustment_hwp = if let Some(session) = kerning.as_deref_mut() {
                    to_hwp(session.boundary_pair_adjustment(cursor.line_start_idx, *end_idx)?)
                } else {
                    0
                };
                let token_fits = text_token_fits_line_hwp(
                    cursor.lw,
                    w_hwp_fit.with_pair_adjustment(pair_adjustment_hwp),
                    cursor.line_space_savings,
                    effective_width,
                    *max_font_size,
                );

                // 단일 문자 CJK/한글 토큰의 줄바꿈 가능 지점 처리
                // 이 글자를 포함한 후 break point 갱신 (end_idx 사용)
                // → 초과 시 이 글자까지 L0에 포함하고 다음 토큰부터 다음 줄
                //
                // **One predicate governs.** This used to ask its own question —
                // the condensed width against the box, without
                // `condense_pull_allowed` — so a glyph the fit test refused could
                // still be registered as a legal line end, and the emitted row
                // runs to the registered point, which put the refused glyph on
                // the line anyway. Measured on `76076_regulatory_analysis` p81
                // (`직접비용 근거설명`, box 36572 HWPUNIT): the fit test refuses
                // at pen 35566 + 1400 = 36966 > 36572, while the recorder
                // admitted the same glyph on 36966 − 980 = 35708 ≤ 36587. Sharing
                // `text_token_fits_line_hwp` makes all six rows of that cell
                // character-identical to the HWP 2024 PDF.
                if *end_idx - *start_idx == 1 && *start_idx > cursor.line_start_idx && token_fits {
                    let c = text_chars[*start_idx];
                    let allow_break = if is_hangul(c) {
                        // [#2185] bit7=1 = 글자 단위 break 허용 (위 주석 참조)
                        korean_break_unit == 1
                    } else {
                        is_cjk_ideograph(c)
                    };
                    if allow_break {
                        cursor.last_break_token_idx = Some(ti);
                        cursor.last_break_char_idx = *end_idx; // 이 글자 다음 (이 글자 포함)
                        cursor.width_at_last_break = cursor.lw + w_hwp; // 이 글자 폭 포함
                        cursor.space_savings_at_last_break = cursor.line_space_savings;
                        cursor.fs_at_last_break = cursor.line_max_fs;
                    }
                }
                if !token_fits {
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
                            // [#5678] 주 판정과 같은 피연산자(`w_hwp_fit`)를 쓴다. 종전에는
                            // 이 자리만 `w_hwp` 를 넘겨, 같은 토큰이 한 반복 안에서 두 방식으로
                            // 측정됐다 — 자간이 0 이 아닌 문단에서만 갈리므로 어떤 테스트도
                            // 이 차이를 잡지 못했다. 펜은 여기서도 전체 폭을 그대로 전진한다.
                            if text_token_fits_line_hwp(
                                cursor.lw,
                                w_hwp_fit.with_pair_adjustment(
                                    if let Some(session) = kerning.as_deref_mut() {
                                        to_hwp(session.boundary_pair_adjustment(
                                            cursor.line_start_idx,
                                            *end_idx,
                                        )?)
                                    } else {
                                        0
                                    },
                                ),
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
                ..
            } => {
                if *max_font_size > line_max_fs {
                    line_max_fs = *max_font_size;
                }

                let w_hwp = to_hwp(*width);

                // 단일 문자 CJK/한글 토큰의 줄바꿈 가능 지점 처리
                // 이 글자를 포함한 후 break point 갱신 (end_idx 사용)
                // → 초과 시 이 글자까지 L0에 포함하고 다음 토큰부터 다음 줄
                let effective_width = eff_w(is_first_line);
                let token_fits = text_token_fits_line_hwp(
                    lw,
                    FitWidthHwp(w_hwp),
                    line_space_savings,
                    effective_width,
                    *max_font_size,
                );
                // Same single predicate as the live fill — see the note there.
                if *end_idx - *start_idx == 1 && *start_idx > line_start_idx && token_fits {
                    let c = text_chars[*start_idx];
                    let allow_break = if is_hangul(c) {
                        // [#2185] bit7=1 = 글자 단위 break 허용 (위 주석 참조)
                        korean_break_unit == 1
                    } else {
                        is_cjk_ideograph(c)
                    };
                    if allow_break {
                        last_break_token_idx = Some(ti);
                        last_break_char_idx = *end_idx; // 이 글자 다음 (이 글자 포함)
                        width_at_last_break = lw + w_hwp; // 이 글자 폭 포함
                        space_savings_at_last_break = line_space_savings;
                        fs_at_last_break = line_max_fs;
                    }
                }
                if !text_token_fits_line_hwp(
                    lw,
                    FitWidthHwp(w_hwp),
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
                                FitWidthHwp(w_hwp),
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
                start_idx,
                base_width,
                ..
            } if *start_idx >= new_line_start => {
                w += to_hwp(*base_width);
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
    line_max_fs: f64,
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
            Control::Form(form) if form.common.treat_as_char => Some(form.height as i32),
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
            // [#5785 후속] 선언 폭 우선 — 원시 열 합은 행별 구획이 다른 표에서 과대집계된다.
            let width = table.flow_width_hu() as i32;
            (width, table.common.height as i32)
        }
        Control::Equation(eq) if eq.common.treat_as_char => {
            (eq.common.width as i32, eq.common.height as i32)
        }
        Control::Form(form) if form.common.treat_as_char => (form.width as i32, form.height as i32),
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

/// Controls that claim no inline advance and own no layout box of their own.
///
/// Structural section/column markers qualify because the caller's current
/// Frame already embodies their resolved page/column geometry. A field
/// (`ClickHere` and friends) qualifies for a different reason that reaches the
/// same place: it is a *marker pair* around ordinary paragraph text.
/// `inline_control_size_hwp` returns `None` for it, so `flow_inline_controls`
/// never emits a token for it and the fill measures it as nothing.
///
/// Fields have to be in this set, because the alternative is that a body
/// paragraph carrying one has **no layout owner at all**. That is the state
/// `hwp_doc_fill_fields` leaves behind: `set_field_text_at`
/// (`queries/field_query.rs`) replaces the field's text and shifts the stored
/// `LineSeg` offsets, but never calls `reflow_line_segs`, so a paragraph that
/// went from 9 bytes to 5,109 still carries the one record that described the
/// empty form. Nothing downstream re-wraps it if the frame declines to look.
/// **A property, not a list.** This used to enumerate three variants, and the
/// enumeration was the defect: `Bookmark`, `PageNumberPos`, `PageHide`,
/// `HiddenComment` and HWP3's `Hyperlink` are every bit as width-neutral as a
/// `Field`, and a body paragraph carrying one had no layout owner at all — it
/// fell to `compose_lines`' 45-character heuristic with no stored record, or
/// got no repair with a stale one. HWP5/HWPX hyperlinks escaped only by an
/// accident of parsing: `%hlk` becomes a `Control::Field` through a prefix
/// test. Fields were added to the list for exactly this reason once already;
/// the class was the thing to fix.
///
/// The property is: **contributes no inline width, and owns no layout box.**
///
/// - No inline width is `inline_control_size_hwp` returning `None` — the same
///   oracle `flow_inline_controls` uses, so a control the fill would measure as
///   nothing is a control the frame may ignore. That is one source of truth
///   rather than two lists that must be kept in step.
/// - No layout box of its own is the closed set below. It cannot come from the
///   width oracle, because a *floating* Picture, Shape or Table also returns
///   `None` there — they contribute no inline width precisely because they are
///   laid out elsewhere. A body frame carries no exclusion for them
///   (`models_exclusions()` is false), so it must decline and leave them with
///   their established owner; that is what `layout_picture_band` exists for.
///
/// Enumerating the controls that *have* geometry rather than the ones that do
/// not also fails in the safer direction. A control variant nobody has
/// classified is laid out as ordinary text by the frame, which is at worst the
/// same treatment its text would get anyway — where the previous shape left the
/// whole paragraph unowned.
fn control_is_width_neutral_marker(control: &Control) -> bool {
    inline_control_size_hwp(control).is_none() && !control_owns_a_layout_box(control)
}

/// Controls that resolve their own geometry, inline or floating, and therefore
/// have a layout owner that is not this frame.
///
/// `Header`/`Footer`/`Footnote`/`Endnote` are nested flows with their own
/// frames; the rest carry a `CommonObjAttr` box. `Footnote`/`Endnote` also
/// occupy a flow slot (`Control::is_logical_inline`).
fn control_owns_a_layout_box(control: &Control) -> bool {
    matches!(
        control,
        Control::Table(_)
            | Control::Shape(_)
            | Control::Picture(_)
            | Control::Equation(_)
            | Control::Form(_)
            | Control::Header(_)
            | Control::Footer(_)
            | Control::Footnote(_)
            | Control::Endnote(_)
    )
}

/// [#6102] 자리차지(TopAndBottom) 비-TAC 표는 자기 레이아웃 상자를 갖지만
/// **줄 폭**은 소비하지 않는다 — 세로 밴드를 예약할 뿐 본문 프레임의 가로
/// 줄바꿈에는 폭-중립이다. 결재문서본문 계열(구역 첫 문단 + 자리차지 표
/// host)의 저장 `textpos` 가 실폭보다 길게 적혀 있을 때, 프레임이 표 컨트롤
/// 때문에 사양하면 검증(admission)조차 못 해 본문 첫 줄이 우단 밖까지
/// 그려진다(36360328 +75px — 한글 2020 은 재래핑한다). 어울림(Square)은
/// 배제 밴드가 줄 폭을 바꾸므로, 그림/도형 float 는 별도 계보
/// (`layout_picture_band`)가 소유하므로 계속 사양한다.
fn control_is_line_width_neutral_float_table(control: &Control) -> bool {
    matches!(control, Control::Table(t)
        if !t.common.treat_as_char
            && matches!(t.common.text_wrap, crate::model::shape::TextWrap::TopAndBottom))
}

pub(super) fn supports_cached_body_frame_controls(para: &Paragraph) -> bool {
    para.controls.iter().all(|control| {
        control_is_width_neutral_marker(control)
            || control_is_line_width_neutral_float_table(control)
    })
}

/// The picture-band frame intentionally admits only its floating host, the
/// already-supported treat-as-character Equation flow, and width-neutral
/// markers. Other controls have their own layout owners and must leave this
/// transaction untouched.
fn supports_picture_band_frame_controls(para: &Paragraph) -> bool {
    let mut non_tac_pictures = 0usize;
    for control in &para.controls {
        match control {
            Control::Picture(picture) if !picture.common.treat_as_char => {
                non_tac_pictures += 1;
            }
            Control::Equation(equation) if equation.common.treat_as_char => {}
            other if control_is_width_neutral_marker(other) => {}
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

/// `baseline_distance` from a row's height — the one expression, so the frame
/// and the edit path cannot publish different baselines for the same paragraph.
///
/// They did: `frame_metrics_for_line` rounded while `make_line_seg` truncated,
/// and `make_line_seg`'s output is written to the file. Any half-point font
/// size split them — 10.5pt gives `1050 * 0.85 = 892.5`, so one published 893
/// and the other 892.
fn baseline_distance_hwp(line_height_hwp: i32) -> i32 {
    (line_height_hwp as f64 * 0.85).round() as i32
}

fn apply_inline_control_line_height(seg: &mut LineSeg, height_hwp: i32) {
    if height_hwp > seg.line_height {
        seg.line_height = height_hwp;
        seg.text_height = height_hwp;
        seg.baseline_distance = baseline_distance_hwp(height_hwp);
    }
}

fn apply_inline_control_frame_height(metrics: &mut FrameRowMetrics, height_hwp: i32) {
    if height_hwp > metrics.line_height {
        metrics.line_height = height_hwp;
        metrics.text_height = height_hwp;
        metrics.baseline_distance = baseline_distance_hwp(height_hwp);
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
        baseline_distance: baseline_distance_hwp(line_height),
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
    layout_paragraph_in_frame_impl(para, frame, styles, dpi, true)
}

fn layout_paragraph_in_frame_impl(
    para: &Paragraph,
    frame: &mut LayoutFrame,
    styles: &ResolvedStyleSet,
    dpi: f64,
    allow_kerning: bool,
) -> Option<Vec<LineSeg>> {
    // [#6102] 폭-중립 자리차지 표 host 도 fill 대상 — 표는 줄 폭을 소비하지
    // 않으므로(자기 레이아웃 소유자가 따로 배치) 텍스트만 재래핑하면 된다.
    if !supports_picture_band_frame_controls(para) && !supports_cached_body_frame_controls(para) {
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
    // [#3128] 프레임이 들여쓰기 추적을 잃던 자리. 종전에는 여기서 `false` 를 박아
    // 두어, 저장 `LINE_SEG` 가 없는 들여쓴 셀 문단도 글꼴 고유 공백 폭으로 쟀다.
    // 실측: `76076_regulatory_analysis.hwp` 에서 이 술어를 만족하는 문단이 74 개고,
    // 그 전부가 이 경로로 들어온다.
    let space_metric =
        if super::missing_lineseg_indented_cell_has_uniform_metrics_with_tracking(para, styles) {
            SpaceMetric::HalfCell
        } else {
            SpaceMetric::Stored
        };
    let mut tokens = tokenize_paragraph_with_regenerated_space_metric(
        &text_chars,
        &para.char_offsets,
        &para.char_shapes,
        styles,
        english_break_unit,
        korean_break_unit,
        space_metric,
        &inline_controls,
    );
    let prepared_kerning = allow_kerning
        .then(|| {
            prepare_paragraph_kerning(para, &text_chars, styles, space_metric, &inline_controls)
        })
        .flatten()
        .and_then(|prepared| {
            apply_paragraph_kerning_to_tokens(&mut tokens, &prepared.measurement).map(|_| prepared)
        });
    let mut kerning_transaction = prepared_kerning
        .as_ref()
        .map(|prepared| prepared.context.layout_session());
    let mut kerning_break_session = prepared_kerning.as_ref().and_then(|prepared| {
        crate::renderer::kerning::KerningParagraphBreakSession::new(
            &para.text,
            &prepared.scalar_styles,
            &prepared.hard_boundaries,
            &prepared.measurement,
            kerning_transaction.as_mut()?,
        )
        .ok()
    });
    let letter_spacing_px =
        resolved_letter_spacing_px(&text_chars, &para.char_offsets, &para.char_shapes, styles);
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
    // Fresh rows inherit only provenance. Page/column-first, empty, indent,
    // paragraph-head, and FIRST/LAST are properties of the newly projected
    // physical row and must not leak from the cached first row.
    let source_tag = para
        .line_segs
        .first()
        .map(|segment| segment.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY)
        .unwrap_or(LineSeg::TAG_IMPLEMENTATION_PROPERTY);
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
                // Asked before filling so a degenerate carve costs no
                // tokenization, but it is the same question
                // `commit_carved_row` asks — one rule, not two.
                if !frame.carved_row_is_usable() {
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
                        &letter_spacing_px,
                        &mut cursor,
                        kerning_break_session.as_mut(),
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

    let kerning_failed = kerning_break_session
        .as_ref()
        .and_then(|session| session.failed_reason())
        .is_some();
    if result.is_none() {
        frame.restore_checkpoint(frame_checkpoint);
    }
    drop(kerning_break_session);
    drop(kerning_transaction);
    if kerning_failed {
        // 한 boundary라도 예산/범위 검증에 실패하면 일부 K1 row를 게시하지
        // 않고 문단 전체를 원래 scalar transaction으로 다시 실행한다.
        return layout_paragraph_in_frame_impl(para, frame, styles, dpi, false);
    }
    result
}

/// The metrics for one stored physical row, computed — never read off the
/// record.
///
/// §1.4.1 recomputes `vertical_pos`, `line_height`, `text_height`,
/// `baseline_distance` and `line_spacing` on both arms and writes them back
/// over the stored values, so the frame must never take them from the file.
/// The row's span is its own `text_start` to the next row's, and its size is
/// the largest char shape active over that span.
///
/// `line_spacing` agrees with HWP's stored value on 98.53% of rows
/// (1,133,166 measured); the residual is §2.14's metricCtx settlement — three
/// independent ceilings — and is characterised, not modelled.
pub(crate) fn stored_row_metrics(
    para: &Paragraph,
    styles: &ResolvedStyleSet,
    dpi: f64,
    row: &[LineSeg],
) -> Option<FrameRowMetrics> {
    let para_style = styles.para_styles.get(para.para_shape_id as usize);
    let line_spacing_type = para_style
        .map(|style| style.line_spacing_type)
        .unwrap_or(LineSpacingType::Percent);
    let line_spacing_value = para_style.map(|style| style.line_spacing).unwrap_or(160.0);
    let fallback_font_size = para
        .char_shapes
        .first()
        .and_then(|shape| styles.char_styles.get(shape.char_shape_id as usize))
        .map(|style| style.font_size)
        .unwrap_or(12.0);

    let span_start = row.first()?.text_start;
    let row_end = row.last()?.text_start;
    let span_end = para
        .line_segs
        .iter()
        .map(|segment| segment.text_start)
        .filter(|start| *start > row_end)
        .min()
        .unwrap_or(u32::MAX);
    let active = para
        .char_shapes
        .iter()
        .filter(|shape| shape.start_pos <= span_start)
        .map(|shape| shape.start_pos)
        .max();
    let max_font_size = para
        .char_shapes
        .iter()
        .filter(|shape| {
            (shape.start_pos >= span_start && shape.start_pos < span_end)
                || Some(shape.start_pos) == active
        })
        .filter_map(|shape| styles.char_styles.get(shape.char_shape_id as usize))
        .map(|style| style.font_size)
        .fold(0.0f64, f64::max);

    Some(frame_metrics_for_line(
        max_font_size,
        fallback_font_size,
        line_spacing_type,
        line_spacing_value,
        dpi,
    ))
}

/// Resolve a paragraph's stored rows through the caller's frame.
///
/// The stored records are a cache, not frame inputs. The frame always computes
/// the physical-row key `(count, column_start, segment_width)` first and admits
/// the cache only on exact equality. Staleness is an additional invalidator: a
/// geometrically matching record that describes obsolete text is still rebuilt.
pub(crate) fn resolve_stored_line_segs_in_frame(
    para: &Paragraph,
    frame: &mut LayoutFrame,
    styles: &ResolvedStyleSet,
    dpi: f64,
    legacy_hwp3_stored_geometry: bool,
    miss_policy: StoredRowMissPolicy,
    stale: bool,
    // [#6175] 같은 세로 band의 용지/쪽 기준 어울림 개체 증거(HWPUNIT).
    // 저장 행의 결손 폭과 세로 위치가 함께 맞으면 좁음의 출처가 외부 기하다.
    float_carve_evidence: &[crate::renderer::float_placement::FloatCarveEvidence],
    known_square_band: bool,
) -> Option<StoredRowResolution> {
    // [#6102] 폭-중립 float 표 host 는 본문 프레임 게이트가 이미 통과시킨
    // 문단이다 — picture-band 게이트만으로 사양하면 저장 textpos 의
    // admission 검증이 통째로 빠져 결재문서본문 계열의 첫 줄이 우단 밖으로
    // 넘친다(36360328 +75px, 한글 2020 은 재래핑). 단, 이 계보는 **stale
    // (물리 위반) 증거가 있을 때만** 개입한다 — 증거 없이 admission 불일치
    // 만으로 재래핑하면 종전 무소유였던 float-host 문단의 확정 핀이 흔들린다.
    if !supports_picture_band_frame_controls(para) {
        if !supports_cached_body_frame_controls(para) || !stale {
            return None;
        }
    }

    // NO_LS: there is no stored record to compare, so this is the rebuild case
    // outright and belongs to the frame's fill. The fill tokenizes through
    // `para.char_shapes`, which is the char-shape re-splitting #2632 needed —
    // a mixed-char-shape paragraph must measure the way it renders.
    if para.line_segs.is_empty() {
        let mut fill_input = para.clone();
        fill_input.line_segs.clear();
        return layout_paragraph_in_frame(&fill_input, frame, styles, dpi)
            .map(|_| StoredRowResolution::Reflowed);
    }

    // HWP3-lineage stored rows use a legacy horizontal-origin lane that the
    // common ParaShape does not carry. The first sample16 mismatch is a
    // non-stale row stored at 2500..50024 while the common style resolves a
    // 5000..50024 Frame (margin 5000, hanging indent -2500). HWP3→HWP5 keeps
    // that stored origin too (#1892 round-trip: stored 0..42520 versus Frame
    // 1200..42520), so provenance — not the current container — owns the gate.
    // This is not a cache-key mismatch the common Frame can repair; it lacks
    // jurisdiction. Edited/stale text still takes the fresh fill below, and
    // NO_LS already took the Frame-owned branch above.
    if legacy_hwp3_stored_geometry && !stale {
        return None;
    }

    // #4755 §1 recomputes a FIRST..LAST split row as one physical row, and the
    // picture-band frame does exactly that because it holds the exclusion that
    // split it. A frame with no exclusions cannot: it carves one full-width
    // interval, so reflowing a split row there flattens it and destroys the
    // stored fragment geometry (#4690 `30098` p3 pi=48 stores `0..3402` and
    // `45305..48188` around a float; a full-width recompute loses the right
    // fragment entirely).
    //
    // An exclusion need not split one physical row into multiple slots. It can
    // leave one narrowed slot for several rows and then end, returning later
    // rows to the full frame width. #4090 pi=45 is exactly that shape: six
    // `0+26319` rows beside a Square table, followed by one `0+48188` row below
    // it. Different single-slot row extents therefore prove the same missing
    // per-band geometry as FIRST..LAST splitting. That is a limit of
    // jurisdiction, not a comparison tolerance — the frame declines rather
    // than claiming rows whose geometry it cannot compute, and the paragraph
    // stays with its established owner.
    //
    // This guard is live and load-bearing: it keeps cache admission and strict
    // reflow from flattening a split row when this frame lacks the exclusion
    // geometry that originally produced it. The multi-slot comparison becomes
    // live when a caller supplies those exclusions.
    if !frame.models_exclusions()
        && stored_rows_require_external_geometry(
            para,
            frame,
            float_carve_evidence,
            known_square_band,
        )
    {
        return None;
    }

    let entry_checkpoint = frame.clone();
    let cache_geometry_matches = stored_rows_reproduce_frame_expectation(para, frame, styles, dpi);
    if cache_geometry_matches && !stale {
        return Some(StoredRowResolution::Stored);
    }

    // Admission commits recomputed rows into the frame. A stale cache must not
    // leave those rows behind before the fresh fill starts. Rejection already
    // restores internally, so this is deliberately idempotent on that arm.
    frame.restore_checkpoint(entry_checkpoint);
    if !cache_geometry_matches
        && !stale
        && miss_policy == StoredRowMissPolicy::UnmodelledUnlessStale
    {
        return None;
    }
    if !cache_geometry_matches {
        report_stored_row_key_mismatch(para, frame, styles, dpi);
    }

    let mut reflow_input = para.clone();
    // A rejected row cannot remain the provenance template for fresh rows:
    // placement/empty flags describe the stored physical rows, not this
    // frame's newly carved projection.
    reflow_input.line_segs.clear();
    layout_paragraph_in_frame(&reflow_input, frame, styles, dpi)
        .map(|_| StoredRowResolution::Reflowed)
}

/// [#6175] 저장 행의 결손 폭이 어울림 개체의 흐름 폭과 같다고 볼 허용 오차(HWPUNIT).
///
/// 개체와 본문 사이의 간격(한컴이 넣는 여백)이 이 안에 든다 — 156655489 실측 284,
/// 156647303 실측 도 같은 자릿수다. 1200HU ≈ 16px 로, 글자 한 칸(약 1000HU)보다
/// 크지 않게 잡아 서로 다른 폭의 개체를 우연히 맞추지 않는다.
const FLOAT_CARVE_MATCH_TOLERANCE_HU: i32 = 1200;

/// A one-unit layout quantum is enough to distinguish a real narrowed lane
/// from a full-width row after integer projection. Keep this local to stored
/// row provenance rather than exposing layout-frame internals.
const UNIFORM_INSET_MIN_DELTA_HU: i32 = 4;

/// Whether stored physical rows prove that their frame changed by vertical
/// band and therefore required exclusion geometry.
///
/// A multi-slot FIRST..LAST row is direct evidence. A sequence of complete
/// single-slot rows with different horizontal extents is equivalent evidence:
/// a scalar frame has one immutable horizontal range, so it cannot produce the
/// transition without an exclusion entering or leaving the row band.
fn stored_rows_require_external_geometry(
    para: &Paragraph,
    frame: &LayoutFrame,
    float_carve_evidence: &[crate::renderer::float_placement::FloatCarveEvidence],
    known_square_band: bool,
) -> bool {
    let line_segs = &para.line_segs;
    let split_or_varying = line_segs
        .iter()
        .any(|segment| !segment.is_first_segment() || !segment.is_last_segment())
        || line_segs.windows(2).any(|pair| {
            pair[0].column_start != pair[1].column_start
                || pair[0].segment_width != pair[1].segment_width
        });
    if split_or_varying {
        return true;
    }

    // [#6175] 문단 전체가 개체 옆에 들어가면 폭 변화가 사라져 위 증거가
    // 소멸한다. 그때는 문서에 실재하는 어울림 개체가 증거다 — 저장 행이 남긴
    // 결손 폭을 그 개체의 흐름 폭이 설명하면, 좁음의 출처는 이 문단 자신이
    // 아니라 외부 기하다.
    //
    // ⚠ 이 판별자는 개체 폭과 **같은 세로 band**의 대조여야 한다. 균일하게 좁다는 것만으로는 문단
    // 테두리 박스의 inset 과 구별되지 않아 #547·#1440 핀이 깨진다(#6129 에서
    // 국소 판별자 2종이 그렇게 반증됐다). 셀에서는 #5818 이 같은 혼동을 "같은
    // 셀에 Square float 실재"로 이미 갈랐고, 이것은 그 계약의 본문 판이다.
    //
    // 156655489 1쪽 실측: 본문 폭 48188, 저장 cs=0·sw=26692 → 결손 21496.
    // 용지 기준 Square 그림 폭 21212 (offset 32361 → 프레임 좌표 26692) 로,
    // 저장 사다리의 끝이 개체 왼쪽 변과 단위까지 맞는다.
    if !float_carve_evidence.is_empty() && !line_segs.is_empty() {
        let uniform_narrow = line_segs.iter().all(|segment| {
            segment.column_start == frame.horizontal.start && segment.segment_width > 0
        }) && line_segs
            .windows(2)
            .all(|pair| pair[0].segment_width == pair[1].segment_width);
        if uniform_narrow {
            let occupied = line_segs[0]
                .column_start
                .saturating_add(line_segs[0].segment_width);
            let missing = frame.horizontal.end.saturating_sub(occupied);
            if missing > FLOAT_CARVE_MATCH_TOLERANCE_HU
                && float_carve_evidence.iter().any(|evidence| {
                    evidence.matches_stored_rows(missing, line_segs, FLOAT_CARVE_MATCH_TOLERANCE_HU)
                })
            {
                return true;
            }
        }
    }

    // Empty carrier rows can hold a narrowed origin supplied by a surrounding
    // legacy/wrap owner even when every physical row is one complete slot.
    // With no text and no control, this paragraph provides no local geometry
    // from which a scalar Frame could derive that inset. Reflowing it full
    // width destroys the carrier contract; leave it with the external owner.
    let differs_from_frame = |segment: &crate::model::paragraph::LineSeg| {
        segment.column_start != frame.horizontal.start
            || segment.column_start.checked_add(segment.segment_width) != Some(frame.horizontal.end)
    };
    let empty_carrier = para.controls.is_empty()
        && para
            .text
            .chars()
            .all(|ch| ch.is_whitespace() || ch == '\r' || ch == '\n');
    if empty_carrier {
        return line_segs.iter().any(differs_from_frame);
    }

    // A uniform inset by itself is not proof of an unmodelled exclusion: it
    // can also be ordinary paragraph indentation. Preserve it only when the
    // caller carries a real non-TAC Square Picture/Shape anchor. #6175's
    // GroupShape path validates the anchor and lane width before it sets this
    // flag; every ordinary body frame leaves the rows eligible for reflow.
    let inset_inside_frame = |segment: &crate::model::paragraph::LineSeg| {
        let end = match segment.column_start.checked_add(segment.segment_width) {
            Some(end) => end,
            None => return false,
        };
        segment.column_start >= frame.horizontal.start
            && end <= frame.horizontal.end
            && (frame.horizontal.end - frame.horizontal.start) - segment.segment_width
                > UNIFORM_INSET_MIN_DELTA_HU
    };
    known_square_band
        && line_segs.iter().any(differs_from_frame)
        && line_segs.iter().all(inset_inside_frame)
}

/// Whether the frame's own carve reproduces every stored row — §1.4.1's
/// `(count, horzpos, horzsize)` cache key, computed before reuse.
///
/// Takes the frame by `&mut` and leaves an admitted paragraph's rows committed
/// in it, because that is what makes the answer checkable — a caller that wants
/// the frame's version of an admitted paragraph can `project_line_segs()` it.
/// A rejection restores the entry checkpoint, so the frame is exactly as it was
/// handed in.
///
/// **One reachability limit, stated rather than implied.** The multi-slot
/// FIRST..LAST comparison inside `try_admit_stored_rows` has no production
/// caller and cannot acquire one until float exclusions are wired through:
/// `ParagraphBox::frame` builds every body frame with an empty exclusion list,
/// so `models_exclusions()` is false at the only production call site and the
/// guard above turns split rows away before the comparison sees them. The
/// picture-band frame does hold exclusions, but it clears `line_segs` and
/// reflows rather than comparing. Until a float set reaches a body frame, the
/// multi-slot path is exercised by unit tests only.
pub(crate) fn stored_rows_reproduce_frame_expectation(
    para: &Paragraph,
    frame: &mut LayoutFrame,
    styles: &ResolvedStyleSet,
    dpi: f64,
) -> bool {
    frame.try_admit_stored_rows(&para.line_segs, |row| {
        stored_row_metrics(para, styles, dpi, row)
    })
}

/// Report a cache-key mismatch already found by the production admission gate.
///
/// Off unless `RHWP_DIAG_STORED_ROW_KEY` is set, so the production path pays
/// one `var_os` and no allocation. Probes a clone: a report must not be able to
/// move the frame it is reporting on.
fn report_stored_row_key_mismatch(
    para: &Paragraph,
    frame: &LayoutFrame,
    styles: &ResolvedStyleSet,
    dpi: f64,
) {
    if std::env::var_os("RHWP_DIAG_STORED_ROW_KEY").is_none() {
        return;
    }
    let mut probe = frame.clone();
    if !stored_rows_reproduce_frame_expectation(para, &mut probe, styles, dpi) {
        eprintln!(
            "DIAG_STORED_ROW_KEY mismatch rows={} stored=[{}]",
            para.line_segs.len(),
            para.line_segs
                .iter()
                .map(|segment| format!("{}+{}", segment.column_start, segment.segment_width))
                .collect::<Vec<_>>()
                .join(" "),
        );
    }
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
///
/// Takes the column width in **pixels**, because that is what makes the
/// paragraph box here the same object the body path builds. It used to take
/// HWPUNIT and derive its own horizontal range as
/// `column_width_hwp - px_to_hwpunit(margin_right)`, where `ParagraphBox::body`
/// computes `px_to_hwpunit(column_width_px - margin_right_px)` — one truncation
/// against two. Those disagree by one HWPUNIT often enough to matter, and the
/// geometry pitch turns a one-unit disagreement into **four** whenever it
/// straddles a multiple of the pitch, because flooring `x - 1` where `x ≡ 0`
/// gives `x - 4`. Both routes publish `line_segs` that are persisted, so the
/// same paragraph could be written to disk with two different
/// `segment_width`s depending on whether a float was in its band.
///
/// The *column* box stays a local range rather than becoming a `ParagraphBox`:
/// float placement needs the column's own edges, unindented by paragraph
/// margins ([`picture_band_paragraph_reference`],
/// `float_placement::resolve_picture_exclusion`), and that is not what
/// `ParagraphBox` models. Stretching the type to cover both would put the
/// two coordinate systems back into one object, which is the confusion it
/// exists to prevent.
pub(crate) fn layout_picture_band(
    paragraphs: &[Paragraph],
    host_index: usize,
    column_width_px: f64,
    styles: &ResolvedStyleSet,
    dpi: f64,
) -> Option<PictureBandLayout> {
    let host = paragraphs.get(host_index)?;
    let column_horizontal = 0..px_to_hwpunit(column_width_px, dpi);
    let margins_for = |paragraph: &Paragraph| {
        let style = styles.para_styles.get(paragraph.para_shape_id as usize);
        (
            style.map(|value| value.margin_left).unwrap_or(0.0),
            style.map(|value| value.margin_right).unwrap_or(0.0),
        )
    };
    // `body_for_style`, like every other body route. An earlier revision used
    // bare `body` here on the argument that the list-origin blocker was a
    // separate decision — that was scope deferral, not a reason, and it left
    // this route publishing a different origin from the edit route for the same
    // list paragraph.
    let box_for = |paragraph: &Paragraph| {
        let paragraph_box = ParagraphBox::body_for_style(
            column_width_px,
            styles.para_styles.get(paragraph.para_shape_id as usize),
            dpi,
        );
        // Supersedes the old `(start < end)` test, and is strictly stronger:
        // that one ran before the geometry pitch, so a base of width 1..3 with
        // a misaligned left edge passed it and then inverted.
        paragraph_box.is_usable().then_some(paragraph_box)
    };
    let host_box = box_for(host)?;
    let host_horizontal = host_box.effective();
    let host_margin_left = px_to_hwpunit(margins_for(host).0, dpi);
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
    let mut anchor_frame = host_box.frame(0);
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
    let mut frame = host_box.frame_with(0, vec![exclusion]);
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
            || box_for(paragraph)?.effective() != host_horizontal
            || (paragraph_index != host_index
                && paragraph.controls.iter().any(|control| {
                    matches!(control, Control::Picture(picture) if !picture.common.treat_as_char)
                }))
        {
            return None;
        }

        let mut input = paragraph.clone();
        // A picture-band projection is freshly computed implementation state,
        // even when the source paragraph had authentic cached rows. Clearing
        // the cache prevents row-local flags from leaking and keeps vpos ladder
        // repair from treating the fresh zero-origin row as a saved reset.
        input.line_segs.clear();
        let paragraph_lines = layout_paragraph_in_frame(&input, &mut frame, styles, dpi)?;
        line_segs.push(paragraph_lines);
    }

    (!line_segs.is_empty() && frame.top >= exclusion_end).then_some(PictureBandLayout {
        paragraph_range: host_index..host_index + line_segs.len(),
        line_segs,
    })
}

/// 문단의 line_segs를 텍스트 내용과 **문단 상자**에 맞게 재계산한다.
///
/// 텍스트 편집(삽입/삭제) 후 호출하여 줄 바꿈을 재배치한다.
///
/// 인자는 폭이 아니라 [`ParagraphBox`] 다 — 호출자가 자기 좌표계를 말해야 한다.
/// 폭만 받던 종전 계약은 원점을 잃었고, 그 결과 본문 편집이 `column_start=0`,
/// `segment_width=가용폭` 을 발행해 HWP 의 저장 기록(열 기준 `column_start=여백`)
/// 과도, 렌더 프레임이 깎는 상자와도 어긋났다.
pub(crate) fn reflow_line_segs(
    para: &mut Paragraph,
    paragraph_box: ParagraphBox,
    styles: &ResolvedStyleSet,
    dpi: f64,
) {
    let _ = reflow_line_segs_impl(para, paragraph_box, styles, dpi, None, false);
}

/// 셀 분할로 저장 폭이 stale해진 문단을 다시 조판한다.
///
/// 한컴은 좁아진 셀에서만 본문 뒤의 inline control을 별도 source line으로 저장한다.
/// 이 규칙을 일반 reflow에 적용하면 원본 문서의 이미 권위적인 control host line까지
/// 분리되어 pagination이 달라진다 (#4138/#2424). 호출자는 split 직후 stale-cell
/// 복구 경로로 한정한다.
pub(crate) fn reflow_line_segs_after_cell_split(
    para: &mut Paragraph,
    paragraph_box: ParagraphBox,
    styles: &ResolvedStyleSet,
    dpi: f64,
) {
    let _ = reflow_line_segs_impl(para, paragraph_box, styles, dpi, None, true);
}

/// 저장 LINE_SEG가 유효한 셀 텍스트 편집은 수정된 줄 이전의 경계를 그대로 둔다.
///
/// 한컴은 중간 줄의 짧은 edit에서 문단 전체를 다시 나누지 않는다. prefix 경계를 다시
/// 계산하면 뒤 줄의 가용 폭이 인위적으로 커져 실제 HWP 저장본과 다른 다음 줄 전환을 만들 수
/// 있다. 단, prefix가 유효한 token 경계일 때만 보존하며, 합성 문단·첫 줄 edit·inline control은
/// 기존 full reflow로 안전하게 폴백한다.
pub(crate) fn reflow_line_segs_after_cell_text_edit(
    para: &mut Paragraph,
    paragraph_box: ParagraphBox,
    styles: &ResolvedStyleSet,
    dpi: f64,
    edit_char_offset: usize,
) -> bool {
    reflow_line_segs_impl(
        para,
        paragraph_box,
        styles,
        dpi,
        Some(edit_char_offset),
        false,
    )
}

fn reflow_line_segs_impl(
    para: &mut Paragraph,
    paragraph_box: ParagraphBox,
    styles: &ResolvedStyleSet,
    dpi: f64,
    preserve_prefix_for_edit: Option<usize>,
    split_stale_cell_reflow: bool,
) -> bool {
    // An impossible box publishes nothing, and this must be the first thing
    // that happens — before the memo invalidation below, which is already a
    // mutation.
    //
    // The render path has refused since it had a box to refuse
    // (`recompose_stored_lines_in_frame`); the edit path never did, so a
    // paragraph whose margins exceed its column reached `make_line_seg` with a
    // negative `seg_width_hwp` and published `segment_width < 0` on every row.
    // That is not a rendering artifact: `segment_width` goes to disk through
    // `serializer::body_text`'s `write_i32` and raw into the HWPX
    // `linesegarray`, so the corrupt extent is written to the file. Both paths
    // now ask `ParagraphBox::is_usable()`, which is where the reasoning for
    // refusing rather than flooring lives.
    //
    // `false` is the right return: it is the "prefix was not preserved" value,
    // so the one consumer that reads it (`reflow_cell_paragraph`) takes its
    // conservative branch.
    if !paragraph_box.is_usable() {
        return false;
    }
    // [#4149] 셀 편집의 단일 관문(reflow_cell_paragraph[_by_path])과 서식 적용
    // (formatting.rs) 이 모두 여기로 수렴한다 — 단일줄 과밀 memo 무효화.
    para.invalidate_single_line_overflow_memo();
    // [#4677] 줄을 다시 계산하면 이전에 붙여 둔 조판 전용 보강 줄은 사라진다 — 표식을
    // 남겨 두면 실제 줄을 저장에서 잘라 내게 된다.
    para.layout_only_fill_lines = 0;
    let orig = para.line_segs.first().cloned();
    // 상자 하나가 발행 기록과 프레임 둘 다의 출처다 — 종전엔 `segment_width` 를
    // 폭에서, 프레임 상자를 또 폭에서 따로 만들어 둘이 어긋날 수 있었다.
    let published_horizontal = paragraph_box.effective();
    let seg_width_hwp = paragraph_box.width_hwp();
    let available_width_px = paragraph_box.width_px(dpi);

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
        let baseline_distance_hwp = baseline_distance_hwp(line_height_hwp);
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
            column_start: published_horizontal.start,
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
            para.replace_line_segs(new_line_segs);
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
            para.replace_line_segs(vec![seg]);
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
    let reflow_space_metric = if split_stale_cell_reflow {
        SpaceMetric::HancomRegenerated
    } else {
        SpaceMetric::Stored
    };
    let mut tokens = tokenize_paragraph_with_regenerated_space_metric(
        &text_chars,
        &para.char_offsets,
        &para.char_shapes,
        styles,
        english_break_unit,
        korean_break_unit,
        // 종전 동작 보존: 이 인자가 곧 공백 규칙이었다.
        reflow_space_metric,
        &inline_controls,
    );
    let prepared_kerning = prepare_paragraph_kerning(
        para,
        &text_chars,
        styles,
        reflow_space_metric,
        &inline_controls,
    )
    .and_then(|prepared| {
        apply_paragraph_kerning_to_tokens(&mut tokens, &prepared.measurement).map(|_| prepared)
    });
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
        // 편집 프레임과 렌더 프레임이 이제 같은 상자에서 나온다 — 종전엔 이쪽만
        // `0..seg_width_hwp`(문단 기준, 미스냅)라 열 기준 렌더 프레임과 어긋났다.
        let mut frame =
            paragraph_box.frame(orig.as_ref().map(|line| line.vertical_pos).unwrap_or(0));
        if let Some(projected) = layout_paragraph_in_frame(para, &mut frame, styles, dpi) {
            para.replace_line_segs(projected);
            return false;
        }
    }

    let mut kerning_transaction = prepared_kerning
        .as_ref()
        .map(|prepared| prepared.context.layout_session());
    let mut kerning_break_session = prepared_kerning.as_ref().and_then(|prepared| {
        crate::renderer::kerning::KerningParagraphBreakSession::new(
            &para.text,
            &prepared.scalar_styles,
            &prepared.hard_boundaries,
            &prepared.measurement,
            kerning_transaction.as_mut()?,
        )
        .ok()
    });
    let mut line_breaks = fill_lines(
        &tokens[token_start..],
        &text_chars,
        available_width_px,
        indent_px,
        tab_width,
        korean_break_unit,
        condense_min_space,
        &resolved_letter_spacing_px(&text_chars, &para.char_offsets, &para.char_shapes, styles),
        reflow_start_idx,
        reflow_is_first_line,
        kerning_break_session.as_mut(),
    );
    let kerning_failed = kerning_break_session
        .as_ref()
        .and_then(|session| session.failed_reason())
        .is_some();
    drop(kerning_break_session);
    drop(kerning_transaction);
    if kerning_failed {
        line_breaks = fill_lines(
            &tokens[token_start..],
            &text_chars,
            available_width_px,
            indent_px,
            tab_width,
            korean_break_unit,
            condense_min_space,
            &resolved_letter_spacing_px(&text_chars, &para.char_offsets, &para.char_shapes, styles),
            reflow_start_idx,
            reflow_is_first_line,
            None,
        );
    }
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

    para.replace_line_segs(new_line_segs);
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
    // Round, don't truncate. `px_to_hwpunit` is `(px * 7200 / dpi) as i32`,
    // which floors toward zero, and the px it is handed came from a HWPUNIT
    // font size in the first place — so the round trip loses a unit whenever
    // the division is inexact. Measured over 1,133,228 admitted rows, that
    // truncation is 5,012 of the 5,215 `line_height` disagreements against
    // HWP's stored records, every one of them exactly -1.
    round_px_to_hwpunit(font_size_px, dpi)
}

/// `px_to_hwpunit` with round-half-away-from-zero instead of truncation.
fn round_px_to_hwpunit(px: f64, dpi: f64) -> i32 {
    (px * 7_200.0 / dpi).round() as i32
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
                (line_height_hwp as f64 * (ls_value - 100.0) / 100.0).round() as i32
            } else {
                0
            }
        }
        LineSpacingType::Fixed => {
            // ls_value = 고정 줄 피치 (px, resolver가 HWPUNIT→px 변환 완료)
            // line_spacing = 고정값 - line_height
            let fixed_hwp = round_px_to_hwpunit(ls_value, dpi);
            (fixed_hwp - line_height_hwp).max(0)
        }
        LineSpacingType::SpaceOnly => {
            // ls_value = 줄 사이 추가 간격만 (px)
            round_px_to_hwpunit(ls_value, dpi)
        }
        LineSpacingType::Minimum => {
            // 최소값: 콘텐츠가 최소값보다 크면 추가 간격 없음
            let min_hwp = round_px_to_hwpunit(ls_value, dpi);
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
            &[],
            &mut cursor,
            None,
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
            &[],
            initial_start_idx,
            initial_is_first_line,
            None,
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
            base_width: 100.0,
            width: 100.0,
            max_font_size: 12.0,
            base_char_widths: vec![10.0; text_chars.len()],
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
                base_width: 20.0,
                width: 20.0,
                max_font_size: 12.0,
                base_char_widths: vec![10.0, 10.0],
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
                base_width: 10.0,
                width: 10.0,
                max_font_size: 12.0,
                base_char_widths: vec![10.0],
                char_widths: vec![10.0],
            },
            BreakToken::Tab {
                idx: 4,
                max_font_size: 12.0,
            },
            BreakToken::Text {
                start_idx: 5,
                end_idx: 6,
                base_width: 10.0,
                width: 10.0,
                max_font_size: 12.0,
                base_char_widths: vec![10.0],
                char_widths: vec![10.0],
            },
            BreakToken::LineBreak { idx: 6 },
            BreakToken::Text {
                start_idx: 7,
                end_idx: 9,
                base_width: 20.0,
                width: 20.0,
                max_font_size: 12.0,
                base_char_widths: vec![10.0, 10.0],
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
            SpaceMetric::Stored,
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
                    baseline_distance: baseline_distance_hwp(line_height),
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
        a_frame_that_expects_the_stored_rows_admits_them_and_skips_the_reflow();
        width_neutrality_is_a_property_of_geometry_not_a_list_of_variants();
        body_stored_route_admits_only_width_neutral_structural_controls();
        hwp3_formatting_rebuild_preserves_legacy_stored_geometry_jurisdiction();
        frame_rejected_rows_reflow_without_propagating_cached_source_flags();
        picture_band_frame_fill_inherits_provenance_not_cached_row_state();
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

    fn a_frame_that_expects_the_stored_rows_admits_them_and_skips_the_reflow() {
        let styles = styles(&[12.0]);
        let mut para = paragraph(
            "alpha beta gamma delta",
            vec![CharShapeRef {
                start_pos: 0,
                char_shape_id: 0,
            }],
        );
        para.line_segs = vec![
            LineSeg {
                text_start: 0,
                vertical_pos: 700,
                line_height: 321,
                text_height: 300,
                baseline_distance: 250,
                line_spacing: 17,
                column_start: 1_000,
                segment_width: 4_000,
                tag: LineSeg::TAG_SINGLE_SEGMENT_LINE,
            },
            LineSeg {
                // This deliberately is not a fresh word boundary. A matching
                // Frame must preserve the cached text partition verbatim.
                text_start: 2,
                vertical_pos: 1_038,
                line_height: 654,
                text_height: 600,
                baseline_distance: 500,
                line_spacing: 23,
                column_start: 1_000,
                segment_width: 4_000,
                tag: LineSeg::TAG_SINGLE_SEGMENT_LINE,
            },
        ];
        let mut frame = LayoutFrame::new(1_000..5_000, 700, Vec::new());

        // Reuse requires the current frame to reproduce the stored physical-row
        // key exactly. Only then may the cached text partition stand.
        let resolution = resolve_stored_line_segs_in_frame(
            &para,
            &mut frame,
            &styles,
            96.0,
            false,
            StoredRowMissPolicy::Reflow,
            false,
            &[],
            false,
        )
        .expect("a scalar cached paragraph is frame-resolvable");
        assert_eq!(resolution, StoredRowResolution::Stored);
        assert_eq!(frame.row_count(), 2);

        // Geometry is preserved because the carve reproduced it — that is the
        // admission test. Vertical metrics are **not**: §1.4.1 recomputes them
        // on both arms and writes them back over the stored record, so the
        // frame publishes what it computed, not what the file carried. The
        // stored heights here (321/654) are fixture values with no relation to
        // the 12pt char shape the provider measures.
        let projected = frame.project_line_segs();
        assert_eq!(projected.len(), 2);
        assert!(projected
            .iter()
            .zip(&para.line_segs)
            .all(|(row, stored)| row.text_start == stored.text_start
                && row.column_start == stored.column_start
                && row.segment_width == stored.segment_width));
        let computed = frame_metrics_for_line(12.0, 12.0, LineSpacingType::Percent, 160.0, 96.0);
        assert!(projected
            .iter()
            .all(|row| row.line_height == computed.line_height
                && row.line_spacing == computed.line_spacing));
        assert_eq!(
            frame.top,
            700 + 2 * (computed.line_height + computed.line_spacing)
        );
    }

    fn width_neutrality_is_a_property_of_geometry_not_a_list_of_variants() {
        // Every one of these contributes no inline width and owns no layout
        // box, so the frame owns their paragraph. Only `SectionDef`,
        // `ColumnDef` and `Field` used to qualify; a body paragraph carrying
        // any of the others had no layout owner at all.
        for control in [
            Control::SectionDef(Box::default()),
            Control::ColumnDef(Default::default()),
            Control::Field(Default::default()),
            Control::Bookmark(Default::default()),
            Control::PageNumberPos(Default::default()),
            Control::PageHide(Default::default()),
            Control::HiddenComment(Box::default()),
            // HWP3 keeps hyperlinks as their own control. HWP5/HWPX turn `%hlk`
            // into a `Control::Field` by a prefix test, which is the only
            // reason those ever reached the frame.
            Control::Hyperlink(Default::default()),
            Control::AutoNumber(Default::default()),
            Control::NewNumber(Default::default()),
            Control::Unknown(Default::default()),
        ] {
            assert!(
                control_is_width_neutral_marker(&control),
                "{control:?} has no geometry, so the frame must own its paragraph"
            );
        }

        // These resolve their own geometry — inline or floating — so the frame
        // must decline and leave them with their established owner. A body
        // frame carries no exclusion, so claiming a floating object's paragraph
        // would flatten the wrap it cannot see.
        for control in [
            Control::Table(Box::default()),
            Control::Shape(Box::new(crate::model::shape::ShapeObject::Rectangle(
                Default::default(),
            ))),
            Control::Picture(Box::default()),
            Control::Equation(Box::default()),
            Control::Form(Box::default()),
            Control::Header(Box::default()),
            Control::Footer(Box::default()),
            Control::Footnote(Box::default()),
            Control::Endnote(Box::default()),
        ] {
            assert!(
                !control_is_width_neutral_marker(&control),
                "{control:?} owns its own layout box"
            );
        }
    }

    fn body_stored_route_admits_only_width_neutral_structural_controls() {
        let styles = styles(&[12.0]);
        let mut structural = paragraph(
            "section body",
            vec![CharShapeRef {
                start_pos: 0,
                char_shape_id: 0,
            }],
        );
        structural.line_segs = vec![LineSeg {
            vertical_pos: 100,
            line_height: 900,
            text_height: 900,
            baseline_distance: 765,
            line_spacing: 540,
            column_start: 0,
            segment_width: 9_000,
            tag: LineSeg::TAG_SINGLE_SEGMENT_LINE,
            ..Default::default()
        }];
        structural.controls = vec![
            Control::SectionDef(Box::default()),
            Control::ColumnDef(Default::default()),
        ];
        assert!(supports_cached_body_frame_controls(&structural));
        let mut structural_frame = LayoutFrame::new(0..9_000, 100, Vec::new());
        assert!(matches!(
            resolve_stored_line_segs_in_frame(
                &structural,
                &mut structural_frame,
                &styles,
                96.0,
                false,
                StoredRowMissPolicy::Reflow,
                false,
                &[],
                false,
            ),
            Some(StoredRowResolution::Stored)
        ));
        // Width-neutral markers do not move the carve, so production admission
        // has committed the matching cached row into the frame.
        assert_eq!(structural_frame.row_count(), 1);
        assert_eq!(
            structural_frame
                .project_line_segs()
                .iter()
                .map(|line| (line.column_start, line.segment_width))
                .collect::<Vec<_>>(),
            vec![(0, 9_000)]
        );

        let mut specialized = structural.clone();
        specialized.controls = vec![Control::Table(Box::default())];
        assert!(!supports_cached_body_frame_controls(&specialized));
        let mut specialized_frame = LayoutFrame::new(0..9_000, 100, Vec::new());
        let checkpoint = specialized_frame.clone();
        assert!(resolve_stored_line_segs_in_frame(
            &specialized,
            &mut specialized_frame,
            &styles,
            96.0,
            false,
            StoredRowMissPolicy::Reflow,
            true,
            &[],
            false,
        )
        .is_none());
        assert_eq!(specialized_frame, checkpoint);

        // A Square exclusion can leave one slot per row and still change that
        // slot when the band ends. An exclusion-less body frame cannot infer
        // the missing band from these cached outputs, so varying single-slot
        // geometry is unmodelled rather than a reason to flatten and reflow.
        let mut varying_single_slot = structural.clone();
        varying_single_slot.controls.clear();
        varying_single_slot.line_segs = vec![
            LineSeg {
                text_start: 0,
                column_start: 0,
                segment_width: 4_000,
                tag: LineSeg::TAG_SINGLE_SEGMENT_LINE,
                ..structural.line_segs[0].clone()
            },
            LineSeg {
                text_start: 4,
                column_start: 0,
                segment_width: 9_000,
                tag: LineSeg::TAG_SINGLE_SEGMENT_LINE,
                ..structural.line_segs[0].clone()
            },
        ];
        let mut scalar_frame = LayoutFrame::new(0..9_000, 100, Vec::new());
        let scalar_checkpoint = scalar_frame.clone();
        assert!(resolve_stored_line_segs_in_frame(
            &varying_single_slot,
            &mut scalar_frame,
            &styles,
            96.0,
            false,
            StoredRowMissPolicy::Reflow,
            false,
            &[],
            false,
        )
        .is_none());
        assert_eq!(scalar_frame, scalar_checkpoint);

        let mut empty_external_carrier = structural.clone();
        empty_external_carrier.text.clear();
        empty_external_carrier.char_offsets.clear();
        empty_external_carrier.char_count = 1;
        empty_external_carrier.controls.clear();
        empty_external_carrier.line_segs = vec![LineSeg {
            text_start: 0,
            column_start: 4_000,
            segment_width: 5_000,
            tag: LineSeg::TAG_SINGLE_SEGMENT_LINE,
            ..structural.line_segs[0].clone()
        }];
        let mut carrier_frame = LayoutFrame::new(0..9_000, 100, Vec::new());
        let carrier_checkpoint = carrier_frame.clone();
        assert!(resolve_stored_line_segs_in_frame(
            &empty_external_carrier,
            &mut carrier_frame,
            &styles,
            96.0,
            false,
            StoredRowMissPolicy::Reflow,
            false,
            &[],
            false,
        )
        .is_none());
        assert_eq!(carrier_frame, carrier_checkpoint);
    }

    fn hwp3_formatting_rebuild_preserves_legacy_stored_geometry_jurisdiction() {
        use crate::document_core::DocumentCore;
        use std::fs;
        use std::path::Path;

        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("samples/issue1892_hwp3_drawing_group_roundtrip.hwp");
        let bytes = fs::read(&path).unwrap_or_else(|error| panic!("read {path:?}: {error}"));
        let mut core = DocumentCore::from_bytes(&bytes).expect("load HWP3 formatting fixture");
        assert!(
            core.document.layout_profile().legacy_hwp3_stored_geometry(),
            "the HWP3 fixture must retain legacy stored-geometry jurisdiction"
        );

        // Choose a plain, nonempty single-row paragraph. With no controls and
        // no varying slots, only HWP3 lineage can make this mismatching Frame
        // unmodelled; clearing the lineage flag below proves the reversal.
        let (section_index, paragraph_index) = core
            .document
            .sections
            .iter()
            .enumerate()
            .find_map(|(section_index, section)| {
                section
                    .paragraphs
                    .iter()
                    .enumerate()
                    .find(|(_, paragraph)| {
                        !paragraph.text.trim().is_empty()
                            && paragraph.controls.is_empty()
                            && paragraph.line_segs.len() == 1
                            && paragraph.line_segs[0].is_first_segment()
                            && paragraph.line_segs[0].is_last_segment()
                            && paragraph.line_segs[0].segment_width > 2_400
                    })
                    .map(|(paragraph_index, _)| (section_index, paragraph_index))
            })
            .expect("fixture has a plain legacy stored row");
        let stored_before = line_fields(
            &core.document.sections[section_index].paragraphs[paragraph_index].line_segs,
        );

        // Underline is paint-only, but the public formatting lifecycle still
        // calls rebuild_section(). The legacy row itself must remain untouched.
        core.apply_char_format_native(
            section_index,
            paragraph_index,
            0,
            1,
            r#"{"underline":true}"#,
        )
        .expect("apply paint-only format");
        assert_eq!(
            line_fields(
                &core.document.sections[section_index].paragraphs[paragraph_index].line_segs,
            ),
            stored_before
        );
        assert!(core.document.layout_profile().legacy_hwp3_stored_geometry());

        let paragraph = &core.document.sections[section_index].paragraphs[paragraph_index];
        let stored = &paragraph.line_segs[0];
        let frame_horizontal = stored.column_start.saturating_add(1_200)
            ..stored.column_start.saturating_add(stored.segment_width);

        let mut ordinary_frame =
            LayoutFrame::new(frame_horizontal.clone(), stored.vertical_pos, Vec::new());
        assert!(matches!(
            resolve_stored_line_segs_in_frame(
                paragraph,
                &mut ordinary_frame,
                &core.styles,
                96.0,
                false,
                StoredRowMissPolicy::Reflow,
                false,
                &[],
                false,
            ),
            Some(StoredRowResolution::Reflowed)
        ));

        let mut legacy_frame = LayoutFrame::new(frame_horizontal, stored.vertical_pos, Vec::new());
        assert!(
            resolve_stored_line_segs_in_frame(
                paragraph,
                &mut legacy_frame,
                &core.styles,
                96.0,
                true,
                StoredRowMissPolicy::Reflow,
                false,
                &[],
                false,
            )
            .is_none(),
            "untouched HWP3 stored origin remains outside the common Frame's jurisdiction"
        );
    }

    fn frame_rejected_rows_reflow_without_propagating_cached_source_flags() {
        let styles = styles(&[12.0]);
        let mut para = paragraph(
            "abcdef ghijkl",
            vec![CharShapeRef {
                start_pos: 0,
                char_shape_id: 0,
            }],
        );
        para.line_segs = vec![LineSeg {
            text_start: 0,
            vertical_pos: 100,
            line_height: 900,
            text_height: 900,
            baseline_distance: 765,
            line_spacing: 540,
            column_start: 0,
            segment_width: 9_000,
            tag: LineSeg::TAG_SINGLE_SEGMENT_LINE
                | LineSeg::TAG_FIRST_LINE_OF_PAGE
                | LineSeg::TAG_FIRST_LINE_OF_COLUMN
                | LineSeg::TAG_EMPTY_SEGMENT,
        }];
        let mut frame = LayoutFrame::new(
            0..9_000,
            100,
            vec![FrameExclusion {
                horizontal: 3_000..5_000,
                vertical: 0..10_000,
                policy: FrameExclusionPolicy::BothSides,
            }],
        );

        let resolution = resolve_stored_line_segs_in_frame(
            &para,
            &mut frame,
            &styles,
            96.0,
            false,
            StoredRowMissPolicy::Reflow,
            false,
            &[],
            false,
        )
        .expect("a scalar cached paragraph is frame-resolvable");

        let StoredRowResolution::Reflowed = resolution else {
            panic!("rows differing from the frame expectation must reflow");
        };
        // The frame holds the rows now; projecting is the only way out.
        let lines = frame.project_line_segs();
        assert_eq!(
            lines
                .iter()
                .map(|line| (line.text_start, line.column_start, line.segment_width))
                .collect::<Vec<_>>(),
            vec![(0, 0, 3_000), (7, 5_000, 4_000)]
        );
        assert_eq!(frame.row_count(), 1);
        assert_eq!(frame.top, 1_540);
        let rejected_cache_flags = LineSeg::TAG_FIRST_LINE_OF_PAGE
            | LineSeg::TAG_FIRST_LINE_OF_COLUMN
            | LineSeg::TAG_EMPTY_SEGMENT;
        assert!(lines
            .iter()
            .all(|line| line.tag & rejected_cache_flags == 0));
        assert!(lines
            .iter()
            .all(|line| line.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0));
    }

    fn picture_band_frame_fill_inherits_provenance_not_cached_row_state() {
        let styles = styles(&[12.0]);
        let mut para = paragraph(
            "alpha beta gamma delta epsilon zeta eta theta iota kappa",
            vec![CharShapeRef {
                start_pos: 0,
                char_shape_id: 0,
            }],
        );
        let cached_row_flags = LineSeg::TAG_FIRST_LINE_OF_PAGE
            | LineSeg::TAG_FIRST_LINE_OF_COLUMN
            | LineSeg::TAG_EMPTY_SEGMENT
            | LineSeg::TAG_AUTO_HYPHENATION
            | LineSeg::TAG_INDENTATION
            | LineSeg::TAG_PARAGRAPH_HEAD;
        para.line_segs = vec![LineSeg {
            tag: LineSeg::TAG_SINGLE_SEGMENT_LINE | cached_row_flags,
            ..Default::default()
        }];

        let fill = |paragraph: &Paragraph| {
            let mut frame = LayoutFrame::new(0..3_000, 0, Vec::new());
            layout_paragraph_in_frame(paragraph, &mut frame, &styles, 96.0)
                .expect("Picture-band fill must produce fresh rows")
        };
        let mut picture_band_input = para;
        picture_band_input.line_segs.clear();
        let synthetic = fill(&picture_band_input);
        assert!(synthetic.len() > 1, "fixture must exercise later rows");
        assert!(synthetic
            .iter()
            .all(|line| line.tag & cached_row_flags == 0));
        assert!(synthetic
            .iter()
            .all(|line| line.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0));
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

        reflow_line_segs(
            &mut para,
            ParagraphBox::content_width_px(50.0, 96.0),
            &styles,
            96.0,
        );

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

        let band = layout_picture_band(
            &[host],
            0,
            crate::renderer::hwpunit_to_px(COLUMN_WIDTH, DPI),
            &styles,
            DPI,
        )
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
        let column_width = page_layout.column_areas[0].width;
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
        let column_width = page_layout.column_areas[0].width;
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
        let column_width = page_layout.column_areas[0].width;
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

        reflow_line_segs(
            &mut para,
            ParagraphBox::content_width_px(500.0, 96.0),
            &ResolvedStyleSet::default(),
            96.0,
        );

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
