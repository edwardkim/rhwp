//! 첫 저장 글줄의 앞 간격을 측정과 실제 배치의 같은 쪽 원점으로 해석한다.

use crate::model::{control::Control, paragraph::Paragraph};
use crate::renderer::{hwpunit_to_px, style_resolver::ResolvedStyleSet};

pub(crate) mod initial_column_band;

/// 문단 기준 위·아래 배치 그림은 본문 밴드를 따로 점유할 수 있다.
/// 글앞/글뒤 개체는 그려져도 본문 글줄의 앞 간격을 대체하지 않는다.
fn has_paragraph_float_graphic(paragraph: &Paragraph) -> bool {
    use crate::model::shape::{TextWrap, VertRelTo};
    paragraph.controls.iter().any(|control| {
        let common = match control {
            Control::Shape(shape) => shape.common(),
            Control::Picture(picture) => &picture.common,
            _ => return false,
        };
        !common.treat_as_char
            && common.text_wrap == TextWrap::TopAndBottom
            && common.vert_rel_to == VertRelTo::Para
    })
}

/// 문단 기준 자리차지 표가 소유하는 완전한 바깥 프레임의 끝.
/// 저장 쪽 원점과 다음 문단의 전진을 확인하는 두 경로가 같은 프레임을 사용한다.
fn paragraph_float_frame_end_hu(paragraph: &Paragraph) -> Option<i64> {
    use crate::model::shape::{TextWrap, VertAlign, VertRelTo};
    let [Control::Table(table)] = paragraph.controls.as_slice() else {
        return None;
    };
    if table.common.treat_as_char
        || !table.common.flow_with_text
        || table.common.allow_overlap
        || table.caption.is_some()
        || table.common.text_wrap != TextWrap::TopAndBottom
        || table.common.vert_rel_to != VertRelTo::Para
        || table.common.vert_align != VertAlign::Top
        || table.common.height == 0
        || table.common.height >= 0x8000_0000
        || (table.common.vertical_offset as i32) < 0
        || paragraph.stored_text_partition_is_dirty()
        || paragraph.cell_format_vpos_dirty
    {
        return None;
    }
    Some(
        i64::from(table.common.vertical_offset)
            + i64::from(table.common.height)
            + i64::from(table.outer_margin_top)
            + i64::from(table.outer_margin_bottom),
    )
}

/// 제어 전용 문단에서 실제 인라인 개체가 겹친 마지막 저장 줄을 소유하고,
/// 그 줄이 개체의 완전한 바깥 상자를 보존하는지 확인한다.
fn control_only_overlapping_shape_frame(first: &Paragraph) -> bool {
    use crate::model::paragraph::LineSeg;
    if !first.text.is_empty()
        || !first.char_offsets.is_empty()
        || first.stored_text_partition_is_dirty()
        || first.cell_format_vpos_dirty
        || first.line_segs.len() < 2
        || !first.line_segs.windows(2).enumerate().all(|(index, _)| {
            super::stored_line_ladder::stored_overlap_advance_hu(
                first,
                index,
                first.line_segs.len(),
                true,
            )
            .is_some()
        })
    {
        return false;
    }
    let mut shapes = first
        .controls
        .iter()
        .enumerate()
        .filter_map(|(index, control)| {
            if let Control::Shape(shape) = control {
                shape.common().treat_as_char.then_some((index, shape))
            } else {
                None
            }
        });
    let Some((index, shape)) = shapes.next() else {
        return false;
    };
    if shapes.next().is_some()
        || first.controls.iter().enumerate().any(|(other, control)| {
            other != index
                && (control.is_logical_inline()
                    || matches!(
                        control,
                        Control::Shape(_)
                            | Control::Picture(_)
                            | Control::Table(_)
                            | Control::Form(_)
                            | Control::Equation(_)
                            | Control::AutoNumber(_)
                            | Control::Ruby(_)
                            | Control::Unknown(_)
                    ))
        })
        || first.empty_control_stream_position(index).is_none()
        || super::composer::control_slot_line_index(first, index) != Some(first.line_segs.len() - 1)
        || super::composer::shape_caption_for_layout(shape).is_some()
    {
        return false;
    }
    let last = first.line_segs.last().expect("두 저장 줄을 확인함");
    let common = shape.common();
    let frame = i64::from(shape.flow_height_hu())
        + i64::from(common.margin.top)
        + i64::from(common.margin.bottom);
    last.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0
        && frame > 0
        && i64::from(last.line_height) == frame
}

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
        if paragraph_float_frame_end_hu(first) == Some(i64::from(next_seg.vertical_pos))
            && i64::from(last_seg.vertical_pos) + i64::from(last_seg.line_height)
                <= i64::from(table.common.vertical_offset) + i64::from(table.outer_margin_top))
        && !first.stored_text_partition_is_dirty()
        && !next.stored_text_partition_is_dirty()
        && !first.cell_format_vpos_dirty
        && !next.cell_format_vpos_dirty
        && next.controls.is_empty();
    // 손상 입력의 저장 줄 값은 i32 뺄셈을 넘칠 수 있어 위 등식처럼 i64 로 잰다.
    let stored_gap_hu = i64::from(next_seg.vertical_pos)
        - i64::from(last_seg.vertical_pos)
        - i64::from(last_seg.line_height)
        - i64::from(last_seg.line_spacing);
    let stored_gap = stored_gap_hu as f64 * dpi / crate::renderer::HWPUNIT_PER_INCH;
    // 명시적 나눔만으로 저장 축을 정하지 않는다. 첫 줄의 앞 간격과
    // 두 문단 사이의 아래·위 간격이 함께 닫혀야 새 쪽의 절대 원점이다.
    // 아래 간격이 0이어도 완전한 저장 단계다. 명시적 쪽 나눔의 첫
    // 앞 간격을 원점으로 빼면 제목만 맞고 이후 본문이 그만큼 위로 당겨진다.
    // 다음 문단의 앞 간격이 0이어도 같은 저장 경계 등식이 유효하다.
    // 양수 간격을 필수로 요구하면 제목이 보존한 첫 여백만 뒤 본문에서 빠진다.
    let complete_margin_step = (stored_gap - first_after - next_before).abs() <= 0.5;
    let next_margin_step = first.column_type != crate::model::paragraph::ColumnBreakType::Page
        && (stored_gap - next_before).abs() <= 0.5;
    // 글자처럼 취급되는 도형이 있는 첫 문단은 제목과 개체가 저장 원점을 함께
    // 소유한다. 문단 앞 여백만으로 쪽 원점을 다시 정하면 후속 표가 그 여백만큼
    // 위로 이동한다. 일반 텍스트 문단의 여백 사다리와 구분한다(#6797 7쪽).
    let overlapping_inline_frame = control_only_overlapping_shape_frame(first)
        && !next.stored_text_partition_is_dirty()
        && !next.cell_format_vpos_dirty;
    let first_vpos_px = hwpunit_to_px(first_seg.vertical_pos, dpi);
    let first_margin_matches = (first_vpos_px - first_before).abs() <= 0.5
        || (overlapping_inline_frame && first_vpos_px > 0.0 && first_vpos_px <= first_before);
    (overlapping_inline_frame
        || !first.controls.iter().any(
            |control| matches!(control, Control::Shape(shape) if shape.common().treat_as_char),
        ))
        && !has_paragraph_float_graphic(first)
        && !has_paragraph_float_graphic(next)
        && first_seg.tag & crate::model::paragraph::LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0
        && last_seg.tag & crate::model::paragraph::LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0
        && next_seg.tag & crate::model::paragraph::LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0
        && next_before >= 0.0
        && first_before > 0.5
        && first_margin_matches
        && (float_closes_next || complete_margin_step || next_margin_step)
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
        || first.vertical_pos != 0
        || first.line_height <= 0
        || first.line_spacing < 0
        || [previous, current, next]
            .iter()
            .any(|p| p.stored_text_partition_is_dirty() || p.cell_format_vpos_dirty)
        || [object_line, first, successor].iter().any(|seg| {
            seg.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0
                || seg.tag & LineSeg::TAG_FIRST_SEGMENT == 0
        })
        || object_line.segment_width <= 0
        || successor.segment_width <= 0
    {
        return false;
    }
    let line_advance = i64::from(first.line_height) + i64::from(first.line_spacing);
    let current_advance = match current.controls.as_slice() {
        [] if !current.text.trim().is_empty() && first.segment_width > 0 => line_advance,
        [Control::Table(table)]
            if current
                .text
                .chars()
                .all(|ch| ch.is_whitespace() || ch.is_control() || ch == '\u{fffc}') =>
        {
            if table.common.treat_as_char {
                let frame = i64::from(table.common.height)
                    + i64::from(table.outer_margin_top)
                    + i64::from(table.outer_margin_bottom);
                if first.segment_width <= 0 || i64::from(first.line_height) != frame {
                    return false;
                }
                line_advance
            } else if let Some(frame) =
                paragraph_float_frame_end_hu(current).filter(|_| table.common.width > 0)
            {
                // 폭 없는 앵커 줄도 표의 완전한 프레임을 점유한다. 작은
                // placeholder 줄 높이만으로 뒤 저장 사다리를 판정하지 않는다.
                line_advance.max(frame)
            } else {
                return false;
            }
        }
        _ => return false,
    };
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
    let expected = current_advance + (spacing * 7200.0 / dpi).round() as i64;
    // 새로운 제목과 그 다음 줄은 같은 원본 사다리를 닫는다. 문단마다
    // 0을 쓰는 로컬 좌표와 편집 후 재조판에는 이 독립 증거가 없다.
    i64::from(successor.vertical_pos) == expected
}
