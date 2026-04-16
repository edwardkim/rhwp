//! 텍스트 폭 측정, 문자 클러스터 분할, CJK 판별 관련 함수

use super::super::font_metrics_data;
use super::super::style_resolver::ResolvedStyleSet;
use super::super::{hwpunit_to_px, TabLeaderInfo, TabStop, TextStyle};
use crate::model::style::UnderlineType;

// ── TextMeasurer trait ──────────────────────────────────────────────

/// 텍스트 폭 측정 추상화 트레이트
///
/// 플랫폼별 텍스트 측정 구현체를 추상화한다.
/// - EmbeddedTextMeasurer: 내장 폰트 메트릭 기반 (모든 플랫폼)
/// - WasmTextMeasurer: JS Canvas 브릿지 + 내장 메트릭 (WASM 전용)
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

/// 스타일에서 공통 파라미터 추출 (font_size, ratio, tab_w)
fn style_params(style: &TextStyle) -> (f64, f64, f64) {
    let font_size = if style.font_size > 0.0 {
        style.font_size
    } else {
        12.0
    };
    let ratio = if style.ratio > 0.0 { style.ratio } else { 1.0 };
    let tab_w = if style.default_tab_width > 0.0 {
        style.default_tab_width
    } else {
        font_size * 4.0
    };
    (font_size, ratio, tab_w)
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
        // 탭 위치가 사용 가능 너비를 초과하면 available_width로 클램핑
        let pos = if ts.position > available_width && available_width > 0.0 {
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

/// 탭 문자의 위치로부터 탭 리더 정보를 추출한다.
pub fn extract_tab_leaders(text: &str, positions: &[f64], style: &TextStyle) -> Vec<TabLeaderInfo> {
    extract_tab_leaders_with_extended(text, positions, style, &[])
}

/// 탭 리더 추출 (tab_extended 지원)
/// tab_extended: HWPX 인라인 탭 또는 HWP 탭 확장 데이터 (ext[1] = leader/fill_type)
pub fn extract_tab_leaders_with_extended(
    text: &str,
    positions: &[f64],
    style: &TextStyle,
    tab_extended: &[[u16; 7]],
) -> Vec<TabLeaderInfo> {
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

            // 1. tab_extended에서 leader 가져오기 (HWPX 인라인 탭)
            let ext_fill = if tab_idx < tab_extended.len() {
                tab_extended[tab_idx][1] as u8 // ext[1] = leader/fill_type
            } else {
                0
            };

            // 2. TabDef에서 fill_type 가져오기 (HWP TabDef)
            let tabdef_fill = if !style.tab_stops.is_empty() || style.auto_tab_right {
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
                leaders.push(TabLeaderInfo {
                    start_x: before_x,
                    end_x: (after_x - space_gap).max(before_x),
                    fill_type,
                });
            }
            tab_idx += 1;
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
pub struct EmbeddedTextMeasurer;

impl TextMeasurer for EmbeddedTextMeasurer {
    fn estimate_text_width(&self, text: &str, style: &TextStyle) -> f64 {
        let (font_size, ratio, tab_w) = style_params(style);
        let chars: Vec<char> = text.chars().collect();
        let cluster_len = build_cluster_len(&chars);
        let char_count = chars.len();
        let has_custom_tabs = !style.tab_stops.is_empty() || style.auto_tab_right;

        let char_width = |i: usize| -> f64 {
            let c = chars[i];
            if c == '\u{2007}' {
                return font_size * 0.5 * ratio + style.letter_spacing + style.extra_char_spacing;
            }
            let base_w = if let Some(w) = measure_char_width_embedded(
                &style.font_family,
                style.bold,
                style.italic,
                c,
                font_size,
            ) {
                w
            } else if cluster_len[i] > 1 || is_cjk_char(c) || is_fullwidth_symbol(c) {
                font_size
            } else {
                font_size * 0.5
            };
            let mut w = base_w * ratio + style.letter_spacing + style.extra_char_spacing;
            if c == ' ' {
                w += style.extra_word_spacing;
            }
            w
        };

        let mut total = 0.0;
        for i in 0..char_count {
            let c = chars[i];
            if cluster_len[i] == 0 {
                continue;
            }
            if c == '\t' {
                if has_custom_tabs {
                    let abs_x = style.line_x_offset + total;
                    let (tab_pos, tab_type, _) = find_next_tab_stop(
                        abs_x,
                        &style.tab_stops,
                        tab_w,
                        style.auto_tab_right,
                        style.available_width,
                    );
                    let rel_tab = tab_pos - style.line_x_offset;
                    match tab_type {
                        1 => {
                            // 오른쪽
                            let seg_w =
                                measure_segment_from(&chars, &cluster_len, i + 1, &char_width);
                            total = (rel_tab - seg_w).max(total);
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
                } else {
                    // 기본 등간격 탭: 라인 절대 위치(line_x_offset + total) 기준으로 계산
                    let abs_x = style.line_x_offset + total;
                    let next_abs = ((abs_x / tab_w).floor() + 1.0) * tab_w;
                    total = (next_abs - style.line_x_offset).max(total);
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
        let (font_size, ratio, tab_w) = style_params(style);
        let chars: Vec<char> = text.chars().collect();
        let char_count = chars.len();
        let mut positions = Vec::with_capacity(char_count + 1);
        let mut x = 0.0;
        positions.push(x);

        let cluster_len = build_cluster_len(&chars);
        let has_custom_tabs = !style.tab_stops.is_empty() || style.auto_tab_right;

        let char_width = |i: usize| -> f64 {
            let c = chars[i];
            if c == '\u{2007}' {
                return font_size * 0.5 * ratio + style.letter_spacing + style.extra_char_spacing;
            }
            let base_w = if let Some(w) = measure_char_width_embedded(
                &style.font_family,
                style.bold,
                style.italic,
                c,
                font_size,
            ) {
                w
            } else if cluster_len[i] > 1 || is_cjk_char(c) || is_fullwidth_symbol(c) {
                font_size
            } else {
                font_size * 0.5
            };
            let mut w = base_w * ratio + style.letter_spacing + style.extra_char_spacing;
            if c == ' ' {
                w += style.extra_word_spacing;
            }
            w
        };

        let mut tab_char_idx = 0usize; // inline_tabs 인덱스
        for i in 0..char_count {
            let c = chars[i];
            if cluster_len[i] == 0 {
                positions.push(x);
                continue;
            }
            if c == '\t' {
                // HWPX 인라인 탭: inline_tabs에서 width/type 사용
                if tab_char_idx < style.inline_tabs.len() {
                    let ext = &style.inline_tabs[tab_char_idx];
                    let tab_width_px = ext[0] as f64 * 96.0 / 7200.0;
                    let tab_type = ext[2];
                    let tab_target = x + tab_width_px;
                    match tab_type {
                        1 => {
                            // 오른쪽
                            let seg_w =
                                measure_segment_from(&chars, &cluster_len, i + 1, &char_width);
                            x = (tab_target - seg_w).max(x);
                        }
                        2 => {
                            // 가운데
                            let seg_w =
                                measure_segment_from(&chars, &cluster_len, i + 1, &char_width);
                            x = (tab_target - seg_w / 2.0).max(x);
                        }
                        _ => {
                            // 왼쪽(0)
                            x = tab_target.max(x);
                        }
                    }
                    tab_char_idx += 1;
                } else if has_custom_tabs {
                    let abs_x = style.line_x_offset + x;
                    let (tab_pos, tab_type, _) = find_next_tab_stop(
                        abs_x,
                        &style.tab_stops,
                        tab_w,
                        style.auto_tab_right,
                        style.available_width,
                    );
                    let rel_tab = tab_pos - style.line_x_offset;
                    match tab_type {
                        1 => {
                            // 오른쪽
                            let seg_w =
                                measure_segment_from(&chars, &cluster_len, i + 1, &char_width);
                            if tab_type == 1 {
                                eprintln!("[DEBUG_TAB_POS] RIGHT tab: abs_x={:.2}, tab_pos={:.2}, line_x_offset={:.2}, rel_tab={:.2}, seg_w={:.2}, avail_w={:.2}, result_x={:.2}",
                                    abs_x, tab_pos, style.line_x_offset, rel_tab, seg_w, style.available_width, (rel_tab - seg_w).max(x));
                            }
                            x = (rel_tab - seg_w).max(x);
                        }
                        2 => {
                            // 가운데
                            let seg_w =
                                measure_segment_from(&chars, &cluster_len, i + 1, &char_width);
                            x = (rel_tab - seg_w / 2.0).max(x);
                        }
                        _ => {
                            // 왼쪽(0), 소수점(3)
                            x = rel_tab.max(x);
                        }
                    }
                    tab_char_idx += 1;
                } else {
                    // 기본 등간격 탭
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
}

// ── WASM 전용 내부 코드 ─────────────────────────────────────────────
//
// JS Canvas measureText 브릿지, LRU 캐시, HWP 단위 양자화 등
// WASM 빌드에서만 컴파일된다.

#[cfg(target_arch = "wasm32")]
mod wasm_internals {
    use crate::renderer::TextStyle;
    use std::cell::RefCell;
    use wasm_bindgen::prelude::*;

    // globalThis.measureTextWidth(font, text) → width in pixels
    // editor.html/index.html의 <head>에 정의된 글로벌 함수를 호출한다.
    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen(js_namespace = globalThis, js_name = "measureTextWidth")]
        fn js_measure_text_width(font: &str, text: &str) -> f64;
    }

    // ── JS measureText 결과 LRU 캐시 ──
    //
    // js_measure_text_width()는 항상 1000px 고정 크기로 측정하므로
    // (measure_font, char) 쌍을 키로 캐싱하면 모든 font_size에서 재사용 가능하다.
    // WASM은 단일 스레드이므로 thread_local + RefCell로 충분하다.

    /// Vec 기반 LRU 캐시 (256 엔트리)
    ///
    /// 용량 ≤ 256이므로 선형 탐색(수 μs)이 JS 브릿지 호출(~50μs)보다 빠르다.
    /// 용량 초과 시 가장 오래된 25%를 제거한다 (webhwp 방식).
    struct MeasureCache {
        entries: Vec<(u64, f64)>, // (key_hash, raw_px) — 접근 순서 (최근이 뒤)
        capacity: usize,
    }

    impl MeasureCache {
        fn new(capacity: usize) -> Self {
            Self {
                entries: Vec::with_capacity(capacity),
                capacity,
            }
        }

        fn get(&mut self, key: u64) -> Option<f64> {
            if let Some(idx) = self.entries.iter().position(|(k, _)| *k == key) {
                let entry = self.entries.remove(idx);
                let val = entry.1;
                self.entries.push(entry); // MRU로 이동
                Some(val)
            } else {
                None
            }
        }

        fn insert(&mut self, key: u64, value: f64) {
            if self.entries.len() >= self.capacity {
                // 가장 오래된 25% 제거
                let remove_count = self.capacity / 4;
                self.entries.drain(0..remove_count);
            }
            self.entries.push((key, value));
        }
    }

    thread_local! {
        static JS_MEASURE_CACHE: RefCell<MeasureCache> = RefCell::new(MeasureCache::new(256));
    }

    /// 캐시 키 생성: hash(measure_font + char)
    fn measure_cache_key(measure_font: &str, c: char) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut h = DefaultHasher::new();
        measure_font.hash(&mut h);
        c.hash(&mut h);
        h.finish()
    }

    /// JS measureText 캐싱 래퍼
    ///
    /// 캐시 히트 시 WASM↔JS 브릿지 호출 없이 즉시 반환.
    /// 미스 시 js_measure_text_width() 호출 후 결과를 캐시에 저장.
    fn cached_js_measure(measure_font: &str, c: char) -> f64 {
        let key = measure_cache_key(measure_font, c);
        JS_MEASURE_CACHE.with(|cache| {
            if let Some(val) = cache.borrow_mut().get(key) {
                return val;
            }
            let val = js_measure_text_width(measure_font, &c.to_string());
            cache.borrow_mut().insert(key, val);
            val
        })
    }

    /// 1000pt 측정용 CSS font 문자열 생성
    pub(super) fn build_1000pt_font_string(style: &TextStyle) -> String {
        let font_weight = if style.bold { "bold " } else { "" };
        let font_style = if style.italic { "italic " } else { "" };
        let font_family = if style.font_family.is_empty() {
            "sans-serif".to_string()
        } else {
            let fallback = crate::renderer::generic_fallback(&style.font_family);
            format!("\"{}\", {}", style.font_family, fallback)
        };
        format!("{}{}1000px {}", font_style, font_weight, font_family)
    }

    /// 한컴 webhwp 방식 문자 폭 측정 (HWP 단위 양자화)
    ///
    /// 파이프라인: 내장 메트릭 → JS 1000px 측정 → font_size/1000 스케일링 → HWP 단위(×75) → 정수 반올림 → px
    pub(super) fn measure_char_width_hwp(
        measure_font: &str,
        font_family: &str,
        bold: bool,
        italic: bool,
        c: char,
        hangul_width_hwp: i32,
        font_size: f64,
    ) -> f64 {
        // 1차: 내장 메트릭 (JS 브릿지 호출 불필요)
        if let Some(w) = super::measure_char_width_embedded(font_family, bold, italic, c, font_size)
        {
            return w;
        }

        // 2차: 한글 음절 → '가' 대리 측정값 재사용 (이미 HWP 단위)
        if c >= '\u{AC00}' && c <= '\u{D7A3}' {
            return hangul_width_hwp as f64 / 75.0;
        }

        // 3차: JS 폴백 (미등록 폰트)
        let raw_px = cached_js_measure(measure_font, c);
        let actual_px = raw_px * font_size / 1000.0;
        let hwp = (actual_px * 75.0).round() as i32;
        hwp as f64 / 75.0
    }

    /// 한글 '가' 대리 측정값 (HWP 단위, 정수)
    /// 내장 메트릭이 있으면 JS 호출 없이 반환.
    pub(super) fn measure_hangul_width_hwp(
        measure_font: &str,
        font_family: &str,
        bold: bool,
        italic: bool,
        font_size: f64,
    ) -> i32 {
        if let Some(w) =
            super::measure_char_width_embedded(font_family, bold, italic, '\u{AC00}', font_size)
        {
            return (w * 75.0).round() as i32;
        }
        let raw_px = cached_js_measure(measure_font, '\u{AC00}');
        let actual_px = raw_px * font_size / 1000.0;
        (actual_px * 75.0).round() as i32
    }
}

// ── WasmTextMeasurer ────────────────────────────────────────────────

/// JS Canvas 브릿지 기반 텍스트 측정기 (WASM 전용)
///
/// 1000pt 측정 + HWP 단위 양자화로 한컴과 동일한 정밀도를 확보한다.
/// 내장 메트릭 우선, 미등록 폰트만 JS 브릿지 사용 (LRU 캐시 256 엔트리).
#[cfg(target_arch = "wasm32")]
pub struct WasmTextMeasurer;

#[cfg(target_arch = "wasm32")]
impl TextMeasurer for WasmTextMeasurer {
    fn estimate_text_width(&self, text: &str, style: &TextStyle) -> f64 {
        let (font_size, ratio, tab_w) = style_params(style);
        let measure_font = wasm_internals::build_1000pt_font_string(style);
        let hangul_hwp = wasm_internals::measure_hangul_width_hwp(
            &measure_font,
            &style.font_family,
            style.bold,
            style.italic,
            font_size,
        );

        let chars: Vec<char> = text.chars().collect();
        let cluster_len = build_cluster_len(&chars);
        let char_count = chars.len();
        let has_custom_tabs = !style.tab_stops.is_empty() || style.auto_tab_right;

        let char_width = |i: usize| -> f64 {
            let c = chars[i];
            if c == '\u{2007}' {
                return font_size * 0.5 * ratio + style.letter_spacing + style.extra_char_spacing;
            }
            let char_px = if cluster_len[i] > 1 {
                hangul_hwp as f64 / 75.0
            } else {
                wasm_internals::measure_char_width_hwp(
                    &measure_font,
                    &style.font_family,
                    style.bold,
                    style.italic,
                    c,
                    hangul_hwp,
                    font_size,
                )
            };
            let mut w = char_px * ratio + style.letter_spacing + style.extra_char_spacing;
            if c == ' ' {
                w += style.extra_word_spacing;
            }
            w
        };

        let mut total = 0.0;
        for i in 0..char_count {
            let c = chars[i];
            if cluster_len[i] == 0 {
                continue;
            }
            if c == '\t' {
                if has_custom_tabs {
                    let abs_x = style.line_x_offset + total;
                    let (tab_pos, tab_type, _) = find_next_tab_stop(
                        abs_x,
                        &style.tab_stops,
                        tab_w,
                        style.auto_tab_right,
                        style.available_width,
                    );
                    let rel_tab = tab_pos - style.line_x_offset;
                    match tab_type {
                        1 => {
                            let seg_w =
                                measure_segment_from(&chars, &cluster_len, i + 1, &char_width);
                            total = (rel_tab - seg_w).max(total);
                        }
                        2 => {
                            let seg_w =
                                measure_segment_from(&chars, &cluster_len, i + 1, &char_width);
                            total = (rel_tab - seg_w / 2.0).max(total);
                        }
                        _ => {
                            total = rel_tab.max(total);
                        }
                    }
                } else {
                    // 기본 등간격 탭: 라인 절대 위치(line_x_offset + total) 기준으로 계산
                    let abs_x = style.line_x_offset + total;
                    let next_abs = ((abs_x / tab_w).floor() + 1.0) * tab_w;
                    total = (next_abs - style.line_x_offset).max(total);
                }
                continue;
            }
            total += char_width(i);
        }
        total
    }

    fn compute_char_positions(&self, text: &str, style: &TextStyle) -> Vec<f64> {
        let (font_size, ratio, tab_w) = style_params(style);
        let chars: Vec<char> = text.chars().collect();
        let char_count = chars.len();
        let mut positions = Vec::with_capacity(char_count + 1);
        let mut x = 0.0;
        positions.push(x);

        let cluster_len = build_cluster_len(&chars);
        let has_custom_tabs = !style.tab_stops.is_empty() || style.auto_tab_right;

        let measure_font = wasm_internals::build_1000pt_font_string(style);
        let hangul_hwp = wasm_internals::measure_hangul_width_hwp(
            &measure_font,
            &style.font_family,
            style.bold,
            style.italic,
            font_size,
        );

        let char_width = |i: usize| -> f64 {
            let c = chars[i];
            if c == '\u{2007}' {
                return font_size * 0.5 * ratio + style.letter_spacing + style.extra_char_spacing;
            }
            let char_px = if cluster_len[i] > 1 {
                hangul_hwp as f64 / 75.0
            } else {
                wasm_internals::measure_char_width_hwp(
                    &measure_font,
                    &style.font_family,
                    style.bold,
                    style.italic,
                    c,
                    hangul_hwp,
                    font_size,
                )
            };
            let mut w = char_px * ratio + style.letter_spacing + style.extra_char_spacing;
            if c == ' ' {
                w += style.extra_word_spacing;
            }
            w
        };

        for i in 0..char_count {
            let c = chars[i];
            if cluster_len[i] == 0 {
                positions.push(x);
                continue;
            }
            if c == '\t' {
                if has_custom_tabs {
                    let abs_x = style.line_x_offset + x;
                    let (tab_pos, tab_type, _) = find_next_tab_stop(
                        abs_x,
                        &style.tab_stops,
                        tab_w,
                        style.auto_tab_right,
                        style.available_width,
                    );
                    let rel_tab = tab_pos - style.line_x_offset;
                    match tab_type {
                        1 => {
                            let seg_w =
                                measure_segment_from(&chars, &cluster_len, i + 1, &char_width);
                            x = (rel_tab - seg_w).max(x);
                        }
                        2 => {
                            let seg_w =
                                measure_segment_from(&chars, &cluster_len, i + 1, &char_width);
                            x = (rel_tab - seg_w / 2.0).max(x);
                        }
                        _ => {
                            x = rel_tab.max(x);
                        }
                    }
                } else {
                    // 기본 등간격 탭: 라인 절대 위치(line_x_offset + x) 기준으로 계산
                    let abs_x = style.line_x_offset + x;
                    let next_abs = ((abs_x / tab_w).floor() + 1.0) * tab_w;
                    x = (next_abs - style.line_x_offset).max(x);
                }
                positions.push(x);
                continue;
            }
            x += char_width(i);
            positions.push(x);
        }

        positions
    }
}

// ── 플랫폼별 기본 측정기 선택 ───────────────────────────────────────

#[cfg(target_arch = "wasm32")]
fn default_measurer() -> WasmTextMeasurer {
    WasmTextMeasurer
}

#[cfg(not(target_arch = "wasm32"))]
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
            letter_spacing: cs.letter_spacing_for_lang(lang_index),
            ratio: cs.ratio_for_lang(lang_index),
            default_tab_width: 0.0,
            tab_stops: Vec::new(),
            auto_tab_right: false,
            available_width: 0.0,
            line_x_offset: 0.0,
            tab_leaders: Vec::new(),
            inline_tabs: Vec::new(),
            extra_word_spacing: 0.0,
            extra_char_spacing: 0.0,
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

// ── 내장 폰트 메트릭 측정 ───────────────────────────────────────────

/// 내장 폰트 메트릭으로 문자 폭 측정 (em 단위 → px 변환)
///
/// 내장 메트릭이 있으면 JS 브릿지 호출 없이 즉시 반환.
/// 없으면 None을 반환하여 폴백 경로를 사용하게 한다.
fn measure_char_width_embedded(
    font_family: &str,
    bold: bool,
    italic: bool,
    c: char,
    font_size: f64,
) -> Option<f64> {
    // CSS font-family 체인에서 첫 번째 폰트명으로 메트릭 조회
    let primary_name = font_family.split(',').next().unwrap_or(font_family).trim();
    let mm = font_metrics_data::find_metric(primary_name, bold, italic)?;
    // HWP 반각 처리: space 및 한컴이 반각으로 처리하는 구두점/기호
    let w = if c == ' ' {
        mm.metric.em_size / 2
    } else {
        let glyph_w = mm.metric.get_width(c)?;
        // 한컴은 스마트 따옴표, 가운뎃점 등을 반각으로 처리
        // 폰트 메트릭에서 전각(em_size)으로 기록되어 있어도 em/2로 강제
        let is_halfwidth_punct = matches!(
            c,
            '\u{2018}'
                ..='\u{2027}' | // ''‚‛""„‟†‡•‣․‥…‧ 구두점/기호
            '\u{00B7}' // · MIDDLE DOT
        );
        if is_halfwidth_punct && glyph_w >= mm.metric.em_size {
            mm.metric.em_size / 2
        } else {
            glyph_w
        }
    };
    // em 단위 → px: w / em_size * font_size, 그 후 HWP 양자화
    let em = mm.metric.em_size as f64;
    let mut actual_px = w as f64 * font_size / em;

    // Bold 폴백: Regular 메트릭으로 폴백된 경우
    // 한컴은 faux bold(합성 Bold) 시 렌더링만 획을 두껍게 하고,
    // 텍스트 메트릭(폭 계산)에는 Regular 폭을 그대로 사용한다.
    // bold_fallback 보정을 적용하면 Justify 정렬에서 공백이 축소됨.
    // (26글자 × 1.02px/글자 = 26.5px 과대 → 공백 소멸)

    // 한컴과 동일한 HWPUNIT 정수 변환: w * base_size / em (내림)
    // round가 아닌 truncate (as i32)로 처리하여 한컴 정수 나눗셈과 일치
    let hwp = (actual_px * 75.0) as i32;
    Some(hwp as f64 / 75.0)
}

// ── 호환 래퍼 (기존 호출부 변경 없음) ──────────────────────────────

/// 텍스트 폭 추정
///
/// 플랫폼별 기본 TextMeasurer를 자동 선택하여 위임한다.
/// WASM: WasmTextMeasurer (JS Canvas + HWP 양자화)
/// 네이티브: EmbeddedTextMeasurer (내장 메트릭 + 휴리스틱)
pub(crate) fn estimate_text_width(text: &str, style: &TextStyle) -> f64 {
    default_measurer().estimate_text_width(text, style)
}

/// 텍스트 폭 추정 (round 없이 raw px 반환)
///
/// 줄바꿈 엔진 전용. 단일 문자 토큰의 반올림 누적 오차를 방지한다.
/// 한컴은 HWPUNIT 정수로 폭을 누적하므로, round 없이 px를 합산한 뒤
/// 줄바꿈 비교 시점에서 available_width와 비교하는 것이 더 정확하다.
pub(crate) fn estimate_text_width_unrounded(text: &str, style: &TextStyle) -> f64 {
    let measurer = EmbeddedTextMeasurer;
    let (font_size, ratio, tab_w) = style_params(style);
    let chars: Vec<char> = text.chars().collect();
    let cluster_len = build_cluster_len(&chars);
    let char_count = chars.len();

    let char_width = |i: usize| -> f64 {
        let c = chars[i];
        if c == '\u{2007}' {
            return font_size * 0.5 * ratio + style.letter_spacing + style.extra_char_spacing;
        }
        let base_w = if let Some(w) =
            measure_char_width_embedded(&style.font_family, style.bold, style.italic, c, font_size)
        {
            w
        } else if cluster_len[i] > 1 || is_cjk_char(c) || is_fullwidth_symbol(c) {
            font_size
        } else {
            font_size * 0.5
        };
        let mut w = base_w * ratio + style.letter_spacing + style.extra_char_spacing;
        if c == ' ' {
            w += style.extra_word_spacing;
        }
        w
    };

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

/// 글자별 X 위치 경계값 계산
///
/// N글자 → N+1개 경계값을 반환한다 (0번째는 0.0, N번째는 전체 폭).
/// run 내부 상대 좌표이며, 절대 좌표는 run.bbox.x + charX[i]로 계산한다.
pub(crate) fn compute_char_positions(text: &str, style: &TextStyle) -> Vec<f64> {
    default_measurer().compute_char_positions(text, style)
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

/// 한컴이 전각으로 처리하는 기호 (메트릭 폴백 시 font_size 사용)
fn is_fullwidth_symbol(c: char) -> bool {
    matches!(
        c,
        '\u{20A9}' |                   // ₩ WON SIGN
        '\u{20AC}' |                   // € EURO SIGN
        '\u{00A3}' |                   // £ POUND SIGN
        '\u{00A5}' // ¥ YEN SIGN
    )
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
}
