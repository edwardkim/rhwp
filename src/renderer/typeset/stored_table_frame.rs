//! 통째 표와 이어받기의 선언 프레임이 저장 내용과 행 형상을 소유하는지 확인한다.
//!
//! 셀 안의 줄만 들어간다는 사실로 표 전체 수용을 판정하지 않는다.
//! 셀·표의 공통 유효 안 여백을 포함한 물리 점유가 선언 셀 안에 있어야 한다.

use crate::renderer::typeset::{hwpunit_to_px, is_synthetic_line_seg, raw_table_ctrl_height_px};

/// Whether every text-bearing cell proves, through its stored line segments,
/// that its content belongs inside the declared cell box. This distinguishes a
/// renderer metric expansion from a source-owned row growth without a table
/// size ratio or a pixel cap. Rowspans and controls have independent physical
/// ownership, so they stay on the measured-row path.
pub(super) fn table_declared_height_has_stored_cell_content_frame(
    table: &crate::model::table::Table,
    dpi: f64,
) -> bool {
    !table.cells.is_empty()
        && table.cells.iter().all(|cell| {
            if cell.row_span != 1 || cell.height >= 0x8000_0000 {
                return false;
            }
            let mut has_text = false;
            let mut stored_bottom = None;
            for para in &cell.paragraphs {
                if !para.controls.is_empty() {
                    return false;
                }
                let text = para.text.replace(|c: char| c.is_control(), "");
                if text.trim().is_empty() {
                    continue;
                }
                has_text = true;
                for seg in para
                    .line_segs
                    .iter()
                    .filter(|seg| !is_synthetic_line_seg(seg))
                    .filter(|seg| seg.vertical_pos >= 0 && seg.line_height > 0)
                {
                    let bottom =
                        hwpunit_to_px(seg.vertical_pos.saturating_add(seg.line_height), dpi);
                    stored_bottom =
                        Some(stored_bottom.map_or(bottom, |current: f64| current.max(bottom)));
                }
            }
            !has_text
                || stored_bottom.is_some_and(|bottom| {
                    let padding = cell.effective_padding(&table.padding);
                    let physical_bottom = bottom
                        + hwpunit_to_px(i32::from(padding.top) + i32::from(padding.bottom), dpi);
                    physical_bottom <= hwpunit_to_px(cell.height as i32, dpi)
                })
        })
}

/// A cell-local stored frame proves that the cell's text fits its own box, but
/// not that the table object's declared frame owns every row.  Some native HWP
/// RowBreak tables retain a stale object height while their individual cells
/// carry the full multi-row geometry.  Treating that short object as the whole
/// table collapses a source-owned first fragment into one floating table.
///
/// This checks the other half of the source contract without relying on a
/// measured/declared ratio: each non-rowspan row must have a declared cell box
/// and their geometry, including cell spacing, must fit inside the declared
/// table object frame.
pub(super) fn table_declared_object_covers_cell_row_frames(
    table: &crate::model::table::Table,
    dpi: f64,
) -> bool {
    let row_count = table.row_count as usize;
    if row_count == 0 || table.cells.is_empty() {
        return false;
    }

    let declared_object_height = raw_table_ctrl_height_px(table, dpi)
        .unwrap_or_else(|| hwpunit_to_px(table.common.height as i32, dpi).max(0.0));
    if declared_object_height <= 0.0 {
        return false;
    }

    let mut row_heights: Vec<Option<f64>> = vec![None; row_count];
    for cell in &table.cells {
        let row = cell.row as usize;
        if row >= row_count || cell.row_span != 1 || cell.height >= 0x8000_0000 {
            return false;
        }
        let height = hwpunit_to_px(cell.height as i32, dpi);
        if height <= 0.0 {
            return false;
        }
        row_heights[row] = Some(row_heights[row].unwrap_or(0.0).max(height));
    }

    let declared_row_geometry =
        row_heights
            .into_iter()
            .collect::<Option<Vec<_>>>()
            .map(|heights| {
                heights.iter().sum::<f64>()
                    + hwpunit_to_px(table.cell_spacing as i32, dpi)
                        * heights.len().saturating_sub(1) as f64
            });
    declared_row_geometry.is_some_and(|height| height <= declared_object_height + 0.5)
}
