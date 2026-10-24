//! 원본 행 선언을 이미 충족한 표의 본체와 닫힌 후속 줄 프레임.
//! 측정이 보존한 행 본체를 format·paint와 후속 줄 원점이 함께 소비한다.

use crate::model::{paragraph::Paragraph, table::Table};
use crate::renderer::float_placement::ParagraphFloatPlacement;
use crate::renderer::hwpunit_to_px;

/// 모든 저장 행이 실제 측정 결과와 같으면 더 큰 개체 캐시로 다시 늘리지 않는다.
/// 보호 크기, 병합 행, 실제 행 성장 및 저장 줄 없는 재조판은 기존 계약을 유지한다.
pub(crate) fn stored_rows_own_body(table: &Table, rows: &[f64], dpi: f64) -> bool {
    if !dpi.is_finite()
        || dpi <= 0.0
        || table.common.treat_as_char
        || table.common.size_protect
        || rows.len() < 2
        || rows.len() != usize::from(table.row_count)
        || table.cells.is_empty()
        || table.cells.iter().any(|cell| {
            cell.row_span != 1
                || cell.height == 0
                || cell.height > i32::MAX as u32
                || cell.dirty_flag
                || cell.paragraphs.is_empty()
                || cell.paragraphs.iter().any(|para| {
                    para.stored_text_partition_is_dirty()
                        || para.cell_format_vpos_dirty
                        || crate::renderer::para_has_no_stored_line_segs(para)
                })
        })
    {
        return false;
    }
    let unit = dpi / 7200.0;
    rows.iter().enumerate().all(|(index, measured)| {
        let declared = table
            .cells
            .iter()
            .filter(|cell| usize::from(cell.row) == index)
            .map(|cell| cell.height)
            .max()
            .map(|height| hwpunit_to_px(height as i32, dpi));
        declared.is_some_and(|height| {
            measured.is_finite()
                && (measured - height).abs()
                    <= unit * 0.5 + (measured.abs() + height.abs()) * f64::EPSILON
        })
    })
}

/// 한컴 저장 상자가 후속 줄에서 정확히 닫히면 정규화된 본체 차이만 투영한다.
/// 원점은 현재 단의 실제 흐름이며 앞 단이나 문서 전체의 저장 원점을 이식하지 않는다.
pub(crate) fn normalized_empty_control_frame(
    host: &Paragraph,
    successor: &Paragraph,
    table: &Table,
    rows: &[f64],
    flow_origin: f64,
    dpi: f64,
    hwpx_object_frame: bool,
) -> Option<ParagraphFloatPlacement> {
    if !stored_rows_own_body(table, rows, dpi)
        || table.caption.is_some()
        || !flow_origin.is_finite()
        || table.common.height == 0
        || table.common.height > i32::MAX as u32
    {
        return None;
    }
    let anchor = host.line_segs.first()?;
    let source_origin = host
        .source_line_seg_vertical_pos
        .as_ref()
        .and_then(|positions| positions.first().copied())
        .unwrap_or(anchor.vertical_pos);
    let declared = hwpunit_to_px(table.common.height as i32, dpi);
    let body = rows.iter().sum::<f64>()
        + hwpunit_to_px(i32::from(table.cell_spacing), dpi) * rows.len().saturating_sub(1) as f64;
    if body >= declared {
        return None;
    }
    let mut frame = crate::renderer::float_placement::stored_empty_control_table_frame(
        host,
        successor,
        table,
        declared,
        0.0,
        source_origin,
        dpi,
        hwpx_object_frame,
    )?;
    let change = body - declared;
    frame.anchor_y += flow_origin;
    frame.table_top += flow_origin;
    frame.occupied_bottom += flow_origin + change;
    frame.stored_host_origin = frame.stored_host_origin.map(|origin| origin + flow_origin);
    frame.stored_successor_line_origin = frame
        .stored_successor_line_origin
        .map(|origin| origin + flow_origin + change);
    Some(frame)
}
