//! 저장 줄이 없는 글자처럼 그림을 공통 물리 줄 결과에 연결한다.

use crate::model::{control::Control, image::Picture, paragraph::Paragraph};
use crate::renderer::style_resolver::{ResolvedParaStyle, ResolvedStyleSet};
use crate::renderer::{
    composer::{inline_picture_ascent_ratio, inline_picture_size_hu},
    hwpunit_to_px,
};

use super::{InlineFlowBox, InlineFlowContent};

/// 글자가 없어도 문단 끝의 글자모양은 그림 줄의 기준선과 캐럿을 소유한다.
/// 같은 폭0 상자를 측정·paint에 전달해 그림 높이를 글자 크기로 오인하지 않는다.
pub(super) fn empty_host_text_box(
    para: &Paragraph,
    styles: &ResolvedStyleSet,
    style: &ResolvedParaStyle,
    dpi: f64,
) -> Option<InlineFlowBox> {
    if !para.text.is_empty() {
        return None;
    }
    let shape = crate::renderer::composer::find_active_char_shape(&para.char_shapes, u32::MAX);
    let size = styles.char_styles.get(shape as usize)?.font_size;
    let metrics = crate::renderer::composer::frame_metrics_for_line(
        size,
        size,
        style.line_spacing_type,
        style.line_spacing,
        dpi,
    );
    let height = hwpunit_to_px(metrics.line_height, dpi);
    Some(InlineFlowBox {
        content: InlineFlowContent::Text {
            range: 0..0,
            style: shape,
            lang: 0,
            alignment_baseline: (style.vertical_alignment != 0)
                .then(|| height * inline_picture_ascent_ratio(style.vertical_alignment)),
        },
        x: 0.0,
        y: 0.0,
        width: 0.0,
        height,
        baseline: hwpunit_to_px(metrics.baseline_distance, dpi),
    })
}

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
