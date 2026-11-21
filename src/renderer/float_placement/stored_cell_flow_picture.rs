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
    /// 다음 문단의 첫 저장 줄이 이 그림 바닥에서 시작한다(뒤 문단이 띠를 닫는다).
    pub closed_by_next_line: bool,
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
    // 호스트 줄 원점 + 양수 오프셋 + 그림 높이 = 다음 문단의 첫 저장 줄이면 그 그림 띠는
    // 뒤 문단이 닫는다. 호스트에 글이 있든 뒤 문단이 글자처럼 취급 그림 줄이든 같은 흐름이다.
    let signed_offset = super::signed_hwpunit(common.vertical_offset);
    let closed_by_next_line = signed_offset >= 0
        && !common.allow_overlap
        && common.vert_align == crate::model::shape::VertAlign::Top
        && common.margin.top == 0
        && common.margin.bottom == 0
        && para
            .controls
            .iter()
            .filter(|c| matches!(c, Control::Picture(p) if !p.common.treat_as_char))
            .count()
            == 1
        && paragraphs.get(index + 1).is_some_and(|next| {
            para.line_segs[0]
                .vertical_pos
                .checked_add(signed_offset)
                .and_then(|top| top.checked_add(height_hu))
                == Some(next.line_segs[0].vertical_pos)
        });
    if !first_closed_frame
        && !closed_by_next_line
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
    Some(StoredCellFlowPictureFrame {
        top_hu,
        bottom_hu,
        closed_by_next_line,
    })
}

pub(crate) fn stored_cell_flow_picture_extent_hu(
    paragraphs: &[Paragraph],
    stored_layout: bool,
) -> Option<i32> {
    let mut closed_by_next_line = false;
    let picture_end = paragraphs
        .iter()
        .enumerate()
        .flat_map(|(index, para)| {
            para.controls.iter().filter_map(move |control| {
                let Control::Picture(picture) = control else {
                    return None;
                };
                stored_cell_flow_picture_frame(paragraphs, index, picture, stored_layout)
            })
        })
        .map(|frame| {
            closed_by_next_line |= frame.closed_by_next_line;
            frame.bottom_hu
        })
        .max()?;
    // 뒤 문단이 그림 띠를 닫은 칸은 줄 전진의 합이 띠를 세지 못한다. 같은 저장 사다리의
    // 마지막 줄 끝이 칸 내용의 끝이다(프레임 인증이 사다리의 온전함과 단조 증가를 확인했다).
    let content_end = closed_by_next_line
        .then(|| super::stored_cell_content_extent_hu(paragraphs))
        .flatten()
        .unwrap_or(picture_end);
    Some(picture_end.max(content_end))
}
