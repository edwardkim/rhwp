//! 글자처럼 그림이 줄 구성에 제공하는 HWPUNIT 상자 크기.
//!
//! 용지·쪽 상대 비율로 확대된 인쇄 잉크와 줄 상자의 기준선 정렬 크기를 구분한다.
//! 저장 줄과 재조판이 소비하는 동일한 크기를 실제 그림 배치에도 전달한다.

use crate::model::image::Picture;

pub(crate) fn inline_picture_size_hu(picture: &Picture) -> (i32, i32) {
    crate::renderer::stored_picture_metrics::stored_picture_size_hu(picture)
}

/// 글줄이 예약하는 그림 상자는 잉크와 바깥 여백을 함께 포함한다.
/// 독립 한컴 재저장본도 위·아래 여백을 포함한 높이로 LineSeg를 발행한다.
/// 실제 잉크 배치는 기존 원점에서 같은 여백 안쪽을 소비한다.
pub(crate) fn inline_picture_occupied_size_hu(picture: &Picture) -> (i32, i32) {
    let (width, height) = inline_picture_size_hu(picture);
    let margin = &picture.common.margin;
    (
        width
            .saturating_add(i32::from(margin.left))
            .saturating_add(i32::from(margin.right)),
        height
            .saturating_add(i32::from(margin.top))
            .saturating_add(i32::from(margin.bottom)),
    )
}

/// HWP 5.0 표 44의 세로 정렬점. 줄 구성과 실제 그림 원점이 같은 값을 사용한다.
pub(crate) fn inline_picture_ascent_ratio(vertical_alignment: u8) -> f64 {
    match vertical_alignment {
        1 => 0.0,
        2 => 0.5,
        3 => 1.0,
        _ => 0.85,
    }
}

/// 저장 줄이 없는 혼합 주석 문단도 기존 줄 소속·탭 배치를 유지하며 그림 상자를 공유한다.
pub(super) fn refresh_picture_rows(
    composed: &mut super::ComposedParagraph,
    para: &crate::model::paragraph::Paragraph,
    styles: &crate::renderer::style_resolver::ResolvedStyleSet,
) {
    use crate::model::control::Control;
    use crate::renderer::{hwpunit_to_px, px_to_hwpunit, DEFAULT_DPI};
    if !para.line_segs.is_empty() {
        return;
    }
    let Some(style) = styles.para_styles.get(para.para_shape_id as usize) else {
        return;
    };
    // shortcut: 비기본 세로 정렬은 저장 줄과 주석 기준선의 별도 계약을 검증한 뒤 연결한다.
    if style.vertical_alignment != 0
        || !para.controls.iter().all(|control| match control {
            Control::Picture(picture) => picture.common.treat_as_char && picture.caption.is_none(),
            Control::Footnote(_) | Control::Endnote(_) => true,
            other => super::control_is_width_neutral_marker(other),
        })
    {
        return;
    }
    let dpi = styles.resolved_dpi.unwrap_or(DEFAULT_DPI);
    let count = composed.lines.len();
    for index in 0..count {
        let end = composed
            .lines
            .get(index + 1)
            .map_or(para.text.chars().count(), |line| line.char_start);
        let line = &mut composed.lines[index];
        let pictures: Vec<_> = composed
            .tac_controls
            .iter()
            .filter(|(position, _, _)| {
                *position >= line.char_start
                    && (*position < end || (*position == end && index + 1 == count))
            })
            .filter_map(|(_, _, control)| match para.controls.get(*control)? {
                Control::Picture(picture) => Some(picture),
                _ => None,
            })
            .collect();
        if pictures.is_empty() {
            continue;
        }
        let font = crate::renderer::composed_line_max_font_size(line, para, styles);
        if !font.is_finite() || font <= 0.0 {
            continue;
        }
        let text_height = px_to_hwpunit(font, dpi);
        let mut ascent = (f64::from(text_height) * 0.85).round() as i32;
        let mut descent = text_height - ascent;
        let mut spacing_height = text_height;
        for picture in pictures {
            let margin = &picture.common.margin;
            let height = inline_picture_size_hu(picture)
                .1
                .saturating_add(i32::from(margin.top))
                .saturating_add(i32::from(margin.bottom));
            let baseline = (f64::from(height)
                * inline_picture_ascent_ratio(style.vertical_alignment))
            .round() as i32;
            ascent = ascent.max(baseline);
            descent = descent.max(height - baseline);
            if picture.common.affect_line_spacing {
                spacing_height = spacing_height.max(height);
            }
        }
        if matches!(
            style.line_spacing_type,
            crate::model::style::LineSpacingType::Fixed
                | crate::model::style::LineSpacingType::Minimum
        ) {
            spacing_height = ascent + descent;
        }
        let metrics = super::frame_metrics_for_line(
            hwpunit_to_px(spacing_height, dpi),
            font,
            style.line_spacing_type,
            style.line_spacing,
            dpi,
        );
        line.line_height = ascent + descent;
        line.baseline_distance = ascent;
        line.line_spacing = metrics.line_spacing;
    }
}
