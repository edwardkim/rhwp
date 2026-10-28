//! 저장 셀 흐름의 그림 원점과 물리 점유 끝을 측정·정렬·배치에 공유한다.

use crate::model::{
    control::Control,
    image::Picture,
    paragraph::{LineSeg, Paragraph},
    shape::{TextWrap, VertRelTo},
};

pub(crate) struct StoredCellFlowPictureFrame {
    pub top_hu: i32,
    pub bottom_hu: i32,
}

/// 앞 내용이 있는 실제 저장 셀에서 문단 기준 자리차지 그림은 같은 줄 원점을 쓴다.
/// 첫 그림이 밀어낸 빈 앵커, 재조판 줄, 저장 리셋과 캡션은 이 계약의 근거가 아니다.
pub(crate) fn stored_cell_flow_picture_frame(
    paragraphs: &[Paragraph],
    index: usize,
    picture: &Picture,
    stored_layout: bool,
) -> Option<StoredCellFlowPictureFrame> {
    let para = paragraphs.get(index)?;
    let common = &picture.common;
    if !stored_layout
        || index == 0
        || picture.caption.is_some()
        || common.treat_as_char
        || !common.flow_with_text
        || !matches!(common.text_wrap, TextWrap::TopAndBottom)
        || !matches!(common.vert_rel_to, VertRelTo::Para)
        || para.line_segs.len() != 1
        || !crate::renderer::cell_vpos_ladder_is_intact(paragraphs)
        || paragraphs.iter().any(|p| {
            p.stored_text_partition_is_dirty()
                || p.cell_format_vpos_dirty
                || p.line_segs.is_empty()
                || p.line_segs.iter().any(|s| {
                    s.vertical_pos < 0 || s.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0
                })
        })
        || paragraphs.windows(2).any(|pair| {
            pair[1].line_segs[0].vertical_pos
                < pair[0].line_segs.last().expect("저장 줄 확인").vertical_pos
        })
        || !paragraphs[..index].iter().any(|p| {
            !p.text.trim().is_empty()
                || p.controls
                    .iter()
                    .any(|c| matches!(c, Control::Table(_) | Control::Picture(_)))
        })
    {
        return None;
    }
    let top_hu = para.line_segs[0]
        .vertical_pos
        .checked_add(super::signed_hwpunit(common.vertical_offset))?;
    let bottom_hu = top_hu.checked_add(i32::try_from(common.height).ok()?)?;
    if top_hu < 0 || bottom_hu <= top_hu {
        return None;
    }
    Some(StoredCellFlowPictureFrame { top_hu, bottom_hu })
}

pub(crate) fn stored_cell_flow_picture_extent_hu(
    paragraphs: &[Paragraph],
    stored_layout: bool,
) -> Option<i32> {
    paragraphs
        .iter()
        .enumerate()
        .flat_map(|(index, para)| {
            para.controls.iter().filter_map(move |control| {
                let Control::Picture(picture) = control else {
                    return None;
                };
                stored_cell_flow_picture_frame(paragraphs, index, picture, stored_layout)
                    .map(|frame| frame.bottom_hu)
            })
        })
        .max()
}
