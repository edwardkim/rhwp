//! 저장 줄이 이미 소유한 TAC 그림도 원래 물리 셀 프레임을 인증한다.

use crate::model::control::Control;
use crate::model::paragraph::{LineSeg, Paragraph};
use crate::model::shape::ShapeObject;

/// HWP5 표 62의 줄 상자와 표 69의 개체 크기·바깥여백을 같은 HU로 대조한다.
/// 별도 캡션이나 흐름 밖 개체는 이 원자 글줄의 높이만으로 인증하지 않는다.
pub(super) fn source_line_controls_fit(paragraph: &Paragraph) -> bool {
    paragraph
        .controls
        .iter()
        .enumerate()
        .all(|(index, control)| {
            let picture = match control {
                Control::Footnote(_) | Control::Endnote(_) => return true,
                Control::Picture(picture) => picture.as_ref(),
                Control::Shape(shape) => match shape.as_ref() {
                    ShapeObject::Picture(picture) => picture,
                    _ => return false,
                },
                _ => return false,
            };
            let common = &picture.common;
            if !common.treat_as_char || common.height == 0 || picture.caption.is_some() {
                return false;
            }
            let Some(owner) = super::super::control_line_seg_index(paragraph, index) else {
                return false;
            };
            let Some(line) = paragraph.line_segs.get(owner) else {
                return false;
            };
            // 줄 구성과 실제 배치가 사용하는 동일한 TAC 크기를 소비한다.
            // 원본 common 크기만 재계산해 확대·저장 프레임 계약을 잃지 않는다.
            let Some(height) = crate::renderer::tac_object_flow_height_hu(control) else {
                return false;
            };
            let occupied = i64::from(height)
                + i64::from(common.margin.top)
                + i64::from(common.margin.bottom);
            line.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0
                && height > 0
                && line.line_height > 0
                && occupied <= i64::from(line.line_height)
        })
}
