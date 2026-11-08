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

/// 앞 내용 또는 다음 저장 글줄이 닫는 실제 셀 그림은 같은 흐름 원점을 쓴다.
/// 그림이 밀어낸 빈 앵커, 재조판 줄, 저장 리셋과 캡션은 이 계약의 근거가 아니다.
pub(crate) fn stored_cell_flow_picture_frame(
    paragraphs: &[Paragraph],
    index: usize,
    picture: &Picture,
    stored_layout: bool,
) -> Option<StoredCellFlowPictureFrame> {
    let para = paragraphs.get(index)?;
    let common = &picture.common;
    if !stored_layout
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
    {
        return None;
    }
    // 첫 빈 호스트의 다음 본문이 그림의 물리 바닥에서 시작하면 그림과 본문이
    // 하나의 저장 흐름을 소유한다. 그림만 셀 중앙에 다시 정렬하면 뒤 글줄 높이의
    // 절반만큼 그림이 내려간다. 음수 offset은 이 비겹침 흐름의 앞 공간이 아니다.
    let height_hu = i32::try_from(common.height).ok()?;
    let first_closed_frame = index == 0
        && para.text.trim().is_empty()
        && para.controls.len() == 1
        && para.line_segs[0].vertical_pos == 0
        && !common.allow_overlap
        && common.vert_align == crate::model::shape::VertAlign::Top
        && common.margin.top == 0
        && common.margin.bottom == 0
        && paragraphs.get(1).is_some_and(|next| {
            !next.text.trim().is_empty()
                && next.controls.is_empty()
                && super::topbottom_flow_vertical_offset_hu(common).checked_add(height_hu)
                    == Some(next.line_segs[0].vertical_pos)
        });
    if !first_closed_frame
        && !paragraphs[..index].iter().any(|p| {
            !p.text.trim().is_empty()
                || p.controls
                    .iter()
                    .any(|c| matches!(c, Control::Table(_) | Control::Picture(_)))
        })
    {
        return None;
    }
    let offset = if first_closed_frame {
        super::topbottom_flow_vertical_offset_hu(common)
    } else {
        super::signed_hwpunit(common.vertical_offset)
    };
    let top_hu = para.line_segs[0].vertical_pos.checked_add(offset)?;
    let bottom_hu = top_hu.checked_add(height_hu)?;
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
