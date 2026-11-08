//! 번호·글머리표가 공통으로 점유하는 줄 앞 영역과 마커 문자열 측정.

use super::super::text_measurement::{
    char_style_to_text_style, estimate_text_width_exact, resolved_to_text_style,
};
use super::{hwpunit_to_px, MarkerHead, ResolvedStyleSet, TextStyle};

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct ListMarkerGeometry {
    /// 마커 글자모양의 기본 크기. 상대 크기를 적용한 glyph 표시 크기와 구별한다.
    pub line_font_size_px: f64,
    /// 첫 줄 시작에 더하는 양.
    pub first_line_px: f64,
    /// 이어지는 줄 시작에 더하는 양.
    pub rest_line_px: f64,
    /// 마커 글자의 x — 첫 줄 본문 시작 기준(보통 음수).
    pub marker_dx_px: f64,
    /// 영역과 표시 문자열이 모두 있어 마커를 그린다.
    pub marker_visible: bool,
}

impl ListMarkerGeometry {
    pub(crate) fn line_px(&self, visual_line_idx: usize) -> f64 {
        if visual_line_idx == 0 {
            self.first_line_px
        } else {
            self.rest_line_px
        }
    }

    /// 줄 나눔용 상자: `(가용폭에서 뺄 양, 합성 들여쓰기)`.
    ///
    /// 줄 채움은 첫 줄에서 `max(들여쓰기,0)`, 이어지는 줄에서 `max(−들여쓰기,0)` 을 뺀다.
    /// 마커를 얹은 두 차감(첫 줄 R1, 이어지는 줄 R2)을 같은 규칙으로 나타내려면
    /// `min(R1,R2)` 를 가용폭에서 빼고 `R1 − R2` 를 들여쓰기로 준다.
    pub(crate) fn breaker_box(&self, indent_px: f64) -> (f64, f64) {
        let first = indent_px.max(0.0) + self.first_line_px;
        let rest = (-indent_px).max(0.0) + self.rest_line_px;
        (first.min(rest), first - rest)
    }
}

/// 글자모양을 본문에서 상속해도 언어별 크기·장평·글꼴은 마커 문자열로 선택한다.
/// ASCII 번호를 첫 한글 run의 슬롯으로 재거나 그리지 않는다.
pub(super) fn marker_text_style(
    styles: &ResolvedStyleSet,
    char_shape_id: u32,
    text: &str,
) -> TextStyle {
    resolved_to_text_style(styles, char_shape_id, marker_lang(text))
}

fn marker_lang(text: &str) -> usize {
    text.chars()
        .find(|&ch| !crate::renderer::composer::is_lang_neutral(ch))
        .map(crate::renderer::style_resolver::detect_lang_category)
        // 독립 ASCII 구두점에는 따라갈 본문 언어가 없다. 마커 자체의 Latin 슬롯을 쓴다.
        .unwrap_or_else(|| if text.is_ascii() { 1 } else { 0 })
}

/// 기준 크기 0 인 글자 모양. 한/글 '본문 글자 크기 따름'은 번호 글자 모양을 이렇게 저장하고
/// 번호를 문단 끝 글자 모양의 크기로 그린다(한/글 12.30 Mac).
fn follows_body_size(styles: &ResolvedStyleSet, id: u32) -> bool {
    styles
        .source_char_shapes
        .get(id as usize)
        .is_some_and(|shape| shape.base_size == 0)
}

/// 머리 모양이 참조한 글자 모양 `id` 의 마커 스타일. 기준 크기 0 이면 문단 끝 글자 모양
/// `end_id` 의 기준 크기로 다시 해소하고 글꼴·장평·꾸밈은 참조한 글자 모양을 따른다.
pub(super) fn head_marker_text_style(
    styles: &ResolvedStyleSet,
    id: u32,
    end_id: u32,
    text: &str,
) -> TextStyle {
    let lang = marker_lang(text);
    match (
        styles.char_styles.get(id as usize),
        styles.source_char_shapes.get(id as usize),
        styles.char_styles.get(end_id as usize),
    ) {
        (Some(style), Some(shape), Some(body)) if follows_body_size(styles, id) => {
            char_style_to_text_style(
                styles,
                &style.with_base_size(shape, body.font_size),
                id,
                lang,
            )
        }
        _ => resolved_to_text_style(styles, id, lang),
    }
}

/// 마커 뒤 거리는 별도 속성이므로 문자열 마지막 글자 뒤 자간은 번호 너비에 넣지 않는다.
/// 글자 사이 자간은 보존하고, 줄 측정과 실제 마커 BBox가 같은 소수 폭을 소비한다.
pub(super) fn marker_text_width(text: &str, style: &TextStyle) -> f64 {
    let width = estimate_text_width_exact(text, style);
    let Some(last) = text.chars().next_back() else {
        return width;
    };
    let last = last.to_string();
    let mut terminal = style.clone();
    terminal.letter_spacing = 0.0;
    if let Some(latin) = terminal.ascii_punct_latin.as_deref_mut() {
        latin.letter_spacing = 0.0;
    }
    width - estimate_text_width_exact(&last, style) + estimate_text_width_exact(&last, &terminal)
}

/// 상대 크기는 glyph에 적용한다. '%' 거리의 기준은 참조 글자모양의 기본 크기다.
/// 명시된 머리 글자모양과 본문 상속 경로가 같은 선택 규칙을 사용한다. 본문 글자 크기를 따르는
/// 머리 글자모양은 번호를 그리는 크기인 문단 끝 글자모양 `end_id` 의 크기를 기준으로 한다.
pub(super) fn nominal_marker_size(
    styles: &ResolvedStyleSet,
    head: MarkerHead,
    fallback_id: u32,
    end_id: u32,
    fallback_size: f64,
) -> f64 {
    let id = if head.char_shape_id != 0
        && head.char_shape_id != u32::MAX
        && (head.char_shape_id as usize) < styles.char_styles.len()
    {
        if follows_body_size(styles, head.char_shape_id) {
            end_id
        } else {
            head.char_shape_id
        }
    } else {
        fallback_id
    };
    styles
        .char_styles
        .get(id as usize)
        .map_or(fallback_size, |style| style.font_size)
}

pub(super) fn list_marker_geometry_for(
    text: &str,
    head: MarkerHead,
    style: &TextStyle,
    distance_font_size: f64,
    indent_px: f64,
    bullet: bool,
) -> ListMarkerGeometry {
    let hu = |v: i16| hwpunit_to_px(v as i32, crate::renderer::DEFAULT_DPI);
    // 한/글은 ASCII 가 아닌 글머리표 글자를 글자 크기 1em 칸으로 잰다 — 글꼴에 없는 `❍`
    // (맑은 고딕 10pt, 대체 글꼴 폭 0.97em → 한/글 1000), Wingdings `U+F09F`(글리프 폭
    // 0.46em → 12pt 에서 한/글 영역 792 = 1200 − 1000 + 600, `chemical-labeling` 54쪽).
    // ASCII 글머리표(`-`)는 글자 폭 그대로다(`footnote-01` 14pt 반각 706).
    let glyph_px = if bullet && !text.is_ascii() {
        style.font_size * text.chars().count() as f64
    } else {
        marker_text_width(text, style)
    };
    let distance_px = if head.attr & (1 << 4) != 0 {
        hu(head.text_distance)
    } else {
        distance_font_size * head.text_distance as f64 / 100.0
    };
    // 기본 개요에서 bit2가 꺼졌으면 형식의 예약 칸을 쓴다. 실제 글리프 폭은
    // 정렬과 paint에 계속 사용한다. 이 영역을 줄 나눔과 최종 배치가 함께 소비한다.
    let reference_px = if !bullet && head.attr & (1 << 2) == 0 {
        head.reference_width_em
            .map(|em| f64::from(em) * style.font_size * style.ratio)
            .unwrap_or(glyph_px)
    } else {
        glyph_px
    };
    let area_px = (reference_px + hu(head.width_adjust) + distance_px).max(0.0);
    let (first_line_px, rest_line_px) = if head.attr & (1 << 3) != 0 {
        // 첫 줄 본문 = max(영역, 들여쓰기⁺), 이어지는 줄 = 첫 줄 − 들여쓰기.
        // 문단 들여쓰기가 준 시작 위에 얹는 양은 두 줄이 같다.
        let extra = (area_px - indent_px.max(0.0)).max(0.0);
        (extra, extra)
    } else {
        (area_px, 0.0)
    };
    // 가운데·오른쪽 정렬은 영역에서 본문과의 거리를 뺀 칸 안에서 글자를 옮긴다.
    let cell_px = area_px - distance_px;
    let align_dx = match head.attr & 0x03 {
        1 => ((cell_px - glyph_px) / 2.0).max(0.0),
        2 => (cell_px - glyph_px).max(0.0),
        _ => 0.0,
    };
    ListMarkerGeometry {
        line_font_size_px: distance_font_size,
        first_line_px,
        rest_line_px,
        marker_dx_px: align_dx - area_px,
        marker_visible: area_px > 0.0 && !text.is_empty(),
    }
}
