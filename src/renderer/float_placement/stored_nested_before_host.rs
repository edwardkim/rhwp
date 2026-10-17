//! 원본의 빈 호스트 줄 앞에 저장된 표 상자를 측정·배치에서 공유한다.

use super::{hwpunit_to_px, is_para_topbottom_float, ParagraphFloatFlow, ParagraphFloatPlacement};
use crate::model::control::Control;
use crate::model::paragraph::{LineSeg, Paragraph};
use crate::model::table::{Cell, Table};
use crate::renderer::style_resolver::ResolvedStyleSet;

pub(super) fn original_lines(para: &Paragraph) -> bool {
    para.raw_break_type == 0
        && !para.stored_text_partition_is_dirty()
        && !para.cell_format_vpos_dirty
        && !para.line_segs.is_empty()
        && para.line_segs.iter().all(|line| {
            line.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0
                && line.vertical_pos >= 0
                && line.line_height > 0
                && line.line_spacing >= 0
        })
        && para
            .line_segs
            .windows(2)
            .all(|pair| pair[1].vertical_pos >= pair[0].vertical_pos)
}

/// 선행 줄 끝과 호스트 줄 사이를 표의 실제 바깥 상자가 정확히 닫아야 한다.
/// 단순한 큰 vpos 점프는 근거가 아니다. 호스트 뒤 줄도 같은 저장 상자를 닫는다.
pub(crate) fn stored_nested_before_host_placement(
    cell: &Cell,
    para_index: usize,
    table: &Table,
    control_index: usize,
    origin: f64,
    table_height: f64,
    styles: &ResolvedStyleSet,
    dpi: f64,
) -> Option<ParagraphFloatPlacement> {
    if cell.dirty_flag {
        return None;
    }
    stored_table_before_host_frame(
        &cell.paragraphs,
        para_index,
        table,
        control_index,
        origin,
        table_height,
        styles,
        dpi,
    )
}

/// 측정도 같은 저장 앞·뒤 줄과 표 바깥 상자로 호스트의 줄 소속을 확인한다.
/// 셀 편집 여부는 셀을 소유한 호출자가, 저장 줄의 유효성은 이 함수가 확인한다.
pub(crate) fn stored_table_before_host_frame(
    paragraphs: &[Paragraph],
    para_index: usize,
    table: &Table,
    control_index: usize,
    origin: f64,
    table_height: f64,
    styles: &ResolvedStyleSet,
    dpi: f64,
) -> Option<ParagraphFloatPlacement> {
    let para = paragraphs.get(para_index)?;
    let previous = paragraphs.get(para_index.checked_sub(1)?)?;
    let next = paragraphs.get(para_index.checked_add(1)?)?;
    let [host] = para.line_segs.as_slice() else {
        return None;
    };
    if !dpi.is_finite()
        || dpi <= 0.0
        || !origin.is_finite()
        || !table_height.is_finite()
        || table.common.height > i32::MAX as u32
        || table.caption.is_some()
        || !is_para_topbottom_float(&table.common)
        || !table.common.flow_with_text
        || (table.common.vertical_offset as i32) < 0
        || !para.text.trim().is_empty()
        || control_index != 0
        || !matches!(para.controls.as_slice(), [Control::Table(_)])
        || !previous.controls.is_empty()
        || !next.controls.is_empty()
        || !original_lines(para)
        || !original_lines(previous)
        || !original_lines(next)
    {
        return None;
    }
    // 전체 셀 좌표를 되감은 뒤의 줄은 앞 프레임의 표 원점을 복원하지 않는다.
    let prefix: Vec<_> = paragraphs[..=para_index]
        .iter()
        .flat_map(|paragraph| &paragraph.line_segs)
        .collect();
    if !paragraphs[..=para_index].iter().all(original_lines)
        || prefix
            .windows(2)
            .any(|pair| pair[1].vertical_pos < pair[0].vertical_pos)
    {
        return None;
    }
    let previous_line = previous.line_segs.last()?;
    let previous_style = styles.para_styles.get(previous.para_shape_id as usize)?;
    let style = styles.para_styles.get(para.para_shape_id as usize)?;
    let next_style = styles.para_styles.get(next.para_shape_id as usize)?;
    let declared = hwpunit_to_px(table.common.height as i32, dpi);
    let epsilon = hwpunit_to_px(2, dpi);
    let lead = hwpunit_to_px(i32::from(table.outer_margin_top), dpi);
    let tail = hwpunit_to_px(i32::from(table.outer_margin_bottom), dpi);
    let offset = hwpunit_to_px(table.common.vertical_offset as i32, dpi);
    let outer = offset + lead + table_height + tail;
    let previous_end = hwpunit_to_px(
        previous_line
            .vertical_pos
            .checked_add(previous_line.line_height)?
            .checked_add(previous_line.line_spacing)?,
        dpi,
    ) + previous_style.spacing_after
        + style.spacing_before;
    let source_host = hwpunit_to_px(host.vertical_pos, dpi);
    let host_end = source_host
        + hwpunit_to_px(host.line_height.checked_add(host.line_spacing)?, dpi)
        + style.spacing_after
        + next_style.spacing_before;
    if (table_height - declared).abs() > epsilon
        || (previous_end + outer - source_host).abs() > epsilon
        || (host_end - hwpunit_to_px(next.line_segs.first()?.vertical_pos, dpi)).abs() > epsilon
    {
        return None;
    }
    let host_origin = origin + style.spacing_before;
    let anchor_y = host_origin - outer;
    Some(ParagraphFloatPlacement {
        flow: ParagraphFloatFlow::NextLine,
        anchor_y,
        stored_host_origin: None,
        stored_successor_line_origin: None,
        table_left: None,
        table_top: anchor_y + offset + lead,
        occupied_bottom: host_origin,
    })
}
