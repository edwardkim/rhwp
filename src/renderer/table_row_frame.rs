//! 원본 행 선언을 이미 충족한 표의 본체와 닫힌 후속 줄 프레임.
//! 측정이 보존한 행 본체를 format·paint와 후속 줄 원점이 함께 소비한다.

use crate::model::{
    control::Control,
    paragraph::{LineSeg, Paragraph},
    table::Table,
};
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

/// 원본 저장 줄과 선언 상자로 입증한 마지막 행의 빈 띠만 회수한다.
/// 측정 유무와 무관하게 같은 행 벡터를 소비하며, 쪽을 가른 행에는 호출하지 않는다.
pub(crate) fn declared_tail_row_height(
    table: &Table,
    row_heights: &[f64],
    cell_spacing: f64,
    dpi: f64,
) -> Option<f64> {
    let row_count = row_heights.len();
    if row_count < 2 || table.row_count as usize != row_count || table.common.height == 0 {
        return None;
    }
    let last_row = row_count - 1;

    // 셀 저장 줄 사다리가 되감기면(다음 줄 vpos 가 앞 줄보다 작다) 한/글이 그 표를 쪽에서
    // 갈랐다는 저장 사실이다. 그 표의 선언 높이는 첫 조각 프레임일 뿐 표 전체가 아니고,
    // 아래 내용 하한도 되감긴 사다리로는 잴 수 없다(pic-in-head-01 pi=65 3×3: 선언 905.8 =
    // 18쪽 조각, 마지막 행 내용은 19쪽으로 이어진다).
    let has_stored_page_rewind = table.cells.iter().any(|cell| {
        let mut previous: Option<i32> = None;
        cell.paragraphs
            .iter()
            .flat_map(|paragraph| paragraph.line_segs.iter())
            .filter(|seg| seg.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0)
            .any(|seg| {
                let rewinds = previous.is_some_and(|vpos| seg.vertical_pos < vpos);
                previous = Some(seg.vertical_pos);
                rewinds
            })
    });
    if has_stored_page_rewind {
        return None;
    }

    // 마지막 행이 저장 선언으로만 잡힌 행인지 — 콘텐츠가 밀어 키운 행이면 여유가 없다.
    let declared_tail = table
        .cells
        .iter()
        .filter(|cell| {
            cell.row as usize == last_row && cell.row_span == 1 && cell.height < 0x8000_0000
        })
        .map(|cell| hwpunit_to_px(cell.height as i32, dpi))
        .fold(0.0f64, f64::max);
    if declared_tail <= 0.0 || (row_heights[last_row] - declared_tail).abs() > 0.5 {
        return None;
    }

    // 축소 하한 = 마지막 행 셀들의 저장 줄 내용 높이 + 상하 패딩.
    let mut content_floor = 0.0f64;
    for cell in &table.cells {
        if cell.row as usize != last_row || cell.row_span != 1 {
            continue;
        }
        // 합성 줄 셀과 중첩 표 tail 은 여기서 판단하지 않는다 (후자는
        // `fit_measured_table_nested_tail_to_declared_height` 담당).
        if cell
            .paragraphs
            .iter()
            .any(crate::renderer::para_has_no_stored_line_segs)
            || cell.paragraphs.iter().any(|paragraph| {
                paragraph
                    .controls
                    .iter()
                    .any(|control| matches!(control, Control::Table(_)))
            })
        {
            return None;
        }
        let content_hu = cell
            .paragraphs
            .iter()
            .flat_map(|paragraph| paragraph.line_segs.iter())
            .map(|seg| i64::from(seg.vertical_pos) + i64::from(seg.line_height))
            .max()
            .unwrap_or(0);
        let pad = hwpunit_to_px(cell.effective_vertical_padding_hu(&table.padding), dpi);
        let floor = hwpunit_to_px(content_hu as i32, dpi) + pad;
        if floor > content_floor {
            content_floor = floor;
        }
    }
    if content_floor <= 0.0 {
        return None;
    }

    let spacing_total = cell_spacing * row_count.saturating_sub(1) as f64;
    let target_body_height = hwpunit_to_px(table.common.height as i32, dpi);
    let target_row_sum = (target_body_height - spacing_total).max(0.0);
    let current_row_sum = row_heights.iter().sum::<f64>();
    let reduction = current_row_sum - target_row_sum;
    // 반올림 급(≤0.5px)은 건드리지 않는다. 회수량의 상한은 비율이 아니라 마지막 행의 저장
    // 내용 하한(아래 검사)과 되감기 없는 사다리(위 검사)다 — 한/글은 표 선언 높이를 지키며
    // 앞 행이 커진 만큼을 마지막 행의 빈 공간에서 거둔다. 2025 행정업무운영 편람 구역 10
    // pi=85(6×5): 정본 괘선 행 12.3/33.3/20.2/13.3/308.1/**9.6** = 396.9 ≒ 선언 397.0,
    // 마지막 행 선언 21.5 · 내용 하한 8.9(감소 11.8px, 선언의 3%). 하한 아래로는 줄이지 않으므로
    // 거둘 공간이 모자라면 회수하지 않는다(issue3587 labnote pi=12: 감소 13.7 > 여유 7.5,
    // 정본도 마지막 행 24.6 그대로 표가 커진다).
    if reduction <= 0.5 {
        return None;
    }
    let target_tail = row_heights[last_row] - reduction;
    if target_tail < content_floor - 0.5 {
        return None;
    }

    Some(target_tail)
}
