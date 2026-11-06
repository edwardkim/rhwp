//! 글줄 사이에 이미 예약된 문단 기준 자리차지 띠는 저장 줄 오프셋이 소유한다.

use crate::model::{control::Control, paragraph::Paragraph, shape::TextWrap};

/// 본문 사다리를 다시 연결해도 이 문단 안의 물리 간격은 평탄화하지 않는다.
/// 개체가 줄 높이에 포함된 계약과 줄 아래에 별도로 쌓이는 계약은 기존 경로에 맡긴다.
pub(crate) fn owns_embedded_float_bands(paragraph: &Paragraph, dpi: f64) -> bool {
    use crate::model::paragraph::LineSeg;
    use crate::model::shape::VertAlign;

    if !dpi.is_finite()
        || dpi <= 0.0
        || paragraph.stored_text_partition_is_dirty()
        || paragraph.cell_format_vpos_dirty
        || paragraph.line_segs.len() < 2
        || paragraph.line_segs.iter().any(|line| {
            line.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0
                || line.line_height <= 0
                || line.line_height != line.text_height
        })
        || paragraph.line_segs.windows(2).any(|pair| {
            pair[1].vertical_pos < pair[0].vertical_pos || pair[1].text_start < pair[0].text_start
        })
    {
        return false;
    }

    let mut has_band = false;
    for (index, control) in paragraph.controls.iter().enumerate() {
        let (common, caption, margin_top, margin_bottom) = match control {
            Control::Table(table) => (
                &table.common,
                &table.caption,
                table.outer_margin_top,
                table.outer_margin_bottom,
            ),
            Control::Picture(picture) => (
                &picture.common,
                &picture.caption,
                picture.common.margin.top,
                picture.common.margin.bottom,
            ),
            Control::Shape(shape)
                if !shape.common().treat_as_char
                    && shape.common().text_wrap == TextWrap::TopAndBottom =>
            {
                return false;
            }
            _ => continue,
        };
        if common.treat_as_char || common.text_wrap != TextWrap::TopAndBottom {
            continue;
        }
        if !crate::renderer::float_placement::is_para_topbottom_float(common)
            || common.vert_align != VertAlign::Top
            || common.height == 0
            || common.height > i32::MAX as u32
        {
            return false;
        }
        let Some(anchor) = crate::renderer::layout::control_line_seg_index(paragraph, index)
            .and_then(|index| paragraph.line_segs.get(index))
        else {
            return false;
        };
        let top = i64::from(anchor.vertical_pos) + i64::from(common.vertical_offset as i32);
        let height = crate::renderer::composer::captioned_shape_height_px(
            crate::renderer::hwpunit_to_px(common.height as i32, dpi),
            caption,
            dpi,
        );
        let bottom = top
            + (height * 7200.0 / dpi).round() as i64
            + i64::from(margin_top)
            + i64::from(margin_bottom);
        let anchor_end = i64::from(anchor.vertical_pos)
            + i64::from(anchor.line_height)
            + i64::from(anchor.line_spacing);
        if bottom <= top
            || anchor_end > top
            || paragraph.line_segs.iter().any(|line| {
                let start = i64::from(line.vertical_pos);
                let end = start + i64::from(line.line_height);
                start < bottom && end > top
            })
            || !paragraph
                .line_segs
                .iter()
                .any(|line| i64::from(line.vertical_pos) >= bottom)
        {
            return false;
        }
        has_band = true;
    }
    has_band
}

/// 글줄 사이 개체 띠까지 포함한 원본 단위의 점유 높이·흐름 전진량.
pub(crate) fn stored_float_row_span_hu(
    paragraph: &Paragraph,
    range: std::ops::Range<usize>,
    dpi: f64,
) -> Option<(i32, i32)> {
    if !owns_embedded_float_bands(paragraph, dpi) || range.is_empty() {
        return None;
    }
    let rows = paragraph.line_segs.get(range)?;
    let first = rows.first()?.vertical_pos;
    let content_bottom = rows.iter().try_fold(first, |bottom, row| {
        row.vertical_pos
            .checked_add(row.line_height)
            .map(|end| bottom.max(end))
    })?;
    let last = rows.last()?;
    let flow_end = last
        .vertical_pos
        .checked_add(last.line_height)?
        .checked_add(last.line_spacing)?;
    Some((
        content_bottom.checked_sub(first)?,
        flow_end.checked_sub(first)?,
    ))
}
