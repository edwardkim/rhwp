//! 그림의 여백·캡션 상자와 호스트 줄이 공유하는 물리 점유.

use super::{hwpunit_to_px, picture_flow_frame_size_hu, Control, Paragraph, Picture};
use crate::model::{paragraph::LineSeg, shape::CaptionDirection};

#[derive(Clone, Copy)]
pub(crate) struct PictureOuterBox {
    pub width: f64,
    pub height: f64,
    pub margin_left: f64,
    pub margin_top: f64,
    pub caption_left: f64,
    pub caption_top: f64,
}

/// #6596의 상자 계약: 바깥 여백을 포함해 정렬하고 잉크는 그 안쪽에 놓는다.
/// 실제 그림 크기는 호출자가 확정한 회전·표시 프레임을 그대로 받는다.
pub(crate) fn picture_outer_box(
    picture: &Picture,
    width: f64,
    height: f64,
    caption_height: f64,
    dpi: f64,
) -> PictureOuterBox {
    let (mut caption_width, mut caption_extra, mut caption_left, mut caption_top) =
        (0.0, 0.0, 0.0, 0.0);
    if let Some(caption) = &picture.caption {
        let spacing = hwpunit_to_px(caption.spacing as i32, dpi);
        match caption.direction {
            CaptionDirection::Top | CaptionDirection::Bottom => {
                caption_extra = caption_height + spacing;
                if caption.direction == CaptionDirection::Top {
                    caption_top = caption_extra;
                }
            }
            CaptionDirection::Left | CaptionDirection::Right => {
                caption_width = hwpunit_to_px(caption.width as i32, dpi) + spacing;
                if caption.direction == CaptionDirection::Left {
                    caption_left = caption_width;
                }
            }
        }
    }
    let margin = &picture.common.margin;
    let (left, right, top, bottom) = if picture.common.treat_as_char {
        (0.0, 0.0, 0.0, 0.0)
    } else {
        (
            hwpunit_to_px(i32::from(margin.left), dpi),
            hwpunit_to_px(i32::from(margin.right), dpi),
            hwpunit_to_px(i32::from(margin.top), dpi),
            hwpunit_to_px(i32::from(margin.bottom), dpi),
        )
    };
    PictureOuterBox {
        width: width + caption_width + left + right,
        height: height + caption_extra + top + bottom,
        margin_left: left,
        margin_top: top,
        caption_left,
        caption_top,
    }
}

pub(crate) struct ParagraphPictureBand {
    pub boxes: Vec<PictureOuterBox>,
    /// 현재 호스트 줄에서 시작하는 점유. 개체의 흐름 앞 공간도 이 문단 소유다.
    pub height: f64,
    /// 앞 조각이 호스트 오프셋을 소비한 띠의 점유.
    pub deferred_height: f64,
}

/// 첫 콘텐츠의 셀 정렬과 앞 본문 뒤의 문단 흐름을 구분한다.
pub(crate) fn picture_host_follows_cell_content(paragraphs: &[Paragraph], index: usize) -> bool {
    paragraphs.iter().take(index).any(|para| {
        !para.text.trim().is_empty()
            || para.controls.iter().any(|control| {
                matches!(
                    control,
                    Control::Table(_)
                        | Control::Picture(_)
                        | Control::Shape(_)
                        | Control::Equation(_)
                )
            })
    })
}

/// 실제 빈 한 줄의 병렬 TopAndBottom 그림은 한 문단 원점을 공유한다.
/// 다른 wrap·TAC·명시적 개행·위/옆 캡션은 각각의 기존 흐름 계약을 유지한다.
pub(crate) fn paragraph_picture_band(para: &Paragraph, dpi: f64) -> Option<ParagraphPictureBand> {
    let [line] = para.line_segs.as_slice() else {
        return None;
    };
    if !para.text.trim().is_empty()
        || para.text.contains(['\r', '\n'])
        || para.stored_text_partition_is_dirty()
        || para.cell_format_vpos_dirty
        || line.line_height <= 0
        || line.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0
        || para.controls.is_empty()
    {
        return None;
    }
    let mut band = ParagraphPictureBand {
        boxes: Vec::with_capacity(para.controls.len()),
        height: 0.0,
        deferred_height: 0.0,
    };
    for control in &para.controls {
        let Control::Picture(picture) = control else {
            return None;
        };
        let common = &picture.common;
        if !super::is_para_topbottom_float(common)
            || !common.flow_with_text
            || common.vert_align != super::VertAlign::Top
            || picture
                .caption
                .as_ref()
                .is_some_and(|caption| caption.direction != CaptionDirection::Bottom)
        {
            return None;
        }
        let (width, height) = picture_flow_frame_size_hu(picture);
        if height <= 0 {
            return None;
        }
        let frame = picture_outer_box(
            picture,
            hwpunit_to_px(width, dpi),
            hwpunit_to_px(height, dpi),
            crate::renderer::composer::caption_height_px(&picture.caption, dpi),
            dpi,
        );
        band.height = band.height.max(
            frame.height + hwpunit_to_px(super::topbottom_flow_vertical_offset_hu(common), dpi),
        );
        band.deferred_height = band.deferred_height.max(frame.height);
        band.boxes.push(frame);
    }
    Some(band)
}
