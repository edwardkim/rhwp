//! 페이지 분할 표 레이아웃 (layout_partial_table)

use crate::model::paragraph::Paragraph;
use crate::model::style::{Alignment, BorderLine};
use crate::model::control::Control;
use crate::model::bin_data::BinDataContent;
use crate::model::shape::CaptionDirection;
use super::utils::find_bin_data;
use super::super::render_tree::*;
use super::super::page_layout::LayoutRect;
use super::super::composer::compose_paragraph;
use super::super::style_resolver::ResolvedStyleSet;
use super::super::{hwpunit_to_px, ShapeStyle};
use super::{LayoutEngine, CellContext, CellPathEntry};
use super::border_rendering::{build_row_col_x, collect_cell_borders, render_edge_borders, render_transparent_borders};
use super::text_measurement::{resolved_to_text_style, estimate_text_width};
use super::table_layout::{NestedTableSplit, calc_nested_split_rows};
use super::super::height_measurer::MeasuredTable;

// 표 수평 정렬 보조 타입은 table_layout.rs에 통합됨

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
        col_area: &LayoutRect,
        y_start: f64,
        bin_data_content: &[BinDataContent],
        start_row: usize,
        end_row: usize,
        is_continuation: bool,
        split_start_content_offset: f64,
        split_end_content_limit: f64,
        host_margin_left: f64,
        host_margin_right: f64,
        measured_table: Option<&MeasuredTable>,
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

        // 분할 표 첫 부분: vert_offset 적용 (자리차지 표의 세로 오프셋)
        let y_start = if !is_continuation && !table.common.treat_as_char
            && matches!(table.common.text_wrap, crate::model::shape::TextWrap::TopAndBottom)
            && matches!(table.common.vert_rel_to, crate::model::shape::VertRelTo::Para)
            && table.common.vertical_offset > 0
        {
            y_start + hwpunit_to_px(table.common.vertical_offset as i32, self.dpi)
        } else {
            y_start
        };

        let col_count = table.col_count as usize;
        let row_count = table.row_count as usize;
        let cell_spacing = hwpunit_to_px(table.cell_spacing as i32, self.dpi);

        // ── 1. 열 폭 계산 + 2. 행 높이 계산 (table_layout 공유 메서드) ──
        let col_widths = self.resolve_column_widths(table, col_count);
        let mut row_heights = self.resolve_row_heights(table, col_count, row_count, measured_table, styles);

        // ── 2b. 분할 행 높이 오버라이드 ──
        if split_start_content_offset > 0.0 && start_row < row_count {
            // start_row가 continuation: 줄 범위 기반으로 실제 렌더링될 콘텐츠 높이 계산
            // (continuous total-offset 방식 대신 discrete line-range 방식 사용)
            let mut max_remaining_h = 0.0f64;
            for cell in &table.cells {
                if cell.row_span == 1 && cell.row as usize == start_row {
                    let (_, _, pad_top, pad_bottom) = self.resolve_cell_padding(cell, table);

                    // Task #324 v3: 중첩 표 포함 여부와 무관하게 통일된 경로 사용.
                    // compute_cell_line_ranges 가 cumulative position 기반으로 nested table
                    // atomic 처리를 정확히 수행하므로 별도 분기 불필요.
                    let composed: Vec<_> = cell.paragraphs.iter()
                        .map(|p| compose_paragraph(p))
                        .collect();
                    let ranges = self.compute_cell_line_ranges(cell, &composed, split_start_content_offset, 0.0, styles);
                    // [Task #362] split_start 시 한 페이지보다 큰 nested table 의 잔여 높이가
                    // 정확히 계산되도록 content_offset 을 함께 전달.
                    let remaining = self.calc_visible_content_height_from_ranges_with_offset(
                        &composed, &cell.paragraphs, &ranges, styles, split_start_content_offset,
                    );
                    let cell_h = remaining + pad_top + pad_bottom;
                    if cell_h > max_remaining_h {
                        max_remaining_h = cell_h;
                    }
                }
            }
            if max_remaining_h > 0.0 {
                row_heights[start_row] = max_remaining_h;
            }
        }
        if split_end_content_limit > 0.0 {
            let last_row = end_row.saturating_sub(1);
            if last_row < row_count {
                // last_row가 인트라-로우 분할: 제한된 높이 적용
                let mut max_split_h = 0.0f64;
                for cell in &table.cells {
                    if cell.row_span == 1 && cell.row as usize == last_row {
                        let (_, _, pad_top, pad_bottom) = self.resolve_cell_padding(cell, table);
                        let cell_h = split_end_content_limit + pad_top + pad_bottom;
                        if cell_h > max_split_h {
                            max_split_h = cell_h;
                        }
                    }
                }
                if max_split_h > 0.0 {
                    row_heights[last_row] = max_split_h;
                }
            }
        }

        // ── 3. 누적 위치 계산 ──
        let mut col_x = vec![0.0f64; col_count + 1];
        for i in 0..col_count {
            col_x[i + 1] = col_x[i] + col_widths[i] + if i + 1 < col_count { cell_spacing } else { 0.0 };
        }

        // 행별 열 위치 계산 (셀별 독립 너비 지원)
        let row_col_x = build_row_col_x(table, &col_widths, col_count, row_count, cell_spacing, self.dpi);

        let table_width = row_col_x.iter()
            .map(|rx| rx.last().copied().unwrap_or(0.0))
            .fold(col_x.last().copied().unwrap_or(0.0), f64::max);

        // ── 표 수평 위치 (table_layout 공유 메서드) ──
        let pw = self.current_paper_width.get();
        let paper_w = if pw > 0.0 { Some(pw) } else { None };
        let table_x = self.compute_table_x_position(
            table, table_width, col_area, 0, Alignment::Left, host_margin_left, host_margin_right, None, paper_w,
        );

        // ── 4. 렌더링할 행 목록 구성 ──
        // is_continuation && repeat_header → 제목행(0)에 제목 셀(is_header)이 있으면 반복
        let render_header = is_continuation && table.repeat_header && start_row > 0
            && table.cells.iter()
                .filter(|c| c.row == 0)
                .any(|c| c.is_header);
        let mut render_rows: Vec<usize> = Vec::new();
        if render_header {
            render_rows.push(0); // 제목행
        }
        for r in start_row..end_row.min(row_count) {
            render_rows.push(r);
        }

        // 렌더링 영역의 행별 y 위치 계산 (0부터 시작)
        let mut render_row_y: Vec<f64> = Vec::new(); // 각 render_rows 항목의 시작 y
        let mut y_accum = 0.0;
        for (i, &r) in render_rows.iter().enumerate() {
            render_row_y.push(y_accum);
            y_accum += row_heights[r] + if i + 1 < render_rows.len() { cell_spacing } else { 0.0 };
        }
        let partial_table_height = y_accum;


        // 엣지 기반 테두리 수집을 위한 그리드 (렌더링 행 기준)
        let render_row_count = render_rows.len();
        let mut h_edges: Vec<Vec<Option<BorderLine>>> = vec![vec![None; col_count]; render_row_count + 1];
        let mut v_edges: Vec<Vec<Option<BorderLine>>> = vec![vec![None; render_row_count]; col_count + 1];
        let mut grid_row_y = render_row_y.clone();
        grid_row_y.push(partial_table_height);

        // ── 4b. 캡션 처리 (첫 번째 파트에서만 렌더링) ──
        let is_first_part = start_row == 0 && !is_continuation && split_start_content_offset == 0.0;
        let is_last_part = end_row >= row_count && split_end_content_limit == 0.0;
        let (caption_height, caption_spacing) = if is_first_part || is_last_part {
            let ch = self.calculate_caption_height(&table.caption, styles);
            let cs = table.caption.as_ref()
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
        let cap_width_px = table.caption.as_ref()
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
                    tree, &mut table_node, Some(tbl_bs),
                    table_x, table_y, table_width, partial_table_height,
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
            let render_range_start = if render_header { 0 } else { start_row };
            let render_range_end = end_row.min(row_count);

            // 제목행 반복으로 렌더링되는 셀인지 판별
            // (원래 범위 밖이지만 render_header 때문에 포함되는 행0 셀)
            let is_repeated_header_cell = render_header && cell_row == 0 && cell_end_row <= start_row;

            // 셀이 렌더링 범위와 겹치는지 확인
            if cell_row >= render_range_end || cell_end_row <= render_range_start {
                // 제목행 렌더링 시 행0 셀은 포함
                if !is_repeated_header_cell {
                    continue;
                }
            }

            // render_rows에서 이 셀의 시작 행 위치 찾기
            // row_span이 페이지 경계를 넘는 셀: cell_row가 render_rows에 없을 수 있음
            // 이 경우 셀 span 범위 내에서 render_rows에 포함된 첫 번째 행을 찾음
            let render_idx = render_rows.iter().position(|&r| r == cell_row)
                .or_else(|| {
                    render_rows.iter().position(|&r| r > cell_row && r < cell_end_row)
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
            let is_split_start_row = split_start_content_offset > 0.0 && cell_row == start_row;
            let is_split_end_row = split_end_content_limit > 0.0 && cell_row == end_row.saturating_sub(1);
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
            self.render_cell_background(tree, &mut cell_node, border_style, cell_x, cell_y, cell_w, cell_h);

            // 셀 패딩
            let (mut pad_left, mut pad_right, pad_top, pad_bottom) = self.resolve_cell_padding(cell, table);

            // 셀 내 문단 구성
            let composed_paras: Vec<_> = cell.paragraphs.iter()
                .map(|p| compose_paragraph(p))
                .collect();

            // 텍스트 오버플로우 시 좌우 패딩 축소
            let (new_pl, new_pr) = self.shrink_cell_padding_for_overflow(
                pad_left, pad_right, cell_w, &composed_paras, styles,
            );
            pad_left = new_pl;
            pad_right = new_pr;

            let inner_x = cell_x + pad_left;
            let inner_width = (cell_w - pad_left - pad_right).max(0.0);
            let inner_height = (cell_h - pad_top - pad_bottom).max(0.0);


            // 분할 행: compute_cell_line_ranges()로 표시할 줄 범위 계산
            let line_ranges: Option<Vec<(usize, usize)>> = if is_in_split_row {
                let co = if is_split_start_row { split_start_content_offset } else { 0.0 };
                let cl = if is_split_end_row { split_end_content_limit } else { 0.0 };
                Some(self.compute_cell_line_ranges(cell, &composed_paras, co, cl, styles))
            } else {
                None
            };

            // 셀 내 텍스트 높이 (분할 행이면 줄 범위 내만 계산)
            // spacing_before: 셀 첫 문단 제외, spacing_after: 셀 마지막 문단 제외
            let split_para_count = cell.paragraphs.len();
            let total_content_height = if let Some(ref ranges) = line_ranges {
                let mut total = 0.0;
                for (pi, ((comp, para), &(start, end))) in composed_paras.iter()
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
                let has_nested = cell.paragraphs.iter()
                    .any(|p| p.controls.iter().any(|c| matches!(c, Control::Table(_))));
                if has_nested {
                    let last_seg_end: i32 = cell.paragraphs.iter()
                        .flat_map(|p| p.line_segs.last())
                        .map(|s| s.vertical_pos + s.line_height)
                        .max()
                        .unwrap_or(0);
                    let vpos_h = hwpunit_to_px(last_seg_end, self.dpi);
                    let line_h = self.calc_composed_paras_content_height(
                        &composed_paras, &cell.paragraphs, styles,
                    );
                    vpos_h.max(line_h)
                } else {
                    self.calc_composed_paras_content_height(
                        &composed_paras, &cell.paragraphs, styles,
                    )
                }
            };

            // 수직 정렬
            // 분할 행에서도 셀 콘텐츠가 visible area에 모두 들어가면 원래 정렬 적용
            use crate::model::table::VerticalAlign;
            // 분할 행에서는 항상 Top 정렬 (컨텐츠가 페이지를 넘어 분할되었으므로)
            let effective_align = if is_in_split_row {
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
                    tree, &mut cell_node, &composed_paras, &cell.paragraphs,
                    styles, &vert_inner_area, cell.vertical_align, cell.text_direction,
                    section_index, Some((para_index, control_index)), cell_idx, None,
                );
                // 세로쓰기 셀도 테두리를 엣지 그리드에 수집
                if let Some(bs) = border_style {
                    let cell_end_row_idx = cell_row + cell.row_span as usize;
                    let first_ri = render_rows.iter().position(|&r| r == cell_row)
                        .or_else(|| render_rows.iter().position(|&r| r > cell_row && r < cell_end_row_idx));
                    let last_ri = render_rows.iter().rposition(|&r| r >= cell_row && r < cell_end_row_idx);
                    if let (Some(fri), Some(lri)) = (first_ri, last_ri) {
                        collect_cell_borders(
                            &mut h_edges, &mut v_edges,
                            cell_col, fri, cell.col_span as usize, lri + 1 - fri,
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
            // 분할 셀에서 중첩 표 오프셋 계산을 위한 누적 콘텐츠 높이 추적
            let mut content_y_accum = 0.0f64;
            for (cp_idx, (composed, para)) in composed_paras.iter().zip(cell.paragraphs.iter()).enumerate() {
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

                // 분할 셀에서 offset에 의해 완전히 소비된 문단은 스킵
                // (중첩 표 포함 문단도 range가 (n,n)이면 이전 페이지에서 이미 렌더링됨)
                let has_nested_table = para.controls.iter().any(|c| matches!(c, Control::Table(_)));
                if start_line >= end_line {
                    // 중첩 표 문단: offset 범위 안에 있으면 스킵, 아니면 렌더링 필요
                    if has_nested_table && is_in_split_row && split_start_content_offset > 0.0 {
                        // content_y_accum으로 이 문단의 중첩 표가 완전히 offset 이전인지 판단
                        let nested_h: f64 = para.controls.iter().map(|ctrl| {
                            if let Control::Table(t) = ctrl {
                                self.calc_nested_table_height(t, styles)
                            } else { 0.0 }
                        }).sum();
                        let nested_end = content_y_accum + nested_h;
                        if nested_end <= split_start_content_offset {
                            // 이전 페이지에서 완전히 렌더링됨 → content_y_accum 전진 후 스킵
                            content_y_accum += nested_h;
                            continue;
                        }
                    } else if has_nested_table && is_in_split_row && split_end_content_limit > 0.0 {
                        // 분할 끝 행: compute_cell_line_ranges가 중첩 표를 limit 초과로
                        // 다음 페이지로 미뤘음(=(line_count, line_count)). 이 페이지에서는 스킵.
                        let nested_h: f64 = para.controls.iter().map(|ctrl| {
                            if let Control::Table(t) = ctrl {
                                self.calc_nested_table_height(t, styles)
                            } else { 0.0 }
                        }).sum();
                        content_y_accum += nested_h;
                        continue;
                    } else if !has_nested_table {
                        // 일반 문단이 offset 으로 완전히 소비됨 → content_y_accum 갱신 후 스킵.
                        // (Task #324 후속: 이 갱신 누락으로 후속 nested-table 문단의 content_y_accum
                        //  이 부정확하여 split_start 페이지에서 렌더 위치가 잘못 판정되던 결함 수정.)
                        if is_in_split_row {
                            let p_style = styles.para_styles.get(para.para_shape_id as usize);
                            let sp_before = if cp_idx > 0 { p_style.map(|s| s.spacing_before).unwrap_or(0.0) } else { 0.0 };
                            let sp_after = if cp_idx + 1 != split_para_count { p_style.map(|s| s.spacing_after).unwrap_or(0.0) } else { 0.0 };
                            let lc = composed.lines.len();
                            let is_lp = cp_idx + 1 == split_para_count;
                            let h = if lc == 0 {
                                sp_before + hwpunit_to_px(400, self.dpi) + sp_after
                            } else {
                                composed.lines.iter().enumerate().map(|(li, line)| {
                                    let lh = hwpunit_to_px(line.line_height, self.dpi);
                                    let ls = hwpunit_to_px(line.line_spacing, self.dpi);
                                    let is_cell_last = is_lp && li + 1 == lc;
                                    let mut h = if !is_cell_last { lh + ls } else { lh };
                                    if li == 0 { h += sp_before; }
                                    if li == lc - 1 { h += sp_after; }
                                    h
                                }).sum()
                            };
                            content_y_accum += h;
                        }
                        continue;
                    }
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
                let total_inline_width: f64 = para.controls.iter().map(|ctrl| {
                    match ctrl {
                        Control::Picture(pic) if pic.common.treat_as_char => {
                            hwpunit_to_px(pic.common.width as i32, self.dpi)
                        }
                        Control::Shape(shape) if shape.common().treat_as_char => {
                            hwpunit_to_px(shape.common().width as i32, self.dpi)
                        }
                        Control::Equation(eq) => {
                            hwpunit_to_px(eq.common.width as i32, self.dpi)
                        }
                        _ => 0.0,
                    }
                }).sum();

                // 이 문단의 전체 텍스트 높이 계산 (분할 셀에서 콘텐츠 위치 추적용)
                // (보이지 않는 줄 포함 — content_y_accum은 실제 콘텐츠 위치를 추적)
                let para_full_text_h: f64 = if is_in_split_row {
                    let p_style = styles.para_styles.get(para.para_shape_id as usize);
                    let sp_before = p_style.map(|s| s.spacing_before).unwrap_or(0.0);
                    let sp_after = p_style.map(|s| s.spacing_after).unwrap_or(0.0);
                    let lc = composed.lines.len();
                    let is_lp = cp_idx + 1 == split_para_count;
                    let line_based_h = if lc == 0 {
                        sp_before + hwpunit_to_px(400, self.dpi) + sp_after
                    } else {
                        composed.lines.iter().enumerate().map(|(li, line)| {
                            let h = hwpunit_to_px(line.line_height, self.dpi);
                            let ls = hwpunit_to_px(line.line_spacing, self.dpi);
                            let is_cell_last = is_lp && li + 1 == lc;
                            let mut lh = if !is_cell_last { h + ls } else { h };
                            if li == 0 { lh += sp_before; }
                            if li == lc - 1 { lh += sp_after; }
                            lh
                        }).sum()
                    };
                    // 중첩 표가 있으면 실제 높이로 대체
                    let nested_h: f64 = para.controls.iter().map(|ctrl| {
                        if let Control::Table(t) = ctrl {
                            self.calc_nested_table_height(t, styles)
                        } else {
                            0.0
                        }
                    }).sum();
                    if nested_h > 0.0 { nested_h.max(line_based_h) } else { line_based_h }
                } else {
                    0.0
                };

                // 표 컨트롤이 없는 문단: 텍스트 먼저, 컨트롤 나중 (기존 동작)
                // 표 컨트롤이 있는 문단: 문단 앞 간격 적용 → 표 먼저 배치 → 텍스트(엔터 등) 나중
                if !has_table_ctrl {
                    let is_last_para = cp_idx == last_rendered_para_idx;
                    para_y = self.layout_composed_paragraph(
                        tree,
                        &mut cell_node,
                        composed,
                        styles,
                        &inner_area,
                        para_y,
                        start_line,
                        end_line,
                        section_index, cp_idx,
                        Some(cell_context.clone()),
                        is_last_para,
                        0.0,
                        None, Some(para), Some(bin_data_content),
                    );

                    let has_visible_text = composed.lines.iter()
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
                    let para_alignment = styles.para_styles
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
                                    let will_render_inline = composed.tac_controls.iter().any(|&(abs_pos, _, ci)| {
                                        ci == ctrl_idx && composed.lines.iter().any(|line| {
                                            let line_chars: usize = line.runs.iter().map(|r| r.text.chars().count()).sum();
                                            abs_pos >= line.char_start && abs_pos < line.char_start + line_chars
                                        })
                                    });
                                    if !will_render_inline {
                                        // 단독 이미지(텍스트 없는 문단): 직접 렌더링
                                        let pic_h = hwpunit_to_px(pic.common.height as i32, self.dpi);
                                        let pic_area = LayoutRect {
                                            x: inline_x,
                                            y: para_y_before_compose,
                                            width: pic_w,
                                            height: pic_h,
                                        };
                                        self.layout_picture(tree, &mut cell_node, pic, &pic_area, bin_data_content, Alignment::Left, None, None, None);
                                    }
                                    inline_x += pic_w;
                                } else {
                                    // 비인라인 이미지: 기존 동작
                                    let pic_area = LayoutRect {
                                        y: para_y,
                                        height: (inner_area.height - (para_y - inner_area.y)).max(0.0),
                                        ..inner_area
                                    };
                                    self.layout_picture(tree, &mut cell_node, pic, &pic_area, bin_data_content, para_alignment, None, None, None);
                                    let pic_h = hwpunit_to_px(pic.common.height as i32, self.dpi);
                                    para_y += pic_h;
                                }
                                has_preceding_text = true;
                            }
                            Control::Shape(shape) => {
                                if shape.common().treat_as_char {
                                    // 인라인 도형: 순차 X 위치로 배치
                                    let shape_w = hwpunit_to_px(shape.common().width as i32, self.dpi);
                                    let shape_area = LayoutRect {
                                        x: inline_x,
                                        y: para_y_before_compose,
                                        width: shape_w,
                                        height: inner_area.height,
                                    };
                                    self.layout_cell_shape(tree, &mut cell_node, shape, &shape_area, para_y_before_compose, Alignment::Left, styles, bin_data_content);
                                    inline_x += shape_w;
                                } else {
                                    // 비인라인 도형: 기존 동작
                                    self.layout_cell_shape(tree, &mut cell_node, shape, &inner_area, para_y, para_alignment, styles, bin_data_content);
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
                                    .get_inline_shape_position(section_index, cp_idx, ctrl_idx)
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

                                let tokens = super::super::equation::tokenizer::tokenize(&eq.script);
                                let ast = super::super::equation::parser::EqParser::new(tokens).parse();
                                let font_size_px = hwpunit_to_px(eq.font_size as i32, self.dpi);
                                let layout_box = super::super::equation::layout::EqLayout::new(font_size_px).layout(&ast);
                                let color_str = super::super::equation::svg_render::eq_color_to_svg(eq.color);
                                let svg_content = super::super::equation::svg_render::render_equation_svg(
                                    &layout_box, &color_str, font_size_px,
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
                                    }),
                                    BoundingBox::new(eq_x, eq_y, eq_w, eq_h),
                                );
                                cell_node.children.push(eq_node);
                            }
                            Control::Table(nested_table) => {
                                let nested_h = self.calc_nested_table_height(nested_table, styles);

                                // 분할 셀에서 중첩 표의 가시성 판단 및 행 범위 필터링
                                if is_in_split_row {
                                    // LINE_SEG.lh에 중첩 표 높이가 이미 포함되어 있으므로
                                    // 중첩 표 시작 위치는 content_y_accum (현재 문단 시작)으로 추정
                                    let nested_content_start = content_y_accum;
                                    let nested_content_end = nested_content_start + nested_h;

                                    // 중첩 표가 split_start_content_offset 이전에 완전히 끝나면 스킵
                                    // (LINE_SEG.lh에 이미 포함되므로 content_y_accum에 별도 추가 불필요)
                                    if split_start_content_offset > 0.0 && nested_content_end <= split_start_content_offset {
                                        continue;
                                    }
                                    // 중첩 표가 split_end_content_limit 이후에 시작하면 스킵
                                    if split_end_content_limit > 0.0 && nested_content_start >= split_end_content_limit {
                                        continue;
                                    }

                                    // 중첩 표 내에서의 오프셋 (연속 페이지: 표 시작이 오프셋 이전)
                                    let offset_into_table = if split_start_content_offset > nested_content_start {
                                        (split_start_content_offset - nested_content_start).min(nested_h)
                                    } else {
                                        0.0
                                    };

                                    // 중첩 표에 할당 가능한 공간 계산
                                    let visible_space = if split_end_content_limit > 0.0 {
                                        let end_in_table = (split_end_content_limit - nested_content_start).min(nested_h);
                                        (end_in_table - offset_into_table).max(0.0)
                                    } else {
                                        nested_h - offset_into_table
                                    };

                                    // 행 범위 계산: 보이는 부분에 해당하는 행만 렌더링
                                    let ncol = nested_table.col_count as usize;
                                    let nrow = nested_table.row_count as usize;
                                    let nrow_heights = self.resolve_row_heights(nested_table, ncol, nrow, None, styles);
                                    let ncell_spacing = hwpunit_to_px(nested_table.cell_spacing as i32, self.dpi);
                                    let split_info = calc_nested_split_rows(&nrow_heights, ncell_spacing, offset_into_table, visible_space);

                                    // 전체 행이 모두 보이면 split 없이, 아니면 행 범위 필터 적용
                                    let need_split = split_info.start_row > 0 || split_info.end_row < nrow;

                                    let nested_y = if has_preceding_text {
                                        para_y
                                    } else {
                                        inner_area.y
                                    };
                                    // TAC(글자처럼 취급) 표: 앞 텍스트 너비만큼 x 오프셋 적용
                                    let tac_text_offset = if nested_table.common.treat_as_char {
                                        let mut text_w = 0.0;
                                        for line in &composed.lines {
                                            for run in &line.runs {
                                                if !run.text.is_empty() {
                                                    let ts = resolved_to_text_style(
                                                        styles, run.char_style_id, run.lang_index);
                                                    text_w += estimate_text_width(&run.text, &ts);
                                                }
                                            }
                                        }
                                        text_w
                                    } else {
                                        0.0
                                    };
                                    let ctrl_area = LayoutRect {
                                        x: inner_area.x + tac_text_offset,
                                        y: nested_y,
                                        width: (inner_area.width - tac_text_offset).max(0.0),
                                        height: visible_space,
                                    };
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
                                    let split_ref = if need_split { Some(&split_info) } else { None };
                                    let table_h_rendered = self.layout_table(
                                        tree, &mut cell_node, nested_table,
                                        section_index, styles, &ctrl_area, nested_y,
                                        bin_data_content, None, 1,
                                        None, para_alignment,
                                        nested_ctx,
                                        0.0, 0.0, None, split_ref, None,
                                    );
                                    // 렌더링된 높이만큼 para_y 전진
                                    para_y = nested_y + table_h_rendered;
                                    // LINE_SEG.lh에 이미 중첩 표 높이가 반영되어 있으므로
                                    // content_y_accum에 nested_h를 별도로 추가하면 이중 계산됨.
                                    // para_full_text_h (line 817에서 추가)에 이미 포함됨.
                                    has_preceding_text = true;
                                } else {
                                    // 비분할 행: 중첩 표가 셀 가용 공간을 초과하면 행 범위 필터 적용
                                    let nested_y = if has_preceding_text {
                                        para_y
                                    } else {
                                        inner_area.y
                                    };
                                    let available_h = (inner_area.height - (nested_y - inner_area.y)).max(0.0);
                                    // TAC(글자처럼 취급) 표: 앞 텍스트 너비만큼 x 오프셋 적용
                                    let tac_text_offset = if nested_table.common.treat_as_char {
                                        let mut text_w = 0.0;
                                        for line in &composed.lines {
                                            for run in &line.runs {
                                                if !run.text.is_empty() {
                                                    let ts = resolved_to_text_style(
                                                        styles, run.char_style_id, run.lang_index);
                                                    text_w += estimate_text_width(&run.text, &ts);
                                                }
                                            }
                                        }
                                        text_w
                                    } else {
                                        0.0
                                    };
                                    let ctrl_area = LayoutRect {
                                        x: inner_area.x + tac_text_offset,
                                        y: nested_y,
                                        width: (inner_area.width - tac_text_offset).max(0.0),
                                        height: available_h,
                                    };

                                    // 중첩 표가 가용 공간을 초과하면 NestedTableSplit 적용
                                    let split_info = if nested_h > available_h + 0.5 {
                                        let ncol = nested_table.col_count as usize;
                                        let nrow = nested_table.row_count as usize;
                                        let nrow_heights = self.resolve_row_heights(nested_table, ncol, nrow, None, styles);
                                        let ncell_spacing = hwpunit_to_px(nested_table.cell_spacing as i32, self.dpi);
                                        Some(calc_nested_split_rows(&nrow_heights, ncell_spacing, 0.0, available_h))
                                    } else {
                                        None
                                    };
                                    let split_ref = split_info.as_ref().filter(|s| s.start_row > 0 || s.end_row < nested_table.row_count as usize);

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
                                        tree, &mut cell_node, nested_table,
                                        section_index, styles, &ctrl_area, nested_y,
                                        bin_data_content, None, 1,
                                        None, para_alignment,
                                        nested_ctx,
                                        0.0, 0.0, None, split_ref, None,
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
                                let next_vpos_y = text_y_start + hwpunit_to_px(
                                    next_seg.vertical_pos, self.dpi);
                                para_y = para_y.max(next_vpos_y);
                            }
                        }
                    }
                }

                // 분할 셀 콘텐츠 위치 추적: 텍스트 높이 누적
                if is_in_split_row {
                    content_y_accum += para_full_text_h;
                }
            }

            // 각주 참조 번호
            for para in &cell.paragraphs {
                self.add_footnote_superscripts(tree, &mut cell_node, para, styles);
            }

            // 셀 테두리를 엣지 그리드에 수집 (인접 셀 중복 제거)
            if let Some(bs) = border_style {
                let cell_end_row_idx = cell_row + cell.row_span as usize;
                let first_ri = render_rows.iter().position(|&r| r == cell_row)
                    .or_else(|| render_rows.iter().position(|&r| r > cell_row && r < cell_end_row_idx));
                let last_ri = render_rows.iter().rposition(|&r| r >= cell_row && r < cell_end_row_idx);
                if let (Some(fri), Some(lri)) = (first_ri, last_ri) {
                    collect_cell_borders(
                        &mut h_edges, &mut v_edges,
                        cell_col, fri, cell.col_span as usize, lri + 1 - fri,
                        &bs.borders,
                    );
                }
            }

            table_node.children.push(cell_node);
        }

        // 엣지 기반 테두리 렌더링
        table_node.children.extend(render_edge_borders(
            tree, &h_edges, &v_edges, &row_col_x, &grid_row_y, table_x, table_y,
        ));
        if self.show_transparent_borders.get() {
            table_node.children.extend(render_transparent_borders(
                tree, &h_edges, &v_edges, &row_col_x, &grid_row_y, table_x, table_y,
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
                    tree, col_node, caption, styles, col_area,
                    table_x, table_width, y_start,
                    &mut self.auto_counter.borrow_mut(),
                    cap_cell_ctx.clone(),
                );
            }
        }
        if render_bottom_caption {
            if let Some(ref caption) = table.caption {
                let host_line_spacing = para.line_segs.first()
                    .map(|seg| hwpunit_to_px(seg.line_spacing, self.dpi))
                    .unwrap_or(0.0);
                let caption_y = table_y + partial_table_height + host_line_spacing + caption_spacing;
                self.layout_caption(
                    tree, col_node, caption, styles, col_area,
                    table_x, table_width, caption_y,
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
                    CaptionVertAlign::Center => table_y + (partial_table_height - caption_height).max(0.0) / 2.0,
                    CaptionVertAlign::Bottom => table_y + (partial_table_height - caption_height).max(0.0),
                };
                self.layout_caption(
                    tree, col_node, caption, styles, col_area,
                    cap_x, cap_width_px, cap_y,
                    &mut self.auto_counter.borrow_mut(),
                    cap_cell_ctx.clone(),
                );
            }
        }

        let caption_total = if render_top_caption {
            caption_height + if caption_height > 0.0 { caption_spacing } else { 0.0 }
        } else if render_bottom_caption {
            let host_line_spacing = para.line_segs.first()
                .map(|seg| hwpunit_to_px(seg.line_spacing, self.dpi))
                .unwrap_or(0.0);
            caption_height + host_line_spacing + if caption_height > 0.0 { caption_spacing } else { 0.0 }
        } else {
            // Left/Right 캡션은 표 높이에 영향 없음
            0.0
        };
        y_start + partial_table_height + caption_total
    }
}
