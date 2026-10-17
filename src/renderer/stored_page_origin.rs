//! 첫 저장 글줄의 앞 간격을 측정과 실제 배치의 같은 쪽 원점으로 해석한다.

use crate::model::{control::Control, paragraph::Paragraph};
use crate::renderer::{
    hwpunit_to_px, layout::para_has_overlay_shape, style_resolver::ResolvedStyleSet,
};

/// 쪽 첫 저장 줄의 vpos가 문단 앞 간격을 포함하면 쪽 원점은 0이다.
/// 다음 문단의 저장 사다리도 앞 간격을 계상하는지 확인하여 측정과 배치가
/// 같은 원점을 소비한다. 명시적 쪽 나눔도 원본 앞 간격과 다음 줄이 같은
/// 사다리를 닫으면 이 원점을 소유한다. 합성 줄은 이 증거로 사용하지 않는다.
pub(crate) fn stored_first_margin_is_page_relative(
    first: &Paragraph,
    next: Option<&Paragraph>,
    styles: &ResolvedStyleSet,
    dpi: f64,
) -> bool {
    let Some(first_seg) = first.line_segs.first() else {
        return false;
    };
    let Some(last_seg) = first.line_segs.last() else {
        return false;
    };
    let Some(next) = next else {
        return false;
    };
    let Some(next_seg) = next.line_segs.first() else {
        return false;
    };
    let first_before = styles
        .para_styles
        .get(first.para_shape_id as usize)
        .map(|style| style.spacing_before)
        .unwrap_or(0.0);
    let next_before = styles
        .para_styles
        .get(next.para_shape_id as usize)
        .map(|style| style.spacing_before)
        .unwrap_or(0.0);
    let first_after = styles
        .para_styles
        .get(first.para_shape_id as usize)
        .map(|style| style.spacing_after)
        .unwrap_or(0.0);
    // A paragraph-relative float can reserve the intervening band instead
    // of ordinary text advances. Its complete outer frame independently
    // closes the next saved line, so the first margin is still page-relative.
    let float_closes_next = matches!(first.controls.as_slice(), [Control::Table(table)]
        if !table.common.treat_as_char && table.common.flow_with_text
            && !table.common.allow_overlap
            && table.caption.is_none()
            && table.common.text_wrap == crate::model::shape::TextWrap::TopAndBottom
            && table.common.vert_rel_to == crate::model::shape::VertRelTo::Para
            && table.common.vert_align == crate::model::shape::VertAlign::Top
            && table.common.height > 0 && table.common.height < 0x8000_0000
            && (table.common.vertical_offset as i32) >= 0
            && i64::from(table.common.vertical_offset) + i64::from(table.common.height)
                + i64::from(table.outer_margin_top) + i64::from(table.outer_margin_bottom)
                == i64::from(next_seg.vertical_pos)
            && i64::from(last_seg.vertical_pos) + i64::from(last_seg.line_height)
                <= i64::from(table.common.vertical_offset) + i64::from(table.outer_margin_top))
        && !first.stored_text_partition_is_dirty()
        && !next.stored_text_partition_is_dirty()
        && !first.cell_format_vpos_dirty
        && !next.cell_format_vpos_dirty
        && next.controls.is_empty();
    // 다음 앞 간격이 0이어도 저장 전진 끝과 정확히 이어지면 같은 쪽 상대 사다리다.
    // 빈 첫 문단의 유효한 앞 간격도 줄 점유에 포함되며 쪽 원점으로 빼지 않는다.
    // 글자처럼 취급되는 도형이 있는 첫 문단은 제목과 개체가 저장 원점을 함께
    // 소유한다. 문단 앞 여백만으로 쪽 원점을 다시 정하면 후속 표가 그 여백만큼
    // 위로 이동한다. 일반 텍스트 문단의 여백 사다리와 구분한다(#6797 7쪽).
    !first
        .controls
        .iter()
        .any(|control| matches!(control, Control::Shape(shape) if shape.common().treat_as_char))
        && !para_has_overlay_shape(first)
        && !para_has_overlay_shape(next)
        && first_seg.tag & crate::model::paragraph::LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0
        && last_seg.tag & crate::model::paragraph::LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0
        && next_seg.tag & crate::model::paragraph::LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0
        && first_before > 0.5
        && next_before >= 0.0
        && (hwpunit_to_px(first_seg.vertical_pos, dpi) - first_before).abs() <= 0.5
        && {
            let gap = hwpunit_to_px(
                next_seg.vertical_pos
                    - last_seg.vertical_pos
                    - last_seg.line_height
                    - last_seg.line_spacing,
                dpi,
            );
            // 기존 0 앞간격 사다리와 뒤간격을 따로 저장한 사다리를 같은 원점에서 판정한다.
            (gap - next_before).abs() <= 0.5
                || (gap - first_after - next_before).abs() <= 0.5
                || (next_before > 0.5 && float_closes_next)
        }
}

/// 완전한 인라인 표의 저장 상자 뒤에서 새 본문 사다리가 0으로 재시작하면
/// 다음 문단이 새 쪽을 소유한다. 표 완료 뒤 좌표 축을 닫는 것과 쪽 경계
/// 보존은 별개다. 단순 vpos 되감김이나 표 크기만으로 판정하지 않는다.
pub(crate) fn stored_inline_frame_restarts_page(
    previous: &Paragraph,
    current: &Paragraph,
    next: Option<&Paragraph>,
    styles: &ResolvedStyleSet,
    dpi: f64,
) -> bool {
    use crate::model::paragraph::LineSeg;
    let [Control::Table(table)] = previous.controls.as_slice() else {
        return false;
    };
    let [object_line] = previous.line_segs.as_slice() else {
        return false;
    };
    let [first] = current.line_segs.as_slice() else {
        return false;
    };
    let Some(next) = next else {
        return false;
    };
    let Some(successor) = next.line_segs.first() else {
        return false;
    };
    if !table.common.treat_as_char
        || table.common.height == 0
        || object_line.vertical_pos <= 0
        || object_line.line_height <= 0
        || object_line.line_spacing < 0
        || i64::from(object_line.line_height)
            != i64::from(table.common.height)
                + i64::from(table.outer_margin_top)
                + i64::from(table.outer_margin_bottom)
        || previous
            .text
            .chars()
            .any(|ch| !ch.is_whitespace() && !ch.is_control() && ch != '\u{FFFC}')
        || !current.controls.is_empty()
        || current.text.trim().is_empty()
        || first.vertical_pos != 0
        || first.line_height <= 0
        || first.line_spacing < 0
        || [previous, current, next]
            .iter()
            .any(|p| p.stored_text_partition_is_dirty() || p.cell_format_vpos_dirty)
        || [object_line, first, successor].iter().any(|seg| {
            seg.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0
                || seg.tag & LineSeg::TAG_FIRST_SEGMENT == 0
                || seg.segment_width <= 0
        })
    {
        return false;
    }
    let Some(current_style) = styles.para_styles.get(current.para_shape_id as usize) else {
        return false;
    };
    let Some(next_style) = styles.para_styles.get(next.para_shape_id as usize) else {
        return false;
    };
    let spacing = current_style.spacing_after + next_style.spacing_before;
    if !spacing.is_finite() || spacing < 0.0 || !dpi.is_finite() || dpi <= 0.0 {
        return false;
    }
    let expected = i64::from(first.line_height)
        + i64::from(first.line_spacing)
        + (spacing * 7200.0 / dpi).round() as i64;
    // 새로운 제목과 그 다음 줄은 같은 원본 사다리를 닫는다. 문단마다
    // 0을 쓰는 로컬 좌표와 편집 후 재조판에는 이 독립 증거가 없다.
    i64::from(successor.vertical_pos) == expected
}
