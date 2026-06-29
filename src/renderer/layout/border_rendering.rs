//! 표 테두리 수집/렌더링 + 문단 테두리 라인 생성

use super::super::render_tree::*;
use super::super::style_resolver::ResolvedBorderStyle;
use super::super::{LineStyle, StrokeDash};
use crate::model::style::{BorderLine, BorderLineType, CenterLine};
use crate::model::table::Table;

fn merge_border(a: &BorderLine, b: &BorderLine) -> BorderLine {
    if a.line_type == BorderLineType::None {
        return *b;
    }
    if b.line_type == BorderLineType::None {
        return *a;
    }

    let a_w = border_width_to_px(a.width);
    let b_w = border_width_to_px(b.width);
    if (a_w - b_w).abs() > 0.01 {
        return if a_w > b_w { *a } else { *b };
    }

    let priority = |lt: BorderLineType| -> u8 {
        match lt {
            BorderLineType::None => 0,
            BorderLineType::ThinThickThinTriple => 4,
            BorderLineType::Double
            | BorderLineType::ThinThickDouble
            | BorderLineType::ThickThinDouble => 3,
            BorderLineType::Wave | BorderLineType::DoubleWave => 2,
            _ => 1,
        }
    };
    if priority(a.line_type) >= priority(b.line_type) {
        *a
    } else {
        *b
    }
}

/// 엣지 그리드 슬롯에 테두리를 병합 저장
fn merge_edge_slot(slot: &mut Option<BorderLine>, border: &BorderLine) {
    if border.line_type == BorderLineType::None {
        return;
    }
    *slot = Some(match *slot {
        Some(existing) => merge_border(&existing, border),
        None => *border,
    });
}

/// 행별 열 누적 위치를 계산한다.
/// HWP에서는 각 셀이 독립적인 너비를 가질 수 있어, 같은 열이라도 행마다 열 경계 위치가 다를 수 있다.
/// col_span==1인 셀의 실제 너비를 사용하고, 해당 위치에 셀이 없으면 전역 col_widths를 폴백한다.
pub(crate) fn build_row_col_x(
    table: &Table,
    col_widths: &[f64],
    col_count: usize,
    row_count: usize,
    cell_spacing: f64,
    dpi: f64,
) -> Vec<Vec<f64>> {
    use super::super::hwpunit_to_px;
    // 셀 너비 그리드 구축 (O(cells) 탐색 1회)
    let mut cell_width_grid = vec![vec![None::<f64>; col_count]; row_count];
    for cell in &table.cells {
        if cell.col_span == 1
            && cell.width > 0
            && (cell.col as usize) < col_count
            && (cell.row as usize) < row_count
        {
            cell_width_grid[cell.row as usize][cell.col as usize] =
                Some(hwpunit_to_px(cell.width as i32, dpi));
        }
    }
    let mut base_rx = vec![0.0f64; col_count + 1];
    for c in 0..col_count {
        base_rx[c + 1] =
            base_rx[c] + col_widths[c] + if c + 1 < col_count { cell_spacing } else { 0.0 };
    }

    if table.common.treat_as_char {
        return vec![base_rx; row_count];
    }

    let target_total = if table.common.width > 0 {
        hwpunit_to_px(table.common.width as i32, dpi)
            + cell_spacing * col_count.saturating_sub(1) as f64
    } else {
        base_rx.last().copied().unwrap_or(0.0)
    };

    let inferred_local_resize_rows = table.inferred_local_resize_rows();
    if !table.local_resize_rows.is_empty() || !inferred_local_resize_rows.is_empty() {
        let mut row_col_x_from_cells = vec![base_rx.clone(); row_count];
        let mut has_cell_order_row = false;
        for (r, row_x) in row_col_x_from_cells.iter_mut().enumerate().take(row_count) {
            let row_idx = r as u16;
            if !table.local_resize_rows.contains(&row_idx)
                && !inferred_local_resize_rows.contains(&row_idx)
            {
                continue;
            }
            let mut row_cells: Vec<_> = table
                .cells
                .iter()
                .enumerate()
                .filter(|(_, cell)| cell.row as usize == r && cell.row_span == 1)
                .collect();
            row_cells.sort_by_key(|(_, cell)| cell.col);
            let has_width_overrides = row_cells.iter().any(|(cell_idx, _)| {
                table
                    .local_resize_cell_widths
                    .iter()
                    .any(|(idx, _)| idx == cell_idx)
            });

            let mut cursor = 0.0;
            let mut next_col = 0usize;
            let mut candidate = vec![0.0f64; col_count + 1];
            let mut valid = !row_cells.is_empty();
            for (cell_idx, cell) in row_cells {
                let c = cell.col as usize;
                let span = cell.col_span.max(1) as usize;
                let end = (c + span).min(col_count);
                if c != next_col || end <= c {
                    valid = false;
                    break;
                }

                candidate[c] = cursor;
                let cell_w = table
                    .local_resize_cell_widths
                    .iter()
                    .find(|(idx, _)| *idx == cell_idx)
                    .map(|(_, width)| hwpunit_to_px(*width as i32, dpi))
                    .unwrap_or_else(|| {
                        if has_width_overrides {
                            (base_rx[end] - base_rx[c]).max(0.0)
                        } else {
                            hwpunit_to_px(cell.width as i32, dpi)
                        }
                    });
                let end_x = cursor + cell_w;
                for inner_col in c + 1..end {
                    let ratio = (inner_col - c) as f64 / span as f64;
                    candidate[inner_col] = cursor + cell_w * ratio;
                }
                candidate[end] = end_x;
                cursor = end_x + if end < col_count { cell_spacing } else { 0.0 };
                next_col = end;
            }

            if valid && next_col == col_count {
                let residual = target_total - cursor;
                if residual < -0.5 {
                    valid = false;
                } else if residual > 0.5 {
                    candidate[col_count] += residual;
                }
            }

            if valid && next_col == col_count {
                *row_x = candidate;
                has_cell_order_row = true;
            }
        }

        if has_cell_order_row
            && row_col_x_from_cells.iter().any(|rx| {
                rx.iter()
                    .zip(base_rx.iter())
                    .any(|(a, b)| (a - b).abs() > 0.01)
            })
        {
            return row_col_x_from_cells;
        }
    }

    let has_independent_widths = cell_width_grid.iter().any(|row| {
        row.iter().enumerate().any(|(c, w)| {
            w.map(|actual| (actual - col_widths.get(c).copied().unwrap_or(actual)).abs() > 0.01)
                .unwrap_or(false)
        })
    });
    if !has_independent_widths {
        return vec![base_rx; row_count];
    }

    let fallback_w = hwpunit_to_px(1800, dpi);
    let mut row_col_x = vec![vec![0.0f64; col_count + 1]; row_count];
    for r in 0..row_count {
        for c in 0..col_count {
            let w = cell_width_grid[r][c]
                .or_else(|| col_widths.get(c).copied())
                .unwrap_or(fallback_w);
            row_col_x[r][c + 1] =
                row_col_x[r][c] + w + if c + 1 < col_count { cell_spacing } else { 0.0 };
        }
        // 저장 파일의 cell.width는 병합 제약을 풀기 전 보조값일 수 있다.
        // 행별 누적 폭이 표 외곽 폭과 맞지 않으면 독립 segment가 아니라 전역 grid를 따른다.
        // Stage 12의 로컬 segment 리사이즈는 보상 리사이즈로 행 전체 폭을 유지하므로 이 조건을 통과한다.
        if (row_col_x[r][col_count] - target_total).abs() > 0.5 {
            row_col_x[r].clone_from_slice(&base_rx);
        }
    }
    row_col_x
}

/// 셀 테두리를 엣지 그리드에 수집
/// h_edges[row_boundary][col]: 수평 엣지 (row_boundary 0..=row_count, col 0..col_count)
/// v_edges[col_boundary][row]: 수직 엣지 (col_boundary 0..=col_count, row 0..row_count)
/// borders: [좌, 우, 상, 하]
pub(crate) fn collect_cell_borders(
    h_edges: &mut [Vec<Option<BorderLine>>],
    v_edges: &mut [Vec<Option<BorderLine>>],
    col: usize,
    row: usize,
    col_span: usize,
    row_span: usize,
    borders: &[BorderLine; 4],
) {
    let h_rows = h_edges.len();
    let v_cols = v_edges.len();
    let col_count = if h_rows > 0 { h_edges[0].len() } else { return };
    let row_count = if v_cols > 0 { v_edges[0].len() } else { return };

    let end_col = (col + col_span).min(col_count);
    let end_row = (row + row_span).min(row_count);

    // 상 테두리
    if row < h_rows {
        for c in col..end_col {
            merge_edge_slot(&mut h_edges[row][c], &borders[2]);
        }
    }
    // 하 테두리
    if end_row < h_rows {
        for c in col..end_col {
            merge_edge_slot(&mut h_edges[end_row][c], &borders[3]);
        }
    }
    // 좌 테두리
    if col < v_cols {
        for r in row..end_row {
            merge_edge_slot(&mut v_edges[col][r], &borders[0]);
        }
    }
    // 우 테두리
    if end_col < v_cols {
        for r in row..end_row {
            merge_edge_slot(&mut v_edges[end_col][r], &borders[1]);
        }
    }
}

/// 엣지 그리드에서 테두리 Line 노드를 생성
/// 연속된 같은 스타일의 엣지 세그먼트는 하나의 Line으로 병합하여
/// 이중선/삼중선의 교차점 렌더링을 깔끔하게 처리한다.
/// row_col_x: 행별 열 누적 위치 (셀별 독립 너비 지원)
pub(crate) fn render_edge_borders(
    tree: &mut PageRenderTree,
    h_edges: &[Vec<Option<BorderLine>>],
    v_edges: &[Vec<Option<BorderLine>>],
    row_col_x: &[Vec<f64>],
    row_y: &[f64],
    table_x: f64,
    table_y: f64,
) -> Vec<RenderNode> {
    let mut nodes = Vec::new();
    let row_count = if row_y.len() > 1 { row_y.len() - 1 } else { 0 };

    // 수평 엣지 렌더링
    for (ri, h_row) in h_edges.iter().enumerate() {
        let y = table_y + row_y.get(ri).copied().unwrap_or(0.0);
        // 행 경계의 열 위치: 경계 아래 행 (또는 마지막 행) 기준
        let ref_row = ri.min(row_count.saturating_sub(1));
        let ref_cx = &row_col_x[ref_row.min(row_col_x.len() - 1)];
        let mut seg_start: Option<usize> = None;
        let mut seg_border: Option<BorderLine> = None;

        for (ci, edge_opt) in h_row.iter().enumerate() {
            let same_style = match (edge_opt, &seg_border) {
                (Some(e), Some(s)) => {
                    e.line_type == s.line_type && e.width == s.width && e.color == s.color
                }
                _ => false,
            };

            if let Some(border) = edge_opt {
                if same_style {
                    // 같은 스타일 → 세그먼트 연장
                } else {
                    // 다른 스타일 → 이전 세그먼트 마무리
                    if let (Some(start), Some(ref sb)) = (seg_start, seg_border) {
                        let x1 = table_x + ref_cx[start];
                        let x2 = table_x + ref_cx[ci];
                        nodes.extend(create_border_line_nodes(tree, &sb, x1, y, x2, y));
                    }
                    seg_start = Some(ci);
                    seg_border = Some(*border);
                }
            } else {
                if let (Some(start), Some(ref sb)) = (seg_start, seg_border) {
                    let x1 = table_x + ref_cx[start];
                    let x2 = table_x + ref_cx[ci];
                    nodes.extend(create_border_line_nodes(tree, &sb, x1, y, x2, y));
                }
                seg_start = None;
                seg_border = None;
            }
        }
        // 마지막 세그먼트
        if let (Some(start), Some(ref sb)) = (seg_start, seg_border) {
            let x1 = table_x + ref_cx[start];
            let x2 = table_x + ref_cx.get(h_row.len()).copied().unwrap_or(ref_cx[start]);
            nodes.extend(create_border_line_nodes(tree, &sb, x1, y, x2, y));
        }
    }

    // 수직 엣지 렌더링 (행별로 x 위치가 다를 수 있음)
    for (ci, v_col) in v_edges.iter().enumerate() {
        let mut seg_start: Option<usize> = None;
        let mut seg_border: Option<BorderLine> = None;
        let mut seg_x: f64 = 0.0;

        for (ri, edge_opt) in v_col.iter().enumerate() {
            let x = table_x
                + row_col_x
                    .get(ri)
                    .and_then(|rx| rx.get(ci).copied())
                    .unwrap_or(0.0);
            let same_style = match (edge_opt, &seg_border) {
                (Some(e), Some(s)) => {
                    e.line_type == s.line_type
                        && e.width == s.width
                        && e.color == s.color
                        && (x - seg_x).abs() < 0.01
                }
                _ => false,
            };

            if let Some(border) = edge_opt {
                if same_style {
                    // 같은 스타일 + 같은 x → 세그먼트 연장
                } else {
                    if let (Some(start), Some(ref sb)) = (seg_start, seg_border) {
                        let y1 = table_y + row_y[start];
                        let y2 = table_y + row_y[ri];
                        nodes.extend(create_border_line_nodes(tree, &sb, seg_x, y1, seg_x, y2));
                    }
                    seg_start = Some(ri);
                    seg_border = Some(*border);
                    seg_x = x;
                }
            } else {
                if let (Some(start), Some(ref sb)) = (seg_start, seg_border) {
                    let y1 = table_y + row_y[start];
                    let y2 = table_y + row_y[ri];
                    nodes.extend(create_border_line_nodes(tree, &sb, seg_x, y1, seg_x, y2));
                }
                seg_start = None;
                seg_border = None;
            }
        }
        if let (Some(start), Some(ref sb)) = (seg_start, seg_border) {
            let y1 = table_y + row_y[start];
            let y2 = table_y + row_y.get(v_col.len()).copied().unwrap_or(row_y[start]);
            nodes.extend(create_border_line_nodes(tree, &sb, seg_x, y1, seg_x, y2));
        }
    }

    nodes
}

/// 투명 테두리를 빨간색 점선 Line 노드로 생성한다.
/// 엣지 그리드에서 None 슬롯(투명 테두리)을 찾아 연속 구간을 병합한다.
pub(crate) fn render_transparent_borders(
    tree: &mut PageRenderTree,
    h_edges: &[Vec<Option<BorderLine>>],
    v_edges: &[Vec<Option<BorderLine>>],
    row_col_x: &[Vec<f64>],
    row_y: &[f64],
    table_x: f64,
    table_y: f64,
) -> Vec<RenderNode> {
    let mut nodes = Vec::new();
    let color: u32 = 0x0000FF; // BGR: Red
    let width = 0.4_f64;
    let dash = StrokeDash::Dot;
    let row_count = if row_y.len() > 1 { row_y.len() - 1 } else { 0 };

    // 수평 투명 엣지
    for (ri, h_row) in h_edges.iter().enumerate() {
        let y = table_y + row_y.get(ri).copied().unwrap_or(0.0);
        let ref_row = ri.min(row_count.saturating_sub(1));
        let ref_cx = &row_col_x[ref_row.min(row_col_x.len() - 1)];
        let mut seg_start: Option<usize> = None;

        for (ci, edge_opt) in h_row.iter().enumerate() {
            if edge_opt.is_none() {
                if seg_start.is_none() {
                    seg_start = Some(ci);
                }
            } else if let Some(start) = seg_start {
                let x1 = table_x + ref_cx[start];
                let x2 = table_x + ref_cx[ci];
                nodes.extend(create_single_line(tree, color, width, dash, x1, y, x2, y));
                seg_start = None;
            }
        }
        if let Some(start) = seg_start {
            let x1 = table_x + ref_cx[start];
            let x2 = table_x + ref_cx.get(h_row.len()).copied().unwrap_or(ref_cx[start]);
            nodes.extend(create_single_line(tree, color, width, dash, x1, y, x2, y));
        }
    }

    // 수직 투명 엣지 (행별 x 위치)
    for (ci, v_col) in v_edges.iter().enumerate() {
        let mut seg_start: Option<usize> = None;
        let mut seg_x: f64 = 0.0;

        for (ri, edge_opt) in v_col.iter().enumerate() {
            let x = table_x
                + row_col_x
                    .get(ri)
                    .and_then(|rx| rx.get(ci).copied())
                    .unwrap_or(0.0);
            if edge_opt.is_none() {
                if seg_start.is_none() {
                    seg_start = Some(ri);
                    seg_x = x;
                } else if (x - seg_x).abs() >= 0.01 {
                    // x가 바뀌면 이전 세그먼트 마무리 후 새 세그먼트 시작
                    let y1 = table_y + row_y[seg_start.unwrap()];
                    let y2 = table_y + row_y[ri];
                    nodes.extend(create_single_line(
                        tree, color, width, dash, seg_x, y1, seg_x, y2,
                    ));
                    seg_start = Some(ri);
                    seg_x = x;
                }
            } else if let Some(start) = seg_start {
                let y1 = table_y + row_y[start];
                let y2 = table_y + row_y[ri];
                nodes.extend(create_single_line(
                    tree, color, width, dash, seg_x, y1, seg_x, y2,
                ));
                seg_start = None;
            }
        }
        if let Some(start) = seg_start {
            let y1 = table_y + row_y[start];
            let y2 = table_y + row_y.get(v_col.len()).copied().unwrap_or(row_y[start]);
            nodes.extend(create_single_line(
                tree, color, width, dash, seg_x, y1, seg_x, y2,
            ));
        }
    }

    nodes
}

/// 테두리선 Line 노드 생성 (이중선/삼중선 지원)
/// None 타입이면 빈 벡터 반환
pub(crate) fn create_border_line_nodes(
    tree: &mut PageRenderTree,
    border: &BorderLine,
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
) -> Vec<RenderNode> {
    if border.line_type == BorderLineType::None {
        return vec![];
    }

    let base_width = border_width_to_px(border.width);

    match border.line_type {
        BorderLineType::None => vec![],

        // 이중선 (동일 굵기)
        BorderLineType::Double => {
            let total = base_width.max(3.0);
            let sub_w = (total * 0.3).max(0.4);
            let gap = (total * 0.4).max(1.0);
            let offset = (gap + sub_w) / 2.0;
            create_parallel_lines(
                tree,
                border.color,
                x1,
                y1,
                x2,
                y2,
                &[(-offset, sub_w), (offset, sub_w)],
                StrokeDash::Solid,
            )
        }

        // 가는선-굵은선 이중선
        BorderLineType::ThinThickDouble => {
            let total = base_width.max(3.0);
            let thin_w = (total * 0.2).max(0.4);
            let thick_w = (total * 0.4).max(0.6);
            let gap = (total * 0.4).max(1.0);
            let thin_offset = -(gap + thin_w) / 2.0;
            let thick_offset = (gap + thick_w) / 2.0;
            create_parallel_lines(
                tree,
                border.color,
                x1,
                y1,
                x2,
                y2,
                &[(thin_offset, thin_w), (thick_offset, thick_w)],
                StrokeDash::Solid,
            )
        }

        // 굵은선-가는선 이중선
        BorderLineType::ThickThinDouble => {
            let total = base_width.max(3.0);
            let thick_w = (total * 0.4).max(0.6);
            let thin_w = (total * 0.2).max(0.4);
            let gap = (total * 0.4).max(1.0);
            let thick_offset = -(gap + thick_w) / 2.0;
            let thin_offset = (gap + thin_w) / 2.0;
            create_parallel_lines(
                tree,
                border.color,
                x1,
                y1,
                x2,
                y2,
                &[(thick_offset, thick_w), (thin_offset, thin_w)],
                StrokeDash::Solid,
            )
        }

        // 가는선-굵은선-가는선 삼중선
        BorderLineType::ThinThickThinTriple => {
            let total = base_width.max(4.0);
            let thin_w = (total * 0.15).max(0.4);
            let thick_w = (total * 0.3).max(0.6);
            let gap = (total * 0.15).max(0.8);
            let outer_offset = thick_w / 2.0 + gap + thin_w / 2.0;
            create_parallel_lines(
                tree,
                border.color,
                x1,
                y1,
                x2,
                y2,
                &[
                    (-outer_offset, thin_w),
                    (0.0, thick_w),
                    (outer_offset, thin_w),
                ],
                StrokeDash::Solid,
            )
        }

        // 단일선 타입들
        _ => {
            if let Some(dash) = border_line_type_to_dash(border.line_type) {
                create_single_line(tree, border.color, base_width, dash, x1, y1, x2, y2)
            } else {
                vec![]
            }
        }
    }
}

/// 평행선 노드 생성 (이중선/삼중선용)
/// lines: &[(offset, width)] — offset은 선 중심의 수직 이동량
fn create_parallel_lines(
    tree: &mut PageRenderTree,
    color: u32,
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
    lines: &[(f64, f64)],
    dash: StrokeDash,
) -> Vec<RenderNode> {
    let is_horizontal = (y2 - y1).abs() < (x2 - x1).abs();
    let mut nodes = Vec::with_capacity(lines.len());

    for &(offset, width) in lines {
        let (lx1, ly1, lx2, ly2) = if is_horizontal {
            (x1, y1 + offset, x2, y2 + offset)
        } else {
            (x1 + offset, y1, x2 + offset, y2)
        };

        let id = tree.next_id();
        nodes.push(RenderNode::new(
            id,
            RenderNodeType::Line(LineNode::new(
                lx1,
                ly1,
                lx2,
                ly2,
                LineStyle {
                    color,
                    width,
                    dash,
                    ..Default::default()
                },
            )),
            BoundingBox::new(
                lx1.min(lx2),
                ly1.min(ly2),
                (lx2 - lx1).abs().max(width),
                (ly2 - ly1).abs().max(width),
            ),
        ));
    }

    nodes
}

/// 임의 방향 평행선 노드 생성 (대각선 이중선/삼중선용)
fn create_parallel_lines_perpendicular(
    tree: &mut PageRenderTree,
    color: u32,
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
    lines: &[(f64, f64)],
    dash: StrokeDash,
) -> Vec<RenderNode> {
    let dx = x2 - x1;
    let dy = y2 - y1;
    let len = (dx * dx + dy * dy).sqrt();
    if len < 0.01 {
        return vec![];
    }
    let nx = -dy / len;
    let ny = dx / len;
    let mut nodes = Vec::with_capacity(lines.len());

    for &(offset, width) in lines {
        let lx1 = x1 + nx * offset;
        let ly1 = y1 + ny * offset;
        let lx2 = x2 + nx * offset;
        let ly2 = y2 + ny * offset;

        let id = tree.next_id();
        nodes.push(RenderNode::new(
            id,
            RenderNodeType::Line(LineNode::new(
                lx1,
                ly1,
                lx2,
                ly2,
                LineStyle {
                    color,
                    width,
                    dash,
                    ..Default::default()
                },
            )),
            BoundingBox::new(
                lx1.min(lx2),
                ly1.min(ly2),
                (lx2 - lx1).abs().max(width),
                (ly2 - ly1).abs().max(width),
            ),
        ));
    }

    nodes
}

/// 단일선 노드 생성
fn create_single_line(
    tree: &mut PageRenderTree,
    color: u32,
    width: f64,
    dash: StrokeDash,
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
) -> Vec<RenderNode> {
    let id = tree.next_id();
    vec![RenderNode::new(
        id,
        RenderNodeType::Line(LineNode::new(
            x1,
            y1,
            x2,
            y2,
            LineStyle {
                color,
                width,
                dash,
                ..Default::default()
            },
        )),
        BoundingBox::new(
            x1.min(x2),
            y1.min(y2),
            (x2 - x1).abs().max(width),
            (y2 - y1).abs().max(width),
        ),
    )]
}

fn border_line_type_from_code(code: u8) -> BorderLineType {
    match code {
        0 => BorderLineType::None,
        1 => BorderLineType::Solid,
        2 => BorderLineType::Dash,
        3 => BorderLineType::Dot,
        4 => BorderLineType::DashDot,
        5 => BorderLineType::DashDotDot,
        6 => BorderLineType::LongDash,
        7 => BorderLineType::Circle,
        8 => BorderLineType::Double,
        9 => BorderLineType::ThinThickDouble,
        10 => BorderLineType::ThickThinDouble,
        11 => BorderLineType::ThinThickThinTriple,
        12 => BorderLineType::Wave,
        13 => BorderLineType::DoubleWave,
        14 => BorderLineType::Thick3D,
        15 => BorderLineType::Thick3DReverse,
        16 => BorderLineType::Thin3D,
        17 => BorderLineType::Thin3DReverse,
        _ => BorderLineType::Solid,
    }
}

fn create_diagonal_line_nodes(
    tree: &mut PageRenderTree,
    line_type: BorderLineType,
    color: u32,
    width_index: u8,
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
) -> Vec<RenderNode> {
    if line_type == BorderLineType::None {
        return vec![];
    }

    let base_width = border_width_to_px(width_index);
    match line_type {
        BorderLineType::None => vec![],
        BorderLineType::Double => {
            let total = base_width.max(3.0);
            let sub_w = (total * 0.3).max(0.4);
            let gap = (total * 0.4).max(1.0);
            let offset = (gap + sub_w) / 2.0;
            create_parallel_lines_perpendicular(
                tree,
                color,
                x1,
                y1,
                x2,
                y2,
                &[(-offset, sub_w), (offset, sub_w)],
                StrokeDash::Solid,
            )
        }
        BorderLineType::ThinThickDouble => {
            let total = base_width.max(3.0);
            let thin_w = (total * 0.2).max(0.4);
            let thick_w = (total * 0.4).max(0.6);
            let gap = (total * 0.4).max(1.0);
            let thin_offset = -(gap + thin_w) / 2.0;
            let thick_offset = (gap + thick_w) / 2.0;
            create_parallel_lines_perpendicular(
                tree,
                color,
                x1,
                y1,
                x2,
                y2,
                &[(thin_offset, thin_w), (thick_offset, thick_w)],
                StrokeDash::Solid,
            )
        }
        BorderLineType::ThickThinDouble => {
            let total = base_width.max(3.0);
            let thick_w = (total * 0.4).max(0.6);
            let thin_w = (total * 0.2).max(0.4);
            let gap = (total * 0.4).max(1.0);
            let thick_offset = -(gap + thick_w) / 2.0;
            let thin_offset = (gap + thin_w) / 2.0;
            create_parallel_lines_perpendicular(
                tree,
                color,
                x1,
                y1,
                x2,
                y2,
                &[(thick_offset, thick_w), (thin_offset, thin_w)],
                StrokeDash::Solid,
            )
        }
        BorderLineType::ThinThickThinTriple => {
            let total = base_width.max(4.0);
            let thin_w = (total * 0.15).max(0.4);
            let thick_w = (total * 0.3).max(0.6);
            let gap = (total * 0.15).max(0.8);
            let outer_offset = thick_w / 2.0 + gap + thin_w / 2.0;
            create_parallel_lines_perpendicular(
                tree,
                color,
                x1,
                y1,
                x2,
                y2,
                &[
                    (-outer_offset, thin_w),
                    (0.0, thick_w),
                    (outer_offset, thin_w),
                ],
                StrokeDash::Solid,
            )
        }
        _ => {
            if let Some(dash) = border_line_type_to_dash(line_type) {
                create_single_line(tree, color, base_width, dash, x1, y1, x2, y2)
            } else {
                vec![]
            }
        }
    }
}

fn create_crooked_diagonal_line_nodes(
    tree: &mut PageRenderTree,
    line_type: BorderLineType,
    color: u32,
    width_index: u8,
    points: &[(f64, f64)],
) -> Vec<RenderNode> {
    let mut nodes = Vec::new();
    for pair in points.windows(2) {
        let (x1, y1) = pair[0];
        let (x2, y2) = pair[1];
        nodes.extend(create_diagonal_line_nodes(
            tree,
            line_type,
            color,
            width_index,
            x1,
            y1,
            x2,
            y2,
        ));
    }
    nodes
}

/// BorderLine이 시각적으로 차지하는 전체 폭(px).
///
/// `create_border_line_nodes`의 이중선/삼중선 분해 규칙과 같은 값을 써서,
/// 쪽 기준 테두리 박스를 바깥쪽으로 확장할 때 렌더된 선 묶음이 본문 쪽으로
/// 파고들지 않게 한다.
pub(crate) fn border_line_visual_span(border: &BorderLine) -> f64 {
    if border.line_type == BorderLineType::None {
        return 0.0;
    }

    let base_width = border_width_to_px(border.width);
    match border.line_type {
        BorderLineType::Double
        | BorderLineType::ThinThickDouble
        | BorderLineType::ThickThinDouble => base_width.max(3.0),
        BorderLineType::ThinThickThinTriple => base_width.max(4.0),
        _ => base_width,
    }
}

/// 쪽 기준 페이지 테두리를 본문 영역 바깥쪽에 배치할 때 쓰는 보정 폭(px).
///
/// 한컴오피스는 `쪽 기준` 이중선 페이지 테두리에서 저장된 간격값에 선 묶음의
/// 시각 폭을 한 번 더 반영해, 테두리가 본문/객체 쪽으로 파고들지 않게 그린다.
/// 표/문단 테두리의 선 자체 분해 규칙은 그대로 두고, 페이지 테두리 위치 계산에만
/// 이 값을 사용한다.
pub(crate) fn body_page_border_outset(border: &BorderLine) -> f64 {
    const BODY_PAGE_DOUBLE_LINE_OUTSET_FACTOR: f64 = 2.5;
    let span = border_line_visual_span(border);
    match border.line_type {
        BorderLineType::Double
        | BorderLineType::ThinThickDouble
        | BorderLineType::ThickThinDouble
        | BorderLineType::ThinThickThinTriple => span * BODY_PAGE_DOUBLE_LINE_OUTSET_FACTOR,
        _ => span,
    }
}

/// HWP 테두리 굵기 인덱스 → 픽셀 변환
/// HWP 스펙 (표 28): mm 값을 96dpi 기준 px로 변환
pub(crate) fn border_width_to_px(width: u8) -> f64 {
    const WIDTHS_PX: [f64; 16] = [
        0.4,  // 0: 0.1mm
        0.5,  // 1: 0.12mm
        0.6,  // 2: 0.15mm
        0.75, // 3: 0.2mm
        1.0,  // 4: 0.25mm
        1.1,  // 5: 0.3mm
        1.5,  // 6: 0.4mm
        1.9,  // 7: 0.5mm
        2.3,  // 8: 0.6mm
        2.6,  // 9: 0.7mm
        3.8,  // 10: 1.0mm
        5.7,  // 11: 1.5mm
        7.6,  // 12: 2.0mm
        11.3, // 13: 3.0mm
        15.1, // 14: 4.0mm
        18.9, // 15: 5.0mm
    ];
    if (width as usize) < WIDTHS_PX.len() {
        WIDTHS_PX[width as usize]
    } else {
        (width as f64 * 1.2).max(0.4).min(20.0)
    }
}

/// BorderLineType → StrokeDash 변환 (None이면 None 반환)
fn border_line_type_to_dash(lt: BorderLineType) -> Option<StrokeDash> {
    match lt {
        BorderLineType::None => None,
        BorderLineType::Solid => Some(StrokeDash::Solid),
        BorderLineType::Dash | BorderLineType::LongDash => Some(StrokeDash::Dash),
        BorderLineType::Dot | BorderLineType::Circle => Some(StrokeDash::Dot),
        BorderLineType::DashDot => Some(StrokeDash::DashDot),
        BorderLineType::DashDotDot => Some(StrokeDash::DashDotDot),
        _ => Some(StrokeDash::Solid), // Double, Wave 등은 Solid로 대체
    }
}

/// 셀 대각선 렌더링
/// HWP BorderFill.attr 비트:
///   bit 2~4: Slash(`/`) 대각선 모양
///     000=none, 그 외=slash
///   bit 5~7: BackSlash(`\`) 대각선 모양
///     000=none, 그 외=backslash
///   bit 8~9: Slash 대각선 꺾은선
///   bit 10: BackSlash 대각선 꺾은선
///   bit 13: 중심선
pub(crate) fn render_cell_diagonal(
    tree: &mut PageRenderTree,
    border_style: &ResolvedBorderStyle,
    cell_x: f64,
    cell_y: f64,
    cell_w: f64,
    cell_h: f64,
) -> Vec<RenderNode> {
    let attr = border_style.diagonal_attr;
    let slash_bits = (attr >> 2) & 0x07;
    let backslash_bits = (attr >> 5) & 0x07;
    let slash_crooked = (attr >> 8) & 0x03;
    let backslash_crooked = (attr >> 10) & 0x01;
    let center_line = border_style.center_line;

    if slash_bits == 0 && backslash_bits == 0 && center_line == CenterLine::None {
        return vec![];
    }

    let diag = &border_style.diagonal;
    // diagonal_type 0 = 선 종류 없음 → 대각선 그리지 않음
    if diag.diagonal_type == 0 {
        return vec![];
    }
    let color = diag.color;
    let line_type = border_line_type_from_code(diag.diagonal_type);

    let mut nodes = Vec::new();

    let x1 = cell_x;
    let y1 = cell_y;
    let x2 = cell_x + cell_w;
    let y2 = cell_y + cell_h;
    let cx = cell_x + cell_w / 2.0;
    let cy = cell_y + cell_h / 2.0;

    match center_line {
        CenterLine::Vertical => {
            nodes.extend(create_diagonal_line_nodes(
                tree, line_type, color, diag.width, x1, cy, x2, cy,
            ));
        }
        CenterLine::Horizontal => {
            nodes.extend(create_diagonal_line_nodes(
                tree, line_type, color, diag.width, cx, y1, cx, y2,
            ));
        }
        CenterLine::Cross => {
            nodes.extend(create_diagonal_line_nodes(
                tree, line_type, color, diag.width, cx, y1, cx, y2,
            ));
            nodes.extend(create_diagonal_line_nodes(
                tree, line_type, color, diag.width, x1, cy, x2, cy,
            ));
        }
        CenterLine::None => {}
    }

    if slash_bits != 0 {
        if slash_crooked != 0 {
            let p1 = (x1, y2);
            let p2 = (cell_x + cell_w * 0.4, cy);
            let p3 = (cell_x + cell_w * 0.6, cy);
            let p4 = (x2, y1);
            nodes.extend(create_crooked_diagonal_line_nodes(
                tree,
                line_type,
                color,
                diag.width,
                &[p1, p2, p3, p4],
            ));
        } else {
            nodes.extend(create_diagonal_line_nodes(
                tree, line_type, color, diag.width, x1, y2, x2, y1,
            ));
        }
    }

    if backslash_bits != 0 {
        let use_crooked = backslash_crooked != 0 || (slash_bits == 0 && slash_crooked != 0);
        if use_crooked {
            let p1 = (x1, y1);
            let p2 = (cell_x + cell_w * 0.4, cy);
            let p3 = (cell_x + cell_w * 0.6, cy);
            let p4 = (x2, y2);
            nodes.extend(create_crooked_diagonal_line_nodes(
                tree,
                line_type,
                color,
                diag.width,
                &[p1, p2, p3, p4],
            ));
        } else {
            nodes.extend(create_diagonal_line_nodes(
                tree, line_type, color, diag.width, x1, y1, x2, y2,
            ));
        }
    }

    nodes
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::style::DiagonalLine;

    fn center_line_style(center_line: CenterLine) -> ResolvedBorderStyle {
        ResolvedBorderStyle {
            diagonal_attr: if center_line == CenterLine::None {
                0
            } else {
                1 << 13
            },
            diagonal: DiagonalLine {
                diagonal_type: 1,
                width: 0,
                color: 0x00F4_C741,
            },
            center_line,
            ..Default::default()
        }
    }

    fn diagonal_style(attr: u16) -> ResolvedBorderStyle {
        ResolvedBorderStyle {
            diagonal_attr: attr,
            diagonal: DiagonalLine {
                diagonal_type: 1,
                width: 0,
                color: 0,
            },
            ..Default::default()
        }
    }

    fn line_node(node: &RenderNode) -> &LineNode {
        match &node.node_type {
            RenderNodeType::Line(line) => line,
            other => panic!("Line 노드가 아님: {other:?}"),
        }
    }

    #[test]
    fn render_hwpx_vertical_center_line_as_horizontal_bar() {
        let mut tree = PageRenderTree::new(0, 200.0, 100.0);
        let nodes = render_cell_diagonal(
            &mut tree,
            &center_line_style(CenterLine::Vertical),
            10.0,
            20.0,
            100.0,
            40.0,
        );

        assert_eq!(nodes.len(), 1);
        let line = line_node(&nodes[0]);
        assert_eq!(
            (line.x1, line.y1, line.x2, line.y2),
            (10.0, 40.0, 110.0, 40.0)
        );
        assert_eq!(line.style.color, 0x00F4_C741);
    }

    #[test]
    fn render_hwpx_horizontal_center_line_as_vertical_bar() {
        let mut tree = PageRenderTree::new(0, 200.0, 100.0);
        let nodes = render_cell_diagonal(
            &mut tree,
            &center_line_style(CenterLine::Horizontal),
            10.0,
            20.0,
            100.0,
            40.0,
        );

        assert_eq!(nodes.len(), 1);
        let line = line_node(&nodes[0]);
        assert_eq!(
            (line.x1, line.y1, line.x2, line.y2),
            (60.0, 20.0, 60.0, 60.0)
        );
    }

    #[test]
    fn render_cross_center_line_creates_vertical_and_horizontal_lines() {
        let mut tree = PageRenderTree::new(0, 200.0, 100.0);
        let nodes = render_cell_diagonal(
            &mut tree,
            &center_line_style(CenterLine::Cross),
            10.0,
            20.0,
            100.0,
            40.0,
        );

        assert_eq!(nodes.len(), 2);
        let vertical = line_node(&nodes[0]);
        let horizontal = line_node(&nodes[1]);
        assert_eq!(
            (vertical.x1, vertical.y1, vertical.x2, vertical.y2),
            (60.0, 20.0, 60.0, 60.0)
        );
        assert_eq!(
            (horizontal.x1, horizontal.y1, horizontal.x2, horizontal.y2),
            (10.0, 40.0, 110.0, 40.0)
        );
    }

    #[test]
    fn render_nonzero_diagonal_shape_codes_as_basic_x() {
        let mut tree = PageRenderTree::new(0, 200.0, 100.0);
        let nodes = render_cell_diagonal(
            &mut tree,
            &diagonal_style((0b111 << 2) | (0b111 << 5)),
            10.0,
            20.0,
            100.0,
            40.0,
        );

        assert_eq!(nodes.len(), 2);
        let slash = line_node(&nodes[0]);
        let backslash = line_node(&nodes[1]);
        assert_eq!(
            (slash.x1, slash.y1, slash.x2, slash.y2),
            (10.0, 60.0, 110.0, 20.0)
        );
        assert_eq!(
            (backslash.x1, backslash.y1, backslash.x2, backslash.y2),
            (10.0, 20.0, 110.0, 60.0)
        );
    }

    #[test]
    fn render_slash_crooked_with_backslash_as_bent_backslash() {
        let mut tree = PageRenderTree::new(0, 200.0, 100.0);
        let nodes = render_cell_diagonal(
            &mut tree,
            &diagonal_style((2 << 8) | (0b010 << 5)),
            10.0,
            20.0,
            100.0,
            40.0,
        );

        assert_eq!(nodes.len(), 3);
        let first = line_node(&nodes[0]);
        let middle = line_node(&nodes[1]);
        let last = line_node(&nodes[2]);
        assert_eq!(
            (first.x1, first.y1, first.x2, first.y2),
            (10.0, 20.0, 50.0, 40.0)
        );
        assert_eq!(
            (middle.x1, middle.y1, middle.x2, middle.y2),
            (50.0, 40.0, 70.0, 40.0)
        );
        assert_eq!(
            (last.x1, last.y1, last.x2, last.y2),
            (70.0, 40.0, 110.0, 60.0)
        );
    }

    #[test]
    fn render_thick_slim_diagonal_as_parallel_lines() {
        let mut tree = PageRenderTree::new(0, 200.0, 100.0);
        let mut style = diagonal_style(0b010 << 2);
        style.diagonal.diagonal_type = 10;
        style.diagonal.width = 13;
        let nodes = render_cell_diagonal(&mut tree, &style, 10.0, 20.0, 100.0, 40.0);

        assert_eq!(nodes.len(), 2);
        let thick = line_node(&nodes[0]);
        let thin = line_node(&nodes[1]);
        assert!(thick.style.width > thin.style.width);
        assert_ne!((thick.x1, thick.y1), (thin.x1, thin.y1));
        assert_ne!((thick.x2, thick.y2), (thin.x2, thin.y2));
    }
}
