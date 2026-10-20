//! 표의 원문 전용 줄과 어울림 배제 뒤의 물리 슬롯을 같은 프레임으로 확정한다.

use crate::model::{paragraph::LineSeg, paragraph::Paragraph, table::Table};
use crate::renderer::{
    height_measurer::MeasuredTable,
    layout_frame::{FrameRowMetrics, LayoutFrame, ParagraphBox, RowSegment},
    px_to_hwpunit,
};

pub(super) struct StoredSquareRows {
    pub(super) anchor_offset_hwp: i32,
    owner: usize,
}

/// 마지막 저장 줄이 표 슬롯만 소유하고 현재 측정 상자와 같은 높이인지 확인한다.
pub(super) fn source(
    para: &Paragraph,
    control: usize,
    table: &Table,
    measured: &MeasuredTable,
    box_: &ParagraphBox,
    dpi: f64,
) -> Option<StoredSquareRows> {
    if para.stored_text_partition_is_dirty()
        || para.cell_format_vpos_dirty
        || table.caption.is_some()
        || !measured.total_height.is_finite()
        || para.controls.iter().enumerate().any(|(index, item)| {
            index != control && !super::super::composer::control_is_width_neutral_marker(item)
        })
    {
        return None;
    }
    let position = *para.control_text_positions().get(control)?;
    // 가시 문자열 뒤의 컨트롤은 char_offsets에 별도 글자 항목이 없다.
    // 저장 줄과 같은 원본 UTF-16 축에서 컨트롤 슬롯의 시작을 읽는다.
    let offset = *para.control_utf16_positions().get(control)?;
    let owner = para.line_segs.len().checked_sub(1)?;
    let row = para.line_segs.get(owner)?;
    if owner == 0
        || row.text_start != offset
        || para.text.chars().skip(position).any(visible)
        || !para.text.chars().take(position).any(visible)
    {
        return None;
    }
    let horizontal = box_.effective();
    if para.line_segs.iter().any(|line| {
        line.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0
            || line.line_height <= 0
            || line.line_height.saturating_add(line.line_spacing) <= 0
            || line.column_start != horizontal.start
            || line.segment_width != horizontal.end - horizontal.start
    }) || para.line_segs.windows(2).any(|pair| {
        pair[0].text_start >= pair[1].text_start || pair[0].vertical_pos > pair[1].vertical_pos
    }) {
        return None;
    }
    let outer_height = px_to_hwpunit(measured.total_height, dpi)
        .checked_add(i32::from(table.outer_margin_top))?
        .checked_add(i32::from(table.outer_margin_bottom))?;
    if (i64::from(row.line_height) - i64::from(outer_height)).abs() > 1 {
        return None;
    }
    let first = para.line_segs.first()?;
    let preceding = &para.line_segs[owner - 1];
    let anchor_offset_hwp = preceding
        .vertical_pos
        .checked_sub(first.vertical_pos)?
        .checked_add(preceding.line_height)?
        .checked_add(preceding.line_spacing)?;
    (anchor_offset_hwp > 0).then_some(StoredSquareRows {
        anchor_offset_hwp,
        owner,
    })
}

fn visible(character: char) -> bool {
    character > '\u{001f}' && character != '\u{fffc}' && !character.is_whitespace()
}

/// 원래 높이의 슬롯도 배제 영역을 피해야 한다. 높이를 줄여 fit시킨 뒤 늘리지 않는다.
pub(super) fn project(
    para: &Paragraph,
    source: &StoredSquareRows,
    frame: &mut LayoutFrame,
) -> Option<Vec<LineSeg>> {
    let origin = para.line_segs.first()?.vertical_pos;
    for (index, row) in para.line_segs.iter().enumerate() {
        let original_top = row.vertical_pos.checked_sub(origin)?;
        if index < source.owner {
            frame.top = original_top;
        }
        frame.minimum_width = row.segment_width;
        let intervals = frame.carve(row.line_height).to_vec();
        if intervals.len() != 1
            || intervals[0].start != row.column_start
            || intervals[0].end != row.column_start.checked_add(row.segment_width)?
            || (index < source.owner && frame.top != original_top)
            || (index == source.owner
                && original_top != source.anchor_offset_hwp
                && original_top != frame.top)
        {
            return None;
        }
        let end = para
            .line_segs
            .get(index + 1)
            .map_or(para.char_count.saturating_sub(1), |next| next.text_start);
        frame.commit_carved_row(
            FrameRowMetrics {
                vertical_pos: frame.top,
                line_height: row.line_height,
                text_height: row.text_height,
                baseline_distance: row.baseline_distance,
                line_spacing: row.line_spacing,
            },
            vec![RowSegment::new(
                row.text_start..end,
                intervals[0].clone(),
                row.tag,
            )],
        )?;
    }
    Some(frame.project_line_segs())
}
