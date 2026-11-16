//! 저장 빈 호스트 줄이 닫는 겹침 금지 그림 프레임의 세로 점유.

use super::{hwpunit_to_px, signed_hwpunit, ParagraphFloatFlow, ParagraphFloatPlacement};
use crate::model::{
    control::Control,
    paragraph::{ColumnBreakType, LineSeg, Paragraph},
    shape::{HorzAlign, HorzRelTo, TextWrap, VertAlign, VertRelTo},
};

/// 내부 그림의 잉크 크기가 아니라 선언 프레임의 합으로 저장 줄 앞 공간을 인증한다.
/// 본문 그림의 같은 앵커·겹침 금지·저장 닫힘이 모두 일치해야 한다.
pub(crate) fn stored_picture_stack_before_host(
    para: &Paragraph,
    frame_vpos: i32,
    actual_host_flow_y: f64,
    paragraph_left: f64,
    spacing_after: f64,
    dpi: f64,
) -> Option<Vec<(usize, ParagraphFloatPlacement)>> {
    let [host] = para.line_segs.as_slice() else {
        return None;
    };
    if !para.text.is_empty()
        || para.stored_text_partition_is_dirty()
        || para.cell_format_vpos_dirty
        || !matches!(
            para.column_type,
            ColumnBreakType::None | ColumnBreakType::Section
        )
        || host.line_height <= 0
        || host.line_spacing < 0
        || host.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0
        || ![actual_host_flow_y, paragraph_left, spacing_after, dpi]
            .iter()
            .all(|x| x.is_finite())
        || paragraph_left != 0.0
        || spacing_after < 0.0
        || dpi <= 0.0
    {
        return None;
    }
    let mut frames = Vec::new();
    let mut total = 0_i32;
    for (index, control) in para.controls.iter().enumerate() {
        let picture = match control {
            Control::Picture(picture) => picture,
            Control::SectionDef(_) | Control::ColumnDef(_) => continue,
            _ => return None,
        };
        let c = &picture.common;
        if c.treat_as_char
            || !c.flow_with_text
            || c.allow_overlap
            || c.text_wrap != TextWrap::TopAndBottom
            || c.vert_rel_to != VertRelTo::Para
            || c.vert_align != VertAlign::Top
            || !matches!(c.horz_rel_to, HorzRelTo::Column | HorzRelTo::Para)
            || c.horz_align != HorzAlign::Left
            || signed_hwpunit(c.horizontal_offset) != 0
            || signed_hwpunit(c.vertical_offset) != 0
            || c.margin.left != 0
            || c.margin.right != 0
            || c.margin.top != 0
            || c.margin.bottom != 0
            || picture.caption.is_some()
        {
            return None;
        }
        let (width, height) = super::picture_flow_frame_size_hu(picture);
        if width <= 0 || height <= 0 {
            return None;
        }
        frames.push((index, total));
        total = total.checked_add(height)?;
    }
    if frames.len() < 2 {
        return None;
    }
    let source_anchor = host.vertical_pos.checked_sub(total)?;
    // 구역 첫 문단도 제어 앵커를 가진다. 새 구역의 0 원점으로 닫히는
    // 경우만 허용하고 일반 구역 나누기의 위치를 그림 합으로 추정하지 않는다.
    if para.column_type == ColumnBreakType::Section
        && (source_anchor != 0 || actual_host_flow_y != 0.0)
    {
        return None;
    }
    // 첫 저장 줄의 vpos가 그림 뒤를 가리키면 일반 글줄 원점 정규화는
    // 그림 앞 공간까지 제거한다. 원본에서 0으로 닫힌 시작은 본문 0 원점이다.
    let source_top = if source_anchor == 0 && actual_host_flow_y == 0.0 {
        0
    } else {
        source_anchor.checked_sub(frame_vpos)?
    };
    let top = hwpunit_to_px(source_top, dpi);
    if source_top < 0 || (top - actual_host_flow_y).abs() > dpi / 7200.0 {
        return None;
    }
    let host_y = top + hwpunit_to_px(total, dpi);
    let next_flow_y = host_y
        + hwpunit_to_px(host.line_height.checked_add(host.line_spacing)?, dpi)
        + spacing_after;
    Some(
        frames
            .into_iter()
            .map(|(index, offset)| {
                let picture_top = top + hwpunit_to_px(offset, dpi);
                (
                    index,
                    ParagraphFloatPlacement {
                        flow: ParagraphFloatFlow::StoredPictureGroup { next_flow_y },
                        anchor_y: picture_top,
                        stored_host_origin: Some(host_y),
                        stored_successor_line_origin: None,
                        table_left: None,
                        table_top: picture_top,
                        occupied_bottom: host_y,
                    },
                )
            })
            .collect(),
    )
}
