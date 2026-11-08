//! 저장 줄에 포함된 개체 높이와 쪽 원점을 재구성 사다리에 연결한다.

use crate::model::control::Control;
use crate::model::paragraph::{LineSeg, Paragraph};

/// 저장 줄이 없는 텍스트 문단을 건너뛰어 앞 저장 프레임의 점유 끝을 잇는다.
/// 개체나 편집 줄처럼 전진량을 설명할 수 없는 항목을 만나면 추측하지 않는다.
pub(super) fn preceding_saved_span(
    spans: &[Option<(i32, i32)>],
    reflowed_text_advances: &[Option<i32>],
    index: usize,
) -> Option<(i32, i32)> {
    let mut intervening = 0_i32;
    for previous in (0..index).rev() {
        if let Some((first, end)) = spans[previous] {
            return Some((first, end.saturating_add(intervening)));
        }
        intervening = intervening.saturating_add(reflowed_text_advances[previous]?);
    }
    None
}

/// 한 개의 개체가 유효한 저장 줄 전체를 점유한 경우다.
/// 캡션·편집된 표·수식처럼 점유를 같은 상자로 확인할 수 없으면 제외한다.
pub(super) fn owns_saved_object_line(para: &Paragraph) -> bool {
    crate::renderer::stored_frame_boundary::owns_saved_object_line(para)
}

/// 가시 본문과 작은 TAC 도형이 유효한 저장 글줄의 기준선 상자 안에 함께 들어간다.
/// 합성 줄·캡션·변환 후 커진 개체를 원래 텍스트 줄 높이로 수용하지 않는다.
pub(super) fn inline_graphics_fit_saved_text_line(para: &Paragraph, alignment: u8) -> bool {
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
            Control::Shape(shape)
                if shape.drawing().is_some_and(|drawing| {
                    drawing.caption.is_none() && drawing.text_box.is_none()
                }) =>
            {
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

/// 이전 저장 줄 상자가 본문을 채우고 다음 저장 원점이 위로 돌아간 경계다.
/// 큰 개체는 마지막 줄의 시작이 낮아도 그 높이가 쪽 끝까지 점유할 수 있다.
pub(super) fn saved_positive_frame_reset(
    previous: Option<(i32, i32)>,
    first: &LineSeg,
    body_height: i32,
) -> bool {
    crate::renderer::stored_frame_boundary::saved_positive_frame_reset(previous, first, body_height)
}
