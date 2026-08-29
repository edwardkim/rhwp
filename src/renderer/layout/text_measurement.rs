//! 텍스트 폭 측정, 문자 클러스터 분할, CJK 판별 관련 함수

use super::super::font_metrics_data;
use super::super::kerning::{ExactFontSourceHandle, KerningRunMeasurement, KerningSourceSession};
use super::super::style_resolver::ResolvedStyleSet;
use super::super::{TabLeaderInfo, TabStop, TextStyle};

// ── TextMeasurer trait ──────────────────────────────────────────────

/// 텍스트 폭 측정 추상화 트레이트
///
/// 텍스트 측정 구현체를 추상화한다.
/// - EmbeddedTextMeasurer: 내장 폰트 메트릭 기반 (native/WASM 공통 기본값)
pub trait TextMeasurer {
    /// 텍스트 전체 폭 추정 (px)
    fn estimate_text_width(&self, text: &str, style: &TextStyle) -> f64;
    /// 글자별 X 위치 경계값 계산 (N글자 → N+1개 경계)
    fn compute_char_positions(&self, text: &str, style: &TextStyle) -> Vec<f64>;
}

// ── 공통 헬퍼 ───────────────────────────────────────────────────────

/// 자모 클러스터 길이 매핑 계산
///
/// 한글 자모 조합(초+중+종)을 1개 클러스터로 묶는다.
/// cluster_len[i] > 0: 클러스터 시작 (길이), 0: 클러스터 내부 (이전 문자와 동일 위치)
fn build_cluster_len(chars: &[char]) -> Vec<u8> {
    let char_count = chars.len();
    let mut cluster_len = vec![0u8; char_count];
    let mut ci = 0;
    while ci < char_count {
        if is_hangul_choseong(chars[ci]) {
            let start = ci;
            ci += 1;
            if ci < char_count && is_hangul_jungseong(chars[ci]) {
                ci += 1;
                if ci < char_count && is_hangul_jongseong(chars[ci]) {
                    ci += 1;
                }
            }
            cluster_len[start] = (ci - start) as u8;
        } else {
            cluster_len[ci] = 1;
            ci += 1;
        }
    }
    cluster_len
}

/// [#2279] 자간(%)의 픽셀 기여 — 한글은 자간을 **해당 글자의 진행폭 비례**로
/// 적용한다 (fs-비례 아님). 무신축 Justify 마지막 줄 실측(36392557 pi34,
/// 휴먼명조 '*' 14pt 자간 -9%/장평 96%: 0.44em = 0.5×0.96×0.91)으로 확정.
/// 전각(1.0em) 글자는 fs-비례와 동일하므로 CJK 자간 동작은 불변이고,
/// 반각/좁은 글자에서만 압축·확장이 글자폭에 비례해 정확해진다.
/// style.letter_spacing 은 fs×% 로 저장되어 있으므로 (base/fs) 로 환산한다.
#[inline]
fn glyph_letter_spacing(letter_spacing_px: f64, glyph_base_px: f64, font_size: f64) -> f64 {
    if font_size <= 0.0 {
        return letter_spacing_px;
    }
    letter_spacing_px * (glyph_base_px / font_size)
}

/// 스타일에서 공통 파라미터 추출 (font_size, ratio, tab_w)
fn style_params(style: &TextStyle) -> (f64, f64, f64) {
    let base_font_size = if style.font_size > 0.0 {
        style.font_size
    } else {
        12.0
    };
    // [#5756] 위/아래첨자 run 의 **전진폭**은 그리는 크기(0.7배)를 따른다.
    // 한글 저장 lineseg 실측(156732409 3쪽): 위첨자 전진을 본문 크기로 재면
    // 칸 안쪽 폭 327.95px 에 담긴 40글자가 382.9px 로 늘어 칸 오른쪽 괘선 밖
    // 54.9px 에 찍힌다 — 0.7배로 재면 327.4px 로 저장 줄바꿈(textpos)과 정확히
    // 맞는다. 자간/장평은 크기 비례라 함께 줄어든다. 탭 폭 기본값은 본문 크기
    // 기준을 유지한다.
    let font_size = if style.superscript || style.subscript {
        base_font_size * crate::renderer::SCRIPT_FONT_SCALE
    } else {
        base_font_size
    };
    let ratio = if style.ratio > 0.0 { style.ratio } else { 1.0 };
    let tab_w = if style.default_tab_width > 0.0 {
        style.default_tab_width
    } else {
        base_font_size * 4.0
    };
    (font_size, ratio, tab_w)
}

/// inline_tabs ext[2] 에서 탭 종류를 추출.
///
/// HWP `tab_extended` 포맷 (PR #292 / Task #290 실증):
/// - high byte = 탭 종류 enum+1 (1=LEFT, 2=RIGHT, 3=CENTER, 4=DECIMAL)
/// - low  byte = fill_type (TabDef.fill 과 동일)
///
/// 기존 코드는 `ext[2]` 전체 u16 을 탭 종류로 해석하여 실제 HWP 값(최소 256)과
/// 매칭 실패. 이 헬퍼로 고바이트만 추출해 0~4 값으로 정규화.
#[inline]
pub(super) fn inline_tab_type(ext: &[u16; 7]) -> u8 {
    ((ext[2] >> 8) & 0xFF) as u8
}

/// 현재 절대 위치에서 다음 탭 정지를 찾는다.
///
/// Returns (position, tab_type, fill_type).
/// 커스텀 탭이 없으면 기본 등간격 탭을 사용한다.
pub(crate) fn find_next_tab_stop(
    abs_x: f64,
    tab_stops: &[TabStop],
    default_tab_width: f64,
    auto_tab_right: bool,
    available_width: f64,
) -> (f64, u8, u8) {
    // 커스텀 탭 정지에서 현재 위치 뒤의 첫 번째 검색
    for ts in tab_stops {
        // type=1(오른쪽) 탭은 단 기준 절대 위치이므로 available_width 클램핑 제외.
        // 들여쓰기(left_margin)가 있는 문단에서도 오른쪽 탭이 동일 위치에 정렬되도록 한다.
        // type=0(왼쪽)/2(가운데) 탭은 종전대로 클램핑하여 텍스트 영역 밖으로 넘어가지 않게 한다.
        let pos = if ts.tab_type != 1 && ts.position > available_width && available_width > 0.0 {
            available_width
        } else {
            ts.position
        };
        if pos > abs_x + 0.5 {
            return (pos, ts.tab_type, ts.fill_type);
        }
    }
    // auto_tab_right: 커스텀 탭이 모두 지나갔으면 오른쪽 끝을 right 탭으로
    if auto_tab_right && available_width > abs_x + 0.5 {
        return (available_width, 1, 0); // type=1(오른쪽), fill=0(없음)
    }
    // 기본 등간격 탭
    let tab_w = if default_tab_width > 0.0 {
        default_tab_width
    } else {
        48.0
    };
    let next = ((abs_x / tab_w).floor() + 1.0) * tab_w;
    (next, 0, 0) // type=0(왼쪽), fill=0(없음)
}

/// 지정 인덱스부터 다음 탭(또는 문자열 끝)까지의 세그먼트 폭을 측정한다.
fn measure_segment_from(
    chars: &[char],
    cluster_len: &[u8],
    start: usize,
    char_width: &dyn Fn(usize) -> f64,
) -> f64 {
    let mut w = 0.0;
    for i in start..chars.len() {
        if chars[i] == '\t' {
            break;
        }
        if cluster_len[i] == 0 {
            continue;
        }
        w += char_width(i);
    }
    w
}

fn tab_suffix_is_ascii_page_number(chars: &[char], start: usize) -> bool {
    let mut seen_digit = false;
    for ch in chars.iter().skip(start) {
        if *ch == '\t' {
            return false;
        }
        if ch.is_whitespace() {
            continue;
        }
        if ch.is_ascii_digit() {
            seen_digit = true;
            continue;
        }
        return false;
    }
    seen_digit
}

fn right_leader_tab_target_rel(style: &TextStyle, font_size: f64) -> Option<f64> {
    style
        .tab_stops
        .iter()
        .rev()
        .find(|tab| tab.tab_type == 1 && tab.fill_type != 0)
        .map(|tab| tab.position - font_size * 0.25 - style.line_x_offset)
        .filter(|target| target.is_finite())
}

fn right_leader_tab_fill(style: &TextStyle) -> Option<u8> {
    style
        .tab_stops
        .iter()
        .rev()
        .find(|tab| tab.tab_type == 1 && tab.fill_type != 0)
        .map(|tab| tab.fill_type)
}

fn right_leader_body_target_rel(style: &TextStyle) -> Option<f64> {
    if style.available_width <= 0.0 || right_leader_tab_fill(style).is_none() {
        return None;
    }
    let target = style.text_start_offset + style.available_width - style.line_x_offset;
    if target.is_finite() {
        Some(target)
    } else {
        None
    }
}

/// 탭 문자의 위치로부터 탭 리더 정보를 추출한다.
pub fn extract_tab_leaders(text: &str, positions: &[f64], style: &TextStyle) -> Vec<TabLeaderInfo> {
    extract_tab_leaders_with_extended(text, positions, style, &[])
}

/// 탭 리더 추출 (tab_extended 지원)
/// tab_extended: HWPX 인라인 탭 또는 HWP 탭 확장 데이터
/// (ext[0..2] = 탭 폭(UINT32), ext[2] = (탭 종류 << 8) | 채움 종류)
pub fn extract_tab_leaders_with_extended(
    text: &str,
    positions: &[f64],
    style: &TextStyle,
    tab_extended: &[[u16; 7]],
) -> Vec<TabLeaderInfo> {
    let chars: Vec<char> = text.chars().collect();
    let tab_w = if style.default_tab_width > 0.0 {
        style.default_tab_width
    } else {
        48.0
    };
    let mut leaders = Vec::new();
    let mut tab_idx = 0usize; // tab_extended 인덱스
    for (i, c) in text.chars().enumerate() {
        if c == '\t' && i + 1 < positions.len() {
            let before_x = positions[i];
            let after_x = positions[i + 1];
            let has_more_tabs_after = chars.iter().skip(i + 1).any(|ch| *ch == '\t');
            let tabdef_page_number_fill = if tab_extended.is_empty()
                && !has_more_tabs_after
                && tab_suffix_is_ascii_page_number(&chars, i + 1)
            {
                right_leader_tab_fill(style)
            } else {
                None
            };

            // 1. tab_extended 에서 채움 종류 가져오기 (HWP5/HWPX 인라인 탭)
            // 인라인 탭 확장 레코드는 ext[0..2] = 탭 폭(UINT32 저/고워드),
            // ext[2] = (탭 종류 << 8) | 채움 종류 다. 위치 계산부(`inline_tab_x` 의
            // `fill_low = tab_type_raw & 0xFF`)와 같은 자리를 봐야 한다.
            // ext[1] 은 탭 폭의 상위 16비트라 목차 탭 폭(수만 HWPUNIT)에서는 늘 0 →
            // `leader` 가 붙은 탭도 채움 없음으로 읽혀 리더가 통째로 사라졌다 (#5799).
            let ext_fill = if tab_idx < tab_extended.len() {
                (tab_extended[tab_idx][2] & 0x00FF) as u8
            } else {
                0
            };

            // 2. TabDef에서 fill_type 가져오기 (HWP TabDef)
            let tabdef_fill = if let Some(fill) = tabdef_page_number_fill {
                fill
            } else if !style.tab_stops.is_empty() || style.auto_tab_right {
                let abs_before = style.line_x_offset + before_x;
                let (_, _, ft) = find_next_tab_stop(
                    abs_before,
                    &style.tab_stops,
                    tab_w,
                    style.auto_tab_right,
                    style.available_width,
                );
                ft
            } else {
                0
            };

            // 둘 중 하나라도 fill이 있으면 리더 추가
            // 오른쪽 정렬 텍스트 앞에 공백 1개 간격 확보
            let fill_type = if ext_fill > 0 { ext_fill } else { tabdef_fill };
            if fill_type > 0 && after_x > before_x + 1.0 {
                let space_gap = style.font_size * 0.25;
                let content_x = text.chars().enumerate().skip(i + 1).find_map(|(j, ch)| {
                    if ch != '\t' && !ch.is_whitespace() && j < positions.len() {
                        Some(positions[j])
                    } else {
                        None
                    }
                });
                let end_x = content_x
                    .map(|x| x - space_gap)
                    .unwrap_or(after_x - space_gap)
                    .min(after_x - space_gap);
                leaders.push(TabLeaderInfo {
                    start_x: before_x,
                    end_x: end_x.max(before_x),
                    fill_type,
                });
            }
            tab_idx += 1;
        }
    }
    if leaders.len() > 1 {
        let mut min_following_end = f64::INFINITY;
        for leader in leaders.iter_mut().rev() {
            if min_following_end.is_finite() && leader.end_x > min_following_end {
                leader.end_x = min_following_end.max(leader.start_x);
            }
            min_following_end = min_following_end.min(leader.end_x);
        }
    }
    leaders
}

// ── EmbeddedTextMeasurer ────────────────────────────────────────────

/// 내장 폰트 메트릭 기반 텍스트 측정기
///
/// font_metrics_data의 582개 폰트 메트릭을 사용하여 문자 폭을 측정한다.
/// 메트릭이 없는 폰트는 CJK=font_size, Latin=font_size×0.5 휴리스틱을 사용한다.
/// 모든 플랫폼에서 동일하게 동작한다 (WASM 포함).
/// [#2132] 공용 글자-워크 — Embedded/Wasm measurer 의 compute_char_positions 중복 소거.
/// 폭 산출원(char_px_raw)과 인라인 탭 divergent 경로(inline_tab_x)만 measurer 별 훅.
/// 나머지(특수문자, dash leader, 자간 클램프, 공백, 커스텀/기본 탭)는 1벌.
fn compute_char_positions_walk(
    text: &str,
    style: &TextStyle,
    inline_tab_x: &dyn Fn(usize, f64, &[u16; 7], &[char], &[u8], &dyn Fn(usize) -> f64) -> f64,
) -> Vec<f64> {
    let (_, _, tab_w) = style_params(style);
    let chars: Vec<char> = text.chars().collect();
    let char_count = chars.len();
    let mut positions = Vec::with_capacity(char_count + 1);
    let mut x = 0.0;
    positions.push(x);

    let cluster_len = build_cluster_len(&chars);
    let has_custom_tabs = !style.tab_stops.is_empty() || style.auto_tab_right;

    let char_width =
        |i: usize| -> f64 { char_width_decision(&chars, &cluster_len, i, style).final_width_px };

    let mut tab_char_idx = 0usize; // inline_tabs 인덱스
    for i in 0..char_count {
        let c = chars[i];
        if cluster_len[i] == 0 {
            positions.push(x);
            continue;
        }
        if c == '\t' {
            if tab_char_idx < style.inline_tabs.len() {
                let ext = &style.inline_tabs[tab_char_idx];
                x = inline_tab_x(i, x, ext, &chars, &cluster_len, &char_width);
                tab_char_idx += 1;
            } else if has_custom_tabs {
                let has_more_tabs_after = chars[i + 1..].contains(&'\t');
                if !has_more_tabs_after && tab_suffix_is_ascii_page_number(&chars, i + 1) {
                    if let Some(target_rel) = right_leader_body_target_rel(style) {
                        let seg_w = measure_segment_from(&chars, &cluster_len, i + 1, &char_width);
                        x = (target_rel - seg_w).max(x);
                        tab_char_idx += 1;
                        positions.push(x);
                        continue;
                    }
                }
                let abs_x = style.line_x_offset + x;
                let (tab_pos, tab_type, fill_type) = find_next_tab_stop(
                    abs_x,
                    &style.tab_stops,
                    tab_w,
                    style.auto_tab_right,
                    style.available_width,
                );
                let rel_tab = tab_pos - style.line_x_offset;
                // [Task #874] auto_tab_right / leader RIGHT 탭은 col-relative 우측 끝
                // (= text_start_offset + available_width) 까지 정렬.
                let effective_rel_tab = if tab_type == 1
                    && style.available_width > 0.0
                    && (fill_type != 0 || style.auto_tab_right)
                {
                    style.text_start_offset + style.available_width - style.line_x_offset
                } else {
                    rel_tab
                };
                match tab_type {
                    1 => {
                        // 오른쪽
                        let seg_start = if fill_type != 0 {
                            i + 1
                        } else {
                            let mut s = i + 1;
                            while s < chars.len() && chars[s] == ' ' && cluster_len[s] != 0 {
                                s += 1;
                            }
                            s
                        };
                        let seg_w =
                            measure_segment_from(&chars, &cluster_len, seg_start, &char_width);
                        x = (effective_rel_tab - seg_w).max(x);
                    }
                    2 => {
                        // 가운데
                        let seg_w = measure_segment_from(&chars, &cluster_len, i + 1, &char_width);
                        x = (rel_tab - seg_w / 2.0).max(x);
                    }
                    _ => {
                        // 왼쪽(0), 소수점(3)
                        x = rel_tab.max(x);
                    }
                }
                tab_char_idx += 1;
            } else {
                // 기본 등간격 탭: 라인 절대 위치(line_x_offset + x) 기준으로 계산.
                let abs_x = style.line_x_offset + x;
                let next_abs = ((abs_x / tab_w).floor() + 1.0) * tab_w;
                x = (next_abs - style.line_x_offset).max(x);
                tab_char_idx += 1;
            }
            positions.push(x);
            continue;
        }
        x += char_width(i);
        positions.push(x);
    }

    positions
}

pub struct EmbeddedTextMeasurer;

impl TextMeasurer for EmbeddedTextMeasurer {
    fn estimate_text_width(&self, text: &str, style: &TextStyle) -> f64 {
        let (font_size, _, tab_w) = style_params(style);
        let chars: Vec<char> = text.chars().collect();
        let cluster_len = build_cluster_len(&chars);
        let char_count = chars.len();
        let has_custom_tabs = !style.tab_stops.is_empty() || style.auto_tab_right;

        let char_width = |i: usize| -> f64 {
            char_width_decision(&chars, &cluster_len, i, style).final_width_px
        };

        let mut total = 0.0;
        let mut tab_char_idx = 0usize;
        for i in 0..char_count {
            let c = chars[i];
            if cluster_len[i] == 0 {
                continue;
            }
            if c == '\t' {
                // 인라인 탭 (HWP tab_extended / HWPX 인라인 탭)
                // NOTE: 네이티브 경로는 `tab_type = ext[2]` 전체 u16 해석을 유지.
                // 기존 golden SVG (issue-147, issue-267) 가 이 "우연한 LEFT 폴백" 동작에
                // 의존하고 있어, 이를 바꾸면 회귀 발생. WASM 경로만 inline_tab_type 사용.
                // [Issue #630 Stage 4 검증] HWP5 의 `ext[0]` 가 이미 right-tab 결과 위치
                // (= 우측 끝 - 한컴_seg_w) 로 저장되어 있어 LEFT fallback 이 인코딩 의도와
                // 정합. RIGHT 정확 매치 시 seg_w 이중 차감 → ≈seg_w (≈112px) 좌측 이탈
                // (aift p4 1-1 등 23/24 라인 모두 영향). 본 LEFT fallback 동작 유지.
                if tab_char_idx < style.inline_tabs.len() {
                    let ext = &style.inline_tabs[tab_char_idx];
                    let tab_width_px = ext[0] as f64 * 96.0 / 7200.0;
                    let tab_type = ext[2];
                    let tab_target = total + tab_width_px;
                    // [Task #874] auto_tab_right 가 활성된 paragraph 에서 단일 tab 의
                    // 인라인 tab_extended 는 Hancom 의 right-tab 결과 위치(= 우측 끝 -
                    // 한컴_seg_w) 를 ext[0] 로 저장. 우리 폰트의 seg_w 와 다르면 좌측
                    // 이탈 발생 (shortcut.hwp pi=144 `Alt+Shift+C` 27 px 부족). auto_right
                    // 일 때는 우리 metric 기준 right-edge - our_seg_w 로 override.
                    let has_more_tabs_after = chars[i + 1..].contains(&'\t');
                    // [Task #874 #10] ext[2] high-byte 가 명시적 LEFT(1)/DECIMAL(4) 면
                    // auto_tab_right paragraph 라도 override 금지 — exam_math.hwp p7
                    // item 18 (Task #290) 의 inline LEFT tab 회귀 차단.
                    let inline_type_hi = ((tab_type >> 8) & 0xFF) as u8;
                    let inline_is_explicit_left = inline_type_hi == 1 || inline_type_hi == 4;
                    let override_to_right = style.auto_tab_right
                        && !has_more_tabs_after
                        && style.available_width > 0.0
                        && !inline_is_explicit_left;
                    if override_to_right {
                        // [Task #874 #2] lang split 로 post-tab 콘텐츠가 후속 run 으로
                        // 쪼개진 경우 (예: "F3→Alt+I" → "F3"/"→"/"Alt+I"), 현재 run 내부
                        // 측정만으로는 seg_w 가 부족. paragraph_layout 이 미리 합산한
                        // block_w override 가 있으면 그것을 사용.
                        let seg_w = style.right_tab_block_width_override.unwrap_or_else(|| {
                            measure_segment_from(&chars, &cluster_len, i + 1, &char_width)
                        });
                        let right_edge_rel =
                            style.text_start_offset + style.available_width - style.line_x_offset;
                        total = (right_edge_rel - seg_w).max(total);
                    } else if inline_type_hi == 0
                        && !has_more_tabs_after
                        && tab_suffix_is_ascii_page_number(&chars, i + 1)
                    {
                        if let Some(target_rel) = right_leader_tab_target_rel(style, font_size) {
                            let seg_w =
                                measure_segment_from(&chars, &cluster_len, i + 1, &char_width);
                            total = (target_rel - seg_w).max(total);
                        } else {
                            total = tab_target.max(total);
                        }
                    } else {
                        match tab_type {
                            1 => {
                                let seg_w =
                                    measure_segment_from(&chars, &cluster_len, i + 1, &char_width);
                                total = (tab_target - seg_w).max(total);
                            }
                            2 => {
                                let seg_w =
                                    measure_segment_from(&chars, &cluster_len, i + 1, &char_width);
                                total = (tab_target - seg_w / 2.0).max(total);
                            }
                            _ => {
                                total = tab_target.max(total);
                            }
                        }
                    }
                    tab_char_idx += 1;
                } else if has_custom_tabs {
                    let has_more_tabs_after = chars[i + 1..].contains(&'\t');
                    if !has_more_tabs_after && tab_suffix_is_ascii_page_number(&chars, i + 1) {
                        if let Some(target_rel) = right_leader_body_target_rel(style) {
                            let seg_w =
                                measure_segment_from(&chars, &cluster_len, i + 1, &char_width);
                            total = (target_rel - seg_w).max(total);
                            tab_char_idx += 1;
                            continue;
                        }
                    }
                    let abs_x = style.line_x_offset + total;
                    let (tab_pos, tab_type, fill_type) = find_next_tab_stop(
                        abs_x,
                        &style.tab_stops,
                        tab_w,
                        style.auto_tab_right,
                        style.available_width,
                    );
                    let rel_tab = tab_pos - style.line_x_offset;
                    // [Task #874] auto_tab_right 의 tab_pos = available_width 는 텍스트
                    // 영역 시작 기준 상대값. col-relative 우측 끝 = text_start_offset +
                    // available_width. line_x_offset 도 col-relative 이므로 변환.
                    let effective_rel_tab = if tab_type == 1
                        && style.available_width > 0.0
                        && (fill_type != 0 || style.auto_tab_right)
                    {
                        style.text_start_offset + style.available_width - style.line_x_offset
                    } else {
                        rel_tab
                    };
                    match tab_type {
                        1 => {
                            // 오른쪽
                            let seg_w =
                                measure_segment_from(&chars, &cluster_len, i + 1, &char_width);
                            total = (effective_rel_tab - seg_w).max(total);
                        }
                        2 => {
                            // 가운데
                            let seg_w =
                                measure_segment_from(&chars, &cluster_len, i + 1, &char_width);
                            total = (rel_tab - seg_w / 2.0).max(total);
                        }
                        _ => {
                            // 왼쪽(0), 소수점(3) → 왼쪽과 동일 처리
                            total = rel_tab.max(total);
                        }
                    }
                    tab_char_idx += 1;
                } else {
                    // 기본 등간격 탭: 라인 절대 위치(line_x_offset + total) 기준으로 계산
                    let abs_x = style.line_x_offset + total;
                    let next_abs = ((abs_x / tab_w).floor() + 1.0) * tab_w;
                    total = (next_abs - style.line_x_offset).max(total);
                    tab_char_idx += 1;
                }
                continue;
            }
            if cluster_len[i] == 0 {
                continue;
            }
            total += char_width(i);
        }
        total.round()
    }

    fn compute_char_positions(&self, text: &str, style: &TextStyle) -> Vec<f64> {
        let (font_size, _ratio, _tab_w) = style_params(style);
        // [#2132] 인라인 탭 divergent 경로 훅 — HWP5 raw ext 인코딩 legacy 해석 유지
        // (Issue #630 Stage 4/6, Task #874 계열 — 원본 무변경 이동).
        let inline_tab_x = |i: usize,
                            x_in: f64,
                            ext: &[u16; 7],
                            chars: &[char],
                            cluster_len: &[u8],
                            char_width: &dyn Fn(usize) -> f64|
         -> f64 {
            let mut x = x_in;
            let tab_width_px = ext[0] as f64 * 96.0 / 7200.0;
            let tab_type_raw = ext[2];
            let tab_target = x + tab_width_px;
            // [Task #874] auto_tab_right paragraph + 단일 tab: ext[0] = Hancom의
            // right-tab 결과 위치 (= 우측 끝 - 한컴_seg_w). 우리 폰트의 seg_w 와 차이
            // 가 있으면 좌측 이탈. col-relative right edge - our_seg_w 로 override.
            let has_more_tabs_after = chars[i + 1..].contains(&'\t');
            // [Task #874 #10] ext[2] high-byte 가 명시적 LEFT(1)/DECIMAL(4) 면
            // auto_tab_right paragraph 라도 override 금지 — exam_math.hwp p7
            // item 18 (Task #290) 의 inline LEFT tab 회귀 차단.
            let inline_type_hi = ((tab_type_raw >> 8) & 0xFF) as u8;
            let inline_is_explicit_left = inline_type_hi == 1 || inline_type_hi == 4;
            let override_to_right = style.auto_tab_right
                && !has_more_tabs_after
                && style.available_width > 0.0
                && !inline_is_explicit_left;
            // [Issue #630 Stage 6] HWP5 inline tab `ext[2]` 인코딩 = `(enum+1)<<8 | fill`
            // 이므로 high-byte 추출이 정확. 단, RIGHT(high-byte=2) + leader(fill≠0)
            // 의 경우 한컴 ext[0] 가 이미 "(우측 끝 - 한컴_seg_w)" 까지의 거리로
            // 저장 (Stage 4 검증).
            let body_right_text_rel = if style.available_width > 0.0 {
                style.text_start_offset + style.available_width - style.line_x_offset
            } else {
                f64::INFINITY
            };
            let body_right_legacy = if style.available_width > 0.0 {
                style.available_width - style.line_x_offset
            } else {
                f64::INFINITY
            };
            if override_to_right {
                // [Task #874 #2] lang split 후속 run 합산 override.
                let seg_w = if let Some(w) = style.right_tab_block_width_override {
                    w
                } else {
                    let seg_start = {
                        let mut s = i + 1;
                        while s < chars.len() && chars[s] == ' ' && cluster_len[s] != 0 {
                            s += 1;
                        }
                        s
                    };
                    measure_segment_from(&chars, &cluster_len, seg_start, &char_width)
                };
                x = (body_right_text_rel - seg_w).max(x);
            } else if inline_type_hi == 0
                && !has_more_tabs_after
                && tab_suffix_is_ascii_page_number(&chars, i + 1)
            {
                if let Some(target_rel) = right_leader_tab_target_rel(style, font_size) {
                    let seg_w = measure_segment_from(&chars, &cluster_len, i + 1, &char_width);
                    x = (target_rel - seg_w).max(x);
                } else {
                    x = tab_target.max(x);
                }
            } else {
                let high_byte = (tab_type_raw >> 8) & 0xFF;
                let fill_low = tab_type_raw & 0xFF;
                match (high_byte, tab_type_raw) {
                    (_, 1) => {
                        // 기존 raw 1 (LEFT 또는 잘못된 RIGHT 1) — 호환 유지
                        let seg_start = {
                            let mut s = i + 1;
                            while s < chars.len() && chars[s] == ' ' && cluster_len[s] != 0 {
                                s += 1;
                            }
                            s
                        };
                        let seg_w =
                            measure_segment_from(&chars, &cluster_len, seg_start, &char_width);
                        x = (tab_target - seg_w).max(x);
                    }
                    (_, 2) => {
                        // 기존 raw 2 — 호환 유지
                        let seg_w = measure_segment_from(&chars, &cluster_len, i + 1, &char_width);
                        x = (tab_target - seg_w / 2.0).max(x);
                    }
                    (2, _) if fill_low != 0 => {
                        // [Task #874 후속] 단일-run RIGHT + leader (목차 페이지번호) —
                        // Task #874 는 cross-run RIGHT+leader 의 text_start_offset
                        // 미포함 본질을 fix (body_right_text_rel +
                        // right_tab_block_width_override). 단일-run 케이스는
                        // 여전히 body_right_legacy (= available_width - line_x_offset)
                        // 사용 → text_start_offset 미포함 으로 cell right inner
                        // (= text_start_offset + available_width) 미달. 또한 leading
                        // space skip 으로 seg_w 가 space 폭만큼 과소 → digit right
                        // edge 가 cell right inner 보다 좌측에 위치 (정렬 미달).
                        //
                        // Fix: \t 뒤 content 가 있는 단일-run 은 cell_right_run_rel
                        // (= text_start_offset + available_width - line_x_offset) 정렬
                        // + seg_w_full (i+1 부터, leading space 포함). content 없는
                        // trailing space / 끝 케이스 (= cross-run 직전) 는 원본 path
                        // 유지 (다음 run 의 pending_right_tab 분기가 처리).
                        let seg_start_skipped = {
                            let mut s = i + 1;
                            while s < chars.len() && chars[s] == ' ' && cluster_len[s] != 0 {
                                s += 1;
                            }
                            s
                        };
                        let has_content_after = seg_start_skipped < chars.len();
                        if has_content_after {
                            let seg_w_full =
                                measure_segment_from(&chars, &cluster_len, i + 1, &char_width);
                            let cell_right_run_rel = style.text_start_offset
                                + style.available_width
                                - style.line_x_offset;
                            x = (cell_right_run_rel - seg_w_full).max(x);
                        } else {
                            let seg_w = measure_segment_from(
                                &chars,
                                &cluster_len,
                                seg_start_skipped,
                                &char_width,
                            );
                            x = (body_right_legacy - seg_w).max(x);
                        }
                    }
                    // [#5872] leader 없는 RIGHT 탭이 **줄 중간**에 있으면 그것은
                    // 쪽번호 정렬 탭이 아니라 줄 앞머리의 정렬 탭이다(목차 개요번호:
                    // `\tI.\t총 칙\t 1`). 뒤에 탭이 더 있는데도 본문 우측 끝으로
                    // 끌어가면 로마숫자가 쪽번호 자리에 겹친다(113424 6쪽 7줄:
                    // I@709.3 ↔ 한글 I@101.0). 한컴이 저장한 `width` 는 정렬을 이미
                    // 마친 전진 거리이므로 그대로 쓰면 좌표가 재현된다
                    // (본문 좌단 75.6 + 1911HU/25.5px = 101.1 ↔ 한글 101.0).
                    // 줄 끝의 RIGHT 탭(뒤에 탭 없음)은 종전대로 우측 끝 정렬.
                    (2, _) if has_more_tabs_after => {
                        x = tab_target.max(x);
                    }
                    (2, _) => {
                        // RIGHT 인라인 탭 (no leader): 한컴 metrics 차이 흡수.
                        let seg_start = {
                            let mut s = i + 1;
                            while s < chars.len() && chars[s] == ' ' && cluster_len[s] != 0 {
                                s += 1;
                            }
                            s
                        };
                        let seg_w =
                            measure_segment_from(&chars, &cluster_len, seg_start, &char_width);
                        x = (body_right_legacy - seg_w).max(x);
                    }
                    _ => {
                        x = tab_target.max(x);
                    }
                }
            }
            x
        };
        compute_char_positions_walk(text, style, &inline_tab_x)
    }
}

// ── 기본 측정기 선택 ────────────────────────────────────────────────
//
// native/WASM 공통으로 EmbeddedTextMeasurer 를 사용한다. 종전 WASM 전용
// WasmTextMeasurer 는 폴백 사다리 비대칭으로 native 와 SVG 좌표가 발산해
// 제거했다 (#4046). native↔WASM byte 패리티는
// scripts/svg_native_wasm_diff.mjs 로 검증한다.

fn default_measurer() -> EmbeddedTextMeasurer {
    EmbeddedTextMeasurer
}

// ── 스타일 변환 ─────────────────────────────────────────────────────

pub(crate) fn resolved_to_text_style(
    styles: &ResolvedStyleSet,
    char_style_id: u32,
    lang_index: usize,
) -> TextStyle {
    if let Some(cs) = styles.char_styles.get(char_style_id as usize) {
        TextStyle {
            font_family: cs.font_family_for_lang(lang_index).to_string(),
            font_size: cs.font_size,
            color: cs.text_color,
            bold: cs.bold,
            italic: cs.italic,
            underline: cs.underline,
            strikethrough: cs.strikethrough,
            kerning: cs.kerning,
            // 단일 출처: 줄 나눔 고속 경로(resolved_letter_spacing)와 같은 식을 쓴다.
            // 두 경로 동등성 시험(구 letter_spacing_matches_full_style_resolution)을
            // 이 호출이 구조적으로 대체한다.
            letter_spacing: resolved_letter_spacing(styles, char_style_id, lang_index),
            ratio: cs.ratio_for_lang(lang_index),
            default_tab_width: 0.0,
            tab_stops: Vec::new(),
            auto_tab_right: false,
            available_width: 0.0,
            line_x_offset: 0.0,
            text_start_offset: 0.0,
            right_tab_block_width_override: None,
            tab_leaders: Vec::new(),
            inline_tabs: Vec::new(),
            extra_word_spacing: 0.0,
            extra_char_spacing: 0.0,
            extra_dash_advance: 0.0,
            outline_type: cs.outline_type,
            shadow_type: cs.shadow_type,
            shadow_color: cs.shadow_color,
            shadow_offset_x: cs.font_size * cs.shadow_offset_x as f64 / 100.0,
            shadow_offset_y: cs.font_size * cs.shadow_offset_y as f64 / 100.0,
            emboss: cs.emboss,
            engrave: cs.engrave,
            superscript: cs.superscript,
            subscript: cs.subscript,
            emphasis_dot: cs.emphasis_dot,
            underline_shape: cs.underline_shape,
            strike_shape: cs.strike_shape,
            underline_color: cs.underline_color,
            strike_color: cs.strike_color,
            shade_color: cs.shade_color,
        }
    } else {
        TextStyle::default()
    }
}

/// `resolved_to_text_style(..).letter_spacing` 와 같은 값을 `TextStyle` 을 만들지 않고 읽는다.
///
/// [#5678] 줄 나눔이 문단의 **글자마다** 자간을 필요로 하는데, 종전에는 그때마다
/// `resolved_to_text_style` 을 불러 `String` 하나와 `Vec` 셋을 포함한 `TextStyle` 을
/// 통째로 만들었다 — 대부분의 문서에서 `0.0` 인 `f64` 하나를 읽으려고.
///
/// 두 경로가 같은 값을 낸다는 것은 `letter_spacing_matches_full_style_resolution` 이 잡는다.
pub(crate) fn resolved_letter_spacing(
    styles: &ResolvedStyleSet,
    char_style_id: u32,
    lang_index: usize,
) -> f64 {
    styles
        .char_styles
        .get(char_style_id as usize)
        .map(|cs| cs.letter_spacing_for_lang(lang_index))
        // 스타일이 없을 때의 값은 위 함수의 `TextStyle::default()` 갈래와 같아야 한다.
        .unwrap_or_else(|| TextStyle::default().letter_spacing)
}

// ── 내장 폰트 메트릭 측정 ───────────────────────────────────────────

/// 폰트가 고정폭(monospace)인지 판정한다.
///
/// Basic Latin (U+0021~U+007E) 의 0 이 아닌 글자폭이 모두 동일하면 monospace.
/// 돋움체/바탕체/굴림체 등 한컴 고정폭 폰트는 `·` 를 포함한 모든 글리프가
/// em_size 폭을 가지므로, U+00B7 의 `.notdef` 위장값 가드에서 이들을 제외해
/// 전각 측정을 보존하기 위함이다 (Issue #630, aift 목차 right-tab 정합).
fn is_monospace_metric(metric: &font_metrics_data::FontMetric) -> bool {
    let mut common: Option<u16> = None;
    let mut count = 0u32;
    for range in metric.latin_ranges {
        if range.start > 0x007E || range.end < 0x0021 {
            continue;
        }
        for (i, &w) in range.widths.iter().enumerate() {
            let code = range.start + i as u32;
            if !(0x0021..=0x007E).contains(&code) || w == 0 {
                continue;
            }
            count += 1;
            match common {
                None => common = Some(w),
                Some(cw) if cw != w => return false,
                _ => {}
            }
        }
    }
    // 표본이 충분할 때만 monospace 로 판정 (Latin 글리프가 거의 없는 폰트 오판 방지).
    count >= 16
}

/// 요청 폰트의 내장 메트릭 DB 등록 여부.
///
/// `compute_char_positions` 의 advance 가 실제 글리프 폭(메트릭 DB)에서
/// 나온 값인지, 아니면 DB 미등록 폰트의 휴리스틱 폴백(`font_size * 0.5`
/// 등)인지 구분하는 데 쓴다. WASM 캔버스 렌더러는 메트릭이 없는(=브라우저
/// 대체 폰트로 치환되는) 폰트에 대해 글리프별 가로 스케일링(per-glyph
/// x-scale)을 적용하면 안 된다 — 치환 폰트의 실제 advance 와 어긋나
/// l/i/t 같은 좁은 글리프가 과도하게 늘어나기 때문이다 (한컴 바겐세일 M
/// → Pretendard 치환 시 Vocabulary 열 왜곡).
pub(crate) fn font_family_has_metrics(font_family: &str, bold: bool, italic: bool) -> bool {
    let primary_name = font_family.split(',').next().unwrap_or(font_family).trim();
    font_metrics_data::find_metric(primary_name, bold, italic).is_some()
}

/// 내장 폰트 메트릭으로 문자 폭 측정 (em 단위 → px 변환)
///
/// 내장 메트릭이 있으면 JS 브릿지 호출 없이 즉시 반환.
/// 없으면 None을 반환하여 폴백 경로를 사용하게 한다.
fn quantize_hwp_px(px: f64) -> f64 {
    let hwp = (px * 75.0) as i32;
    hwp as f64 / 75.0
}

fn kopub_char_width(primary_name: &str, c: char, font_size: f64) -> Option<f64> {
    let lower = primary_name.to_lowercase();
    let is_dotum = primary_name.contains("KoPub돋움체") || lower.contains("kopub dotum");
    let is_batang = primary_name.contains("KoPub바탕체") || lower.contains("kopub batang");
    if !is_dotum && !is_batang {
        return None;
    }

    if c == ' ' {
        return Some(quantize_hwp_px(font_size * 0.5));
    }
    if is_narrow_punctuation(c) {
        return Some(quantize_hwp_px(font_size * 0.3));
    }
    // [#2239] 괄호 — KoPub 경로는 86712 한컴 PDF 글리프 직독 실측(13px 문서
    // 괄호 4px ≈ 0.3em, #2195 stage23)으로 narrow 유지. is_narrow_punctuation
    // 의 괄호가 폰트 한정(is_narrow_paren_for_font)으로 빠지면서 여기서 보존.
    if matches!(c, '(' | ')') {
        return Some(quantize_hwp_px(font_size * 0.3));
    }
    if c.is_ascii() {
        return Some(quantize_hwp_px(font_size * 0.5));
    }
    if is_cjk_char(c) || is_fullwidth_symbol(c) {
        // [#6389] KoPub돋움체 한글 전각은 872/1000em — 편람 kopub 오라클 PDF 의
        // 임베드 서브셋 CIDFont /W 직독(Light 601·Medium 319·Bold 283 글리프
        // 전원 872). 재구성도 맞는다: 한글 줄폭 = 872×장평0.98−자간 = 825HU/자
        // ≈ 오라클 잉크 실측 829. 종전 1.0 은 #2195 stage57 이 86712 한컴 PDF
        // 에서 실측한 값인데, 그 PDF 는 KoPub 미설치 환경이라 한글이 바탕으로
        // **치환해** 그린 것 — 치환 글꼴(전각 1.0em)의 폭이 KoPub face 상수로
        // 들어와 있었다. 치환 환경 문서군의 폭은 face 상수가 아니라 치환 경로가
        // 소유해야 한다. 그보다 전의 0.84 도 실물(0.872)과 미세하게 어긋나
        // r27 을 -11줄 과소시켰다. 바탕체 0.94 는 같은 방법 실측 936/1000 과
        // 사실상 일치해 유지한다.
        let factor = if is_dotum { 0.872 } else { 0.94 };
        return Some(quantize_hwp_px(font_size * factor));
    }

    None
}

/// #3820 `76076_regulatory_analysis` 한컴 PDF p35의 한양중고딕 공백 advance.
///
/// HWP의 일반적인 U+0020 반각 규약(`em/2`)과 달리, 원명 `한양중고딕`으로
/// 작성된 표 본문은 한컴 PDF의 p35 line decision에 맞춘 550/1024em advance가 필요하다. p35의
/// 107자 무-`LINE_SEG` 셀에서 한글 advance와 cell 폭은 RHWP와 일치하지만, 이
/// 공백 차이(약 2.17px/space)가 누적되어 `…반죽된` 뒤 `용` 한 글자가 잘못
/// 앞줄에 남는다. 자동 생성 TTF hmtx 테이블은 바꾸지 않고 PDF의 line-decision
/// 보정만 이 원명에 국한한다. `HY중고딕`은 별 face이므로 반각 규약을 유지한다.
const HANYANG_JUNGGOTHIC_PDF_SPACE_UNITS: u16 = 550;

fn hanyang_junggothic_pdf_space_width(primary_name: &str) -> Option<u16> {
    (primary_name == "한양중고딕").then_some(HANYANG_JUNGGOTHIC_PDF_SPACE_UNITS)
}

/// #3820 `76076_regulatory_analysis` 한컴 PDF p81의 한양신명조 공백 advance.
///
/// HWP 원본은 `한양신명조`를 지정하지만 기준 PDF의 실제 word gap은
/// 411/1024em(14pt에서 약 5.64pt)이다. 임베드 HFT의 U+0020 hmtx(518/1024em)를
/// 그대로 쓰면 p81의 중첩 표에서 여덟 공백마다 차이가 누적되어 `좌석안전`의
/// `전`이 다음 줄로 밀리고, 뒤의 간접편익 표가 p82로 넘어간다. 글리프·셀 폭·
/// 저장 510HU margin은 PDF와 일치하므로 이 standard-body 원명 U+0020 line-decision만
/// 보정한다.
///
/// 같은 원명이라도 10pt 접수증은 기준 PDF에서 일반 반각을 쓴다. face 이름만으로
/// 411/1024em을 적용하면 날짜의 누적 공백이 줄어 `㊞`이 도장 원 밖으로 밀린다.
const HANYANG_SHINMYEONGJO_PDF_SPACE_UNITS: u16 = 411;
/// `issue1949` HWPX standard-body는 12pt, #3820 p81 표는 14pt다. 두 문서 모두
/// 411/1024em line-decision을 쓰지만, 10pt 접수증은 일반 반각이다.
const HANYANG_SHINMYEONGJO_PDF_SPACE_MIN_FONT_SIZE_PX: f64 = 16.0; // 12pt

fn hanyang_shinmyeongjo_pdf_space_width(primary_name: &str, font_size: f64) -> Option<u16> {
    (primary_name == "한양신명조"
        && font_size + 0.01 >= HANYANG_SHINMYEONGJO_PDF_SPACE_MIN_FONT_SIZE_PX)
        .then_some(HANYANG_SHINMYEONGJO_PDF_SPACE_UNITS)
}

fn hancom_pdf_space_width(primary_name: &str, font_size: f64) -> Option<u16> {
    hanyang_junggothic_pdf_space_width(primary_name)
        .or_else(|| hanyang_shinmyeongjo_pdf_space_width(primary_name, font_size))
}

/// [#2070] ㆍ(U+318D) 폭은 SYMBOL 폰트별: 한양신명조 = 전각(사다리 v3 실측),
/// 명조(HY견명조 치환) 등 여타 = 반각 (80168 개정안{{7}} p9/p13 '시ㆍ도조례'
/// 1줄 오라클, 개정안{{1}} P21 마크와 반각 양립 검증). embedded 메트릭
/// (HY견명조 수록분)이 전각이라 룩업보다 앞서 판정하되, 함초롬(HCR) 계열은
/// embedded 메트릭을 신뢰한다 (None 반환). [#2279] 한컴바탕/한컴돋움
/// (Haansoft 실메트릭, ㆍ=1.0em)도 동일하게 embedded 메트릭을 신뢰한다.
pub(crate) fn area_dot_fallback_width(font_family: &str, font_size: f64) -> Option<f64> {
    let fam = font_family.split(',').next().unwrap_or("").trim();
    if fam.contains("함초롬")
        || fam.contains("HCR")
        || fam.contains("한컴")
        || fam.contains("Haansoft")
    {
        return None;
    }
    Some(if fam.contains("한양신명조") {
        font_size
    } else {
        font_size * 0.5
    })
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct CharWidthDecision<'a> {
    pub(crate) width_source: &'static str,
    pub(crate) base_width_px: f64,
    pub(crate) final_width_px: f64,
    pub(crate) metric: Option<font_metrics_data::MetricLookupDecision<'a>>,
    pub(crate) character_match: &'static str,
    pub(crate) dash_leader: bool,
    pub(crate) negative_spacing_clamped: bool,
}

/// 측정이 전각 글리프를 반각 advance 로 눌렀을 때의 `width_source`.
///
/// 페인트(`forces_halfwidth_cjk_quote`)가 같은 판정을 되묻는 데 쓰므로 문자열
/// literal 로 흩어 두지 않는다.
const HALFWIDTH_PUNCTUATION_WIDTH_SOURCE: &str = "metricHalfwidthPunctuationOverlay";

#[derive(Debug, Clone, Copy)]
struct EmbeddedWidthDecision<'a> {
    width_px: Option<f64>,
    width_source: &'static str,
    metric: Option<font_metrics_data::MetricLookupDecision<'a>>,
    character_match: &'static str,
}

fn measure_char_width_embedded_decision<'a>(
    font_family: &'a str,
    bold: bool,
    italic: bool,
    c: char,
    font_size: f64,
) -> EmbeddedWidthDecision<'a> {
    let primary_name = font_family
        .split(',')
        .next()
        .unwrap_or(font_family)
        .trim()
        .trim_matches('\'')
        .trim_matches('"');
    if let Some(w) = kopub_char_width(primary_name, c, font_size) {
        return EmbeddedWidthDecision {
            width_px: Some(w),
            width_source: "kopubTable",
            metric: None,
            character_match: "hit",
        };
    }
    let Some(mm) = font_metrics_data::find_metric_decision(primary_name, bold, italic) else {
        return EmbeddedWidthDecision {
            width_px: None,
            width_source: "metricMiss",
            metric: None,
            character_match: "notApplicable",
        };
    };
    // HWP 반각 처리: space 및 한컴이 반각으로 처리하는 구두점/기호.
    let (w, width_source) = if c == ' ' {
        if let Some(width) = hancom_pdf_space_width(primary_name, font_size) {
            (width, "metricSpaceOverlay")
        } else {
            (mm.metric.em_size / 2, "metricHalfSpace")
        }
    } else {
        let Some(glyph_w) = mm.metric.get_width(c) else {
            return EmbeddedWidthDecision {
                width_px: None,
                width_source: "metricCharacterMiss",
                metric: Some(mm),
                character_match: "miss",
            };
        };
        let is_halfwidth_punct = matches!(c, '\u{2018}'..='\u{2027}');
        let is_narrow_unicode_punct = matches!(c, '\u{2018}' | '\u{2019}' | '\u{2027}');
        let is_b7_notdef_artifact =
            c == '\u{00B7}' && glyph_w >= mm.metric.em_size && !is_monospace_metric(mm.metric);
        if (is_narrow_unicode_punct && glyph_w >= mm.metric.em_size) || is_b7_notdef_artifact {
            (
                (mm.metric.em_size as f64 * 0.3) as u16,
                "metricNarrowPunctuationOverlay",
            )
        } else if (is_halfwidth_punct
            || (is_halfwidth_cjk_quote(c) && is_monospace_metric(mm.metric)))
            && glyph_w >= mm.metric.em_size
        {
            (mm.metric.em_size / 2, HALFWIDTH_PUNCTUATION_WIDTH_SOURCE)
        } else {
            (glyph_w, "embeddedMetric")
        }
    };
    EmbeddedWidthDecision {
        width_px: Some(quantize_hwp_px(
            w as f64 * font_size / mm.metric.em_size as f64,
        )),
        width_source,
        metric: Some(mm),
        character_match: "hit",
    }
}

pub(crate) fn char_width_decision<'a>(
    chars: &[char],
    cluster_len: &[u8],
    i: usize,
    style: &'a TextStyle,
) -> CharWidthDecision<'a> {
    let (font_size, ratio, _) = style_params(style);
    let c = chars[i];
    if cluster_len[i] == 0 {
        return CharWidthDecision {
            width_source: "clusterContinuation",
            base_width_px: 0.0,
            final_width_px: 0.0,
            metric: None,
            character_match: "notApplicable",
            dash_leader: false,
            negative_spacing_clamped: false,
        };
    }
    if c == '\u{FFFC}' || c == '\u{F081C}' {
        return CharWidthDecision {
            width_source: if c == '\u{FFFC}' {
                "inlineObjectPlaceholder"
            } else {
                "hwpPuaFiller"
            },
            base_width_px: 0.0,
            final_width_px: 0.0,
            metric: None,
            character_match: "notApplicable",
            dash_leader: false,
            negative_spacing_clamped: false,
        };
    }
    if c == '\u{2007}' {
        let base_width_px = font_size * 0.5;
        let final_width_px = base_width_px * ratio
            + glyph_letter_spacing(style.letter_spacing, base_width_px * ratio, font_size)
            + style.extra_char_spacing;
        return CharWidthDecision {
            width_source: "figureSpace",
            base_width_px,
            final_width_px,
            metric: None,
            character_match: "notApplicable",
            dash_leader: false,
            negative_spacing_clamped: false,
        };
    }

    let (base_width_raw, width_source, metric, character_match) = if let Some(w) = (c == '\u{318D}')
        .then(|| area_dot_fallback_width(&style.font_family, font_size))
        .flatten()
    {
        (w, "areaDotFallback", None, "notApplicable")
    } else {
        // [#6172] 사각 안 숫자 PUA(U+F02B0~F02C4) 는 폰트 글리프가 아니라 렌더러가
        // 사각형+숫자로 합성해 그린다(#6127 / PR #6137, `boxed_pua_number`).
        // 그러면 전진폭도 폰트의 (있을 수도 없을 수도 있는) PUA 글리프가 아니라 **합성물이
        // 닮은 `□`(U+25A1) 의 전진폭**으로 재야, 같은 줄에 이어지는 진짜 `□` 와 상자
        // 간격이 균일해진다. 종전에는 이 대역이 아래 어느 분기에도 안 걸려 마지막
        // 폴백(0.5em)으로 떨어졌고, 상자 폭(0.72em)보다 전진폭이 좁아 상자끼리
        // 4.4pt 씩 겹쳤다 — 2599643 1쪽 "󰊲󰊰󰊰□-□□□" (20pt 에서 10.00pt vs □ 20.04pt).
        let c = if crate::renderer::boxed_pua_number(c).is_some() {
            '\u{25A1}'
        } else {
            c
        };
        let embedded = measure_char_width_embedded_decision(
            &style.font_family,
            style.bold,
            style.italic,
            c,
            font_size,
        );
        if let Some(w) = embedded.width_px {
            (
                w,
                embedded.width_source,
                embedded.metric,
                embedded.character_match,
            )
        } else if cluster_len[i] <= 1 && is_unicode_halfwidth_form(c) {
            // [#6023] Halfwidth and Fullwidth Forms 블록(FF00–FFEF)에서
            // FF61–FFDC(｡｢｣､･·반각 가타카나·반각 한글 자모)와 FFE8–FFEE 는
            // 정의상 **반각**이다. is_cjk_char 의 블록 블랭킷이 이들을 전각
            // (1em)으로 오분류해 반각 낫표 ｢｣ 뒤가 전각 공백처럼 벌어졌다
            // (30269 1쪽 한글 2020 PDF 실측: ｢ 전진 7.9pt = 0.5em @ 15.95pt,
            // rhwp 16.0pt). 메트릭 DB 에 항목이 있으면 위 embedded 가 이긴다.
            (
                font_size * 0.5,
                "heuristicHalfwidthForm",
                embedded.metric,
                embedded.character_match,
            )
        } else if cluster_len[i] > 1 || is_cjk_char(c) || is_fullwidth_symbol(c) {
            (
                font_size,
                "heuristicFullwidth",
                embedded.metric,
                embedded.character_match,
            )
        } else if is_narrow_punctuation(c) || is_narrow_paren_for_font(&style.font_family, c) {
            (
                font_size * 0.3,
                "heuristicNarrow",
                embedded.metric,
                embedded.character_match,
            )
        } else {
            (
                font_size * 0.5,
                "heuristicHalfwidth",
                embedded.metric,
                embedded.character_match,
            )
        }
    };
    let dash_leader = is_dash_leader_run(chars, i);
    let base_width_px = if dash_leader {
        base_width_raw.min(font_size * 0.3)
    } else {
        base_width_raw
    };
    let mut final_width_px = base_width_px * ratio
        + glyph_letter_spacing(style.letter_spacing, base_width_px * ratio, font_size)
        + style.extra_char_spacing;
    if c == ' ' {
        final_width_px += style.extra_word_spacing;
    }
    if dash_leader {
        final_width_px += style.extra_dash_advance;
    }
    let mut negative_spacing_clamped = false;
    if style.letter_spacing + style.extra_char_spacing < 0.0 {
        let min_width = base_width_px * ratio * 0.5;
        if final_width_px < min_width {
            final_width_px = min_width;
            negative_spacing_clamped = true;
        }
    }
    CharWidthDecision {
        width_source,
        base_width_px,
        final_width_px,
        metric,
        character_match,
        dash_leader,
        negative_spacing_clamped,
    }
}

fn measure_char_width_embedded(
    font_family: &str,
    bold: bool,
    italic: bool,
    c: char,
    font_size: f64,
) -> Option<f64> {
    measure_char_width_embedded_decision(font_family, bold, italic, c, font_size).width_px
}

// ── 호환 래퍼 (기존 호출부 변경 없음) ──────────────────────────────

/// 텍스트 폭 추정
///
/// 기본 TextMeasurer(EmbeddedTextMeasurer, 내장 메트릭 + 휴리스틱)에 위임한다.
/// native/WASM 공통 — SVG byte 패리티의 전제다 (#4046).
pub(crate) fn estimate_text_width(text: &str, style: &TextStyle) -> f64 {
    default_measurer().estimate_text_width(text, style)
}

/// 텍스트 폭 추정 (round 없이 raw px 반환)
///
/// 줄바꿈 엔진 전용. 단일 문자 토큰의 반올림 누적 오차를 방지한다.
/// 한컴은 HWPUNIT 정수로 폭을 누적하므로, round 없이 px를 합산한 뒤
/// 줄바꿈 비교 시점에서 available_width와 비교하는 것이 더 정확하다.
pub(crate) fn estimate_text_width_unrounded(text: &str, style: &TextStyle) -> f64 {
    let (_, _, tab_w) = style_params(style);
    let chars: Vec<char> = text.chars().collect();
    let cluster_len = build_cluster_len(&chars);
    let char_count = chars.len();

    let char_width =
        |i: usize| -> f64 { char_width_decision(&chars, &cluster_len, i, style).final_width_px };

    let mut total = 0.0;
    for i in 0..char_count {
        if cluster_len[i] == 0 {
            continue;
        }
        let c = chars[i];
        if c == '\t' {
            let abs_x = style.line_x_offset + total;
            let next_abs = ((abs_x / tab_w).floor() + 1.0) * tab_w;
            total = (next_abs - style.line_x_offset).max(total);
            continue;
        }
        total += char_width(i);
    }
    total // round 없이 반환
}

/// 한컴이 폭 변경 뒤 LINE_SEG를 다시 만들 때 쓰는 공백 advance.
///
/// 저장본은 글꼴 고유 공백 폭을 보존할 수 있지만, 한컴의 새 재조판은 반각 공백을
/// 사용한다. 이 규칙을 전역 측정에 넣으면 원본 저장 LINE_SEG의 정합이 깨지므로,
/// stale cell 복구나 LINE_SEG 부재 재조판처럼 새 줄을 만드는 경로만 opt-in한다.
///
/// 저장 metric과 재조판 metric이 같은 style에는 `None`을 반환해 별도 보정이 없도록
/// 한다. 따라서 글꼴명이나 고정 글자 크기에 의존하지 않는다.
pub(crate) fn hancom_regenerated_space_width(style: &TextStyle) -> Option<f64> {
    let (font_size, ratio, _) = style_params(style);
    let base_w = font_size * 0.5;
    let mut width = base_w * ratio
        + glyph_letter_spacing(style.letter_spacing, base_w * ratio, font_size)
        + style.extra_char_spacing
        + style.extra_word_spacing;
    if style.letter_spacing + style.extra_char_spacing < 0.0 {
        width = width.max(base_w * ratio * 0.5);
    }
    let stored_width = estimate_text_width_unrounded(" ", style);
    (width > stored_width + f64::EPSILON).then_some(width)
}

/// 글자별 X 위치 경계값 계산
///
/// N글자 → N+1개 경계값을 반환한다 (0번째는 0.0, N번째는 전체 폭).
/// run 내부 상대 좌표이며, 절대 좌표는 run.bbox.x + charX[i]로 계산한다.
pub(crate) fn compute_char_positions(text: &str, style: &TextStyle) -> Vec<f64> {
    default_measurer().compute_char_positions(text, style)
}

/// 기존 문자 경계값과 exact-font pair positioning을 한 번만 계산하는 공통 진입점.
///
/// R4C의 line/token 소비자와 R4D의 backend replay가 같은 owned measurement를
/// 공유하기 위한 경계다. K0와 exact source 부재에서는 기존
/// [`compute_char_positions`] 결과를 그대로 보존한다.
pub(crate) fn compute_kerning_run_measurement(
    text: &str,
    style: &TextStyle,
    source_handle: Option<&ExactFontSourceHandle>,
    session: &mut KerningSourceSession<'_>,
) -> KerningRunMeasurement {
    let (effective_font_size_px, width_ratio, _) = style_params(style);
    let base_positions = compute_char_positions(text, style);
    super::super::kerning::compute_kerning_run_measurement(
        text,
        style.kerning,
        base_positions,
        effective_font_size_px,
        width_ratio,
        source_handle,
        session,
    )
}

/// 실제 글자 위치 계산과 같은 경로를 사용해 관측 가능한 폭 결정을 반환한다.
/// 탭은 문맥 의존 advance이므로 최종 위치 차이를 정답으로 기록한다.
pub(crate) fn trace_char_width_decisions<'a>(
    text: &str,
    style: &'a TextStyle,
) -> Vec<CharWidthDecision<'a>> {
    let chars: Vec<char> = text.chars().collect();
    let cluster_len = build_cluster_len(&chars);
    let positions = compute_char_positions(text, style);
    chars
        .iter()
        .enumerate()
        .map(|(i, &ch)| {
            let mut decision = char_width_decision(&chars, &cluster_len, i, style);
            if ch == '\t' {
                let advance = positions
                    .get(i + 1)
                    .zip(positions.get(i))
                    .map(|(after, before)| after - before)
                    .unwrap_or(0.0);
                decision.width_source = "tabAdvance";
                decision.base_width_px = advance;
                decision.final_width_px = advance;
                decision.metric = None;
                decision.character_match = "notApplicable";
                decision.dash_leader = false;
                decision.negative_spacing_clamped = false;
            }
            decision
        })
        .collect()
}

// ── 문자 분류 함수 ──────────────────────────────────────────────────

/// CJK 문자 여부 판별 (EmbeddedTextMeasurer의 히우리스틱 폭 계산에서 사용)
pub(crate) fn is_cjk_char(c: char) -> bool {
    ('\u{1100}'..='\u{11FF}').contains(&c)   // 한글 자모
    || ('\u{3130}'..='\u{318F}').contains(&c) // 한글 호환 자모 (ㆍ U+318D 포함)
    || ('\u{AC00}'..='\u{D7AF}').contains(&c) // 한글 음절
    || ('\u{A960}'..='\u{A97F}').contains(&c) // 한글 자모 확장-A (옛한글 초성)
    || ('\u{D7B0}'..='\u{D7FF}').contains(&c) // 한글 자모 확장-B (옛한글 중/종성)
    || ('\u{4E00}'..='\u{9FFF}').contains(&c) // CJK Unified Ideographs
    || ('\u{3400}'..='\u{4DBF}').contains(&c) // CJK Extension A
    || ('\u{F900}'..='\u{FAFF}').contains(&c) // CJK Compatibility
    || ('\u{3040}'..='\u{30FF}').contains(&c) // 히라가나/카타카나
    || ('\u{FF00}'..='\u{FFEF}').contains(&c) // 전각 문자
}

/// 실제 글리프 폭이 반각(em/2)보다 뚜렷이 좁은 구두점·기호.
/// 메트릭 DB 미등록 폰트의 폴백 폭 계산 시 `font_size * 0.5` 대신
/// `font_size * 0.3` 을 쓰도록 분기하는 기준 (Task #257).
///
/// General Punctuation 좁은 글리프 확장: 휴먼명조 U+2027 등 DB 미수록
/// 폰트의 폴백 `font_size * 0.5` 가 한컴 대비 ~10px 과대 (font-size 20px
/// 기준). 한컴은 약 0.25-0.3 em 으로 렌더하므로 동일 분기 적용.
fn is_narrow_punctuation(c: char) -> bool {
    matches!(
        c,
        ',' | '.' | ':' | ';' | '\'' | '"' | '`' |
        '\u{00B7}' |  // · MIDDLE DOT
        '\u{2018}' |  // ' LEFT SINGLE QUOTATION MARK
        '\u{2019}' |  // ' RIGHT SINGLE QUOTATION MARK
        '\u{2027}' |  // ‧ HYPHENATION POINT
        // [Task #1735] 한글 방점. 렌더 경로에서 좁은 가운데 점(·)으로 치환되므로
        // 측정 폭도 narrow 로 맞춰 측정-렌더 폭 정합 유지(0.5em 기본 폴백 방지).
        '\u{302E}' |  // 〮 HANGUL SINGLE DOT TONE MARK (방점)
        '\u{302F}' // 〯 HANGUL DOUBLE DOT TONE MARK (쌍방점)
    )
}

/// [#2239] 괄호 '(' ')' narrow 폭(0.3em) — 사다리 실측 폰트 한정.
///
/// 통제 사다리 실측(#2195 stage30/31): 휴먼명조 '(' = 0.31em(embedded 정합),
/// 한양중고딕 '(' <= 317HU(0.29em) — fallback 0.5em 은 과대
/// (76076 표325 r0 '(정량)영향집단명' 11pt: 8800>8642 로 2줄, 한글 1줄).
/// 단 HY신명조·바탕 계열은 0.5em(#2156 ASCII 폭 표 정합)이므로 폰트 무관
/// `is_narrow_punctuation` 전역 분류는 금지 — 실측된 폰트에서만 좁힌다.
/// (KoPub 계열은 `kopub_char_width` 자체 분기에서 별도 실측 근거로 유지.)
fn is_narrow_paren_for_font(font_family: &str, c: char) -> bool {
    if !matches!(c, '(' | ')') {
        return false;
    }
    let primary = font_family.split(',').next().unwrap_or(font_family).trim();
    primary.contains("휴먼명조") || primary.contains("한양중고딕") || primary.contains("HY중고딕")
}

/// [#6023] Halfwidth and Fullwidth Forms 블록의 **반각** 구간.
///
/// FF00–FF60(전각 ASCII 변형)·FFE0–FFE6(전각 기호)은 전각이 맞지만,
/// FF61–FFDC(｡｢｣､･, 반각 가타카나, 반각 한글 자모)와 FFE8–FFEE(반각 기호)는
/// 유니코드 정의상 반각이다. 폴백 폭 분류에서 이 구간을 전각 블랭킷보다
/// 먼저 가른다.
fn is_unicode_halfwidth_form(c: char) -> bool {
    ('\u{FF61}'..='\u{FFDC}').contains(&c) || ('\u{FFE8}'..='\u{FFEE}').contains(&c)
}

/// 한컴이 수평 조판에서 반각 advance 로 처리하는 CJK 낫표.
///
/// 일부 등록 폰트는 `「」` glyph advance 를 전각으로 제공하지만, 한컴 PDF 기준
/// 본문 조판에서는 법령명 낫표 뒤에 전각 공백처럼 보이는 간격이 생기지 않는다
/// (#2020 돋움체 여권신청서).
///
/// 다만 휴먼명조·HY헤드라인M 에서는 한글이 전폭을 쓴다 (#6060). 메트릭 DB 의
/// 「 폭은 두 계열 모두 `em_size` 이므로 일괄 반각 오버레이가 아니라 글꼴별로
/// 갈라야 한다.
pub(crate) fn is_halfwidth_cjk_quote(c: char) -> bool {
    matches!(c, '\u{300C}' | '\u{300D}')
}

/// 페인트가 「」 글리프를 반각 공간에 눌러 그려야 하는지 — **측정과 같은 판정**이어야 한다.
///
/// 측정은 고정폭 메트릭(`is_monospace_metric`)이고 글리프 advance 가 `em_size` 이상일 때만
/// 반각 오버레이를 적용한다. 페인트가 폰트 **이름 목록**으로 따로 판정하면 두 경로가 갈린다.
///
/// - 이름 목록 밖 고정폭(바탕체·궁서체·D2Coding): advance 는 반각인데 글리프는 전각으로
///   그려져 다음 글자와 겹친다.
/// - 이름에 `돋움체` 를 포함하지만 메트릭 DB 밖(KoPub돋움체): advance 는 전각인데 글리프만
///   반각으로 눌려 오른쪽에 빈 공간이 남는다.
///
/// 그래서 판정을 이름이 아니라 측정 결정 하나로 통일한다. 돋움체 반각(#2020)과
/// 휴먼명조·HY헤드라인M 전폭(#6060)은 메트릭만으로 그대로 갈린다.
pub fn forces_halfwidth_cjk_quote(
    font_family: &str,
    bold: bool,
    italic: bool,
    c: char,
    font_size: f64,
) -> bool {
    if !is_halfwidth_cjk_quote(c) {
        return false;
    }
    measure_char_width_embedded_decision(font_family, bold, italic, c, font_size).width_source
        == HALFWIDTH_PUNCTUATION_WIDTH_SOURCE
}

/// 3 개 이상 연속하는 dash leader 시퀀스의 일부 여부 (Task #352).
///
/// 한컴 문서의 빈칸/구분선은 ASCII '-' 반복으로 구성되며, PDF 도 좁은
/// advance 로 렌더된다. 그러나 일부 한글 폰트(HY신명조 등) 의 메트릭 DB 가
/// '-' 글리프 폭을 0.83 em 으로 저장하고 있어 반복 시 자연 폭이
/// 사용 가능 폭을 크게 초과한다. 본 헬퍼로 leader 시퀀스를 식별해
/// 좁은 advance(`font_size * 0.3`) 로 재산출한다.
///
/// 자연 텍스트의 단발 dash(예: "stimulus-driven", "32.-") 는 ≥3 조건을
/// 만족하지 않으므로 영향 없음.
fn is_dash_leader_run(chars: &[char], i: usize) -> bool {
    if chars[i] != '-' {
        return false;
    }
    let mut count = 1usize;
    let mut j = i;
    while j > 0 && chars[j - 1] == '-' {
        count += 1;
        j -= 1;
        if count >= 3 {
            return true;
        }
    }
    let mut j = i;
    while j + 1 < chars.len() && chars[j + 1] == '-' {
        count += 1;
        j += 1;
        if count >= 3 {
            return true;
        }
    }
    false
}

/// 한컴이 전각으로 처리하는 기호 (메트릭 폴백 시 font_size 사용)
fn is_fullwidth_symbol(c: char) -> bool {
    matches!(c,
        '\u{20A9}' |                   // ₩ WON SIGN
        '\u{20AC}' |                   // € EURO SIGN
        '\u{00A3}' |                   // £ POUND SIGN
        '\u{00A5}'                     // ¥ YEN SIGN
    )
    || ('\u{2190}'..='\u{21FF}').contains(&c) // Arrows (→, ⇨, ⇒ 등)
    || ('\u{2460}'..='\u{24FF}').contains(&c) // Enclosed Alphanumerics (①②③ 등)
    || ('\u{25A0}'..='\u{25FF}').contains(&c) // Geometric Shapes (□■▲◆○ 등, 섹션 머리 기호)
    || ('\u{2600}'..='\u{26FF}').contains(&c) // Miscellaneous Symbols (☆★ 등)
    || ('\u{2700}'..='\u{27BF}').contains(&c) // Dingbats (✓✗ 등)
    || ('\u{3200}'..='\u{32FF}').contains(&c) // Enclosed CJK Letters (㉠㉡ 등)
    || ('\u{3300}'..='\u{33FF}').contains(&c) // CJK Compatibility (㎜㎝ 등)
    || ('\u{2160}'..='\u{217F}').contains(&c) // Roman Numerals (Ⅰ Ⅱ Ⅲ 등)
}

/// 한글 자모 초성 여부 (옛한글 포함)
fn is_hangul_choseong(c: char) -> bool {
    ('\u{1100}'..='\u{115F}').contains(&c) || ('\u{A960}'..='\u{A97F}').contains(&c)
}

/// 한글 자모 중성 여부 (옛한글 포함, ᆞ U+119E 포함)
fn is_hangul_jungseong(c: char) -> bool {
    ('\u{1160}'..='\u{11A7}').contains(&c) || ('\u{D7B0}'..='\u{D7C6}').contains(&c)
}

/// 한글 자모 종성 여부 (옛한글 포함)
fn is_hangul_jongseong(c: char) -> bool {
    ('\u{11A8}'..='\u{11FF}').contains(&c) || ('\u{D7CB}'..='\u{D7FB}').contains(&c)
}

/// 텍스트를 렌더링 클러스터 단위로 분할한다.
/// 한글 자모 조합 시퀀스(초+중+종)를 하나의 클러스터로 묶어
/// 옛한글(아래아 등)이 올바르게 합성될 수 있도록 한다.
/// 반환값: Vec<(시작_문자_인덱스, 클러스터_문자열)>
pub fn split_into_clusters(text: &str) -> Vec<(usize, String)> {
    let chars: Vec<char> = text.chars().collect();
    let mut clusters: Vec<(usize, String)> = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        // 초성으로 시작하는 자모 조합 시퀀스 감지
        if is_hangul_choseong(chars[i]) {
            let start = i;
            let mut cluster = String::new();
            cluster.push(chars[i]);
            i += 1;
            // 중성 (필수)
            if i < chars.len() && is_hangul_jungseong(chars[i]) {
                cluster.push(chars[i]);
                i += 1;
                // 종성 (선택)
                if i < chars.len() && is_hangul_jongseong(chars[i]) {
                    cluster.push(chars[i]);
                    i += 1;
                }
            }
            clusters.push((start, cluster));
        } else {
            clusters.push((i, chars[i].to_string()));
            i += 1;
        }
    }
    clusters
}

/// 세로쓰기에서 CW 90° 회전해야 하는 문자 판별
///
/// text_direction과 무관하게 항상 회전되는 문자:
/// - 괄호류: ( ) [ ] { } < > 〈 〉 《 》 「 」 『 』 【 】
/// - 문장부호: . , _ - ~ … ― ─
pub(crate) fn is_vertical_rotate_char(c: char) -> bool {
    matches!(
        c,
        '(' | ')' | '[' | ']' | '{' | '}' | '<' | '>'
        | '.' | ',' | '_' | '-' | '~'
        | '\u{2026}' // … (ellipsis)
        | '\u{2015}' // ― (horizontal bar)
        | '\u{2500}' // ─ (box drawing horizontal)
        | '\u{2014}' // — (em dash)
        | '\u{2013}' // – (en dash)
        | '\u{3008}' | '\u{3009}' // 〈 〉
        | '\u{300A}' | '\u{300B}' // 《 》
        | '\u{300C}' | '\u{300D}' // 「 」
        | '\u{300E}' | '\u{300F}' // 『 』
        | '\u{3010}' | '\u{3011}' // 【 】
        | '\u{FF08}' | '\u{FF09}' // （ ）
        | '\u{FF3B}' | '\u{FF3D}' // ［ ］
        | '\u{FF5B}' | '\u{FF5D}' // ｛ ｝
    )
}

/// 세로쓰기 기호 대체: 수평 형태 → 세로 형태 Unicode 변환
///
/// CJK Compatibility Forms (U+FE30-FE4F) 및 Vertical Forms 활용.
/// 대체 가능한 문자가 있으면 Some(세로형태)를 반환하고,
/// 없으면 None을 반환한다 (호출측에서 회전 처리).
pub(crate) fn vertical_substitute_char(c: char) -> Option<char> {
    match c {
        // 괄호류
        '(' | '\u{FF08}' => Some('\u{FE35}'), // ︵
        ')' | '\u{FF09}' => Some('\u{FE36}'), // ︶
        '{' | '\u{FF5B}' => Some('\u{FE37}'), // ︷
        '}' | '\u{FF5D}' => Some('\u{FE38}'), // ︸
        '[' | '\u{FF3B}' => Some('\u{FE39}'), // ︹
        ']' | '\u{FF3D}' => Some('\u{FE3A}'), // ︺
        '\u{3010}' => Some('\u{FE3B}'),       // 【 → ︻
        '\u{3011}' => Some('\u{FE3C}'),       // 】 → ︼
        '\u{3008}' => Some('\u{FE3F}'),       // 〈 → ︿
        '\u{3009}' => Some('\u{FE40}'),       // 〉 → ﹀
        '\u{300A}' => Some('\u{FE3D}'),       // 《 → ︽
        '\u{300B}' => Some('\u{FE3E}'),       // 》 → ︾
        '\u{300C}' => Some('\u{FE41}'),       // 「 → ﹁
        '\u{300D}' => Some('\u{FE42}'),       // 」 → ﹂
        '\u{300E}' => Some('\u{FE43}'),       // 『 → ﹃
        '\u{300F}' => Some('\u{FE44}'),       // 』 → ﹄
        // 대시/선
        '\u{2014}' => Some('\u{FE31}'), // — → ︱ (em dash)
        '\u{2013}' => Some('\u{FE32}'), // – → ︲ (en dash)
        '\u{2015}' => Some('\u{FE31}'), // ― → ︱ (horizontal bar)
        '\u{2500}' => Some('\u{2502}'), // ─ → │ (box drawing)
        // 말줄임
        '\u{2026}' => Some('\u{FE19}'), // … → ︙ (vertical ellipsis)
        // 물결표
        '~' => Some('\u{FE34}'), // ~ → ︴ (vertical wavy low line)
        // 밑줄
        '_' => Some('\u{FE33}'), // _ → ︳ (vertical low line)
        _ => None,
    }
}

// ── 테스트 ──────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn issue_4442_noto_sans_kr_regular_ascii_shared_width_uses_tracked_advances() {
        let style = TextStyle {
            font_family: "Noto Sans KR".to_string(),
            font_size: 1000.0,
            ..Default::default()
        };

        assert_eq!(estimate_text_width_unrounded("AVATAR", &style), 3633.0);
        assert_eq!(
            compute_char_positions("AVATAR", &style),
            vec![0.0, 608.0, 1183.0, 1791.0, 2390.0, 2998.0, 3633.0]
        );
    }

    /// [진단] 흔한 폰트의 라틴 metric 커버리지 — 없으면 0.5em 폴백(캐럿·재래핑 부정확).
    /// `cargo test --lib latin_metric_coverage_report -- --nocapture` 로 표를 본다.
    #[test]
    fn latin_metric_coverage_report() {
        let fonts = [
            "바탕",
            "돋움",
            "굴림",
            "궁서",
            "맑은 고딕",
            "함초롬바탕",
            "함초롬돋움",
            "한컴바탕",
            "한컴돋움",
            "HY견고딕",
            "HY견명조",
            "HY중고딕",
            "HY신명조",
            "휴먼명조",
            "한양신명조",
            "중고딕",
            "Noto Sans KR",
            "Noto Serif KR",
            "Batang",
            "Dotum",
            "Gulim",
            "Arial",
            "Times New Roman",
            "Calibri",
            "KoPub바탕체",
            "KoPub돋움체",
            "나눔고딕",
            "나눔명조",
            "굴림체",
            "돋움체",
        ];
        let mut have = 0;
        for f in fonts {
            let ok = font_family_has_metrics(f, false, false);
            // 함초롬바탕/HCR 은 자동 생성 hmtx 표를 타므로
            // metric 유무와 무관하게 정확하다 — 표시해 둔다.
            // 함초롬(HCR)·KoPub 은 별도 per-glyph 경로가 있어 metric 유무와 무관하게 정확하다.
            let dedicated = f.contains("함초롬")
                || f.contains("HCR")
                || f.contains("KoPub")
                || measure_char_width_embedded(f, false, false, 'E', 100.0).is_some();
            let tag = if ok {
                "metric 있음"
            } else if dedicated {
                "전용 표(정확)"
            } else {
                "→ 0.5em 폴백"
            };
            println!("  {f:<22} {tag}");
            have += (ok || dedicated) as u32;
        }
        println!(
            "
  {}/{} 정확, 나머지는 라틴 0.5em 폴백",
            have,
            fonts.len()
        );
        // 회귀 가드 — 흔한 폰트 커버리지가 떨어지면 캐럿·재래핑이 부정확해진다(§4.29).
        assert!(have >= 29, "라틴 metric 커버리지 회귀: {have}/30");
    }

    /// 테스트용 고정 폭 텍스트 측정기
    ///
    /// 모든 문자를 동일한 폭으로 측정한다.
    /// 결정론적 테스트와 레이아웃 로직 검증에 사용한다.
    pub struct MockTextMeasurer {
        pub char_width: f64,
    }

    impl TextMeasurer for MockTextMeasurer {
        fn estimate_text_width(&self, text: &str, style: &TextStyle) -> f64 {
            let (font_size, ratio, tab_w) = style_params(style);
            let chars: Vec<char> = text.chars().collect();
            let cluster_len = build_cluster_len(&chars);
            let mut total = 0.0;
            for i in 0..chars.len() {
                if cluster_len[i] == 0 {
                    continue;
                }
                if chars[i] == '\t' {
                    total = ((total / tab_w).floor() + 1.0) * tab_w;
                    continue;
                }
                total += self.char_width * ratio + style.letter_spacing + style.extra_char_spacing;
                if chars[i] == ' ' {
                    total += style.extra_word_spacing;
                }
            }
            total
        }

        fn compute_char_positions(&self, text: &str, style: &TextStyle) -> Vec<f64> {
            let (font_size, ratio, tab_w) = style_params(style);
            let chars: Vec<char> = text.chars().collect();
            let cluster_len = build_cluster_len(&chars);
            let mut positions = Vec::with_capacity(chars.len() + 1);
            let mut x = 0.0;
            positions.push(x);
            for i in 0..chars.len() {
                if cluster_len[i] == 0 {
                    positions.push(x);
                    continue;
                }
                if chars[i] == '\t' {
                    x = ((x / tab_w).floor() + 1.0) * tab_w;
                    positions.push(x);
                    continue;
                }
                x += self.char_width * ratio + style.letter_spacing + style.extra_char_spacing;
                if chars[i] == ' ' {
                    x += style.extra_word_spacing;
                }
                positions.push(x);
            }
            positions
        }
    }

    // ── #4701 함초롬바탕 라틴 = 폰트 hmtx ──

    /// 함초롬바탕(HCR Batang)의 비한글 문자는 **폰트 자신의 hmtx** 로 잰다.
    ///
    /// #2156 이 넣고 §4.31(`69bb0813d`)이 값을 갈아끼운 `HAANSOFT_BATANG_ASCII`
    /// 오버라이드를 제거한 자리다. 그 표는 한글 PDF 의 `가`↔ASCII origin 델타를
    /// **한국어 전각으로 정규화**해 만들었는데, 이 폰트의 한글 advance 가 0.970em
    /// 이라 전 ASCII 가 `1/0.970 = 1.0309` 배 부풀었다. `HANBatang.ttf` hmtx 대비
    /// 평균 +3.54% 였고, 표에 0.970 을 되곱하면 +1.11% 로 줄어 나머지가 origin
    /// 델타 방식의 추출 잡음임이 드러났다(#4701).
    ///
    /// 잡음의 증거: 이 폰트가 같은 advance(0.5500)를 주는 숫자 0~9 에 옛 표는
    /// 0.5651 과 0.5774 두 값을 배정했다. 폰트 메트릭이라면 있을 수 없는 차이다.
    ///
    /// 눈에 띄는 증상은 목차 점선 리더였다 — `·`(U+00B7)가 0.3330 으로 +4.1%
    /// 넓어 줄당 85개가 누적되면 줄 끝에서 약 21px 벌어졌다.
    ///
    /// 자동 생성 표(`font_metrics_data`)는 TTF 에서 추출한 것이라 hmtx 와 불일치가
    /// 0 이다. 그래서 고칠 것은 표가 아니라 그것을 가로채던 오버라이드였다.
    #[test]
    fn issue_4701_hcr_batang_latin_uses_font_hmtx() {
        let fs = 40.0 / 3.0; // 10pt = 13.333px
        let w = |c: char| {
            measure_char_width_embedded("함초롬바탕", false, false, c, fs)
                .unwrap_or_else(|| panic!("측정 실패: {c:?}"))
        };
        let hcr_batang = |c: char| {
            measure_char_width_embedded("HCR Batang", false, false, c, fs)
                .unwrap_or_else(|| panic!("HCR Batang 측정 실패: {c:?}"))
        };
        // HANBatang.ttf hmtx 실측 (upm=1000). 브라우저 measureText 실측과도 일치한다.
        for (c, em) in [
            ('(', 0.3200),
            (',', 0.3200),
            ('.', 0.3200),
            ('·', 0.3200),
            ('0', 0.5500),
            ('9', 0.5500),
            ('A', 0.7060),
        ] {
            assert!(
                (w(c) - fs * em).abs() < 0.05,
                "{c:?} {} ≠ {em}em (hmtx)",
                w(c)
            );
        }
        // 폰트가 같은 폭을 주는 글자에는 같은 값이 나와야 한다 — 옛 표의 잡음 재발 가드.
        for pair in [('0', '9'), ('(', ')'), ('.', ',')] {
            assert_eq!(w(pair.0), w(pair.1), "{pair:?} 는 hmtx 가 동일하다");
        }
        assert_eq!(w('('), hcr_batang('('), "별칭도 같은 경로를 타야 함");
        // 한글 음절·공백은 기존 경로(HCR hmtx / useFontSpace=0 em/2) 유지.
        assert!(
            (w('가') - fs * 0.9700).abs() < 0.05,
            "한글 {} ≠ 0.97em",
            w('가')
        );
    }

    // ── #2279 한컴돋움/한컴바탕 = Haansoft 실메트릭 ──

    /// 한컴돋움/한컴바탕의 실체는 Haansoft Dotum/Batang (HDOTUM.TTF/HBATANG.TTF
    /// name table 한국어명). 한글 PDF 실측(36398599 pi35 단일줄 무신축 '*' run:
    /// 0.583em, 한글 음절 1.0em)과 hmtx 가 일치 — HCR(함초롬) 메트릭('*' 0.498,
    /// 음절 0.97em)으로 회귀하면 '*' 마스킹 구분선·본문 래핑 줄수가 한글 대비
    /// ±1 이탈한다 (92 컨트롤셋 36398599/36399105 −1쪽 계열).
    #[test]
    fn issue_2279_hancom_dotum_batang_use_haansoft_metrics() {
        let fs = 20.0; // 15pt
        let w = |fam: &str, c: char| {
            measure_char_width_embedded(fam, false, false, c, fs)
                .unwrap_or_else(|| panic!("측정 실패: {fam} {c:?}"))
        };
        // 한컴돋움 = Haansoft Dotum
        assert!(
            (w("한컴돋움", '*') - fs * 0.583).abs() < 0.05,
            "'*' {}",
            w("한컴돋움", '*')
        );
        assert!(
            (w("한컴돋움", '0') - fs * 0.583).abs() < 0.05,
            "'0' {}",
            w("한컴돋움", '0')
        );
        assert!(
            (w("한컴돋움", '가') - fs * 1.0).abs() < 0.05,
            "'가' {}",
            w("한컴돋움", '가')
        );
        // 한컴바탕 = Haansoft Batang (음절 1.0em; ASCII 는 #2156 표와 동일)
        assert!(
            (w("한컴바탕", '가') - fs * 1.0).abs() < 0.05,
            "'가' {}",
            w("한컴바탕", '가')
        );
        assert!(
            (w("한컴바탕", '*') - fs * 0.5).abs() < 0.05,
            "'*' {}",
            w("한컴바탕", '*')
        );
        // 함초롬돋움은 종전대로 HCR Dotum 메트릭 유지 (한글 대체 여부 미실측)
        assert!(
            (w("함초롬돋움", '가') - fs * 0.97).abs() < 0.05,
            "HCR '가' {}",
            w("함초롬돋움", '가')
        );
        // ㆍ(U+318D): 한컴 계열은 area_dot 폴백 대신 embedded 메트릭(1.0em) 신뢰
        assert!(area_dot_fallback_width("한컴돋움", fs).is_none());
        assert!(area_dot_fallback_width("한컴바탕", fs).is_none());
    }

    // ── #2430 한양·휴먼 HFT 실측 메트릭의 native/WASM 정합 보장 ──

    /// 한양 4종·휴먼명조의 ASCII 전 구간(0x20..=0x7E)이 embedded 메트릭으로
    /// 해소됨을 고정한다. WASM 도 native 와 같은 EmbeddedTextMeasurer 를
    /// 쓰므로(#4046 통일), 이 커버리지가 성립하는 한 원본 글꼴이 없는
    /// Studio 환경(HY 대체 글리프 표시)에서도 줄바꿈·캐럿·선택 좌표를
    /// 결정하는 문자폭은 native 와 동일하다 — hybrid(HFT 실측 메트릭 +
    /// HY 대체 표시) 정책의 레이아웃 정합 근거.
    /// 회귀 시(원명 미해소 → 휴리스틱 폴백) embedded 커버리지 구멍이
    /// 셀 재래핑 줄수를 바꾼다 (#2430 재래핑 오발동의 재발 형태).
    #[test]
    fn issue_2430_hft_faces_ascii_embedded_coverage() {
        let fs = 40.0 / 3.0; // 10pt = 13.333px
        for fam in [
            "한양신명조",
            "한양중고딕",
            "한양견명조",
            "한양견고딕",
            "휴먼명조",
        ] {
            for code in 0x20..=0x7Eu32 {
                let c = char::from_u32(code).unwrap();
                let w =
                    measure_char_width_embedded(fam, false, false, c, fs).unwrap_or_else(|| {
                        panic!("{fam} {c:?}: embedded 메트릭 미해소 — Canvas 폴백 회귀")
                    });
                assert!(w > 0.0, "{fam} {c:?}: 비정상 폭 {w}");
            }
        }
        // 실측 스팟 체크 (tools/task2430/measured/ ladder 실측 = 커밋 테이블):
        // 명조·중고딕 계열 숫자 0.497em, 견 계열 0.565em.
        let w = |fam: &str, c: char| measure_char_width_embedded(fam, false, false, c, fs).unwrap();
        assert!(
            (w("한양신명조", '0') - fs * 0.497).abs() < 0.05,
            "신명조 '0' {}",
            w("한양신명조", '0')
        );
        assert!(
            (w("휴먼명조", '0') - fs * 0.497).abs() < 0.05,
            "휴먼명조 '0' {}",
            w("휴먼명조", '0')
        );
        assert!(
            (w("한양견명조", '0') - fs * 0.565).abs() < 0.05,
            "견명조 '0' {}",
            w("한양견명조", '0')
        );
        assert!(
            (w("한양견고딕", '0') - fs * 0.565).abs() < 0.05,
            "견고딕 '0' {}",
            w("한양견고딕", '0')
        );
    }

    /// #3820 — 한양중고딕 원명 space만 한컴 PDF p35의 word gap으로 보정한다.
    /// 실제 TTF hmtx 생성 테이블을 변경하지 않아 HY중고딕과 다른 Hanyang face의
    /// 일반 반각 space 계약은 그대로다.
    #[test]
    fn issue_3820_hanyang_junggothic_space_uses_pdf_advance_only() {
        let fs = 40.0 / 3.0; // 10pt = 13.333px
        let hanyang = measure_char_width_embedded("한양중고딕", false, false, ' ', fs)
            .expect("한양중고딕 space metric");
        let hy = measure_char_width_embedded("HY중고딕", false, false, ' ', fs)
            .expect("HY중고딕 space metric");
        let other = measure_char_width_embedded("한양견고딕", false, false, ' ', fs)
            .expect("한양견고딕 space metric");

        assert!(
            (hanyang - quantize_hwp_px(fs * 550.0 / 1024.0)).abs() < f64::EPSILON,
            "한양중고딕 PDF space advance={hanyang:.3}"
        );
        assert!(
            (hy - quantize_hwp_px(fs * 0.5)).abs() < f64::EPSILON,
            "HY중고딕 일반 반각 space={hy:.3}"
        );
        assert!(
            (other - quantize_hwp_px(fs * 0.5)).abs() < f64::EPSILON,
            "다른 한양 face 일반 반각 space={other:.3}"
        );
    }

    /// #3820 — 12/14pt standard body의 한양신명조 411/1024em 보정이 접수증 10pt 날짜
    /// run까지 넓어지면 공백 누적으로 `㊞` anchor가 도장 원 밖으로 이동한다. 크기 하한을
    /// 고정해 issue1949 원본 HWP line boundary·p81의 line decision·복학원서 일반 반각을 함께
    /// 보존한다. 분할 stale-cell의 한컴 재조판 규칙은 별도 helper로만 적용한다.
    #[test]
    fn issue_3820_hanyang_shinmyeongjo_space_is_standard_body_only() {
        let standard_body_fs = 16.0; // 12pt
        let p81_fs = 56.0 / 3.0; // 14pt = 18.666px
        let receipt_fs = 40.0 / 3.0; // 10pt = 13.333px
        let standard_body =
            measure_char_width_embedded("한양신명조", false, false, ' ', standard_body_fs)
                .expect("한양신명조 12pt space metric");
        let p81 = measure_char_width_embedded("한양신명조", false, false, ' ', p81_fs)
            .expect("한양신명조 14pt space metric");
        let receipt = measure_char_width_embedded("한양신명조", false, false, ' ', receipt_fs)
            .expect("한양신명조 10pt space metric");

        assert!(
            (standard_body - quantize_hwp_px(standard_body_fs * 411.0 / 1024.0)).abs()
                < f64::EPSILON,
            "issue1949 원본 한양신명조 12pt PDF space advance={standard_body:.3}"
        );
        assert!(
            (p81 - quantize_hwp_px(p81_fs * 411.0 / 1024.0)).abs() < f64::EPSILON,
            "p81 한양신명조 14pt PDF space advance={p81:.3}"
        );
        assert!(
            (receipt - quantize_hwp_px(receipt_fs * 0.5)).abs() < f64::EPSILON,
            "접수증 한양신명조 10pt 일반 반각 space={receipt:.3}"
        );

        let stale_split_style = TextStyle {
            font_family: "한양신명조".to_string(),
            font_size: standard_body_fs,
            ..Default::default()
        };
        assert!(
            (hancom_regenerated_space_width(&stale_split_style).expect("split 12pt space")
                - standard_body_fs * 0.5)
                .abs()
                < f64::EPSILON,
            "#4138 split stale-cell 12pt space must be half-em"
        );
    }

    // ── MockTextMeasurer 테스트 ──

    #[test]
    fn test_mock_measurer_fixed_width() {
        let m = MockTextMeasurer { char_width: 10.0 };
        let style = TextStyle {
            font_size: 16.0,
            ..Default::default()
        };
        let w = m.estimate_text_width("ABC", &style);
        assert!((w - 30.0).abs() < 0.01, "expected 30.0, got {}", w);
    }

    #[test]
    fn test_mock_measurer_positions() {
        let m = MockTextMeasurer { char_width: 10.0 };
        let style = TextStyle {
            font_size: 16.0,
            ..Default::default()
        };
        let pos = m.compute_char_positions("AB", &style);
        assert_eq!(pos.len(), 3);
        assert!((pos[0]).abs() < 0.01);
        assert!((pos[1] - 10.0).abs() < 0.01);
        assert!((pos[2] - 20.0).abs() < 0.01);
    }

    #[test]
    fn test_mock_measurer_ratio() {
        let m = MockTextMeasurer { char_width: 10.0 };
        let style = TextStyle {
            font_size: 16.0,
            ratio: 0.5,
            ..Default::default()
        };
        let w = m.estimate_text_width("AB", &style);
        assert!(
            (w - 10.0).abs() < 0.01,
            "expected 10.0 (2*10*0.5), got {}",
            w
        );
    }

    #[test]
    fn test_mock_measurer_letter_spacing() {
        let m = MockTextMeasurer { char_width: 10.0 };
        let style = TextStyle {
            font_size: 16.0,
            letter_spacing: 2.0,
            ..Default::default()
        };
        let w = m.estimate_text_width("AB", &style);
        assert!(
            (w - 24.0).abs() < 0.01,
            "expected 24.0 (2*(10+2)), got {}",
            w
        );
    }

    #[test]
    fn test_mock_measurer_extra_word_spacing() {
        let m = MockTextMeasurer { char_width: 10.0 };
        let style = TextStyle {
            font_size: 16.0,
            extra_word_spacing: 5.0,
            ..Default::default()
        };
        // "A B" = A(10) + space(10+5) + B(10) = 35
        let w = m.estimate_text_width("A B", &style);
        assert!((w - 35.0).abs() < 0.01, "expected 35.0, got {}", w);
    }

    #[test]
    fn test_unicode_arrow_uses_symbol_advance() {
        let style = TextStyle {
            font_family: "KoPub돋움체 Light".to_string(),
            font_size: 10.0,
            ..Default::default()
        };

        let arrow = estimate_text_width("⇒", &style);
        let ascii = estimate_text_width("A", &style);
        assert!(
            arrow > ascii,
            "arrow should use symbol advance, arrow={arrow}, ascii={ascii}"
        );
    }

    #[test]
    fn test_mock_measurer_tab() {
        let m = MockTextMeasurer { char_width: 10.0 };
        let style = TextStyle {
            font_size: 16.0,
            ..Default::default()
        };
        // tab_w = font_size * 4 = 64, "\tA" → tab snaps to 64, then A at 74
        let pos = m.compute_char_positions("\tA", &style);
        assert_eq!(pos.len(), 3);
        assert!(
            (pos[1] - 64.0).abs() < 0.01,
            "tab should snap to 64, got {}",
            pos[1]
        );
        assert!(
            (pos[2] - 74.0).abs() < 0.01,
            "A should be at 74, got {}",
            pos[2]
        );
    }

    // ── EmbeddedTextMeasurer 테스트 ──

    #[test]
    fn test_embedded_measurer_latin_heuristic() {
        let m = EmbeddedTextMeasurer;
        let style = TextStyle {
            font_size: 16.0,
            ..Default::default()
        };
        // 기본 폰트("")는 내장 메트릭 없음 → 휴리스틱: Latin = font_size * 0.5
        let w = m.estimate_text_width("AB", &style);
        assert!(
            (w - 16.0).abs() < 0.01,
            "expected 16.0 (2*8.0 heuristic), got {}",
            w
        );
    }

    #[test]
    fn test_embedded_measurer_cjk_heuristic() {
        let m = EmbeddedTextMeasurer;
        let style = TextStyle {
            font_size: 16.0,
            ..Default::default()
        };
        // 기본 폰트("")는 내장 메트릭 없음 → 휴리스틱: CJK = font_size
        let w = m.estimate_text_width("가나", &style);
        assert!(
            (w - 32.0).abs() < 0.01,
            "expected 32.0 (2*16.0 heuristic), got {}",
            w
        );
    }

    #[test]
    fn test_kopub_dotum_hangul_872_advance() {
        let m = EmbeddedTextMeasurer;
        let style = TextStyle {
            font_family: "KoPub돋움체 Light".to_string(),
            font_size: 14.0,
            ..Default::default()
        };

        // [#6389] KoPub돋움체 한글 전각 872/1000em — 편람 kopub 오라클 PDF 임베드
        // 서브셋 CIDFont /W 직독(전 웨이트 1,203 글리프 만장일치 872). 종전 1.0
        // 은 KoPub 미설치 환경의 바탕 치환 렌더(86712)를 face 상수로 오인한 값.
        // 14px × 0.872 = 12.208 → HWPUNIT 양자화 12.2/자, 두 글자 24.4 를
        // estimate_text_width 가 총폭 round 해 24.0 (줄바꿈 비교는 unrounded).
        let w = m.estimate_text_width("가나", &style);
        assert_eq!(w, 24.0);
    }

    #[test]
    fn test_embedded_measurer_known_font() {
        let m = EmbeddedTextMeasurer;
        let style = TextStyle {
            font_family: "함초롬돋움".to_string(),
            font_size: 16.0,
            ..Default::default()
        };
        // 내장 메트릭이 있는 폰트: Latin 문자는 CJK보다 좁아야 함
        let w = m.estimate_text_width("A", &style);
        assert!(
            w > 0.0 && w < 16.0,
            "Latin 'A' should be narrower than CJK, got {}",
            w
        );
    }

    #[test]
    fn test_embedded_matches_free_fn() {
        // 자유 함수 래퍼가 EmbeddedTextMeasurer로 위임하는지 확인
        let style = TextStyle {
            font_size: 16.0,
            ..Default::default()
        };
        let free_fn_result = estimate_text_width("ABC가나다", &style);
        let trait_result = EmbeddedTextMeasurer.estimate_text_width("ABC가나다", &style);
        assert!(
            (free_fn_result - trait_result).abs() < 0.01,
            "free fn ({}) != trait ({})",
            free_fn_result,
            trait_result,
        );
    }

    #[test]
    fn test_embedded_positions_match_free_fn() {
        let style = TextStyle {
            font_size: 16.0,
            ..Default::default()
        };
        let free_fn_result = compute_char_positions("ABC", &style);
        let trait_result = EmbeddedTextMeasurer.compute_char_positions("ABC", &style);
        assert_eq!(free_fn_result.len(), trait_result.len());
        for (a, b) in free_fn_result.iter().zip(trait_result.iter()) {
            assert!((a - b).abs() < 0.01, "position mismatch: {} != {}", a, b);
        }
    }

    #[test]
    fn test_inline_object_placeholder_has_zero_advance() {
        let style = TextStyle {
            font_family: "Haansoft Dotum".to_string(),
            font_size: 12.0,
            ..Default::default()
        };

        assert_eq!(estimate_text_width("\u{FFFC}", &style), 0.0);
        assert_eq!(
            estimate_text_width("\u{FFFC}\u{FFFC}A", &style),
            estimate_text_width("A", &style),
            "U+FFFC placeholder 는 실제 TAC 노드가 따로 폭을 차지하므로 텍스트 폭에 더하면 안 됨"
        );

        let positions = compute_char_positions("\u{FFFC}A", &style);
        assert_eq!(positions[0], positions[1]);
        assert!(positions[2] > positions[1]);
    }

    // ── 오버플로우 압축 회귀 테스트 (Task #229) ──

    /// 음수 extra_char_spacing (오버플로우 압축)에서 narrow glyph(콤마)가
    /// 뒷 글자에 역진 겹침되지 않아야 한다. compute_char_positions 결과는
    /// 단조 비감소여야 한다.
    #[test]
    fn test_overflow_compression_positions_monotonic_comma() {
        let m = EmbeddedTextMeasurer;
        // 실제 재현 케이스: "65,063,026,600" 을 12pt 맑은 고딕으로,
        // extra_char_spacing = -2.88 (셀 오버플로우 압축 시나리오).
        let style = TextStyle {
            font_family: "맑은 고딕".to_string(),
            font_size: 12.0,
            ratio: 1.0,
            extra_char_spacing: -2.88,
            ..Default::default()
        };
        let positions = m.compute_char_positions("65,063,026,600", &style);
        for win in positions.windows(2) {
            assert!(
                win[1] >= win[0] - 1e-6,
                "positions must be non-decreasing: {:?}",
                positions
            );
        }
    }

    /// 실제 문서 재현 케이스: 압축은 CharShape 의 `letter_spacing` 을 통해 오며
    /// `extra_char_spacing` 은 0 일 수 있다. 가드 조건은 둘의 합이어야 한다.
    #[test]
    fn test_charshape_negative_letter_spacing_no_reverse() {
        let m = EmbeddedTextMeasurer;
        let style = TextStyle {
            font_family: "맑은 고딕".to_string(),
            font_size: 12.0,
            ratio: 1.0,
            letter_spacing: -2.88,
            extra_char_spacing: 0.0,
            ..Default::default()
        };
        let positions = m.compute_char_positions("65,063,026,600", &style);
        for win in positions.windows(2) {
            assert!(
                win[1] >= win[0] - 1e-6,
                "positions must be non-decreasing: {:?}",
                positions
            );
        }
    }

    /// 동일 시나리오에서 ASCII 마침표도 역진되지 않아야 한다.
    #[test]
    fn test_overflow_compression_positions_monotonic_period() {
        let m = EmbeddedTextMeasurer;
        let style = TextStyle {
            font_family: "맑은 고딕".to_string(),
            font_size: 12.0,
            ratio: 1.0,
            extra_char_spacing: -2.88,
            ..Default::default()
        };
        let positions = m.compute_char_positions("526.278", &style);
        for win in positions.windows(2) {
            assert!(
                win[1] >= win[0] - 1e-6,
                "positions must be non-decreasing: {:?}",
                positions
            );
        }
    }

    /// extra_char_spacing == 0 (비-압축) 경로는 클램프의 영향을 받지 않아야 한다.
    /// 21a02ec 이후의 동작과 동일해야 함.
    #[test]
    fn test_non_compression_width_unchanged_by_fix() {
        let m = EmbeddedTextMeasurer;
        let style_a = TextStyle {
            font_family: "맑은 고딕".to_string(),
            font_size: 12.0,
            ratio: 1.0,
            ..Default::default()
        };
        let w = m.estimate_text_width("65,063,026,600", &style_a);
        assert!(
            w > 50.0 && w < 200.0,
            "sanity: non-compression width reasonable, got {}",
            w
        );
    }

    // ── build_cluster_len 테스트 ──

    #[test]
    fn test_build_cluster_len_basic() {
        let chars: Vec<char> = "ABC".chars().collect();
        let cl = build_cluster_len(&chars);
        assert_eq!(cl, vec![1, 1, 1]);
    }

    #[test]
    fn test_build_cluster_len_hangul_jamo() {
        // 초성(ㄱ U+1100) + 중성(ㅏ U+1161) + 종성(ㄴ U+11AB) = 3자 1클러스터
        let chars: Vec<char> = "\u{1100}\u{1161}\u{11AB}".chars().collect();
        let cl = build_cluster_len(&chars);
        assert_eq!(cl, vec![3, 0, 0]);
    }

    #[test]
    fn test_build_cluster_len_mixed() {
        // "A" + 초성+중성 + "B"
        let chars: Vec<char> = "A\u{1100}\u{1161}B".chars().collect();
        let cl = build_cluster_len(&chars);
        assert_eq!(cl, vec![1, 2, 0, 1]);
    }

    // ── narrow glyph advance 회귀 (Task #257) ──
    //
    // `is_narrow_punctuation` 폴백 분기 검증. 메트릭 DB 및 `resolve_metric_alias`
    // 양쪽 모두에 등록되지 않은 이름을 사용해야 폴백 경로가 실제로 실행된다.
    // (과거엔 "HY헤드라인M" 을 사용했으나 Task #259 에서 alias 등록되며 폴백이
    // 우회됨 → 임의의 미등록 이름으로 교체.)
    const UNREGISTERED_FONT: &str = "__rhwp_test_unregistered_font__";

    #[test]
    fn test_narrow_glyph_comma_base_width() {
        let m = EmbeddedTextMeasurer;
        let style = TextStyle {
            font_family: UNREGISTERED_FONT.to_string(),
            font_size: 13.333,
            ratio: 1.0,
            ..Default::default()
        };
        // positions of "A,B": A at 0, , at A-advance, B at A-advance + ,-advance
        let positions = m.compute_char_positions("A,B", &style);
        let comma_advance = positions[2] - positions[1];
        assert!(
            comma_advance <= style.font_size * 0.35,
            "narrow comma advance should be ≤ font_size * 0.35 ({:.2}), got {:.2}",
            style.font_size * 0.35,
            comma_advance
        );
    }

    #[test]
    fn test_narrow_glyph_middle_dot_base_width() {
        let m = EmbeddedTextMeasurer;
        let style = TextStyle {
            font_family: UNREGISTERED_FONT.to_string(),
            font_size: 16.667,
            ratio: 1.0,
            ..Default::default()
        };
        let positions = m.compute_char_positions("가\u{00B7}나", &style);
        let dot_advance = positions[2] - positions[1];
        assert!(
            dot_advance <= style.font_size * 0.35,
            "narrow middle-dot advance should be ≤ font_size * 0.35 ({:.2}), got {:.2}",
            style.font_size * 0.35,
            dot_advance
        );
    }

    /// [#2239] 괄호 narrow(0.3em)는 사다리 실측 폰트(휴먼명조/한양중고딕) 한정.
    /// 미실측(미등록) 폰트의 괄호는 0.5em 폴백 유지 — HY신명조·바탕 계열
    /// 0.5em(#2156 ASCII 폭 표) 회귀 방지.
    #[test]
    fn test_paren_narrow_is_font_conditioned() {
        let m = EmbeddedTextMeasurer;
        // 미등록·미실측 폰트: 괄호는 0.5em 폴백.
        let style = TextStyle {
            font_family: UNREGISTERED_FONT.to_string(),
            font_size: 13.333,
            ratio: 1.0,
            ..Default::default()
        };
        let positions = m.compute_char_positions("A(B", &style);
        let advance = positions[2] - positions[1];
        assert!(
            (advance - style.font_size * 0.5).abs() < 0.5,
            "미실측 폰트 '(' 는 0.5em 폴백이어야 함, got {:.2}",
            advance
        );
        // 한양중고딕(사다리 실측 '(' <= 0.29em): narrow 0.3em.
        let style_hy = TextStyle {
            font_family: "한양중고딕".to_string(),
            font_size: 13.333,
            ratio: 1.0,
            ..Default::default()
        };
        let positions_hy = m.compute_char_positions("A(B", &style_hy);
        let advance_hy = positions_hy[2] - positions_hy[1];
        assert!(
            advance_hy <= style_hy.font_size * 0.45,
            "한양중고딕 '(' 는 narrow(≤0.45em)여야 함, got {:.2}",
            advance_hy
        );
    }

    /// [Task #1735] 방점 U+302E/U+302F 는 렌더 경로에서 좁은 가운데 점(·)으로
    /// 치환·렌더되므로, 측정 폭도 narrow(≤0.35em)로 분류해 측정-렌더 폭 정합을
    /// 유지한다(0.5em 기본 폴백 방지).
    #[test]
    fn test_narrow_glyph_tone_marks() {
        let m = EmbeddedTextMeasurer;
        let style = TextStyle {
            font_family: UNREGISTERED_FONT.to_string(),
            font_size: 16.667,
            ratio: 1.0,
            ..Default::default()
        };
        for text in &["가\u{302E}나", "가\u{302F}나"] {
            let positions = m.compute_char_positions(text, &style);
            let advance = positions[2] - positions[1];
            assert!(
                advance <= style.font_size * 0.35,
                "tone mark advance should be ≤ font_size * 0.35 ({:.2}), got {:.2} for {:?}",
                style.font_size * 0.35,
                advance,
                text
            );
        }
    }

    #[test]
    fn test_narrow_glyph_period_and_colon() {
        let m = EmbeddedTextMeasurer;
        let style = TextStyle {
            font_family: UNREGISTERED_FONT.to_string(),
            font_size: 13.333,
            ratio: 1.0,
            ..Default::default()
        };
        for (ch, text) in &[('.', "A.B"), (':', "A:B")] {
            let positions = m.compute_char_positions(text, &style);
            let advance = positions[2] - positions[1];
            assert!(
                advance <= style.font_size * 0.35,
                "narrow '{}' advance should be ≤ font_size * 0.35 ({:.2}), got {:.2}",
                ch,
                style.font_size * 0.35,
                advance
            );
        }
    }

    #[test]
    fn test_non_narrow_char_unchanged() {
        // 회귀 방어: 영문 'A'·한글 '가' 는 narrow 분기에 해당하지 않아야 한다.
        let m = EmbeddedTextMeasurer;
        let style = TextStyle {
            font_family: UNREGISTERED_FONT.to_string(),
            font_size: 13.333,
            ratio: 1.0,
            ..Default::default()
        };
        // 'A' = Latin 반각 = font_size * 0.5 ≈ 6.67 유지
        let pos_a = m.compute_char_positions("AA", &style);
        let a_advance = pos_a[1] - pos_a[0];
        assert!(
            (a_advance - style.font_size * 0.5).abs() < 0.1,
            "Latin 'A' advance should remain font_size * 0.5 ({:.2}), got {:.2}",
            style.font_size * 0.5,
            a_advance
        );
        // '가' = CJK 전각 = font_size 유지
        let pos_k = m.compute_char_positions("가가", &style);
        let k_advance = pos_k[1] - pos_k[0];
        assert!(
            (k_advance - style.font_size).abs() < 0.1,
            "CJK '가' advance should remain font_size ({:.2}), got {:.2}",
            style.font_size,
            k_advance
        );
    }

    /// Issue #630: 등록된 한글 폰트(돋움체)에서 `·`(U+00B7) 가 전각으로 측정되어야
    /// 한컴 저장본 의 tab_extended 와 정합. `is_halfwidth_punct` 의 강제 반각
    /// 처리는 한컴 측정값과 8.67px(반각 1자) 차이 유발.
    #[test]
    fn test_630_middle_dot_full_width_in_registered_font() {
        let m = EmbeddedTextMeasurer;
        let style = TextStyle {
            font_family: "돋움체".to_string(),
            font_size: 17.333,
            ratio: 1.0,
            ..Default::default()
        };
        let positions = m.compute_char_positions("가\u{00B7}나", &style);
        assert!(positions.len() >= 3, "positions should have ≥ 3 entries");
        let dot_advance = positions[2] - positions[1];

        // 전각 = font_size (≈17.33px). 정정 전: 반각 (≈8.67px).
        // HWPUNIT 양자화 + 폰트 메트릭 미세 차이 허용 ±1.5px.
        let expected = style.font_size;
        assert!(
            (dot_advance - expected).abs() < 1.5,
            "DotumChe 의 `·` (U+00B7) advance 가 전각 (={:.2}) 으로 측정되어야 함, got {:.2}\n\
             정정 전: 반각 (≈{:.2}). is_halfwidth_punct 가 U+00B7 강제 반각 처리 (Issue #630).",
            expected,
            dot_advance,
            expected / 2.0
        );
    }

    #[test]
    fn test_2020_corner_quote_halfwidth_in_registered_font() {
        let m = EmbeddedTextMeasurer;
        let style = TextStyle {
            font_family: "돋움체".to_string(),
            font_size: 13.333,
            ratio: 1.0,
            ..Default::default()
        };

        let positions = m.compute_char_positions("「여", &style);
        let quote_advance = positions[1] - positions[0];
        let hangul_advance = positions[2] - positions[1];

        assert!(
            quote_advance <= style.font_size * 0.6,
            "`「` 는 등록 폰트에서도 반각 advance 로 측정되어야 함. got {:.2}",
            quote_advance
        );
        assert!(
            hangul_advance >= style.font_size * 0.9,
            "뒤따르는 한글은 전각 advance 를 유지해야 함. got {:.2}",
            hangul_advance
        );
    }

    /// [U+00B7 .notdef 위장값 정정] 비례폰트(휴먼명조)에서 `·`(U+00B7) 글리프
    /// 부재로 cmap 이 .notdef(em_size) 로 위장 → 전각 측정되던 것을 narrow 로
    /// 정정한다. 한컴은 점 글리프를 가진 대체 폰트(바탕 ≈0.33em)로 `·` 를
    /// 렌더하므로 한컴 PDF 정합. 고정폭 폰트(돋움체)는 영향 없음 —
    /// test_630_middle_dot_full_width_in_registered_font 가 전각 보존을 가드.
    #[test]
    fn test_b7_notdef_artifact_narrow_in_proportional_font() {
        let m = EmbeddedTextMeasurer;
        let style = TextStyle {
            font_family: "휴먼명조".to_string(),
            font_size: 20.0,
            ratio: 1.0,
            ..Default::default()
        };
        let positions = m.compute_char_positions("가\u{00B7}나", &style);
        assert!(positions.len() >= 3, "positions should have ≥ 3 entries");
        let dot_advance = positions[2] - positions[1];
        // 비례폰트의 .notdef 위장 전각(≈20px) 이 아니라 narrow(0.3em ≈ 6px) 여야 함.
        assert!(
            dot_advance <= style.font_size * 0.4,
            "휴먼명조의 `·` (U+00B7) 는 .notdef 위장 전각이 아니라 narrow \
             (≤ font_size * 0.4 = {:.2}) 로 측정되어야 함, got {:.2}",
            style.font_size * 0.4,
            dot_advance
        );
    }

    // Stage 4 검증으로 native tab_type 정정 (정정 2) 은 회귀 발견되어 철회.
    // HWP5 의 `tab_extended[0]` 가 이미 right-tab 결과 위치 (= 우측 끝 - 한컴_seg_w)
    // 로 저장되어 있어 LEFT fallback 이 인코딩 의도와 정합. 본 테스트는 합성 데이터
    // 기반의 잘못된 가정 (RIGHT 정확 매치) 을 검증하던 것이라 삭제.
}
