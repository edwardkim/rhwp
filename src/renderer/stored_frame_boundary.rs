//! 원래 저장 글줄의 점유와 다음 쪽 앞 공간으로 물리 프레임 경계를 판정한다.

use crate::model::control::Control;
use crate::model::paragraph::{LineSeg, Paragraph};

pub(crate) fn saved_positive_frame_reset(
    previous: Option<(i32, i32)>,
    first: &LineSeg,
    body_height: i32,
) -> bool {
    previous.is_some_and(|(previous_first, previous_end)| {
        let next_bottom = first.vertical_pos.saturating_add(first.line_height);
        first.vertical_pos > 0
            && first.line_height > 0
            && first.vertical_pos < previous_first
            && next_bottom <= body_height
            // 새 쪽의 앞 공간도 그 줄의 실제 점유다. 이전 축에 이어 넣을 때
            // 줄 높이만 더하면 표 뒤의 앞 간격을 버려 본문 밖까지 그릴 수 있다.
            && previous_end.saturating_add(next_bottom) > body_height
    })
}

pub(crate) fn owns_saved_object_line(para: &Paragraph) -> bool {
    let [line] = para.line_segs.as_slice() else {
        return false;
    };
    let mut owners = para
        .controls
        .iter()
        .filter(|control| !crate::renderer::control_is_body_flow_neutral_marker(control));
    let Some(control) = owners.next() else {
        return false;
    };
    if owners.next().is_some() {
        return false;
    }
    if para.stored_text_partition_dirty
        || line.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0
        || para
            .text
            .chars()
            .any(|c| !c.is_whitespace() && c > '\u{001f}' && c != '\u{fffc}')
    {
        return false;
    }
    let common = match control {
        Control::Shape(shape)
            if shape
                .drawing()
                .is_some_and(|drawing| drawing.caption.is_none()) =>
        {
            shape.common()
        }
        Control::Picture(picture) if picture.caption.is_none() => &picture.common,
        Control::Table(table)
            if table.caption.is_none()
                && table.cells.iter().all(|cell| {
                    !cell.dirty_flag
                        && cell.paragraphs.iter().all(|paragraph| {
                            !paragraph.stored_text_partition_is_dirty()
                                && !paragraph.cell_format_vpos_dirty
                                && !crate::renderer::para_has_no_stored_line_segs(paragraph)
                        })
                }) =>
        {
            &table.common
        }
        _ => return false,
    };
    common.treat_as_char
        && common.height > 0
        && i64::from(line.line_height)
            == i64::from(common.height)
                + i64::from(common.margin.top)
                + i64::from(common.margin.bottom)
}

/// 가시 본문과 작은 TAC 도형이 유효한 저장 글줄의 기준선 상자 안에 함께 들어간다.
/// 합성 줄·캡션·변환 후 커진 개체를 원래 텍스트 줄 높이로 수용하지 않는다.
pub(crate) fn inline_graphics_fit_saved_text_line(para: &Paragraph, alignment: u8) -> bool {
    let [line] = para.line_segs.as_slice() else {
        return false;
    };
    if alignment != 0
        || para.stored_text_partition_dirty
        || para.cell_format_vpos_dirty
        || line.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0
        || line.text_start != 0
        || line.line_height != line.text_height
        || line.baseline_distance <= 0
        || line.baseline_distance > line.line_height
        || !para
            .text
            .chars()
            .any(|c| !c.is_whitespace() && c > '\u{001f}' && c != '\u{fffc}')
    {
        return false;
    }
    let mut has_graphic = false;
    let ascent_ratio = crate::renderer::composer::inline_picture_ascent_ratio(alignment);
    para.controls.iter().all(|control| {
        if crate::renderer::control_is_body_flow_neutral_marker(control) {
            return true;
        }
        let (common, height) = match control {
            Control::Shape(shape) if text_free_graphic(shape) => {
                (shape.common(), shape.flow_height_hu())
            }
            _ => return false,
        };
        has_graphic = true;
        common.treat_as_char
            && common.vertical_offset == 0
            && height > 0
            && f64::from(height) * ascent_ratio + f64::from(common.margin.top)
                <= f64::from(line.baseline_distance)
            && f64::from(height) * (1.0 - ascent_ratio) + f64::from(common.margin.bottom)
                <= f64::from(line.line_height - line.baseline_distance)
    }) && has_graphic
}

/// 글을 담지 않는 그리기 개체다. 묶음은 바깥 상자가 줄 안의 점유를 정하므로 캡션 없이
/// 같은 조건의 하위 개체만 가지면 단일 도형과 같다. 글상자·캡션은 글줄을 만들어 제외한다.
fn text_free_graphic(shape: &crate::model::shape::ShapeObject) -> bool {
    match shape {
        crate::model::shape::ShapeObject::Group(group) => {
            group.caption.is_none()
                && !group.children.is_empty()
                && group.children.iter().all(text_free_graphic)
        }
        _ => shape
            .drawing()
            .is_some_and(|drawing| drawing.caption.is_none() && drawing.text_box.is_none()),
    }
}
