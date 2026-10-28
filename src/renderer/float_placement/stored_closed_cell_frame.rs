//! 닫힌 저장 표 프레임과 앞뒤 글줄의 셀 원점·점유 끝을 공유한다.

use super::{hwpunit_to_px, stored_table_before_host_frame, StoredBlockCellFlow};
use crate::model::{control::Control, paragraph::Paragraph};
use crate::renderer::style_resolver::ResolvedStyleSet;

/// 마지막 저장 줄의 점유 끝과 후속 문단이 같은 바깥 상자를 닫는 경우의 괘선 하단.
/// 빈 마지막 줄도 물리 공간을 소유한다. 바깥 아래 여백은 괘선 뒤에서 별도로 소비한다.
pub(super) fn terminal_cell_border_before_successor_hu(
    table: &crate::model::table::Table,
    successor: &Paragraph,
) -> Option<i32> {
    use crate::model::paragraph::LineSeg;
    let [cell] = table.cells.as_slice() else {
        return None;
    };
    let tail = cell.paragraphs.last()?;
    let line = tail.line_segs.last()?;
    let [next] = successor.line_segs.as_slice() else {
        return None;
    };
    if cell.dirty_flag
        || tail.stored_text_partition_is_dirty()
        || tail.cell_format_vpos_dirty
        || !tail.controls.is_empty()
        || line.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0
        || line.vertical_pos < 0
        || line.line_height <= 0
        || table.outer_margin_top < 0
        || table.outer_margin_bottom < 0
    {
        return None;
    }
    let padding = cell.effective_padding(&table.padding);
    if padding.top < 0 || padding.bottom < 0 {
        return None;
    }
    let outer_bottom = i64::from(line.vertical_pos)
        + i64::from(line.line_height)
        + i64::from(padding.top)
        + i64::from(padding.bottom)
        + i64::from(table.outer_margin_top)
        + i64::from(table.outer_margin_bottom);
    (outer_bottom == i64::from(next.vertical_pos))
        .then(|| {
            next.vertical_pos
                .checked_sub(i32::from(table.outer_margin_bottom))
        })
        .flatten()
}

/// 모든 블록 표가 호스트 앞의 독립 저장 프레임으로 닫힐 때만 수용한다.
/// 부분 저장 사다리·새 쪽/단·다른 표 앵커가 섞이면 기존 경로에 남긴다.
pub(super) fn closed_before_host_flow(
    paragraphs: &[Paragraph],
    styles: &ResolvedStyleSet,
    heights: &[Vec<f64>],
    dpi: f64,
) -> Option<StoredBlockCellFlow> {
    let mut previous = None;
    let mut has_frame = false;
    let mut result = StoredBlockCellFlow {
        paragraph_origins: Vec::with_capacity(paragraphs.len()),
        origins: paragraphs
            .iter()
            .map(|p| vec![None; p.controls.len()])
            .collect(),
        bottom: 0.0,
        closed_before_host: true,
    };
    for (pi, paragraph) in paragraphs.iter().enumerate() {
        if paragraph.raw_break_type != 0
            || paragraph.column_type != crate::model::paragraph::ColumnBreakType::None
        {
            return None;
        }
        let first = paragraph.line_segs.first()?;
        let source_host = hwpunit_to_px(first.vertical_pos, dpi);
        result.paragraph_origins.push(source_host);
        for line in &paragraph.line_segs {
            if previous.is_some_and(|top| line.vertical_pos < top) {
                return None;
            }
            previous = Some(line.vertical_pos);
            let end = hwpunit_to_px(line.vertical_pos.checked_add(line.line_height)?, dpi);
            result.bottom = result.bottom.max(end);
        }
        for (ci, control) in paragraph.controls.iter().enumerate() {
            let Control::Table(table) = control else {
                continue;
            };
            let body = *heights.get(pi)?.get(ci)?
                - hwpunit_to_px(
                    i32::from(table.outer_margin_top) + i32::from(table.outer_margin_bottom),
                    dpi,
                );
            let before = styles
                .para_styles
                .get(paragraph.para_shape_id as usize)?
                .spacing_before;
            let frame = stored_table_before_host_frame(
                paragraphs,
                pi,
                table,
                ci,
                source_host - before,
                body,
                styles,
                dpi,
            )?;
            // 원점은 바깥여백·offset 앞이며 실제 배치가 그 몫을 한 번 적용한다.
            // 표 뒤의 호스트 줄과 다음 본문은 저장 좌표를 그대로 소비한다.
            result.origins[pi][ci] = Some(frame.anchor_y);
            result.bottom = result.bottom.max(frame.occupied_bottom);
            has_frame = true;
        }
    }
    has_frame.then_some(result)
}
