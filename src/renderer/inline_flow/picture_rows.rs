//! 저장 줄이 없는 글자처럼 그림을 공통 물리 줄 결과에 연결한다.

use crate::model::{control::Control, image::Picture, paragraph::Paragraph};
use crate::renderer::style_resolver::ResolvedParaStyle;
use crate::renderer::{
    composer::{inline_picture_ascent_ratio, inline_picture_size_hu},
    hwpunit_to_px,
};

use super::{InlineFlowBox, InlineFlowContent};

pub(crate) fn supports_picture_rows(para: &Paragraph) -> bool {
    para.line_segs.is_empty()
        && para.controls.iter().any(|c| {
            matches!(c,
            Control::Picture(p) if p.common.treat_as_char)
        })
        && para.controls.iter().all(|c| match c {
            // 캡션의 줄 구성과 배치는 기존 캡션 소유자에 남긴다.
            Control::Picture(p) => p.common.treat_as_char && p.caption.is_none(),
            Control::SectionDef(_)
            | Control::ColumnDef(_)
            | Control::PageNumberPos(_)
            | Control::PageNumCtrl(_) => true,
            _ => false,
        })
}

pub(super) fn picture_box(
    picture: &Picture,
    control: usize,
    style: &ResolvedParaStyle,
    dpi: f64,
) -> InlineFlowBox {
    let (width, height) = inline_picture_size_hu(picture);
    let margin = &picture.common.margin;
    let left = hwpunit_to_px(i32::from(margin.left), dpi);
    let right = hwpunit_to_px(i32::from(margin.right), dpi);
    let top = hwpunit_to_px(i32::from(margin.top), dpi);
    let bottom = hwpunit_to_px(i32::from(margin.bottom), dpi);
    let height = hwpunit_to_px(height, dpi) + top + bottom;
    let ascent = inline_picture_ascent_ratio(style.vertical_alignment);
    InlineFlowBox {
        content: InlineFlowContent::Picture {
            control,
            margin_left: left,
            margin_top: top,
            affect_line_spacing: picture.common.affect_line_spacing,
        },
        x: 0.0,
        y: 0.0,
        width: hwpunit_to_px(width, dpi) + left + right,
        height,
        baseline: height * ascent,
    }
}
