//! 페이지 분할 표 레이아웃 (layout_partial_table)

use super::super::composer::compose_paragraph;
use super::super::height_measurer::MeasuredTable;
use super::super::page_layout::LayoutRect;
use super::super::render_tree::*;
use super::super::style_resolver::ResolvedStyleSet;
use super::super::{hwpunit_to_px, ShapeStyle};
use super::border_rendering::{
    build_row_col_x, collect_cell_borders, render_edge_borders, render_transparent_borders,
};
use super::table_layout::{calc_nested_split_rows, NestedTableSplit};
use super::text_measurement::{estimate_text_width, resolved_to_text_style};
use super::utils::find_bin_data;
use super::{CellContext, CellPathEntry, LayoutEngine};
use crate::model::bin_data::BinDataContent;
use crate::model::control::Control;
use crate::model::paragraph::Paragraph;
use crate::model::shape::CaptionDirection;
use crate::model::style::{Alignment, BorderLine};

// 표 수평 정렬 보조 타입은 table_layout.rs에 통합됨

/// [Task #1025] `row` 를 포함하는 rowspan 블록 범위 `[b_start, b_end)`.
/// rs>1 셀이 겹치는 행을 전이적으로 확장한다(겹침 없으면 `[row, row+1)`).
/// 페이지네이터 `mt.row_block_for` / `advance_row_block_cut` 와 동일한 블록 정의.
fn rowspan_block_range(table: &crate::model::table::Table, row: usize) -> (usize, usize) {
    let mut b_start = row;
    let mut b_end = row + 1;
    loop {
        let mut changed = false;
        for c in &table.cells {
            if c.row_span <= 1 {
                continue;
            }
            let cs = c.row as usize;
            let ce = cs + c.row_span as usize;
            if cs < b_end && ce > b_start {
                if cs < b_start {
                    b_start = cs;
                    changed = true;
                }
                if ce > b_end {
                    b_end = ce;
                    changed = true;
                }
            }
        }
        if !changed {
            break;
        }
    }
    (b_start, b_end)
}

/// [Task #1025] 블록 `[b_start, b_end)` 컷 벡터에서 `cell` 의 인덱스.
/// `advance_row_block_cut` 과 동일한 `(row, col)` 안정 순서. 없으면 None.
fn block_cut_index(
    table: &crate::model::table::Table,
    b_start: usize,
    b_end: usize,
    cell: &crate::model::table::Cell,
) -> Option<usize> {
    let mut cells: Vec<&crate::model::table::Cell> = table
        .cells
        .iter()
        .filter(|c| {
            let cr = c.row as usize;
            let ce = cr + (c.row_span as usize).max(1);
            cr < b_end && ce > b_start
        })
        .collect();
    cells.sort_by_key(|c| (c.row, c.col));
    cells
        .iter()
        .position(|c| c.row == cell.row && c.col == cell.col)
}

impl LayoutEngine {
    /// 표의 일부 행만 레이아웃한다 (페이지 분할).
    ///
    /// `start_row..end_row` 범위의 행만 렌더링한다.
    /// `is_continuation`이 true이고 repeat_header인 표면 행0(제목행)을 먼저 렌더링한다.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn layout_partial_table(
        &self,
        tree: &mut PageRenderTree,
        col_node: &mut RenderNode,
        paragraphs: &[Paragraph],
        para_index: usize,
        control_index: usize,
        section_index: usize,
        styles: &ResolvedStyleSet,
        outline_numbering_id: u16,
        col_area: &LayoutRect,
        y_start: f64,
        bin_data_content: &[BinDataContent],
        start_row: usize,
        end_row: usize,
        is_continuation: bool,
        start_cut: &[usize],
        end_cut: &[usize],
        is_block_split: bool,
        host_margin_left: f64,
        host_margin_right: f64,
        measured_table: Option<&MeasuredTable>,
        clamp_header_negative_para_offset: bool,
    ) -> f64 {
        let para = match paragraphs.get(para_index) {
            Some(p) => p,
            None => return y_start,
        };
        let table = match para.controls.get(control_index) {
            Some(Control::Table(t)) => t,
            _ => return y_start,
        };

        if table.cells.is_empty() {
            return y_start;
        }

        // 분할 표 첫 부분: vert_offset 적용 (자리차지 표의 세로 오프셋).
        // [Task #712] HwpUnit=u32 이라 `vertical_offset > 0` 는 음수 비트표현
        // (예: -1796 HU = 0xFFFFF8FC = 4294965500u32) 도 양수로 통과시켜
        // 후속 `as i32` 캐스트에서 음수가 적용 → 표가 위로 점프, 직전 인라인
        // 표 영역 침범. 비-Partial 경로(`table_layout.rs:1069+`)는 동일 분기에
        // `raw_y.max(y_start)` 클램프가 있어 음수 무력화. Partial 경로에는
        // 클램프가 없으므로 게이트를 signed 비교로 정정해 동등 효과.
        let vert_off_signed = table.common.vertical_offset as i32;
        let y_start = if !is_continuation
            && !table.common.treat_as_char
            && matches!(
                table.common.text_wrap,
                crate::model::shape::TextWrap::TopAndBottom
            )
            && matches!(
                table.common.vert_rel_to,
                crate::model::shape::VertRelTo::Para
            )
            && vert_off_signed > 0
        {
            y_start + hwpunit_to_px(vert_off_signed, self.dpi)
        } else {
            y_start
        };

        let col_count = table.col_count as usize;
        let row_count = table.row_count as usize;
        let cell_spacing = hwpunit_to_px(table.cell_spacing as i32, self.dpi);

        // ── 1. 열 폭 계산 + 2. 행 높이 계산 (table_layout 공유 메서드) ──
        let col_widths = self.resolve_column_widths(table, col_count);
        let mut row_heights =
            self.resolve_row_heights(table, col_count, row_count, measured_table, styles);

        // ── 2b. 행 높이 오버라이드 (Task #993: 컷 기반) ──
        // 렌더 대상 모든 행의 높이를 페이지네이터와 동일한 컷 측정
        // (row_cut_content_height)으로 정정한다. 페이지네이터(typeset)와 렌더러가
        // 단일 측정 공간(advance_row_cut/cell_units)을 공유해야 분할 표가
        // 페이지를 넘지 않는다. 분할 행은 start_cut/end_cut 범위, 그 외 행은
        // 전체 콘텐츠. rowspan 연속 행(컷 0)은 resolve_row_heights 결과 유지.
        {
            let split_last_row = end_row.saturating_sub(1);
            let mut rows_to_set: std::collections::BTreeSet<usize> = (start_row..end_row).collect();
            // 연속분 머리행 반복 — start_row 이전의 is_header 행도 렌더된다.
            if is_continuation && table.repeat_header && start_row > 0 {
                for c in &table.cells {
                    if c.is_header && (c.row as usize) < start_row {
                        rows_to_set.insert(c.row as usize);
                    }
                }
            }
            // [Task #1025] page-larger 블록 분할(is_block_split)이면 컷이 rowspan
            // 블록-셀 인덱스 → 블록 범위(rowspan-확장)로 per-row 컷 매핑. 그 외(일반
            // 분할)는 기존 per-row(row_span==1) 경로 유지(rowspan 행은 atomic).
            let start_block = if is_block_split && !start_cut.is_empty() {
                Some(rowspan_block_range(table, start_row))
            } else {
                None
            };
            let end_block = if is_block_split && !end_cut.is_empty() {
                Some(rowspan_block_range(table, split_last_row))
            } else {
                None
            };
            for r in rows_to_set {
                if r >= row_count {
                    continue;
                }
                let rowspan_touched = table.cells.iter().any(|c| {
                    c.row_span > 1
                        && (c.row as usize) <= r
                        && r < c.row as usize + c.row_span as usize
                });
                if is_block_split {
                    let in_start = start_block.is_some_and(|(s, e)| s <= r && r < e);
                    let in_end = end_block.is_some_and(|(s, e)| s <= r && r < e);
                    // 분할 블록 밖 rowspan 행은 컷 모델 밖 — resolve_row_heights 유지.
                    if rowspan_touched && !in_start && !in_end {
                        continue;
                    }
                    // 행 r 의 row_span==1 셀(col 순)별 블록 컷 → per-row 컷 매핑.
                    let mut rcells: Vec<&crate::model::table::Cell> = table
                        .cells
                        .iter()
                        .filter(|c| c.row as usize == r && c.row_span == 1)
                        .collect();
                    rcells.sort_by_key(|c| c.col);
                    let mut per_start: Vec<usize> = Vec::with_capacity(rcells.len());
                    let mut per_end: Vec<usize> = Vec::with_capacity(rcells.len());
                    let mut has_visible_range = false;
                    let mut has_row_cut = false;
                    for c in &rcells {
                        let units = self.cell_units(c, table, styles);
                        let su = match (in_start, start_block) {
                            (true, Some((bs, be))) => block_cut_index(table, bs, be, c)
                                .and_then(|i| start_cut.get(i).copied())
                                .unwrap_or(0),
                            _ => 0,
                        }
                        .min(units.len());
                        let eu = match (in_end, end_block) {
                            (true, Some((bs, be))) => block_cut_index(table, bs, be, c)
                                .and_then(|i| end_cut.get(i).copied())
                                .unwrap_or(units.len()),
                            _ => units.len(),
                        }
                        .clamp(su, units.len());
                        if eu > su {
                            has_visible_range = true;
                        }
                        if su > 0 || eu < units.len() {
                            has_row_cut = true;
                        }
                        per_start.push(su);
                        per_end.push(eu);
                    }
                    let h = if !has_visible_range {
                        0.0
                    } else if has_row_cut {
                        self.row_cut_content_height(table, r, &per_start, &per_end, styles)
                    } else {
                        self.row_cut_content_height(table, r, &[], &[], styles)
                    };
                    if h > 0.0 {
                        row_heights[r] = h;
                    }
                } else {
                    let su: &[usize] = if r == start_row { start_cut } else { &[] };
                    let eu: &[usize] = if r == split_last_row { end_cut } else { &[] };
                    // 기존 per-row 경로에서 rowspan 행은 기본적으로 atomic
                    // (resolve_row_heights) 유지. 단 RowBreak 의 큰 rowspan 블록 내부
                    // 행을 typeset 이 per-row cut 으로 분할한 split boundary 에서는
                    // 렌더러도 같은 cut 높이를 적용해야 한다.
                    if rowspan_touched && su.is_empty() && eu.is_empty() {
                        continue;
                    }
                    let h = self.row_cut_content_height(table, r, su, eu, styles);
                    if h > 0.0 {
                        row_heights[r] = h;
                    }
                }
            }
        }

        // ── 3. 누적 위치 계산 ──
        let mut col_x = vec![0.0f64; col_count + 1];
        for i in 0..col_count {
            col_x[i + 1] =
                col_x[i] + col_widths[i] + if i + 1 < col_count { cell_spacing } else { 0.0 };
        }

        // 행별 열 위치 계산 (셀별 독립 너비 지원)
        let row_col_x = build_row_col_x(
            table,
            &col_widths,
            col_count,
            row_count,
            cell_spacing,
            self.dpi,
        );

        let table_width = row_col_x
            .iter()
            .map(|rx| rx.last().copied().unwrap_or(0.0))
            .fold(col_x.last().copied().unwrap_or(0.0), f64::max);

        // ── 표 수평 위치 (table_layout 공유 메서드) ──
        let pw = self.current_paper_width.get();
        let paper_w = if pw > 0.0 { Some(pw) } else { None };
        let table_x = self.compute_table_x_position(
            table,
            table_width,
            col_area,
            0,
            Alignment::Left,
            host_margin_left,
            host_margin_right,
            None,
            paper_w,
        );

        // ── 4. 렌더링할 행 목록 구성 ──
        // is_continuation && repeat_header → start_row 이전의 is_header 행만 반복
        let mut header_rows: Vec<usize> = Vec::new();
        if is_continuation && table.repeat_header && start_row > 0 {
            let mut seen = vec![false; row_count];
            for c in &table.cells {
                let r = c.row as usize;
                if c.is_header && r < start_row && r < row_count && !seen[r] {
                    seen[r] = true;
                    header_rows.push(r);
                }
            }
            header_rows.sort_unstable();
        }
        let mut render_rows: Vec<usize> = Vec::new();
        render_rows.extend_from_slice(&header_rows);
        for r in start_row..end_row.min(row_count) {
            render_rows.push(r);
        }

        // 렌더링 영역의 행별 y 위치 계산 (0부터 시작)
        let mut render_row_y: Vec<f64> = Vec::new(); // 각 render_rows 항목의 시작 y
        let mut y_accum = 0.0;
        for (i, &r) in render_rows.iter().enumerate() {
            render_row_y.push(y_accum);
            y_accum += row_heights[r]
                + if i + 1 < render_rows.len() {
                    cell_spacing
                } else {
                    0.0
                };
        }
        let partial_table_height = y_accum;

        // 엣지 기반 테두리 수집을 위한 그리드 (렌더링 행 기준)
        let render_row_count = render_rows.len();
        let mut h_edges: Vec<Vec<Option<BorderLine>>> =
            vec![vec![None; col_count]; render_row_count + 1];
        let mut v_edges: Vec<Vec<Option<BorderLine>>> =
            vec![vec![None; render_row_count]; col_count + 1];
        let mut grid_row_y = render_row_y.clone();
        grid_row_y.push(partial_table_height);

        // ── 4b. 캡션 처리 (첫 번째 파트에서만 렌더링) ──
        let is_first_part = start_row == 0 && !is_continuation && start_cut.is_empty();
        let is_last_part = end_row >= row_count && end_cut.is_empty();
        let (caption_height, caption_spacing) = if is_first_part || is_last_part {
            let ch = self.calculate_caption_height(&table.caption, styles);
            let cs = table
                .caption
                .as_ref()
                .map(|c| hwpunit_to_px(c.spacing as i32, self.dpi))
                .unwrap_or(0.0);
            (ch, cs)
        } else {
            (0.0, 0.0)
        };

        let cap_dir = table.caption.as_ref().map(|c| c.direction);
        let is_left_cap = cap_dir == Some(CaptionDirection::Left);
        let is_right_cap = cap_dir == Some(CaptionDirection::Right);
        let is_lr_cap = is_left_cap || is_right_cap;
        let render_top_caption = is_first_part && cap_dir == Some(CaptionDirection::Top);
        let render_bottom_caption = is_last_part && cap_dir == Some(CaptionDirection::Bottom);
        // Left/Right 캡션은 모든 파트에서 렌더링 (표 옆에 배치)
        let render_lr_caption = is_lr_cap;

        // Left 캡션: 표를 오른쪽으로 이동
        let cap_width_px = table
            .caption
            .as_ref()
            .map(|c| hwpunit_to_px(c.width as i32, self.dpi))
            .unwrap_or(0.0);
        let table_x = if is_left_cap {
            table_x + cap_width_px + caption_spacing
        } else {
            table_x
        };

        let table_y = if render_top_caption {
            y_start + caption_height + caption_spacing
        } else {
            y_start
        };

        // ── 5. 표 노드 생성 ──
        let table_id = tree.next_id();
        let mut table_node = RenderNode::new(
            table_id,
            RenderNodeType::Table(TableNode {
                row_count: table.row_count,
                col_count: table.col_count,
                border_fill_id: table.border_fill_id,
                section_index: Some(section_index),
                para_index: Some(para_index),
                control_index: Some(control_index),
            }),
            BoundingBox::new(table_x, table_y, table_width, partial_table_height),
        );

        // ── 5-1. 표 배경 렌더링 (표 > 배경 > 색 > 면색) ──
        if table.border_fill_id > 0 {
            let tbl_idx = (table.border_fill_id as usize).saturating_sub(1);
            if let Some(tbl_bs) = styles.border_styles.get(tbl_idx) {
                self.render_cell_background(
                    tree,
                    &mut table_node,
                    Some(tbl_bs),
                    table_x,
                    table_y,
                    table_width,
                    partial_table_height,
                    bin_data_content,
                );
            }
        }

        // ── 6. 셀 렌더링 (render_rows 범위 내 셀만) ──
        for (cell_idx, cell) in table.cells.iter().enumerate() {
            let cell_row = cell.row as usize;
            let cell_col = cell.col as usize;
            if cell_col >= col_count || cell_row >= row_count {
                continue;
            }

            // 이 셀이 렌더링 범위에 포함되는지 확인
            let cell_end_row = cell_row + cell.row_span as usize;
            let render_range_start = if !header_rows.is_empty() {
                *header_rows.first().unwrap()
            } else {
                start_row
            };
            let render_range_end = end_row.min(row_count);

            // 제목행 반복으로 렌더링되는 셀인지 판별
            let is_repeated_header_cell = !header_rows.is_empty()
                && header_rows.contains(&cell_row)
                && cell_end_row <= start_row;

            // 셀이 렌더링 범위와 겹치는지 확인
            if cell_row >= render_range_end || cell_end_row <= render_range_start {
                if !is_repeated_header_cell {
                    continue;
                }
            }

            // render_rows에서 이 셀의 시작 행 위치 찾기
            // row_span이 페이지 경계를 넘는 셀: cell_row가 render_rows에 없을 수 있음
            // 이 경우 셀 span 범위 내에서 render_rows에 포함된 첫 번째 행을 찾음
            let render_idx = render_rows.iter().position(|&r| r == cell_row).or_else(|| {
                render_rows
                    .iter()
                    .position(|&r| r > cell_row && r < cell_end_row)
            });
            let render_y_offset = match render_idx {
                Some(idx) => render_row_y[idx],
                None => continue, // 렌더링 범위에 없음
            };

            let rcx = &row_col_x[cell_row.min(row_count - 1)];
            let cell_x = table_x + rcx[cell_col];
            let cell_y = table_y + render_y_offset;

            // 병합 셀 크기
            let end_col = (cell_col + cell.col_span as usize).min(col_count);
            let cell_w = rcx[end_col] - rcx[cell_col];

            // 행 높이: 병합 셀의 경우 렌더링 범위 내의 행만 합산
            let mut cell_h = 0.0;
            let mut span_count = 0;
            for rs in 0..cell.row_span as usize {
                let target_r = cell_row + rs;
                if let Some(ri) = render_rows.iter().position(|&r| r == target_r) {
                    cell_h += row_heights[target_r];
                    if span_count > 0 {
                        cell_h += cell_spacing;
                    }
                    span_count += 1;
                    let _ = ri;
                }
            }
            if cell_h <= 0.0 {
                continue;
            }

            // 이 셀이 분할 행에 속하는지 판별 (clip 플래그에 사용)
            // [Task #1025] page-larger 블록 분할이면 컷이 블록-셀 인덱스 → 블록 범위
            // (rowspan-확장)와 셀 교차로 판정. 그 외는 기존 per-row 판정.
            let split_start_block = if is_block_split && !start_cut.is_empty() {
                Some(rowspan_block_range(table, start_row))
            } else {
                None
            };
            let split_end_block = if is_block_split && !end_cut.is_empty() {
                Some(rowspan_block_range(table, end_row.saturating_sub(1)))
            } else {
                None
            };
            let is_split_start_row = if is_block_split {
                split_start_block.is_some_and(|(s, e)| cell_row < e && cell_end_row > s)
            } else {
                !start_cut.is_empty() && cell_row == start_row
            };
            let is_split_end_row = if is_block_split {
                split_end_block.is_some_and(|(s, e)| cell_row < e && cell_end_row > s)
            } else {
                !end_cut.is_empty() && cell_row == end_row.saturating_sub(1)
            };
            let is_in_split_row = is_split_start_row || is_split_end_row;

            let cell_id = tree.next_id();
            let mut cell_node = RenderNode::new(
                cell_id,
                RenderNodeType::TableCell(TableCellNode {
                    col: cell.col,
                    row: cell.row,
                    col_span: cell.col_span,
                    row_span: cell.row_span,
                    border_fill_id: cell.border_fill_id,
                    text_direction: cell.text_direction,
                    clip: is_in_split_row,
                    model_cell_index: Some(cell_idx as u32),
                }),
                BoundingBox::new(cell_x, cell_y, cell_w, cell_h),
            );

            // 셀 BorderFill 조회
            let border_style = if cell.border_fill_id > 0 {
                let idx = (cell.border_fill_id as usize).saturating_sub(1);
                styles.border_styles.get(idx)
            } else {
                None
            };

            // 셀 배경
            self.render_cell_background(
                tree,
                &mut cell_node,
                border_style,
                cell_x,
                cell_y,
                cell_w,
                cell_h,
                bin_data_content,
            );

            // 셀 패딩
            let (mut pad_left, mut pad_right, pad_top, pad_bottom) =
                self.resolve_cell_padding(cell, table);

            // 셀 내 문단 구성
            let mut composed_paras: Vec<_> = cell
                .paragraphs
                .iter()
                .map(|p| compose_paragraph(p))
                .collect();

            // 텍스트 오버플로우 시 좌우 패딩 축소
            let (new_pl, new_pr) = self.shrink_cell_padding_for_overflow(
                pad_left,
                pad_right,
                cell_w,
                &composed_paras,
                &cell.paragraphs,
                styles,
                cell.apply_inner_margin,
            );
            pad_left = new_pl;
            pad_right = new_pr;

            let inner_x = cell_x + pad_left;
            let inner_width = (cell_w - pad_left - pad_right).max(0.0);
            let inner_height = (cell_h - pad_top - pad_bottom).max(0.0);

            // [Task #671] line_segs 비어 있는 셀 paragraph 의 단일 ComposedLine 압축
            // 결과를 셀 가용 너비 (inner_width) 에 맞춰 다중 ComposedLine 으로 재분할.
            for (cpi, para) in cell.paragraphs.iter().enumerate() {
                if let Some(comp) = composed_paras.get_mut(cpi) {
                    crate::renderer::composer::recompose_for_cell_width(
                        comp,
                        para,
                        inner_width,
                        styles,
                    );
                }
            }

            // 분할 행: [Task #993/#1025] start_cut/end_cut(유닛 컷)으로 표시할 줄 범위 계산.
            // 블록 분할이면 블록-셀 (row,col) 인덱스, 그 외는 행내 row_span==1 col 인덱스.
            let cut_units: Option<(usize, usize)> = if is_in_split_row {
                let pair = if is_block_split {
                    let su = match (is_split_start_row, split_start_block) {
                        (true, Some((bs, be))) => block_cut_index(table, bs, be, cell)
                            .and_then(|i| start_cut.get(i).copied())
                            .unwrap_or(0),
                        _ => 0,
                    };
                    let eu = match (is_split_end_row, split_end_block) {
                        (true, Some((bs, be))) => block_cut_index(table, bs, be, cell)
                            .and_then(|i| end_cut.get(i).copied())
                            .unwrap_or(usize::MAX),
                        _ => usize::MAX,
                    };
                    (su, eu)
                } else {
                    let cut_idx = table
                        .cells
                        .iter()
                        .filter(|c| c.row_span == 1 && c.row == cell.row && c.col < cell.col)
                        .count();
                    let su = if is_split_start_row {
                        start_cut.get(cut_idx).copied().unwrap_or(0)
                    } else {
                        0
                    };
                    let eu = if is_split_end_row {
                        end_cut.get(cut_idx).copied().unwrap_or(usize::MAX)
                    } else {
                        usize::MAX
                    };
                    (su, eu)
                };
                Some(pair)
            } else {
                None
            };
            let line_ranges: Option<Vec<(usize, usize)>> = cut_units
                .map(|(su, eu)| self.cell_line_ranges_from_cut(cell, table, styles, su, eu));
            // [Task #1073] 이 셀이 per-중첩행 분해 대상(단일 문단 + 가시 텍스트 없음 + 단일
            // 중첩 표 2행+)이면 cut 유닛 인덱스가 곧 중첩행 범위 → 렌더 NestedTableSplit 에
            // start_row 로 전달(연속 페이지가 중첩행 0부터 재렌더되는 결함 정정).
            let nested_cut_range: Option<(usize, usize)> = cut_units.filter(|_| {
                cell.paragraphs.len() == 1
                    && cell.paragraphs[0].text.trim().is_empty()
                    && cell.paragraphs[0]
                        .controls
                        .iter()
                        .filter(|c| matches!(c, crate::model::control::Control::Table(_)))
                        .count()
                        == 1
            });

            // 셀 내 텍스트 높이 (분할 행이면 줄 범위 내만 계산)
            // spacing_before: 셀 첫 문단 제외, spacing_after: 셀 마지막 문단 제외
            let split_para_count = cell.paragraphs.len();
            let total_content_height = if let Some(ref ranges) = line_ranges {
                let mut total = 0.0;
                for (pi, ((comp, para), &(start, end))) in composed_paras
                    .iter()
                    .zip(cell.paragraphs.iter())
                    .zip(ranges.iter())
                    .enumerate()
                {
                    let para_style = styles.para_styles.get(para.para_shape_id as usize);
                    let is_last_para = pi + 1 == split_para_count;
                    // spacing_before: 셀 첫 문단(pi==0) 제외
                    if start == 0 && end > 0 && pi > 0 {
                        let spacing_before = para_style.map(|s| s.spacing_before).unwrap_or(0.0);
                        total += spacing_before;
                    }
                    let line_count = comp.lines.len();
                    for li in start..end {
                        if li < line_count {
                            let line = &comp.lines[li];
                            let h = hwpunit_to_px(line.line_height, self.dpi);
                            let is_cell_last_line = is_last_para && li + 1 == line_count;
                            if !is_cell_last_line {
                                total += h + hwpunit_to_px(line.line_spacing, self.dpi);
                            } else {
                                total += h;
                            }
                        }
                    }
                    // spacing_after: 셀 마지막 문단 제외
                    if end == comp.lines.len() && end > start && !is_last_para {
                        let spacing_after = para_style.map(|s| s.spacing_after).unwrap_or(0.0);
                        total += spacing_after;
                    }
                }
                total
            } else {
                // 중첩 표가 있는 셀: LINE_SEG.line_height에 중첩 표 높이가 미포함되므로
                // vpos 기반으로 전체 콘텐츠 높이를 계산
                let has_nested = cell
                    .paragraphs
                    .iter()
                    .any(|p| p.controls.iter().any(|c| matches!(c, Control::Table(_))));
                if has_nested {
                    let last_seg_end: i32 = cell
                        .paragraphs
                        .iter()
                        .flat_map(|p| p.line_segs.last())
                        .map(|s| s.vertical_pos + s.line_height)
                        .max()
                        .unwrap_or(0);
                    let vpos_h = hwpunit_to_px(last_seg_end, self.dpi);
                    let line_h = self.calc_composed_paras_content_height(
                        &composed_paras,
                        &cell.paragraphs,
                        styles,
                    );
                    let nested_bottom =
                        self.calc_nested_controls_bottom_height(&cell.paragraphs, styles);
                    vpos_h
                        .max(line_h)
                        .max(nested_bottom)
                        .max(self.calc_non_inline_controls_flow_height(&cell.paragraphs))
                        .max(self.calc_cell_wrap_objects_bottom_height(&cell.paragraphs))
                } else {
                    self.calc_composed_paras_content_height(
                        &composed_paras,
                        &cell.paragraphs,
                        styles,
                    )
                    .max(self.calc_non_inline_controls_flow_height(&cell.paragraphs))
                    .max(self.calc_cell_wrap_objects_bottom_height(&cell.paragraphs))
                }
            };

            // 수직 정렬
            use crate::model::table::VerticalAlign;
            // [Task #697 후속] 분할 행이라도 이 셀의 line_ranges 가 셀의 모든 paragraph line 을
            // 그대로 visible 처리한다면 (= 실제 split 적용 안 받은 cell, 예: inner-table-01.hwp
            // cell[10] '사업개요' 라벨) 원본 cell.vertical_align 을 사용한다. split 적용으로
            // line 일부가 잘린 cell 만 Top 강제.
            let cell_was_split = if let Some(ref ranges) = line_ranges {
                ranges.iter().enumerate().any(|(i, &(s, e))| {
                    let total = composed_paras.get(i).map(|c| c.lines.len()).unwrap_or(0);
                    s != 0 || e != total
                })
            } else {
                false
            };
            let effective_align = if is_in_split_row && cell_was_split {
                VerticalAlign::Top
            } else {
                cell.vertical_align
            };
            let text_y_start = match effective_align {
                VerticalAlign::Top => cell_y + pad_top,
                VerticalAlign::Center => {
                    cell_y + pad_top + (inner_height - total_content_height).max(0.0) / 2.0
                }
                VerticalAlign::Bottom => {
                    cell_y + pad_top + (inner_height - total_content_height).max(0.0)
                }
            };

            // 세로쓰기 셀: 별도 레이아웃 경로 (가로 레이아웃 루프 대신)
            if cell.text_direction != 0 {
                let vert_inner_area = LayoutRect {
                    x: inner_x,
                    y: cell_y + pad_top,
                    width: inner_width,
                    height: inner_height,
                };
                self.layout_vertical_cell_text(
                    tree,
                    &mut cell_node,
                    &composed_paras,
                    &cell.paragraphs,
                    styles,
                    &vert_inner_area,
                    cell.vertical_align,
                    cell.text_direction,
                    section_index,
                    Some((para_index, control_index)),
                    cell_idx,
                    None,
                );
                // 세로쓰기 셀도 테두리를 엣지 그리드에 수집
                if let Some(bs) = border_style {
                    let cell_end_row_idx = cell_row + cell.row_span as usize;
                    let first_ri = render_rows.iter().position(|&r| r == cell_row).or_else(|| {
                        render_rows
                            .iter()
                            .position(|&r| r > cell_row && r < cell_end_row_idx)
                    });
                    let last_ri = render_rows
                        .iter()
                        .rposition(|&r| r >= cell_row && r < cell_end_row_idx);
                    if let (Some(fri), Some(lri)) = (first_ri, last_ri) {
                        collect_cell_borders(
                            &mut h_edges,
                            &mut v_edges,
                            cell_col,
                            fri,
                            cell.col_span as usize,
                            lri + 1 - fri,
                            &bs.borders,
                        );
                    }
                }
                table_node.children.push(cell_node);
                continue;
            }

            let inner_area = LayoutRect {
                x: inner_x,
                y: text_y_start,
                width: inner_width,
                height: inner_height,
            };

            // 셀 내 문단 + 컨트롤 통합 레이아웃
            // 분할 셀에서 실제 렌더링되는 마지막 문단 인덱스 계산
            // (뒤쪽 문단이 line_ranges=(0,0)으로 스킵되면 composed_paras.len()-1이 아님)
            let last_rendered_para_idx = if let Some(ref ranges) = line_ranges {
                let mut last_idx = 0usize;
                for (i, &(s, e)) in ranges.iter().enumerate() {
                    if s < e {
                        last_idx = i;
                    }
                }
                last_idx
            } else {
                composed_paras.len().saturating_sub(1)
            };

            let mut para_y = text_y_start;
            let mut has_preceding_text = false;
            for (cp_idx, (composed, para)) in composed_paras
                .iter()
                .zip(cell.paragraphs.iter())
                .enumerate()
            {
                // 분할 행이면 해당 문단의 줄 범위 적용
                let (start_line, end_line) = if let Some(ref ranges) = line_ranges {
                    if cp_idx < ranges.len() {
                        ranges[cp_idx]
                    } else {
                        (0, 0) // 범위 밖 문단은 렌더링하지 않음
                    }
                } else {
                    (0, composed.lines.len())
                };

                // [Task #993] 컷 범위 밖 문단은 이전/다음 페이지 소속 — 이 페이지에서
                // 스킵한다. cell_line_ranges_from_cut 이 가시 유닛만 범위에 넣으므로
                // (중첩 표/빈 문단 포함) start_line>=end_line 이면 비가시가 확정이다.
                // content_y_accum 은 가시 콘텐츠만 추적하므로 스킵 시 전진하지 않는다.
                if start_line >= end_line {
                    continue;
                }

                let cell_context = CellContext {
                    parent_para_index: para_index,
                    path: vec![CellPathEntry {
                        control_index,
                        cell_index: cell_idx,
                        cell_para_index: cp_idx,
                        text_direction: cell.text_direction,
                    }],
                };
                let cell_context_opt = Some(cell_context.clone());

                // 표 컨트롤 유무 판별
                let has_table_ctrl = para.controls.iter().any(|c| matches!(c, Control::Table(_)));

                // 인라인 이미지가 있는 문단: compose 전 위치를 저장
                let para_y_before_compose = para_y;

                // 인라인(treat_as_char) 컨트롤의 총 폭을 미리 계산
                let total_inline_width: f64 = para
                    .controls
                    .iter()
                    .map(|ctrl| match ctrl {
                        Control::Picture(pic) if pic.common.treat_as_char => {
                            hwpunit_to_px(pic.common.width as i32, self.dpi)
                        }
                        Control::Shape(shape) if shape.common().treat_as_char => {
                            hwpunit_to_px(shape.common().width as i32, self.dpi)
                        }
                        Control::Equation(eq) => hwpunit_to_px(eq.common.width as i32, self.dpi),
                        _ => 0.0,
                    })
                    .sum();

                // 표 컨트롤이 없는 문단: 텍스트 먼저, 컨트롤 나중 (기존 동작)
                // 표 컨트롤이 있는 문단: 문단 앞 간격 적용 → 표 먼저 배치 → 텍스트(엔터 등) 나중
                if !has_table_ctrl {
                    let is_last_para = cp_idx == last_rendered_para_idx;
                    let numbered_comp = if start_line == 0 {
                        self.apply_paragraph_numbering(
                            Some(composed),
                            para,
                            styles,
                            outline_numbering_id,
                        )
                    } else {
                        None
                    };
                    let composed_for_layout = numbered_comp.as_ref().unwrap_or(composed);
                    para_y = self.layout_composed_paragraph(
                        tree,
                        &mut cell_node,
                        composed_for_layout,
                        styles,
                        &inner_area,
                        para_y,
                        start_line,
                        end_line,
                        section_index,
                        cp_idx,
                        Some(cell_context.clone()),
                        !matches!(effective_align, VerticalAlign::Top),
                        is_last_para,
                        0.0,
                        None,
                        Some(para),
                        Some(bin_data_content),
                        None, // 셀 컨텍스트 — wrap zone 무관
                    );

                    let has_visible_text = composed
                        .lines
                        .iter()
                        .any(|line| line.runs.iter().any(|run| !run.text.trim().is_empty()));
                    if has_visible_text {
                        has_preceding_text = true;
                    }
                } else {
                    // has_table_ctrl: 표가 포함된 문단
                    // LINE_SEG vpos가 문단 위치를 정확히 지정하므로,
                    // 추가 spacing 없이 para_y를 그대로 사용.
                }

                // 이 문단의 컨트롤(이미지/도형/중첩테이블) 배치
                // 제목행 반복 셀에서는 컨트롤을 건너뜀 (이미지/도형 중복 방지)
                if !is_repeated_header_cell {
                    let para_alignment = styles
                        .para_styles
                        .get(para.para_shape_id as usize)
                        .map(|s| s.alignment)
                        .unwrap_or(Alignment::Left);

                    // 인라인 컨트롤의 시작 X 위치 (정렬 기반)
                    let mut inline_x = match para_alignment {
                        Alignment::Center | Alignment::Distribute => {
                            inner_area.x + (inner_area.width - total_inline_width).max(0.0) / 2.0
                        }
                        Alignment::Right => {
                            inner_area.x + (inner_area.width - total_inline_width).max(0.0)
                        }
                        _ => inner_area.x,
                    };

                    for (ctrl_idx, ctrl) in para.controls.iter().enumerate() {
                        match ctrl {
                            Control::Picture(pic) => {
                                if pic.common.treat_as_char {
                                    let pic_w = hwpunit_to_px(pic.common.width as i32, self.dpi);
                                    // layout_composed_paragraph에서 텍스트 흐름 안에 렌더링됐는지 확인:
                                    // 이미지 위치가 실제 run 범위에 포함될 때만 스킵
                                    let will_render_inline =
                                        composed.tac_controls.iter().any(|&(abs_pos, _, ci)| {
                                            ci == ctrl_idx
                                                && composed.lines.iter().any(|line| {
                                                    let line_chars: usize = line
                                                        .runs
                                                        .iter()
                                                        .map(|r| r.text.chars().count())
                                                        .sum();
                                                    abs_pos >= line.char_start
                                                        && abs_pos < line.char_start + line_chars
                                                })
                                        });
                                    if !will_render_inline {
                                        // 단독 이미지(텍스트 없는 문단): 직접 렌더링
                                        let pic_h =
                                            hwpunit_to_px(pic.common.height as i32, self.dpi);
                                        // [Task #477] 셀 폭 초과 시 비율 유지 클램프
                                        let clamped_w = pic_w.min(inner_area.width);
                                        let clamped_h = if pic_w > 0.0 {
                                            pic_h * (clamped_w / pic_w)
                                        } else {
                                            pic_h
                                        };
                                        let pic_area = LayoutRect {
                                            x: inline_x,
                                            y: para_y_before_compose,
                                            width: clamped_w,
                                            height: clamped_h,
                                        };
                                        // [Task #1151 v4] 셀 안 inline picture (partial 표 path).
                                        self.layout_picture(
                                            tree,
                                            &mut cell_node,
                                            pic,
                                            &pic_area,
                                            bin_data_content,
                                            Alignment::Left,
                                            Some(section_index),
                                            Some(cell_context.parent_para_index),
                                            Some(ctrl_idx),
                                            Some(&cell_context),
                                        );
                                        inline_x += clamped_w;
                                        continue;
                                    }
                                    inline_x += pic_w;
                                } else {
                                    // 비인라인 이미지: TopAndBottom+Para 는 row height 증가와
                                    // 무관하게 LINE_SEG 기준 anchor 를 유지한다.
                                    let anchor_y = if matches!(
                                        pic.common.text_wrap,
                                        crate::model::shape::TextWrap::TopAndBottom
                                    ) && matches!(
                                        pic.common.vert_rel_to,
                                        crate::model::shape::VertRelTo::Para
                                    ) {
                                        para.line_segs
                                            .first()
                                            .filter(|seg| seg.vertical_pos >= 0)
                                            .map(|seg| {
                                                cell_y
                                                    + pad_top
                                                    + hwpunit_to_px(seg.vertical_pos, self.dpi)
                                            })
                                            .unwrap_or(para_y_before_compose)
                                    } else {
                                        para_y
                                    };
                                    let pic_w = hwpunit_to_px(pic.common.width as i32, self.dpi);
                                    let pic_h = hwpunit_to_px(pic.common.height as i32, self.dpi);
                                    let unrestricted_take_place_cell_float =
                                        !pic.common.flow_with_text
                                            && matches!(
                                                pic.common.text_wrap,
                                                crate::model::shape::TextWrap::TopAndBottom
                                            )
                                            && matches!(
                                                pic.common.vert_rel_to,
                                                crate::model::shape::VertRelTo::Para
                                            );
                                    let picture_anchor_y = if unrestricted_take_place_cell_float {
                                        anchor_y
                                            - pic_h
                                            - hwpunit_to_px(
                                                pic.common.vertical_offset as i32,
                                                self.dpi,
                                            )
                                    } else {
                                        anchor_y
                                    };
                                    let cell_area = LayoutRect {
                                        y: picture_anchor_y,
                                        height: (inner_area.height
                                            - (picture_anchor_y - inner_area.y))
                                            .max(0.0),
                                        ..inner_area
                                    };
                                    let (pic_x, pic_y) = self.compute_object_position(
                                        &pic.common,
                                        pic_w,
                                        pic_h,
                                        &cell_area,
                                        &inner_area,
                                        &inner_area,
                                        &inner_area,
                                        picture_anchor_y,
                                        para_alignment,
                                    );
                                    let pic_area = LayoutRect {
                                        x: pic_x,
                                        y: pic_y,
                                        width: pic_w,
                                        height: pic_h,
                                    };
                                    let mut pic_for_layout = pic.clone();
                                    pic_for_layout.common.horizontal_offset = 0;
                                    pic_for_layout.common.vertical_offset = 0;
                                    pic_for_layout.common.horz_align =
                                        crate::model::shape::HorzAlign::Left;
                                    pic_for_layout.common.vert_align =
                                        crate::model::shape::VertAlign::Top;
                                    // [Task #1151 v4] 셀 안 non-inline picture (partial 표 path).
                                    if unrestricted_take_place_cell_float {
                                        self.layout_picture(
                                            tree,
                                            &mut table_node,
                                            &pic_for_layout,
                                            &pic_area,
                                            bin_data_content,
                                            Alignment::Left,
                                            Some(section_index),
                                            Some(cell_context.parent_para_index),
                                            Some(ctrl_idx),
                                            Some(&cell_context),
                                        );
                                    } else {
                                        self.layout_picture(
                                            tree,
                                            &mut cell_node,
                                            &pic_for_layout,
                                            &pic_area,
                                            bin_data_content,
                                            Alignment::Left,
                                            Some(section_index),
                                            Some(cell_context.parent_para_index),
                                            Some(ctrl_idx),
                                            Some(&cell_context),
                                        );
                                    }
                                    para_y += self.non_inline_control_flow_height(&pic.common);
                                }
                                has_preceding_text = true;
                            }
                            Control::Shape(shape) => {
                                if shape.common().treat_as_char {
                                    // 인라인 도형: 순차 X 위치로 배치
                                    let shape_w =
                                        hwpunit_to_px(shape.common().width as i32, self.dpi);
                                    let shape_area = LayoutRect {
                                        x: inline_x,
                                        y: para_y_before_compose,
                                        width: shape_w,
                                        height: inner_area.height,
                                    };
                                    // [Task #1138] 분할 표 셀 컨텍스트
                                    let table_cell_ctx = Some((
                                        section_index,
                                        para_index,
                                        control_index,
                                        cell_idx,
                                        cp_idx,
                                        ctrl_idx,
                                    ));
                                    self.layout_cell_shape(
                                        tree,
                                        &mut cell_node,
                                        shape,
                                        &shape_area,
                                        para_y_before_compose,
                                        Alignment::Left,
                                        styles,
                                        bin_data_content,
                                        clamp_header_negative_para_offset,
                                        table_cell_ctx,
                                    );
                                    inline_x += shape_w;
                                } else {
                                    // 비인라인 도형: 기존 동작
                                    let shape_anchor_y = if matches!(
                                        shape.common().vert_rel_to,
                                        crate::model::shape::VertRelTo::Para
                                    ) {
                                        para_y_before_compose
                                    } else {
                                        para_y
                                    };
                                    // [Task #1138] 분할 표 셀 컨텍스트
                                    let table_cell_ctx = Some((
                                        section_index,
                                        para_index,
                                        control_index,
                                        cell_idx,
                                        cp_idx,
                                        ctrl_idx,
                                    ));
                                    self.layout_cell_shape(
                                        tree,
                                        &mut cell_node,
                                        shape,
                                        &inner_area,
                                        shape_anchor_y,
                                        para_alignment,
                                        styles,
                                        bin_data_content,
                                        clamp_header_negative_para_offset,
                                        table_cell_ctx,
                                    );
                                }
                            }
                            Control::Equation(eq) => {
                                // 분할 표 내 수식: 항상 글자처럼 인라인 배치
                                let eq_w = hwpunit_to_px(eq.common.width as i32, self.dpi);
                                let eq_h = hwpunit_to_px(eq.common.height as i32, self.dpi);

                                // 빈 runs 셀 + TAC 수식: paragraph_layout(Task #287 경로)이
                                // layout_composed_paragraph 안에서 이미 렌더 후
                                // set_inline_shape_position 호출. 중복 emit 방지
                                // (Issue #301 의 분할 표 경로 보강 — Task #318).
                                let already_rendered_inline = tree
                                    .get_inline_shape_position(
                                        section_index,
                                        cp_idx,
                                        ctrl_idx,
                                        cell_context_opt.as_ref(),
                                    )
                                    .is_some();
                                if already_rendered_inline {
                                    inline_x += eq_w;
                                    continue;
                                }

                                let (eq_x, eq_y) = {
                                    let x = inline_x;
                                    inline_x += eq_w;
                                    (x, para_y_before_compose)
                                };

                                let tokens =
                                    super::super::equation::tokenizer::tokenize(&eq.script);
                                let ast =
                                    super::super::equation::parser::EqParser::new(tokens).parse();
                                let font_size_px = hwpunit_to_px(eq.font_size as i32, self.dpi);
                                let layout_box =
                                    super::super::equation::layout::EqLayout::new(font_size_px)
                                        .layout(&ast);
                                let color_str =
                                    super::super::equation::svg_render::eq_color_to_svg(eq.color);
                                let svg_content =
                                    super::super::equation::svg_render::render_equation_svg(
                                        &layout_box,
                                        &color_str,
                                        font_size_px,
                                    );

                                let eq_node = RenderNode::new(
                                    tree.next_id(),
                                    RenderNodeType::Equation(EquationNode {
                                        svg_content,
                                        layout_box,
                                        color_str,
                                        color: eq.color,
                                        font_size: font_size_px,
                                        section_index: Some(section_index),
                                        para_index: Some(para_index),
                                        control_index: Some(ctrl_idx),
                                        cell_index: Some(cell_idx),
                                        cell_para_index: Some(cp_idx),
                                        note_ref: None,
                                    }),
                                    BoundingBox::new(eq_x, eq_y, eq_w, eq_h),
                                );
                                cell_node.children.push(eq_node);
                            }
                            Control::Table(nested_table) => {
                                let nested_h = self.calc_nested_table_height(nested_table, styles);

                                // [Task #993] 컷 모델: 중첩 표는 atomic 유닛이라
                                // line_ranges 가 가시 여부를 이미 결정했다. 가시
                                // 중첩 표는 전체 렌더하되 셀 가용 공간을 초과하면
                                // calc_nested_split_rows 로 행 범위를 필터한다.
                                {
                                    // 중첩 표가 셀 가용 공간을 초과하면 행 범위 필터 적용
                                    let nested_y = if has_preceding_text {
                                        para_y
                                    } else {
                                        inner_area.y
                                    };
                                    let available_h =
                                        (inner_area.height - (nested_y - inner_area.y)).max(0.0);
                                    // TAC(글자처럼 취급) 표: 앞 텍스트 너비만큼 x 오프셋 적용.
                                    // 분할 표 내부에서는 composed 텍스트가 이전 줄까지 포함할 수
                                    // 있으므로, 표가 남은 폭에 들어가지 않으면 셀 좌측 기준으로
                                    // 배치해 페이지 오른쪽 밖으로 밀려나는 것을 막는다.
                                    let tac_text_offset = if nested_table.common.treat_as_char {
                                        let mut text_w = 0.0;
                                        for line in &composed.lines {
                                            for run in &line.runs {
                                                if !run.text.is_empty() {
                                                    let ts = resolved_to_text_style(
                                                        styles,
                                                        run.char_style_id,
                                                        run.lang_index,
                                                    );
                                                    text_w += estimate_text_width(&run.text, &ts);
                                                }
                                            }
                                        }
                                        text_w
                                    } else {
                                        0.0
                                    };
                                    let nested_w = if nested_table.common.width > 0 {
                                        hwpunit_to_px(nested_table.common.width as i32, self.dpi)
                                    } else {
                                        inner_area.width
                                    };
                                    let tac_x_offset = if nested_table.common.treat_as_char
                                        && tac_text_offset > 0.0
                                        && tac_text_offset + nested_w > inner_area.width + 0.5
                                    {
                                        0.0
                                    } else {
                                        tac_text_offset.min(inner_area.width)
                                    };
                                    let ctrl_area = LayoutRect {
                                        x: inner_area.x + tac_x_offset,
                                        y: nested_y,
                                        width: (inner_area.width - tac_x_offset).max(0.0),
                                        height: available_h,
                                    };

                                    // 중첩 표가 가용 공간을 초과하면 NestedTableSplit 적용
                                    let split_info = if let Some((su, eu)) = nested_cut_range {
                                        // [Task #1073] 페이지네이션 컷(중첩행 범위)으로 직접
                                        // NestedTableSplit 구성 — 연속 페이지가 start_row 부터
                                        // 렌더(available_h 휴리스틱의 row0 재렌더 결함 정정).
                                        let ncol = nested_table.col_count as usize;
                                        let nrow = nested_table.row_count as usize;
                                        let nrow_heights = self.resolve_row_heights(
                                            nested_table,
                                            ncol,
                                            nrow,
                                            None,
                                            styles,
                                        );
                                        let ncs = hwpunit_to_px(
                                            nested_table.cell_spacing as i32,
                                            self.dpi,
                                        );
                                        let start_row = su.min(nrow);
                                        let end_row = eu.min(nrow);
                                        let mut vis_h = 0.0;
                                        for r in start_row..end_row {
                                            vis_h += nrow_heights[r];
                                            if r + 1 < end_row {
                                                vis_h += ncs;
                                            }
                                        }
                                        Some(NestedTableSplit {
                                            start_row,
                                            end_row,
                                            visible_height: vis_h,
                                            offset_within_start: 0.0,
                                        })
                                    } else if nested_h > available_h + 0.5 {
                                        let ncol = nested_table.col_count as usize;
                                        let nrow = nested_table.row_count as usize;
                                        let nrow_heights = self.resolve_row_heights(
                                            nested_table,
                                            ncol,
                                            nrow,
                                            None,
                                            styles,
                                        );
                                        let ncell_spacing = hwpunit_to_px(
                                            nested_table.cell_spacing as i32,
                                            self.dpi,
                                        );
                                        Some(calc_nested_split_rows(
                                            &nrow_heights,
                                            ncell_spacing,
                                            0.0,
                                            available_h,
                                        ))
                                    } else {
                                        None
                                    };
                                    let split_ref = split_info.as_ref().filter(|s| {
                                        s.start_row > 0
                                            || s.end_row < nested_table.row_count as usize
                                    });

                                    let nested_ctx = cell_context_opt.as_ref().map(|ctx| {
                                        let mut new_ctx = ctx.clone();
                                        new_ctx.path.push(CellPathEntry {
                                            control_index: ctrl_idx,
                                            cell_index: 0,
                                            cell_para_index: 0,
                                            text_direction: 0,
                                        });
                                        new_ctx
                                    });
                                    let table_h_rendered = self.layout_table(
                                        tree,
                                        &mut cell_node,
                                        nested_table,
                                        section_index,
                                        styles,
                                        outline_numbering_id,
                                        &ctrl_area,
                                        nested_y,
                                        bin_data_content,
                                        None,
                                        1,
                                        None,
                                        para_alignment,
                                        nested_ctx,
                                        0.0,
                                        0.0,
                                        None,
                                        split_ref,
                                        None,
                                        clamp_header_negative_para_offset,
                                    );
                                    para_y = nested_y + table_h_rendered;
                                    has_preceding_text = true;
                                }
                            }
                            _ => {}
                        }
                    }
                }

                if has_table_ctrl {
                    // LINE_SEG vpos 기반으로 para_y 보정.
                    let is_last_para = cp_idx + 1 == composed_paras.len();
                    if !is_last_para {
                        if let Some(next_para) = cell.paragraphs.get(cp_idx + 1) {
                            if let Some(next_seg) = next_para.line_segs.first() {
                                let next_vpos_y =
                                    text_y_start + hwpunit_to_px(next_seg.vertical_pos, self.dpi);
                                para_y = para_y.max(next_vpos_y);
                            }
                        }
                    }
                }
            }

            // 각주 참조 번호
            for para in &cell.paragraphs {
                self.add_footnote_superscripts(tree, &mut cell_node, para, styles);
            }

            // 셀 테두리를 엣지 그리드에 수집 (인접 셀 중복 제거)
            if let Some(bs) = border_style {
                let cell_end_row_idx = cell_row + cell.row_span as usize;
                let first_ri = render_rows.iter().position(|&r| r == cell_row).or_else(|| {
                    render_rows
                        .iter()
                        .position(|&r| r > cell_row && r < cell_end_row_idx)
                });
                let last_ri = render_rows
                    .iter()
                    .rposition(|&r| r >= cell_row && r < cell_end_row_idx);
                if let (Some(fri), Some(lri)) = (first_ri, last_ri) {
                    collect_cell_borders(
                        &mut h_edges,
                        &mut v_edges,
                        cell_col,
                        fri,
                        cell.col_span as usize,
                        lri + 1 - fri,
                        &bs.borders,
                    );
                }
            }

            table_node.children.push(cell_node);
        }

        // 엣지 기반 테두리 렌더링
        table_node.children.extend(render_edge_borders(
            tree,
            &h_edges,
            &v_edges,
            &row_col_x,
            &grid_row_y,
            table_x,
            table_y,
        ));
        if self.show_transparent_borders.get() {
            table_node.children.extend(render_transparent_borders(
                tree,
                &h_edges,
                &v_edges,
                &row_col_x,
                &grid_row_y,
                table_x,
                table_y,
            ));
        }

        col_node.children.push(table_node);

        // ── 캡션 렌더링 ──
        // cell_index = 65534: 캡션 식별 센티널 (셀 0과 구분)
        let cap_cell_ctx = Some(CellContext {
            parent_para_index: para_index,
            path: vec![CellPathEntry {
                control_index,
                cell_index: 65534,
                cell_para_index: 0,
                text_direction: 0,
            }],
        });
        if render_top_caption {
            if let Some(ref caption) = table.caption {
                self.layout_caption(
                    tree,
                    col_node,
                    caption,
                    styles,
                    col_area,
                    table_x,
                    table_width,
                    y_start,
                    &mut self.auto_counter.borrow_mut(),
                    cap_cell_ctx.clone(),
                );
            }
        }
        if render_bottom_caption {
            if let Some(ref caption) = table.caption {
                let host_line_spacing = para
                    .line_segs
                    .first()
                    .map(|seg| hwpunit_to_px(seg.line_spacing, self.dpi))
                    .unwrap_or(0.0);
                let caption_y =
                    table_y + partial_table_height + host_line_spacing + caption_spacing;
                self.layout_caption(
                    tree,
                    col_node,
                    caption,
                    styles,
                    col_area,
                    table_x,
                    table_width,
                    caption_y,
                    &mut self.auto_counter.borrow_mut(),
                    cap_cell_ctx.clone(),
                );
            }
        }
        if render_lr_caption {
            if let Some(ref caption) = table.caption {
                use crate::model::shape::CaptionVertAlign;
                let cap_x = if is_left_cap {
                    table_x - cap_width_px - caption_spacing
                } else {
                    table_x + table_width + caption_spacing
                };
                let cap_y = match caption.vert_align {
                    CaptionVertAlign::Top => table_y,
                    CaptionVertAlign::Center => {
                        table_y + (partial_table_height - caption_height).max(0.0) / 2.0
                    }
                    CaptionVertAlign::Bottom => {
                        table_y + (partial_table_height - caption_height).max(0.0)
                    }
                };
                self.layout_caption(
                    tree,
                    col_node,
                    caption,
                    styles,
                    col_area,
                    cap_x,
                    cap_width_px,
                    cap_y,
                    &mut self.auto_counter.borrow_mut(),
                    cap_cell_ctx.clone(),
                );
            }
        }

        let caption_total = if render_top_caption {
            caption_height
                + if caption_height > 0.0 {
                    caption_spacing
                } else {
                    0.0
                }
        } else if render_bottom_caption {
            let host_line_spacing = para
                .line_segs
                .first()
                .map(|seg| hwpunit_to_px(seg.line_spacing, self.dpi))
                .unwrap_or(0.0);
            caption_height
                + host_line_spacing
                + if caption_height > 0.0 {
                    caption_spacing
                } else {
                    0.0
                }
        } else {
            // Left/Right 캡션은 표 높이에 영향 없음
            0.0
        };
        y_start + partial_table_height + caption_total
    }
}
