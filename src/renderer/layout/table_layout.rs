//! 표 레이아웃 (layout_table + 셀 높이/줄범위 계산)

use crate::model::paragraph::Paragraph;
use crate::model::style::{Alignment, BorderLine};
use crate::model::table::VerticalAlign;
use crate::model::control::Control;
use crate::model::bin_data::BinDataContent;
use super::super::render_tree::*;
use super::super::page_layout::LayoutRect;
use super::super::height_measurer::MeasuredTable;
use super::super::composer::{compose_paragraph, ComposedParagraph};
use super::super::style_resolver::ResolvedStyleSet;
use super::super::{hwpunit_to_px, ShapeStyle};
use super::{LayoutEngine, CellContext, CellPathEntry};
use super::border_rendering::{build_row_col_x, collect_cell_borders, render_cell_diagonal, render_edge_borders, render_transparent_borders};
use super::text_measurement::{resolved_to_text_style, estimate_text_width};
use super::super::composer::effective_text_for_metrics;
use super::utils::find_bin_data;

// 표 수평 정렬: model::shape 타입 사용
use crate::model::shape::{HorzRelTo, HorzAlign};

/// 중첩 표 부분 렌더링을 위한 행 범위 정보
pub(crate) struct NestedTableSplit {
    pub start_row: usize,
    pub end_row: usize,
    /// 실제 표시할 높이 (마지막 행이 부분적으로 보일 때 전체 행 높이 대신 사용)
    pub visible_height: f64,
    /// start_row 내부 오프셋: 이미 이전 페이지에 렌더링된 start_row 상단 부분의 높이
    pub offset_within_start: f64,
}

/// 중첩 표에서 pixel offset/space를 행 범위로 변환한다.
/// 공간이 부족한 마지막 행은 제외하여 다음 페이지에서 렌더링되도록 한다.
pub(crate) fn calc_nested_split_rows(
    row_heights: &[f64],
    cell_spacing: f64,
    offset: f64,
    space: f64,
) -> NestedTableSplit {
    let row_count = row_heights.len();
    if row_count == 0 {
        return NestedTableSplit { start_row: 0, end_row: 0, visible_height: 0.0, offset_within_start: 0.0 };
    }

    // row_y 누적 배열 (layout_table과 동일 방식)
    let mut row_y = vec![0.0f64; row_count + 1];
    for i in 0..row_count {
        row_y[i + 1] = row_y[i] + row_heights[i]
            + if i + 1 < row_count { cell_spacing } else { 0.0 };
    }

    // offset에 해당하는 시작 행 찾기
    let mut start_row = 0;
    if offset > 0.0 {
        start_row = row_count;
        for r in 0..row_count {
            if row_y[r] + row_heights[r] > offset {
                start_row = r;
                break;
            }
        }
    }

    // space에 해당하는 끝 행 찾기
    let visible_end = offset + space;
    let mut end_row = row_count;
    if space > 0.0 && space < f64::MAX {
        for r in 0..row_count {
            if row_y[r] + row_heights[r] >= visible_end {
                end_row = r + 1;
                break;
            }
        }
    }

    // 마지막 행이 거의 들어가지 않으면 제외하여 다음 페이지에서 온전하게 렌더링
    if end_row > start_row {
        let last_r = end_row - 1;
        let last_row_top = row_y[last_r];
        let available_for_last = visible_end - last_row_top;
        let last_h = row_heights[last_r];
        let min_threshold = (last_h * 0.5).min(10.0);
        if available_for_last < last_h && available_for_last < min_threshold {
            end_row -= 1;
        }
    }

    // visible_height: 포함된 행의 실제 높이 (start_row 전체 포함)
    let range_height = if end_row > start_row {
        row_y[end_row] - row_y[start_row]
    } else {
        0.0
    };
    // 연속 페이지(offset>0): start_row를 처음부터 완전히 렌더링하므로
    // offset_within_start=0, visible_height=range_height (포함된 행 전체 높이)
    // 첫 페이지(offset==0): 가용 공간으로 캡
    let visible_height = if offset > 0.0 {
        range_height
    } else {
        space.min(range_height)
    };

    NestedTableSplit { start_row, end_row, visible_height, offset_within_start: 0.0 }
}


impl LayoutEngine {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn layout_table(
        &self,
        tree: &mut PageRenderTree,
        col_node: &mut RenderNode,
        table: &crate::model::table::Table,
        section_index: usize,
        styles: &ResolvedStyleSet,
        col_area: &LayoutRect,
        y_start: f64,
        bin_data_content: &[BinDataContent],
        measured_table: Option<&MeasuredTable>,
        depth: usize,
        table_meta: Option<(usize, usize)>,
        host_alignment: Alignment,
        enclosing_cell_ctx: Option<CellContext>,
        host_margin_left: f64,
        host_margin_right: f64,
        inline_x_override: Option<f64>,
        nested_split: Option<&NestedTableSplit>,
        para_y: Option<f64>,
    ) -> f64 {
        if table.cells.is_empty() {
            if depth == 0 { return y_start; } else { return 0.0; }
        }
        // 1x1 래퍼 표 감지: 외곽 표를 무시하고 내부 표를 직접 렌더링
        if table.row_count == 1 && table.col_count == 1 && table.cells.len() == 1 {
            let cell = &table.cells[0];
            let has_visible_text = cell.paragraphs.iter()
                .any(|p| p.text.chars().any(|ch| !ch.is_whitespace() && ch != '\r' && ch != '\n'));
            if !has_visible_text {
                if let Some(nested) = cell.paragraphs.iter()
                    .flat_map(|p| p.controls.iter())
                    .find_map(|c| if let Control::Table(t) = c { Some(t.as_ref()) } else { None })
                {
                    return self.layout_table(
                        tree, col_node, nested,
                        section_index, styles, col_area, y_start,
                        bin_data_content, None, depth,
                        table_meta, host_alignment, enclosing_cell_ctx, host_margin_left,
                        host_margin_right, inline_x_override, nested_split, para_y,
                    );
                }
            }
        }

        let col_count = table.col_count as usize;
        let row_count = table.row_count as usize;
        let cell_spacing = hwpunit_to_px(table.cell_spacing as i32, self.dpi);

        // ── 1. 열 폭 + 행 높이 계산 ──
        let col_widths = self.resolve_column_widths(table, col_count);
        let row_heights = self.resolve_row_heights(table, col_count, row_count, measured_table, styles);

        // ── 2. 누적 위치 계산 ──
        let mut col_x = vec![0.0f64; col_count + 1];
        for i in 0..col_count {
            col_x[i + 1] = col_x[i] + col_widths[i] + if i + 1 < col_count { cell_spacing } else { 0.0 };
        }
        let mut row_y = vec![0.0f64; row_count + 1];
        for i in 0..row_count {
            row_y[i + 1] = row_y[i] + row_heights[i] + if i + 1 < row_count { cell_spacing } else { 0.0 };
        }

        // 중첩 표 부분 렌더링: row_y를 시프트하여 보이는 행만 표시
        let (row_y_shift, split_row_range, split_y_offset) = if let Some(split) = nested_split {
            let sr = split.start_row.min(row_count);
            let er = split.end_row.min(row_count);
            let shift = row_y[sr];
            // row_y를 시프트하여 start_row가 0에서 시작하도록 함
            for y in row_y.iter_mut() {
                *y -= shift;
            }
            // end_row 이후의 모든 row_y를 캡하여 spanning 셀이 보이는 영역을 초과하지 않도록 함
            let cap_y = if split.visible_height > 0.0 {
                split.visible_height.min(row_y[er])
            } else {
                row_y[er]
            };
            for i in er..=row_count {
                row_y[i] = cap_y;
            }
            // start_row 내부 오프셋: 이미 이전 페이지에 표시된 부분만큼 위로 올림
            (shift, Some((sr, er)), split.offset_within_start)
        } else {
            (0.0, None, 0.0)
        };

        let row_col_x = build_row_col_x(table, &col_widths, col_count, row_count, cell_spacing, self.dpi);

        let table_width = row_col_x.iter()
            .map(|rx| rx.last().copied().unwrap_or(0.0))
            .fold(col_x.last().copied().unwrap_or(0.0), f64::max);
        let table_height = if let Some((_, er)) = split_row_range {
            row_y[er].max(0.0)
        } else {
            row_y.last().copied().unwrap_or(0.0)
        };

        // ── 3. 위치 결정 ──
        let pw = self.current_paper_width.get();
        let paper_w = if pw > 0.0 { Some(pw) } else { None };
        let mut table_x = self.compute_table_x_position(
            table, table_width, col_area, depth, host_alignment, host_margin_left, host_margin_right, inline_x_override, paper_w,
        );

        let (caption_height, caption_spacing) = if depth == 0 {
            let ch = self.calculate_caption_height(&table.caption, styles);
            let cs = table.caption.as_ref()
                .map(|c| hwpunit_to_px(c.spacing as i32, self.dpi))
                .unwrap_or(0.0);
            (ch, cs)
        } else {
            (0.0, 0.0)
        };

        // Left 캡션: 표를 캡션 크기만큼 오른쪽으로 이동
        if depth == 0 {
            if let Some(ref cap) = table.caption {
                if matches!(cap.direction, crate::model::shape::CaptionDirection::Left) {
                    let cap_w = hwpunit_to_px(cap.width as i32, self.dpi);
                    table_x += cap_w + caption_spacing;
                }
            }
        }

        let table_text_wrap = if depth == 0 { table.common.text_wrap } else { crate::model::shape::TextWrap::Square };
        let inline_top_caption_offset = if inline_x_override.is_some() && depth == 0 {
            if let Some(ref caption) = table.caption {
                use crate::model::shape::CaptionDirection;
                if matches!(caption.direction, CaptionDirection::Top) {
                    caption_height + caption_spacing
                } else {
                    0.0
                }
            } else {
                0.0
            }
        } else {
            0.0
        };

        // inline_x_override가 있으면 외부에서 inline 위치를 계산했으므로 x/y 기준은 유지한다.
        // 단, Top 캡션은 표 본문 위의 별도 영역이므로 표 본문 y 에 캡션 높이만큼 반영한다.
        let table_y = if inline_x_override.is_some() {
            y_start + inline_top_caption_offset
        } else {
            self.compute_table_y_position(
                table, table_height, y_start, col_area, depth, caption_height, caption_spacing,
                para_y,
            ) - split_y_offset
        };

        // ── 4. 표 노드 생성 ──
        let table_id = tree.next_id();
        let mut table_node = RenderNode::new(
            table_id,
            RenderNodeType::Table(TableNode {
                row_count: table.row_count,
                col_count: table.col_count,
                border_fill_id: table.border_fill_id,
                section_index: Some(section_index),
                para_index: table_meta.map(|(pi, _)| pi),
                control_index: table_meta.map(|(_, ci)| ci),
            }),
            BoundingBox::new(table_x, table_y, table_width, table_height),
        );

        // ── 4-1. 표 배경 렌더링 (표 > 배경 > 색 > 면색) ──
        if table.border_fill_id > 0 {
            let tbl_idx = (table.border_fill_id as usize).saturating_sub(1);
            if let Some(tbl_bs) = styles.border_styles.get(tbl_idx) {
                self.render_cell_background(
                    tree, &mut table_node, Some(tbl_bs),
                    table_x, table_y, table_width, table_height,
                    bin_data_content,
                );
            }
        }

        // ── 4-2. cellzone 배경 렌더링 (zone 전체 영역에 한 번) ──
        for zone in &table.zones {
            if zone.border_fill_id == 0 { continue; }
            let zone_idx = (zone.border_fill_id as usize).saturating_sub(1);
            if let Some(zone_bs) = styles.border_styles.get(zone_idx) {
                // zone 영역의 좌표 계산
                let sc = zone.start_col as usize;
                let ec = (zone.end_col as usize + 1).min(col_count);
                let sr = zone.start_row as usize;
                let er = (zone.end_row as usize + 1).min(row_count);
                if sc < col_count && sr < row_count {
                    let zone_x = table_x + row_col_x.get(sr).and_then(|r| r.get(sc)).copied().unwrap_or(0.0);
                    let zone_y = table_y + row_y.get(sr).copied().unwrap_or(0.0);
                    let zone_x_end = table_x + row_col_x.get(sr).and_then(|r| {
                        if ec < r.len() { Some(r[ec]) } else { r.last().map(|&last_x| {
                            // 마지막 열 끝 = 마지막 열 시작 + 해당 셀 너비
                            let last_col = r.len() - 1;
                            table.cells.iter()
                                .find(|c| c.row as usize == sr && c.col as usize == last_col)
                                .map(|c| last_x + hwpunit_to_px(c.width as i32, self.dpi))
                                .unwrap_or(last_x)
                        })}
                    }).unwrap_or(0.0);
                    let zone_y_end = table_y + row_y.get(er).copied().unwrap_or_else(|| {
                        // 마지막 행 끝 = 마지막 행 시작 + 해당 행 높이
                        row_y.get(er - 1).copied().unwrap_or(0.0) + table.row_sizes.get(er - 1).map(|&h| hwpunit_to_px(h as i32, self.dpi)).unwrap_or(0.0)
                    });
                    let zone_w = (zone_x_end - zone_x).max(0.0);
                    let zone_h = (zone_y_end - zone_y).max(0.0);
                    // [Task #429] 단색/패턴/그라데이션 + 이미지 채우기 (zone 의 별도 image fill 처리는
                    // render_cell_background 가 통합 처리하므로 제거)
                    self.render_cell_background(tree, &mut table_node, Some(zone_bs), zone_x, zone_y, zone_w, zone_h, bin_data_content);
                }
            }
        }

        // ── 5. 셀 레이아웃 ──
        let mut h_edges: Vec<Vec<Option<BorderLine>>> = vec![vec![None; col_count]; row_count + 1];
        let mut v_edges: Vec<Vec<Option<BorderLine>>> = vec![vec![None; row_count]; col_count + 1];

        self.layout_table_cells(
            tree, &mut table_node, table,
            section_index, styles, col_area, bin_data_content,
            depth, table_meta, enclosing_cell_ctx,
            &row_col_x, &row_y, col_count, row_count,
            table_x, table_y,
            &mut h_edges, &mut v_edges,
            split_row_range, row_y_shift,
        );


        // ── 5-1. 표 전체 외곽 테두리 보충 ──
        // 셀 테두리만으로는 표 외곽이 비어있을 수 있음.
        // 셀이 해당 외곽 엣지를 커버하지 않는 곳에만 table.border_fill_id fallback 적용.
        // (셀이 존재하지만 의도적으로 테두리를 없앤 곳에는 적용하지 않음)
        if table.border_fill_id > 0 {
            let tbl_idx = (table.border_fill_id as usize).saturating_sub(1);
            if let Some(tbl_bs) = styles.border_styles.get(tbl_idx) {
                let borders = &tbl_bs.borders; // [left, right, top, bottom]

                // 셀이 커버하는 외곽 엣지 맵 구축
                let mut h_covered = vec![vec![false; col_count]; row_count + 1];
                let mut v_covered = vec![vec![false; row_count]; col_count + 1];
                for cell in &table.cells {
                    let c = cell.col as usize;
                    let r = cell.row as usize;
                    if c >= col_count || r >= row_count { continue; }
                    let ec = (c + cell.col_span as usize).min(col_count);
                    let er = (r + cell.row_span as usize).min(row_count);
                    // 상단
                    if r == 0 { for cc in c..ec { h_covered[0][cc] = true; } }
                    // 하단
                    if er == row_count { for cc in c..ec { h_covered[row_count][cc] = true; } }
                    // 좌측
                    if c == 0 { for rr in r..er { v_covered[0][rr] = true; } }
                    // 우측
                    if ec == col_count { for rr in r..er { v_covered[col_count][rr] = true; } }
                }

                // 셀이 커버하지 않는 외곽 엣지에만 fallback 적용
                for c in 0..col_count {
                    if h_edges[0][c].is_none() && !h_covered[0][c] {
                        let b = &borders[2];
                        if !matches!(b.line_type, crate::model::style::BorderLineType::None) && b.width > 0 {
                            h_edges[0][c] = Some(*b);
                        }
                    }
                    if h_edges[row_count][c].is_none() && !h_covered[row_count][c] {
                        let b = &borders[3];
                        if !matches!(b.line_type, crate::model::style::BorderLineType::None) && b.width > 0 {
                            h_edges[row_count][c] = Some(*b);
                        }
                    }
                }
                for r in 0..row_count {
                    if v_edges[0][r].is_none() && !v_covered[0][r] {
                        let b = &borders[0];
                        if !matches!(b.line_type, crate::model::style::BorderLineType::None) && b.width > 0 {
                            v_edges[0][r] = Some(*b);
                        }
                    }
                    if v_edges[col_count][r].is_none() && !v_covered[col_count][r] {
                        let b = &borders[1];
                        if !matches!(b.line_type, crate::model::style::BorderLineType::None) && b.width > 0 {
                            v_edges[col_count][r] = Some(*b);
                        }
                    }
                }
            }
        }

        // ── 6. 테두리 렌더링 ──
        table_node.children.extend(render_edge_borders(
            tree, &h_edges, &v_edges, &row_col_x, &row_y, table_x, table_y,
        ));
        if self.show_transparent_borders.get() {
            table_node.children.extend(render_transparent_borders(
                tree, &h_edges, &v_edges, &row_col_x, &row_y, table_x, table_y,
            ));
        }

        col_node.children.push(table_node);

        // ── 7. 캡션 렌더링 ──
        if depth == 0 {
            if let Some(ref caption) = table.caption {
                use crate::model::shape::{CaptionDirection, CaptionVertAlign};
                let (cap_x, cap_w, cap_y) = match caption.direction {
                    CaptionDirection::Top => (table_x, table_width, y_start),
                    CaptionDirection::Bottom => (table_x, table_width, table_y + table_height + caption_spacing),
                    CaptionDirection::Left | CaptionDirection::Right => {
                        let cw = hwpunit_to_px(caption.width as i32, self.dpi);
                        let cx = if caption.direction == CaptionDirection::Left {
                            table_x - cw - caption_spacing
                        } else {
                            table_x + table_width + caption_spacing
                        };
                        let cy = match caption.vert_align {
                            CaptionVertAlign::Top => table_y,
                            CaptionVertAlign::Center => table_y + (table_height - caption_height).max(0.0) / 2.0,
                            CaptionVertAlign::Bottom => table_y + (table_height - caption_height).max(0.0),
                        };
                        (cx, cw, cy)
                    }
                };
                let cap_cell_ctx = table_meta.map(|(pi, ci)| CellContext {
                    parent_para_index: pi,
                    path: vec![CellPathEntry {
                        control_index: ci,
                        cell_index: 65534, // 캡션 식별 센티널
                        cell_para_index: 0,
                        text_direction: 0,
                    }],
                });
                self.layout_caption(
                    tree, col_node, caption, styles, col_area,
                    cap_x, cap_w, cap_y,
                    &mut self.auto_counter.borrow_mut(),
                    cap_cell_ctx,
                );
            }
        }

        // ── 8. 반환값 ──
        if depth == 0 {
            // Left/Right 캡션은 표 높이에 영향 없음
            let is_lr_cap = table.caption.as_ref().map_or(false, |c| {
                use crate::model::shape::CaptionDirection;
                matches!(c.direction, CaptionDirection::Left | CaptionDirection::Right)
            });
            let caption_extra = if is_lr_cap {
                0.0
            } else {
                caption_height + if caption_height > 0.0 { caption_spacing } else { 0.0 }
            };
            if matches!(table_text_wrap, crate::model::shape::TextWrap::BehindText | crate::model::shape::TextWrap::InFrontOfText) {
                // 글뒤로/글앞으로: y_offset 변경 없음
                y_start
            } else if matches!(table_text_wrap, crate::model::shape::TextWrap::TopAndBottom) && !table.common.treat_as_char {
                // 자리차지: 표 아래쪽까지 y_offset 진행 (절대 위치 기준)
                let table_bottom = table_y + table_height + caption_extra;
                table_bottom.max(y_start)
            } else {
                let total_height = table_height + caption_extra;
                y_start + total_height
            }
        } else {
            // 중첩 표: outer_margin 포함 높이 반환
            let om_top = hwpunit_to_px(table.outer_margin_top as i32, self.dpi);
            let om_bottom = hwpunit_to_px(table.outer_margin_bottom as i32, self.dpi);
            (table_height + om_top + om_bottom).max(0.0)
        }
    }

    /// 열 폭 계산 (단일 셀 + 병합 셀 해결)
    pub(crate) fn resolve_column_widths(
        &self,
        table: &crate::model::table::Table,
        col_count: usize,
    ) -> Vec<f64> {
        // 1단계: col_span==1인 셀에서 개별 열 폭 추출
        let mut col_widths = vec![0.0f64; col_count];
        for cell in &table.cells {
            if cell.col_span == 1 && (cell.col as usize) < col_count {
                let w = hwpunit_to_px(cell.width as i32, self.dpi);
                if w > col_widths[cell.col as usize] {
                    col_widths[cell.col as usize] = w;
                }
            }
        }

        // 2단계: 병합 셀에서 미지 열 폭을 반복적으로 해결
        {
            let mut constraints: Vec<(usize, usize, f64)> = Vec::new();
            for cell in &table.cells {
                let c = cell.col as usize;
                let span = cell.col_span as usize;
                if span > 1 && c + span <= col_count {
                    let total_w = hwpunit_to_px(cell.width as i32, self.dpi);
                    if let Some(existing) = constraints.iter_mut().find(|x| x.0 == c && x.1 == span) {
                        if total_w > existing.2 { existing.2 = total_w; }
                    } else {
                        constraints.push((c, span, total_w));
                    }
                }
            }
            constraints.sort_by_key(|&(_, span, _)| span);

            let max_iter = col_count + constraints.len();
            for _ in 0..max_iter {
                let mut progress = false;
                for &(c, span, total_w) in &constraints {
                    let known_sum: f64 = (c..c + span).map(|i| col_widths[i]).sum();
                    let unknown_cols: Vec<usize> = (c..c + span)
                        .filter(|&i| col_widths[i] == 0.0)
                        .collect();
                    if unknown_cols.len() == 1 {
                        let remaining = (total_w - known_sum).max(0.0);
                        col_widths[unknown_cols[0]] = remaining;
                        progress = true;
                    }
                }
                if !progress { break; }
            }

            for &(c, span, total_w) in &constraints {
                let known_sum: f64 = (c..c + span).map(|i| col_widths[i]).sum();
                let unknown_cols: Vec<usize> = (c..c + span)
                    .filter(|&i| col_widths[i] == 0.0)
                    .collect();
                if !unknown_cols.is_empty() {
                    let remaining = (total_w - known_sum).max(0.0);
                    let per_col = remaining / unknown_cols.len() as f64;
                    for i in unknown_cols {
                        col_widths[i] = per_col;
                    }
                }
            }
        }

        // 3단계: 여전히 폭이 0인 열에 기본값 할당
        for c in 0..col_count {
            if col_widths[c] <= 0.0 {
                col_widths[c] = hwpunit_to_px(1800, self.dpi);
            }
        }
        col_widths
    }

    /// 행 높이 계산 (MeasuredTable 우선, 없으면 셀/병합/컨텐츠 기반)
    pub(crate) fn resolve_row_heights(
        &self,
        table: &crate::model::table::Table,
        col_count: usize,
        row_count: usize,
        measured_table: Option<&MeasuredTable>,
        styles: &ResolvedStyleSet,
    ) -> Vec<f64> {
        if let Some(mt) = measured_table {
            let mut rh = mt.row_heights.clone();
            rh.resize(row_count, hwpunit_to_px(400, self.dpi));
            return rh;
        }

        // 1단계: row_span==1인 셀에서 개별 행 높이 추출
        let mut row_heights = vec![0.0f64; row_count];
        for cell in &table.cells {
            if cell.row_span == 1 && (cell.row as usize) < row_count {
                let r = cell.row as usize;
                if cell.height < 0x80000000 {
                    let h = hwpunit_to_px(cell.height as i32, self.dpi);
                    if h > row_heights[r] {
                        row_heights[r] = h;
                    }
                }
            }
        }

        // 1-b단계: 셀 내 실제 컨텐츠 높이 계산
        for cell in &table.cells {
            if cell.row_span == 1 && (cell.row as usize) < row_count {
                let r = cell.row as usize;
                let (_, _, pad_top, pad_bottom) = self.resolve_cell_padding(cell, table);

                let content_height = if cell.text_direction != 0 {
                    // 세로쓰기: line_seg.segment_width가 열의 세로 길이
                    self.calc_vertical_cell_content_height(&cell.paragraphs)
                } else {
                    self.calc_cell_paragraphs_content_height(&cell.paragraphs, styles)
                };
                // LINE_SEG의 line_height에 이미 셀 내 중첩 표 높이가 반영되어 있으므로
                // controls_height를 별도로 더하면 이중 계산됨
                let required_height = content_height + pad_top + pad_bottom;
                if required_height > row_heights[r] {
                    row_heights[r] = required_height;
                }
            }
        }

        // 2단계: 병합 셀에서 미지 행 높이를 반복적으로 해결
        {
            let mut constraints: Vec<(usize, usize, f64)> = Vec::new();
            for cell in &table.cells {
                let r = cell.row as usize;
                let span = cell.row_span as usize;
                if span > 1 && r + span <= row_count && cell.height < 0x80000000 {
                    let total_h = hwpunit_to_px(cell.height as i32, self.dpi);
                    if let Some(existing) = constraints.iter_mut().find(|x| x.0 == r && x.1 == span) {
                        if total_h > existing.2 { existing.2 = total_h; }
                    } else {
                        constraints.push((r, span, total_h));
                    }
                }
            }
            constraints.sort_by_key(|&(_, span, _)| span);
            let max_iter = row_count + constraints.len();
            for _ in 0..max_iter {
                let mut progress = false;
                for &(r, span, total_h) in &constraints {
                    let known_sum: f64 = (r..r + span).map(|i| row_heights[i]).sum();
                    let unknown_rows: Vec<usize> = (r..r + span)
                        .filter(|&i| row_heights[i] == 0.0)
                        .collect();
                    if unknown_rows.len() == 1 {
                        let remaining = (total_h - known_sum).max(0.0);
                        row_heights[unknown_rows[0]] = remaining;
                        progress = true;
                    }
                }
                if !progress { break; }
            }
            for &(r, span, total_h) in &constraints {
                let known_sum: f64 = (r..r + span).map(|i| row_heights[i]).sum();
                let unknown_rows: Vec<usize> = (r..r + span)
                    .filter(|&i| row_heights[i] == 0.0)
                    .collect();
                if !unknown_rows.is_empty() {
                    let remaining = (total_h - known_sum).max(0.0);
                    let per_row = remaining / unknown_rows.len() as f64;
                    for i in unknown_rows {
                        row_heights[i] = per_row;
                    }
                }
            }
        }

        // 2-b단계: 병합 셀 컨텐츠 높이 > 결합 행 높이이면 마지막 행 확장
        for cell in &table.cells {
            let r = cell.row as usize;
            let span = cell.row_span as usize;
            if span > 1 && r + span <= row_count {
                let (_, _, pad_top, pad_bottom) = self.resolve_cell_padding(cell, table);
                let content_height = self.calc_cell_paragraphs_content_height(&cell.paragraphs, styles);
                // LINE_SEG의 line_height에 이미 셀 내 중첩 표 높이가 반영되어 있으므로
                // controls_height를 별도로 더하면 이중 계산됨
                let required_height = content_height + pad_top + pad_bottom;
                let combined: f64 = (r..r + span).map(|i| row_heights[i]).sum();
                if required_height > combined {
                    let deficit = required_height - combined;
                    row_heights[r + span - 1] += deficit;
                }
            }
        }

        // 3단계: 높이 0인 행에 기본값
        for r in 0..row_count {
            if row_heights[r] <= 0.0 {
                row_heights[r] = hwpunit_to_px(400, self.dpi);
            }
        }
        row_heights
    }

    /// 셀 문단들의 콘텐츠 높이 합산 (spacing + line_height + line_spacing)
    pub(crate) fn calc_cell_paragraphs_content_height(
        &self,
        paragraphs: &[Paragraph],
        styles: &ResolvedStyleSet,
    ) -> f64 {
        let cell_para_count = paragraphs.len();
        paragraphs.iter()
            .enumerate()
            .map(|(pidx, p)| {
                let comp = compose_paragraph(p);
                self.calc_para_lines_height(&comp.lines, pidx, cell_para_count,
                    styles.para_styles.get(p.para_shape_id as usize))
            })
            .sum()
    }

    /// pre-composed 문단들의 콘텐츠 높이 합산 (compose 생략)
    pub(crate) fn calc_composed_paras_content_height(
        &self,
        composed_paras: &[ComposedParagraph],
        paragraphs: &[Paragraph],
        styles: &ResolvedStyleSet,
    ) -> f64 {
        let cell_para_count = paragraphs.len();
        composed_paras.iter()
            .zip(paragraphs.iter())
            .enumerate()
            .map(|(pidx, (comp, para))| {
                self.calc_para_lines_height(&comp.lines, pidx, cell_para_count,
                    styles.para_styles.get(para.para_shape_id as usize))
            })
            .sum()
    }

    /// 단일 문단의 줄 높이 합산 (공통 로직)
    fn calc_para_lines_height(
        &self,
        lines: &[crate::renderer::composer::ComposedLine],
        pidx: usize,
        total_para_count: usize,
        para_style: Option<&crate::renderer::style_resolver::ResolvedParaStyle>,
    ) -> f64 {
        let is_last_para = pidx + 1 == total_para_count;
        let spacing_before = if pidx > 0 {
            para_style.map(|s| s.spacing_before).unwrap_or(0.0)
        } else {
            0.0
        };
        let spacing_after = if !is_last_para {
            para_style.map(|s| s.spacing_after).unwrap_or(0.0)
        } else {
            0.0
        };
        if lines.is_empty() {
            spacing_before + hwpunit_to_px(400, self.dpi) + spacing_after
        } else {
            let line_count = lines.len();
            let lines_total: f64 = lines.iter()
                .enumerate()
                .map(|(i, line)| {
                    let h = hwpunit_to_px(line.line_height, self.dpi);
                    let is_cell_last_line = is_last_para && i + 1 == line_count;
                    if !is_cell_last_line {
                        h + hwpunit_to_px(line.line_spacing, self.dpi)
                    } else {
                        h
                    }
                })
                .sum();
            spacing_before + lines_total + spacing_after
        }
    }

    /// 세로쓰기 셀의 콘텐츠 높이 계산
    /// 세로쓰기에서 line_seg.segment_width = 열의 세로 길이 (HWPUNIT)
    /// 셀 높이 = 최대 segment_width
    fn calc_vertical_cell_content_height(
        &self,
        paragraphs: &[Paragraph],
    ) -> f64 {
        let mut max_seg_height: f64 = 0.0;
        for para in paragraphs {
            for ls in &para.line_segs {
                let h = hwpunit_to_px(ls.segment_width, self.dpi);
                if h > max_seg_height {
                    max_seg_height = h;
                }
            }
        }
        if max_seg_height <= 0.0 {
            // fallback: 기본 높이
            hwpunit_to_px(400, self.dpi)
        } else {
            max_seg_height
        }
    }

    /// 셀 패딩 계산 (cell.padding이 0이면 table.padding fallback)
    pub(crate) fn resolve_cell_padding(
        &self,
        cell: &crate::model::table::Cell,
        table: &crate::model::table::Table,
    ) -> (f64, f64, f64, f64) {
        // HWP 스펙: aim(apply_inner_margin)=true → cell.padding,
        //           aim=false → table.padding 우선.
        // Task #347: 단, aim=false에서도 cell.padding이 table.padding보다
        // 큰 비대칭 값이면 작성자 의도(예: KTX 목차 R=1417 HU)로 보고 그 축만 cell 사용.
        // (Task #279의 "전 축에서 cell 우선" 휴리스틱은 일반 박스 셀에서 표 padding을
        // 무시해 텍스트가 왼쪽으로 붙어버리는 부작용이 있어 축소 적용.)
        let prefer_cell_axis = |c: i16, t: i16| -> bool {
            if cell.apply_inner_margin {
                c != 0
            } else {
                // aim=false: cell이 table보다 명백히 큰 경우만 cell 우선 (의도된 비대칭)
                (c as i32) > (t as i32)
            }
        };
        let pad_left = if prefer_cell_axis(cell.padding.left, table.padding.left) {
            hwpunit_to_px(cell.padding.left as i32, self.dpi)
        } else {
            hwpunit_to_px(table.padding.left as i32, self.dpi)
        };
        let pad_right = if prefer_cell_axis(cell.padding.right, table.padding.right) {
            hwpunit_to_px(cell.padding.right as i32, self.dpi)
        } else {
            hwpunit_to_px(table.padding.right as i32, self.dpi)
        };
        let pad_top = if prefer_cell_axis(cell.padding.top, table.padding.top) {
            hwpunit_to_px(cell.padding.top as i32, self.dpi)
        } else {
            hwpunit_to_px(table.padding.top as i32, self.dpi)
        };
        let pad_bottom = if prefer_cell_axis(cell.padding.bottom, table.padding.bottom) {
            hwpunit_to_px(cell.padding.bottom as i32, self.dpi)
        } else {
            hwpunit_to_px(table.padding.bottom as i32, self.dpi)
        };
        // [Task #501] 한컴 방어 로직 모방 — cell.padding.top + bottom 합산이
        // cell.height 자체를 초과하면 (mel-001 p2 셀[21]: pad=1700 HU 두 축, h=1280 HU)
        // 한컴은 자체 가드로 cell 안에 콘텐츠가 들어가도록 처리. cell.height 의 절반까지
        // 비례 축소 (HWP 스펙 외 한컴 동작 모방).
        let (pad_top, pad_bottom) = if cell.height < 0x80000000 {
            let cell_h_px = hwpunit_to_px(cell.height as i32, self.dpi);
            let total_v_pad = pad_top + pad_bottom;
            if cell_h_px > 0.0 && total_v_pad >= cell_h_px {
                let max_v_pad = cell_h_px * 0.5;
                let scale = max_v_pad / total_v_pad;
                (pad_top * scale, pad_bottom * scale)
            } else {
                (pad_top, pad_bottom)
            }
        } else {
            (pad_top, pad_bottom)
        };
        (pad_left, pad_right, pad_top, pad_bottom)
    }

    /// 셀 텍스트가 오버플로우할 때 좌우 패딩을 축소하여 공간을 확보한다.
    /// composed 문단의 각 줄 텍스트 폭을 측정하여 최대값이 가용 폭을 초과하면
    /// 패딩을 비례 축소한다 (최소 1px 보장).
    pub(crate) fn shrink_cell_padding_for_overflow(
        &self,
        pad_left: f64,
        pad_right: f64,
        cell_w: f64,
        composed_paras: &[ComposedParagraph],
        styles: &ResolvedStyleSet,
    ) -> (f64, f64) {
        let mut max_line_w = 0.0f64;
        for comp in composed_paras {
            for line in &comp.lines {
                let mut w = 0.0;
                for run in &line.runs {
                    let mut ts = resolved_to_text_style(styles, run.char_style_id, run.lang_index);
                    // 자연 폭 측정: 음수 자간을 제거하여 글리프가 서로 겹치지 않는 최소 폭을 얻음
                    if ts.letter_spacing < 0.0 {
                        ts.letter_spacing = 0.0;
                    }
                    // [Task #555] PUA 옛한글 변환 후 자모 시퀀스 폭 사용.
                    w += estimate_text_width(effective_text_for_metrics(run), &ts);
                }
                if w > max_line_w {
                    max_line_w = w;
                }
            }
        }
        let available = (cell_w - pad_left - pad_right).max(0.0);
        // Task #347: estimate_text_width는 영어 본문(Times New Roman 등) 자연 폭을
        // 5~15%까지 과대 추정할 수 있어, HWP가 이미 줄바꿈한 본문에서도
        // padding 축소가 잘못 트리거됨. 15% 이내 초과는 정상으로 보고 미축소.
        let overflow_threshold = available * 1.15;
        if max_line_w <= overflow_threshold || cell_w <= 2.0 {
            return (pad_left, pad_right);
        }
        let min_pad = 1.0;
        let total_pad = pad_left + pad_right;
        let max_reducible = (total_pad - 2.0 * min_pad).max(0.0);
        if max_reducible <= 0.0 {
            return (pad_left, pad_right);
        }
        let deficit = max_line_w - available;
        let reduction = deficit.min(max_reducible);
        let new_total = total_pad - reduction;
        let new_left = if total_pad > 0.0 {
            pad_left * new_total / total_pad
        } else {
            new_total / 2.0
        };
        let new_right = new_total - new_left;
        (new_left, new_right)
    }

    /// 셀 배경 렌더링 (fill_color + pattern + gradient)
    pub(crate) fn render_cell_background(
        &self,
        tree: &mut PageRenderTree,
        cell_node: &mut RenderNode,
        border_style: Option<&crate::renderer::style_resolver::ResolvedBorderStyle>,
        cell_x: f64, cell_y: f64, cell_w: f64, cell_h: f64,
        bin_data_content: &[BinDataContent],
    ) {
        let fill_color = border_style.and_then(|bs| bs.fill_color);
        let pattern = border_style.and_then(|bs| bs.pattern);
        let gradient = border_style.and_then(|bs| bs.gradient.clone());
        if fill_color.is_some() || gradient.is_some() || pattern.is_some() {
            let rect_id = tree.next_id();
            let rect_node = RenderNode::new(
                rect_id,
                RenderNodeType::Rectangle(RectangleNode::new(
                    0.0,
                    ShapeStyle {
                        fill_color,
                        pattern,
                        stroke_color: None,
                        stroke_width: 0.0,
                        ..Default::default()
                    },
                    gradient,
                )),
                BoundingBox::new(cell_x, cell_y, cell_w, cell_h),
            );
            cell_node.children.push(rect_node);
        }
        // [Task #429] image fill 처리 — zone 처리와 동일 패턴
        if let Some(img_fill) = border_style.and_then(|bs| bs.image_fill.as_ref()) {
            if let Some(img_content) = crate::renderer::layout::find_bin_data(bin_data_content, img_fill.bin_data_id) {
                let img_id = tree.next_id();
                let img_node = RenderNode::new(
                    img_id,
                    RenderNodeType::Image(ImageNode {
                        fill_mode: Some(img_fill.fill_mode),
                        ..ImageNode::new(img_fill.bin_data_id, Some(img_content.data.clone()))
                    }),
                    BoundingBox::new(cell_x, cell_y, cell_w, cell_h),
                );
                cell_node.children.push(img_node);
            }
        }
    }

    /// 표 수평 위치 결정
    pub(crate) fn compute_table_x_position(
        &self,
        table: &crate::model::table::Table,
        table_width: f64,
        col_area: &LayoutRect,
        depth: usize,
        host_alignment: Alignment,
        host_margin_left: f64,
        host_margin_right: f64,
        inline_x_override: Option<f64>,
        paper_width: Option<f64>,
    ) -> f64 {
        if let Some(ix) = inline_x_override {
            // inline_x_override: 外部(テキストフロー)で既に正しい位置が計算済み
            // TAC表のh_offsetはテキストフロー位置には不要 (非TAC表のみ加算)
            if table.common.treat_as_char {
                ix
            } else {
                let h_offset = hwpunit_to_px(table.common.horizontal_offset as i32, self.dpi);
                ix + h_offset
            }
        } else if depth == 0 && table.common.treat_as_char {
            // 글자처럼 취급(treat_as_char)
            // TAC 표의 위치는 텍스트 플로우에 의해 결정되므로 h_offset 미적용
            let ref_x = col_area.x + host_margin_left;
            let ref_w = col_area.width - host_margin_left - host_margin_right;
            match host_alignment {
                Alignment::Center | Alignment::Distribute => ref_x + (ref_w - table_width).max(0.0) / 2.0,
                Alignment::Right => ref_x + (ref_w - table_width).max(0.0),
                _ => ref_x,
            }
        } else if depth == 0 {
            // 표 자체 위치 속성
            let horz_rel_to = table.common.horz_rel_to;
            let horz_align = table.common.horz_align;
            let h_offset = hwpunit_to_px(table.common.horizontal_offset as i32, self.dpi);
            let (ref_x, ref_w) = match horz_rel_to {
                HorzRelTo::Paper => {
                    let paper_w = paper_width.unwrap_or({
                        // fallback: col_area 기반 추정 (paper_width 미전달 시)
                        if table_width > col_area.width {
                            col_area.x * 2.0 + table_width
                        } else {
                            col_area.x * 2.0 + col_area.width
                        }
                    });
                    (0.0, paper_w)
                }
                HorzRelTo::Page => {
                    // Task #347: 본문 영역(body_area) 기준. 미설정 시 col_area 폴백.
                    let body = self.current_body_area.get();
                    if body.2 > 0.0 { (body.0, body.2) } else { (col_area.x, col_area.width) }
                }
                HorzRelTo::Para => (col_area.x + host_margin_left, col_area.width - host_margin_left),
                _ => (col_area.x, col_area.width),
            };
            match horz_align {
                HorzAlign::Left | HorzAlign::Inside => ref_x + h_offset,
                HorzAlign::Center => ref_x + (ref_w - table_width).max(0.0) / 2.0 + h_offset,
                // Task #347: picture_footnote.rs:185와 동일하게 - h_offset (오른쪽 끝에서 안쪽으로 오프셋).
                HorzAlign::Right | HorzAlign::Outside => ref_x + (ref_w - table_width).max(0.0) - h_offset,
            }
        } else {
            // 중첩 표: outer_margin_left 적용 + host_alignment에 따라 셀 내에서 정렬
            let om_left = hwpunit_to_px(table.outer_margin_left as i32, self.dpi);
            let area_x = col_area.x + om_left;
            let area_w = (col_area.width - om_left).max(0.0);
            match host_alignment {
                Alignment::Center | Alignment::Distribute => area_x + (area_w - table_width).max(0.0) / 2.0,
                Alignment::Right => area_x + (area_w - table_width).max(0.0),
                _ => area_x,
            }
        }
    }

    /// 표 세로 위치 결정 (text_wrap + v_offset + 캡션)
    fn compute_table_y_position(
        &self,
        table: &crate::model::table::Table,
        table_height: f64,
        y_start: f64,
        col_area: &LayoutRect,
        depth: usize,
        caption_height: f64,
        caption_spacing: f64,
        para_y: Option<f64>,
    ) -> f64 {
        let table_treat_as_char = table.common.treat_as_char;
        let table_text_wrap = if depth == 0 { table.common.text_wrap } else { crate::model::shape::TextWrap::Square };

        if depth == 0 && !table_treat_as_char && matches!(table_text_wrap, crate::model::shape::TextWrap::TopAndBottom | crate::model::shape::TextWrap::BehindText | crate::model::shape::TextWrap::InFrontOfText) {
            // 자리차지(1) / 글뒤로(2) / 글앞으로(3): v_offset 기반 절대 위치
            
            
            let v_offset = hwpunit_to_px(table.common.vertical_offset as i32, self.dpi);
            // 문단 기준일 때 para_y 사용 (같은 문단의 여러 표가 동일 기준점 공유)
            let anchor_y = para_y.unwrap_or(y_start);
            // bit 13: VertRelTo가 'para'일 때 본문 영역으로 제한
            
            let page_h_approx = col_area.y * 2.0 + col_area.height;
            let vert_rel_to = table.common.vert_rel_to;
            // Task #297: Page는 본문 영역(body area) 기준, Paper는 용지 전체 기준
            // (HWP 스펙: Page=쪽 본문, Paper=용지 전체). 바탕쪽 문맥에서는
            // col_area = paper_area이므로 두 경로 결과가 동일하여 회귀 없음.
            let (ref_y, ref_h) = match vert_rel_to {
                crate::model::shape::VertRelTo::Page => {
                    // Task #347: 본문 영역(body_area) 기준. 미설정 시 col_area 폴백.
                    let body = self.current_body_area.get();
                    if body.3 > 0.0 { (body.1, body.3) } else { (col_area.y, col_area.height) }
                }
                crate::model::shape::VertRelTo::Para => (anchor_y, col_area.height - (anchor_y - col_area.y).max(0.0)),
                crate::model::shape::VertRelTo::Paper => (0.0, page_h_approx),
            };
            // Top 캡션: 표 위치를 캡션 높이만큼 아래로 이동
            let caption_top_offset = if let Some(ref cap) = table.caption {
                use crate::model::shape::CaptionDirection;
                if matches!(cap.direction, CaptionDirection::Top) {
                    caption_height + if caption_height > 0.0 { caption_spacing } else { 0.0 }
                } else {
                    0.0
                }
            } else {
                0.0
            };
            let vert_align = table.common.vert_align;
            let raw_y = match vert_align {
                crate::model::shape::VertAlign::Top | crate::model::shape::VertAlign::Inside => ref_y + v_offset + caption_top_offset,
                crate::model::shape::VertAlign::Center => ref_y + (ref_h - table_height) / 2.0 + v_offset + caption_top_offset,
                crate::model::shape::VertAlign::Bottom | crate::model::shape::VertAlign::Outside => ref_y + ref_h - table_height - v_offset + caption_top_offset,
            };
            // Para 기준 + bit 13: 본문 영역으로 제한
            // 앞선 표/텍스트가 차지한 영역(y_start) 아래로 밀어내고, 본문 영역 내로 클램핑
            // Task #347: TopAndBottom 만 y_start 이하로 밀어냄. 글뒤로(BehindText) /
            // 글앞으로(InFrontOfText) 표는 절대 위치 오버레이이므로 push-down 미적용.
            if matches!(vert_rel_to, crate::model::shape::VertRelTo::Para) {
                let body_top = col_area.y;
                let body_bottom = col_area.y + col_area.height - table_height;
                let pushed = if matches!(table_text_wrap, crate::model::shape::TextWrap::TopAndBottom) {
                    raw_y.max(y_start)
                } else {
                    raw_y
                };
                pushed.clamp(body_top, body_bottom.max(body_top))
            } else {
                raw_y
            }
        } else if depth == 0 {
            let v_offset = if table_treat_as_char {
                hwpunit_to_px(table.common.vertical_offset as i32, self.dpi)
            } else { 0.0 };
            if let Some(ref caption) = table.caption {
                use crate::model::shape::CaptionDirection;
                if matches!(caption.direction, CaptionDirection::Top) {
                    y_start + caption_height + caption_spacing + v_offset
                } else {
                    y_start + v_offset
                }
            } else {
                y_start + v_offset
            }
        } else {
            // 중첩 표: outer_margin_top 적용
            let om_top = hwpunit_to_px(table.outer_margin_top as i32, self.dpi);
            y_start + om_top
        }
    }

    /// 각 셀 레이아웃 (배경, 패딩, 텍스트, 컨트롤, 테두리)
    #[allow(clippy::too_many_arguments)]
    fn layout_table_cells(
        &self,
        tree: &mut PageRenderTree,
        table_node: &mut RenderNode,
        table: &crate::model::table::Table,
        section_index: usize,
        styles: &ResolvedStyleSet,
        col_area: &LayoutRect,
        bin_data_content: &[BinDataContent],
        depth: usize,
        table_meta: Option<(usize, usize)>,
        enclosing_cell_ctx: Option<CellContext>,
        row_col_x: &[Vec<f64>],
        row_y: &[f64],
        col_count: usize,
        row_count: usize,
        table_x: f64,
        table_y: f64,
        h_edges: &mut Vec<Vec<Option<BorderLine>>>,
        v_edges: &mut Vec<Vec<Option<BorderLine>>>,
        row_filter: Option<(usize, usize)>,
        row_y_shift: f64,
    ) {
        for (cell_idx, cell) in table.cells.iter().enumerate() {
            let c = cell.col as usize;
            let r = cell.row as usize;
            if c >= col_count || r >= row_count {
                continue;
            }

            // 행 범위 필터: 보이는 행에 겹치지 않는 셀은 스킵
            let cell_end_row = (r + cell.row_span as usize).min(row_count);
            if let Some((sr, er)) = row_filter {
                if cell_end_row <= sr || r >= er {
                    continue;
                }
            }

            let cell_x = table_x + row_col_x[r][c];
            // row_y는 이미 시프트된 상태이므로 음수일 수 있음 (start_row 이전 행)
            // 행 스패닝 셀의 경우 table_y 이상으로 클램프
            let raw_cell_y = table_y + row_y[r];
            let cell_y = if row_filter.is_some() { raw_cell_y.max(table_y) } else { raw_cell_y };
            let end_col = (c + cell.col_span as usize).min(col_count);
            let end_row = (r + cell.row_span as usize).min(row_count);
            let cell_w = row_col_x[r][end_col] - row_col_x[r][c];
            let raw_cell_h = row_y[end_row] - row_y[r];
            let cell_h = if row_filter.is_some() {
                // 클램프된 y에 맞게 높이도 조정
                (raw_cell_h - (cell_y - raw_cell_y)).max(0.0)
            } else {
                raw_cell_h
            };

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
                    clip: true,
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

            // (a) 셀 배경
            self.render_cell_background(tree, &mut cell_node, border_style, cell_x, cell_y, cell_w, cell_h, bin_data_content);

            // 셀 패딩 (cell.padding이 0이면 table.padding fallback)
            let (mut pad_left, mut pad_right, pad_top, pad_bottom) = self.resolve_cell_padding(cell, table);

            let mut composed_paras: Vec<_> = cell.paragraphs.iter()
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

            // AutoNumber(Page) 치환: 셀 내 쪽번호 필드를 현재 페이지 번호로 변환
            let current_pn = self.current_page_number.get();
            if current_pn > 0 {
                for (cpi, para) in cell.paragraphs.iter().enumerate() {
                    let has_page_auto = para.controls.iter().any(|c|
                        matches!(c, Control::AutoNumber(an)
                            if an.number_type == crate::model::control::AutoNumberType::Page));
                    if has_page_auto {
                        let page_str = current_pn.to_string();
                        if let Some(comp) = composed_paras.get_mut(cpi) {
                            for line in &mut comp.lines {
                                for run in &mut line.runs {
                                    if run.text.contains('\u{0015}') {
                                        run.text = run.text.replace('\u{0015}', &page_str);
                                    } else if run.text.trim().is_empty() {
                                        run.text = page_str.clone();
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // 인라인 이미지/도형 최대 높이
            let mut max_inline_height: f64 = 0.0;

            // 수직 정렬용 콘텐츠 높이
            // (A) composed 기반: LINE_SEG line_height 합산 + 비인라인 도형/그림
            let total_content_height: f64 = {
                let mut text_height: f64 = self.calc_composed_paras_content_height(
                    &composed_paras, &cell.paragraphs, styles,
                );
                for para in &cell.paragraphs {
                    for ctrl in &para.controls {
                        match ctrl {
                            Control::Picture(pic) => {
                                let pic_h = hwpunit_to_px(pic.common.height as i32, self.dpi);
                                if pic.common.treat_as_char {
                                    if pic_h > max_inline_height {
                                        max_inline_height = pic_h;
                                    }
                                } else {
                                    text_height += pic_h;
                                }
                            }
                            Control::Shape(shape) => {
                                let shape_h = hwpunit_to_px(shape.common().height as i32, self.dpi);
                                if shape.common().treat_as_char {
                                    if shape_h > max_inline_height {
                                        max_inline_height = shape_h;
                                    }
                                } else {
                                    text_height += shape_h;
                                }
                            }
                            Control::Equation(eq) => {
                                let eq_h = hwpunit_to_px(eq.common.height as i32, self.dpi);
                                if eq.common.treat_as_char {
                                    if eq_h > max_inline_height {
                                        max_inline_height = eq_h;
                                    }
                                } else {
                                    text_height += eq_h;
                                }
                            }
                            Control::Table(t) => {
                                // 중첩 표 높이: 행 높이 합산
                                let nested_h = self.calc_nested_table_height(t, styles);
                                text_height += nested_h;
                            }
                            _ => {}
                        }
                    }
                }
                let composed_height = text_height.max(max_inline_height);

                // (B) vpos 기반: 마지막 문단의 vpos_end + 중첩 표 보정
                // LINE_SEG lh에 중첩 표 높이가 미반영된 경우를 보정
                let vpos_height = if cell.paragraphs.len() > 1 {
                    let last_para = cell.paragraphs.last().unwrap();
                    if let Some(seg) = last_para.line_segs.last() {
                        let mut last_end = seg.vertical_pos + seg.line_height;
                        // 마지막 문단에 중첩 표가 있고 lh가 표 높이보다 작으면 보정
                        for ctrl in &last_para.controls {
                            if let Control::Table(t) = ctrl {
                                let table_h = t.common.height as i32;
                                if table_h > seg.line_height {
                                    last_end += table_h - seg.line_height;
                                }
                            }
                        }
                        hwpunit_to_px(last_end, self.dpi)
                    } else {
                        0.0
                    }
                } else {
                    0.0
                };

                composed_height.max(vpos_height)
            };

            // 수직 정렬 (분할 표에서는 Top 강제 — 보이는 영역이 전체 셀보다 작음)
            let effective_valign = if row_filter.is_some() {
                VerticalAlign::Top
            } else {
                cell.vertical_align
            };
            // Task #347: HWP는 LineSeg.vertical_pos에 첫 줄의 절대 위치(셀 내부 컨텐츠 상단부터)
            // 를 기록한다. 이 값을 그대로 적용하면 모든 vertical_align (Top/Center/Bottom)에서
            // PDF와 일치하는 텍스트 시작 y가 자동으로 결정됨 (mechanical_offset 불필요).
            // 단, line_segs가 비어있는 케이스는 기존 mechanical_offset 폴백 유지.
            // [Task #362] 셀 안에 nested table 이 있는 경우 vpos 적용 제외.
            // nested table 케이스에서 LineSeg.vpos 가 셀 콘텐츠 시작 오프셋 의미가 아니라
            // 셀 안의 누적 위치로 사용되어, vpos 를 추가하면 콘텐츠가 표 높이를 초과하여 클립 발생.
            // (kps-ai p56 case: 외부 셀 vpos=2000HU 가 추가되어 19.5px 클립.)
            let has_nested_table = cell.paragraphs.iter()
                .any(|p| p.controls.iter().any(|c| matches!(c, Control::Table(_))));
            let first_line_vpos = cell.paragraphs.first()
                .and_then(|p| p.line_segs.first())
                .map(|ls| hwpunit_to_px(ls.vertical_pos, self.dpi));
            let text_y_start = if !has_nested_table && first_line_vpos.filter(|&v| v > 0.0).is_some() {
                // vpos는 셀 컨텐츠 상단(=cell_y+pad_top)으로부터의 첫 줄 top y 오프셋
                cell_y + pad_top + first_line_vpos.unwrap()
            } else {
                match effective_valign {
                    VerticalAlign::Top => cell_y + pad_top,
                    VerticalAlign::Center => {
                        let mechanical_offset = (inner_height - total_content_height).max(0.0) / 2.0;
                        cell_y + pad_top + mechanical_offset
                    }
                    VerticalAlign::Bottom => {
                        cell_y + pad_top + (inner_height - total_content_height).max(0.0)
                    }
                }
            };

            // 세로쓰기 셀
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
                    section_index, table_meta, cell_idx, enclosing_cell_ctx.clone(),
                );
            } else {

            let inner_area = LayoutRect {
                x: inner_x,
                y: text_y_start,
                width: inner_width,
                height: inner_height,
            };

            // 셀 내 문단 + 컨트롤 통합 레이아웃
            let mut para_y = text_y_start;
            let mut has_preceding_text = false;
            for (cp_idx, (composed, para)) in composed_paras.iter().zip(cell.paragraphs.iter()).enumerate() {
                let cell_context = if let Some(ref ctx) = enclosing_cell_ctx {
                    let mut new_ctx = ctx.clone();
                    if let Some(last) = new_ctx.path.last_mut() {
                        last.cell_index = cell_idx;
                        last.cell_para_index = cp_idx;
                        last.text_direction = cell.text_direction;
                    }
                    Some(new_ctx)
                } else {
                    table_meta.map(|(pi, ci)| CellContext {
                        parent_para_index: pi,
                        path: vec![CellPathEntry {
                            control_index: ci,
                            cell_index: cell_idx,
                            cell_para_index: cp_idx,
                            text_direction: cell.text_direction,
                        }],
                    })
                };

                let has_table_ctrl = para.controls.iter().any(|c| matches!(c, Control::Table(_)));

                let para_y_before_compose = para_y;

                // 줄별 TAC 컨트롤 너비 합산: 각 TAC가 속한 줄을 판별하여 줄별 최대 너비 계산
                let tac_line_widths: Vec<f64> = {
                    // 줄별 너비 합산 벡터
                    let mut line_widths = vec![0.0f64; composed.lines.len().max(1)];
                    for ctrl in &para.controls {
                        let (is_tac, w) = match ctrl {
                            Control::Picture(pic) if pic.common.treat_as_char => {
                                (true, hwpunit_to_px(pic.common.width as i32, self.dpi))
                            }
                            Control::Shape(shape) if shape.common().treat_as_char => {
                                (true, hwpunit_to_px(shape.common().width as i32, self.dpi))
                            }
                            Control::Equation(eq) => {
                                (true, hwpunit_to_px(eq.common.width as i32, self.dpi))
                            }
                            Control::Table(t) if t.common.treat_as_char => {
                                (true, hwpunit_to_px(t.common.width as i32, self.dpi))
                            }
                            _ => (false, 0.0),
                        };
                        if !is_tac { continue; }
                        // 줄이 1개이면 무조건 0번 줄
                        if composed.lines.len() <= 1 {
                            line_widths[0] += w;
                        } else {
                            // 아직 줄 분배 전이므로 순서대로 채워넣기:
                            // 현재 줄 너비 + 이 컨트롤 너비 > 셀 너비이면 다음 줄로
                            let mut placed = false;
                            for lw in line_widths.iter_mut() {
                                if *lw == 0.0 || *lw + w <= inner_width + 0.5 {
                                    *lw += w;
                                    placed = true;
                                    break;
                                }
                            }
                            if !placed {
                                if let Some(last) = line_widths.last_mut() {
                                    *last += w;
                                }
                            }
                        }
                    }
                    line_widths
                };
                let total_inline_width: f64 = tac_line_widths.iter().cloned().fold(0.0f64, f64::max);

                if !has_table_ctrl {
                    let is_last_para = cp_idx + 1 == composed_paras.len();
                    // 분할 중첩 표: 셀 하단을 초과하는 줄은 렌더링하지 않음
                    let end_line = if row_filter.is_some() {
                        let cell_bottom = cell_y + cell_h;
                        let mut sim_y = para_y;
                        let mut fit = composed.lines.len();
                        for (li, line) in composed.lines.iter().enumerate() {
                            let lh = hwpunit_to_px(line.line_height, self.dpi);
                            if sim_y + lh > cell_bottom + 0.5 {
                                fit = li;
                                break;
                            }
                            sim_y += lh + hwpunit_to_px(line.line_spacing, self.dpi);
                        }
                        fit
                    } else {
                        composed.lines.len()
                    };
                    para_y = self.layout_composed_paragraph(
                        tree,
                        &mut cell_node,
                        composed,
                        styles,
                        &inner_area,
                        para_y,
                        0,
                        end_line,
                        section_index, cp_idx,
                        cell_context.clone(),
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
                    // (leading spacing은 LINE_SEG vpos에 이미 반영되어 있음)
                }

                let para_alignment = styles.para_styles
                    .get(para.para_shape_id as usize)
                    .map(|s| s.alignment)
                    .unwrap_or(Alignment::Left);

                let mut prev_tac_text_pos: usize = 0;
                // LINE_SEG 기반 줄별 TAC 이미지 배치를 위한 상태
                // 빈 문단(runs 없음)에서 TAC 컨트롤을 LINE_SEG에 순서대로 매핑
                let all_runs_empty = composed.lines.iter().all(|l| l.runs.is_empty());
                let mut tac_seq_index: usize = 0; // TAC 컨트롤 순번 (빈 문단용)
                let mut current_tac_line: usize = 0;
                let mut inline_x = {
                    let line_w = tac_line_widths.first().copied().unwrap_or(total_inline_width);
                    match para_alignment {
                        Alignment::Center | Alignment::Distribute => {
                            inner_area.x + (inner_area.width - line_w).max(0.0) / 2.0
                        }
                        Alignment::Right => {
                            inner_area.x + (inner_area.width - line_w).max(0.0)
                        }
                        _ => inner_area.x,
                    }
                };
                let mut tac_img_y = para_y_before_compose;

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
                                    // LINE_SEG 기반 줄 판별
                                    let target_line = if all_runs_empty && para.line_segs.len() > 1 {
                                        // 빈 문단: TAC 순번으로 LINE_SEG에 1:1 매핑
                                        let li = tac_seq_index.min(para.line_segs.len() - 1);
                                        tac_seq_index += 1;
                                        li
                                    } else {
                                        // 텍스트 있는 문단: char position으로 줄 판별
                                        composed.tac_controls.iter()
                                            .find(|&&(_, _, ci)| ci == ctrl_idx)
                                            .map(|&(abs_pos, _, _)| {
                                                composed.lines.iter().enumerate()
                                                    .rev()
                                                    .find(|(_, line)| abs_pos >= line.char_start)
                                                    .map(|(li, _)| li)
                                                    .unwrap_or(0)
                                            })
                                            .unwrap_or(0)
                                    };

                                    if target_line > current_tac_line {
                                        // 줄이 바뀜: inline_x 리셋, y를 LINE_SEG vpos 기준으로 이동
                                        current_tac_line = target_line;
                                        let line_w = tac_line_widths.get(target_line).copied().unwrap_or(0.0);
                                        inline_x = match para_alignment {
                                            Alignment::Center | Alignment::Distribute => {
                                                inner_area.x + (inner_area.width - line_w).max(0.0) / 2.0
                                            }
                                            Alignment::Right => {
                                                inner_area.x + (inner_area.width - line_w).max(0.0)
                                            }
                                            _ => inner_area.x,
                                        };
                                        if let Some(seg) = para.line_segs.get(target_line) {
                                            tac_img_y = para_y_before_compose + hwpunit_to_px(seg.vertical_pos, self.dpi);
                                        }
                                    }

                                    let pic_h = hwpunit_to_px(pic.common.height as i32, self.dpi);
                                    // [Task #477] 셀 폭 초과 시 비율 유지 클램프
                                    let clamped_w = pic_w.min(inner_area.width);
                                    let clamped_h = if pic_w > 0.0 {
                                        pic_h * (clamped_w / pic_w)
                                    } else {
                                        pic_h
                                    };
                                    let pic_area = LayoutRect {
                                        x: inline_x,
                                        y: tac_img_y,
                                        width: clamped_w,
                                        height: clamped_h,
                                    };
                                    self.layout_picture(tree, &mut cell_node, pic, &pic_area, bin_data_content, Alignment::Left, Some(section_index), None, None);
                                    inline_x += clamped_w;
                                    continue;
                                }
                                inline_x += pic_w;
                            } else {
                                // 비-인라인(자리차지/글뒤로/글앞으로) 이미지:
                                // 본문배치 속성(가로/세로 기준, 정렬, 오프셋) 적용
                                let pic_w = hwpunit_to_px(pic.common.width as i32, self.dpi);
                                let pic_h = hwpunit_to_px(pic.common.height as i32, self.dpi);
                                let cell_area = LayoutRect {
                                    y: para_y,
                                    height: (inner_area.height - (para_y - inner_area.y)).max(0.0),
                                    ..inner_area
                                };
                                let (pic_x, pic_y) = self.compute_object_position(
                                    &pic.common, pic_w, pic_h,
                                    &cell_area, &inner_area, &inner_area, &inner_area,
                                    para_y, para_alignment,
                                );
                                let pic_area = LayoutRect {
                                    x: pic_x,
                                    y: pic_y,
                                    width: pic_w,
                                    height: pic_h,
                                };
                                self.layout_picture(tree, &mut cell_node, pic, &pic_area, bin_data_content, Alignment::Left, Some(section_index), None, None);
                                para_y += pic_h;
                            }
                            has_preceding_text = true;
                        }
                        Control::Shape(shape) => {
                            if shape.common().treat_as_char {
                                let shape_w = hwpunit_to_px(shape.common().width as i32, self.dpi);
                                // Shape 앞의 텍스트 너비 계산: tac_controls에서 이 Shape의 text_pos와
                                // 이전 Shape의 text_pos 차이에 해당하는 텍스트 너비를 inline_x에 반영
                                if let Some(&(tac_pos, _, _)) = composed.tac_controls.iter().find(|&&(_, _, ci)| ci == ctrl_idx) {
                                    // [Task #495] 가드: 사각형이 paragraph 첫 줄(ls[0]) 범위 안에 있을 때만
                                    // text_before 추출/발행. multi-line paragraph 에서 사각형이 ls[1]+ 에
                                    // 있는 경우 composed.lines.first() 만 보던 기존 코드는 첫 줄 전체
                                    // 텍스트를 잘못 추출해 paragraph_layout 결과와 중복 발행했음.
                                    let in_first_line = composed.lines.first()
                                        .map(|line| {
                                            let line_chars: usize = line.runs.iter().map(|r| r.text.chars().count()).sum();
                                            tac_pos >= line.char_start && tac_pos < line.char_start + line_chars
                                        })
                                        .unwrap_or(false);
                                    // 이 Shape 앞에 아직 inline_x에 반영되지 않은 텍스트가 있는지 계산
                                    let text_before: String = if in_first_line {
                                        composed.lines.first()
                                            .map(|line| {
                                                let mut chars_so_far = 0usize;
                                                let mut result = String::new();
                                                for run in &line.runs {
                                                    for ch in run.text.chars() {
                                                        if chars_so_far >= prev_tac_text_pos && chars_so_far < tac_pos {
                                                            result.push(ch);
                                                        }
                                                        chars_so_far += 1;
                                                    }
                                                }
                                                result
                                            })
                                            .unwrap_or_default()
                                    } else {
                                        String::new()
                                    };
                                    if !text_before.is_empty() {
                                        let char_style_id = composed.lines.first()
                                            .and_then(|l| l.runs.first())
                                            .map(|r| r.char_style_id).unwrap_or(0);
                                        let lang_index = composed.lines.first()
                                            .and_then(|l| l.runs.first())
                                            .map(|r| r.lang_index).unwrap_or(0);
                                        let ts = resolved_to_text_style(styles, char_style_id, lang_index);
                                        // [Task #555] PUA 옛한글 char 은 자모 시퀀스로 변환 후 폭 측정.
                                        let text_before_metrics: String = {
                                            use super::super::pua_oldhangul::map_pua_old_hangul;
                                            text_before.chars().flat_map(|ch| {
                                                if let Some(jamos) = map_pua_old_hangul(ch) {
                                                    jamos.iter().copied().collect::<Vec<_>>()
                                                } else { vec![ch] }
                                            }).collect()
                                        };
                                        let text_w = estimate_text_width(&text_before_metrics, &ts);
                                        let text_font_size = ts.font_size;
                                        // 텍스트 렌더링: Shape 사이에 배치
                                        // 텍스트 y를 Shape 하단 baseline에 맞춤
                                        // (Shape 높이 - 폰트 줄 높이)만큼 아래로 이동
                                        let text_baseline = text_font_size * 0.85;
                                        let font_line_h = text_font_size * 1.2;
                                        // 인접 Shape의 높이를 사용하여 텍스트 y를 baseline 정렬
                                        let adjacent_shape_h = para.controls.iter()
                                            .find_map(|c| if let Control::Shape(s) = c {
                                                if s.common().treat_as_char { Some(hwpunit_to_px(s.common().height as i32, self.dpi)) } else { None }
                                            } else { None })
                                            .unwrap_or(0.0);
                                        let text_y = para_y_before_compose + (adjacent_shape_h - font_line_h).max(0.0);
                                        let text_node_id = tree.next_id();
                                        let text_node = RenderNode::new(
                                            text_node_id,
                                            RenderNodeType::TextRun(TextRunNode {
                                                text: text_before,
                                                style: ts,
                                                char_shape_id: Some(char_style_id),
                                                para_shape_id: Some(composed.para_style_id),
                                                section_index: Some(section_index),
                                                para_index: None,
                                                char_start: None,
                                                cell_context: None,
                                                is_para_end: false,
                                                is_line_break_end: false,
                                                rotation: 0.0,
                                                is_vertical: false,
                                                char_overlap: None,
                                                border_fill_id: 0,
                                                baseline: text_baseline,
                                                field_marker: FieldMarkerType::None,
                                            }),
                                            BoundingBox::new(inline_x, text_y, text_w, font_line_h),
                                        );
                                        cell_node.children.push(text_node);
                                        inline_x += text_w;
                                    }
                                    prev_tac_text_pos = tac_pos;
                                }
                                let shape_area = LayoutRect {
                                    x: inline_x,
                                    y: para_y_before_compose,
                                    width: shape_w,
                                    height: inner_area.height,
                                };
                                self.layout_cell_shape(tree, &mut cell_node, shape, &shape_area, para_y_before_compose, Alignment::Left, styles, bin_data_content);
                                inline_x += shape_w;
                            } else {
                                self.layout_cell_shape(tree, &mut cell_node, shape, &inner_area, para_y, para_alignment, styles, bin_data_content);
                            }
                        }
                        Control::Equation(eq) => {
                            // 수식 컨트롤: 글자처럼 인라인 배치
                            let eq_w = hwpunit_to_px(eq.common.width as i32, self.dpi);

                            // 수식이 텍스트 run 사이에 인라인으로 배치되는 경우
                            // layout_composed_paragraph에서 이미 렌더링됨 → 건너뛰기
                            let has_text_in_para = para.text.chars().any(|c| c > '\u{001F}' && c != '\u{FFFC}');
                            // 빈 runs 셀 + TAC 수식: paragraph_layout(Task #287 경로)이 이미
                            // 렌더 후 set_inline_shape_position 호출. 중복 emit 방지(Issue #301).
                            let already_rendered_inline = tree
                                .get_inline_shape_position(section_index, cp_idx, ctrl_idx)
                                .is_some();
                            if has_text_in_para || already_rendered_inline {
                                // paragraph_layout 경로에서 이미 렌더됨
                                inline_x += eq_w;
                            } else {
                                // 수식만 있는 문단: 여기서 직접 렌더링
                                let eq_h = hwpunit_to_px(eq.common.height as i32, self.dpi);
                                let eq_x = {
                                    let x = inline_x;
                                    inline_x += eq_w;
                                    x
                                };
                                let eq_y = para_y_before_compose;

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
                                        para_index: table_meta.map(|(pi, _)| pi),
                                        control_index: Some(ctrl_idx),
                                        cell_index: Some(cell_idx),
                                        cell_para_index: Some(cp_idx),
                                    }),
                                    BoundingBox::new(eq_x, eq_y, eq_w, eq_h),
                                );
                                cell_node.children.push(eq_node);
                            }
                        }
                        Control::Table(nested_table) => {
                            let is_tac_table = nested_table.common.treat_as_char;
                            let nested_y = if has_preceding_text {
                                para_y
                            } else {
                                inner_area.y
                            };
                            let nested_ctx = cell_context.as_ref().map(|ctx| {
                                let mut new_ctx = ctx.clone();
                                new_ctx.path.push(CellPathEntry {
                                    control_index: ctrl_idx,
                                    cell_index: 0,
                                    cell_para_index: 0,
                                    text_direction: 0,
                                });
                                new_ctx
                            });
                            if is_tac_table {
                                // TAC 표: inline_x를 사용하여 수평 배치
                                let tac_w = hwpunit_to_px(nested_table.common.width as i32, self.dpi);
                                let ctrl_area = LayoutRect {
                                    x: inline_x,
                                    y: para_y_before_compose,
                                    width: tac_w,
                                    height: (inner_area.height - (para_y_before_compose - inner_area.y)).max(0.0),
                                };
                                let table_h = self.layout_table(
                                    tree, &mut cell_node, nested_table,
                                    section_index, styles, &ctrl_area, para_y_before_compose,
                                    bin_data_content, None, depth + 1,
                                    None, para_alignment,
                                    nested_ctx,
                                    0.0, 0.0, Some(inline_x), None, None,
                                );
                                inline_x += tac_w;
                                // para_y는 TAC 표 높이만큼 갱신 (같은 문단 내 다음 표도 같은 y)
                                let new_bottom = para_y_before_compose + table_h;
                                if new_bottom > para_y {
                                    para_y = new_bottom;
                                }
                            } else {
                                // 비-TAC 표: 기존 수직 배치
                                // 앞 텍스트 너비만큼 x 오프셋 적용
                                let tac_text_offset = if nested_table.attr & 0x01 != 0 {
                                    let mut text_w = 0.0;
                                    for line in &composed.lines {
                                        for run in &line.runs {
                                            if !run.text.is_empty() {
                                                let ts = resolved_to_text_style(
                                                    styles, run.char_style_id, run.lang_index);
                                                // [Task #555] PUA 옛한글 변환 후 자모 시퀀스 폭.
                                                text_w += estimate_text_width(effective_text_for_metrics(run), &ts);
                                            }
                                        }
                                    }
                                    text_w
                                } else {
                                    0.0
                                };
                                // TAC 표 앞 텍스트 렌더링 (문단부호 등 표시용)
                                if tac_text_offset > 0.0 {
                                    let line_h = composed.lines.first()
                                        .map(|l| hwpunit_to_px(l.line_height, self.dpi))
                                        .unwrap_or(12.0);
                                    let baseline = line_h * 0.85;
                                    let line_id = tree.next_id();
                                    let mut line_node = RenderNode::new(
                                        line_id,
                                        RenderNodeType::TextLine(TextLineNode::new(line_h, baseline)),
                                        BoundingBox::new(inner_area.x, nested_y, tac_text_offset, line_h),
                                    );
                                    let mut run_x = inner_area.x;
                                    for line in &composed.lines {
                                        for run in &line.runs {
                                            if run.text.is_empty() { continue; }
                                            let ts = resolved_to_text_style(
                                                styles, run.char_style_id, run.lang_index);
                                            // [Task #555] PUA 옛한글 변환 후 자모 시퀀스 폭.
                                            let run_w = estimate_text_width(effective_text_for_metrics(run), &ts);
                                            let run_id = tree.next_id();
                                            let run_node = RenderNode::new(
                                                run_id,
                                                RenderNodeType::TextRun(TextRunNode {
                                                    text: run.text.clone(),
                                                    style: ts,
                                                    char_shape_id: Some(run.char_style_id),
                                                    para_shape_id: Some(para.para_shape_id),
                                                    section_index: Some(section_index),
                                                    para_index: None,
                                                    char_start: None,
                                                    cell_context: cell_context.clone(),
                                                    is_para_end: false,
                                                    is_line_break_end: false,
                                                    rotation: 0.0,
                                                    is_vertical: false,
                                                    char_overlap: None,
                                                    border_fill_id: 0,
                                                    baseline,
                                                    field_marker: FieldMarkerType::None,
                                                }),
                                                BoundingBox::new(run_x, nested_y, run_w, line_h),
                                            );
                                            line_node.children.push(run_node);
                                            run_x += run_w;
                                        }
                                    }
                                    cell_node.children.push(line_node);
                                }
                                let ctrl_area = LayoutRect {
                                    x: inner_area.x + tac_text_offset,
                                    y: nested_y,
                                    width: (inner_area.width - tac_text_offset).max(0.0),
                                    height: (inner_area.height - (nested_y - inner_area.y)).max(0.0),
                                };
                                let table_h = self.layout_table(
                                    tree, &mut cell_node, nested_table,
                                    section_index, styles, &ctrl_area, nested_y,
                                    bin_data_content, None, depth + 1,
                                    None, para_alignment,
                                    nested_ctx,
                                    0.0, 0.0, None, None, None,
                                );
                                para_y = nested_y + table_h;
                            }
                            has_preceding_text = true;
                        }
                        _ => {}
                    }
                }

                // 마지막 인라인 Shape 이후의 남은 텍스트 렌더링 (예: "일")
                if prev_tac_text_pos > 0 {
                    let total_text_chars = composed.lines.first()
                        .map(|line| line.runs.iter().map(|r| r.text.chars().count()).sum::<usize>())
                        .unwrap_or(0);
                    if prev_tac_text_pos < total_text_chars {
                        let remaining_text: String = composed.lines.first()
                            .map(|line| {
                                let mut chars_so_far = 0usize;
                                let mut result = String::new();
                                for run in &line.runs {
                                    for ch in run.text.chars() {
                                        if chars_so_far >= prev_tac_text_pos {
                                            result.push(ch);
                                        }
                                        chars_so_far += 1;
                                    }
                                }
                                result
                            })
                            .unwrap_or_default();
                        let remaining_trimmed = remaining_text.trim_end();
                        if !remaining_trimmed.is_empty() {
                            let char_style_id = composed.lines.first()
                                .and_then(|l| l.runs.last())
                                .map(|r| r.char_style_id).unwrap_or(0);
                            let lang_index = composed.lines.first()
                                .and_then(|l| l.runs.last())
                                .map(|r| r.lang_index).unwrap_or(0);
                            let ts = resolved_to_text_style(styles, char_style_id, lang_index);
                            // [Task #555] PUA 옛한글 char 은 자모 시퀀스로 변환 후 폭 측정.
                            let remaining_metrics: String = {
                                use super::super::pua_oldhangul::map_pua_old_hangul;
                                remaining_trimmed.chars().flat_map(|ch| {
                                    if let Some(jamos) = map_pua_old_hangul(ch) {
                                        jamos.iter().copied().collect::<Vec<_>>()
                                    } else { vec![ch] }
                                }).collect()
                            };
                            let text_w = estimate_text_width(&remaining_metrics, &ts);
                            let text_baseline = ts.font_size * 0.85;
                            let text_h = ts.font_size * 1.2;
                            // 마지막 Shape 높이 기준으로 텍스트 y 계산
                            let last_shape_h = para.controls.iter().rev()
                                .find_map(|c| if let Control::Shape(s) = c {
                                    if s.common().treat_as_char { Some(hwpunit_to_px(s.common().height as i32, self.dpi)) } else { None }
                                } else { None })
                                .unwrap_or(0.0);
                            let text_y = para_y_before_compose + (last_shape_h - text_h).max(0.0);
                            let text_node_id = tree.next_id();
                            let text_node = RenderNode::new(
                                text_node_id,
                                RenderNodeType::TextRun(TextRunNode {
                                    text: remaining_trimmed.to_string(),
                                    style: ts,
                                    char_shape_id: Some(char_style_id),
                                    para_shape_id: Some(composed.para_style_id),
                                    section_index: Some(section_index),
                                    para_index: None,
                                    char_start: None,
                                    cell_context: None,
                                    is_para_end: false,
                                    is_line_break_end: false,
                                    rotation: 0.0,
                                    is_vertical: false,
                                    char_overlap: None,
                                    border_fill_id: 0,
                                    baseline: text_baseline,
                                    field_marker: FieldMarkerType::None,
                                }),
                                BoundingBox::new(inline_x, text_y, text_w, text_h),
                            );
                            cell_node.children.push(text_node);
                        }
                    }
                }

                if has_table_ctrl {
                    // LINE_SEG vpos 기반으로 para_y 보정.
                    // LINE_SEG.line_height에는 중첩 표 높이가 미포함될 수 있으므로
                    // layout_table 반환값과 vpos 기반 중 적절한 값을 선택한다.
                    let is_last_para = cp_idx + 1 == composed_paras.len();
                    // 다음 문단의 vpos가 있으면 그것을 기준으로 para_y 보정
                    if !is_last_para {
                        if let Some(next_para) = cell.paragraphs.get(cp_idx + 1) {
                            if let Some(next_seg) = next_para.line_segs.first() {
                                let next_vpos_y = text_y_start + hwpunit_to_px(
                                    next_seg.vertical_pos, self.dpi);
                                // layout_table 기반 para_y와 다음 문단 vpos 중
                                // 더 큰 값 사용 (표가 LINE_SEG보다 클 수 있으므로)
                                para_y = para_y.max(next_vpos_y);
                            }
                        }
                    }
                    // 음수 line_spacing 처리 (중첩 구조에서 para_y 되돌리기)
                    if !(is_last_para && enclosing_cell_ctx.is_some()) {
                        if let Some(last_line) = composed.lines.last() {
                            let ls = hwpunit_to_px(last_line.line_spacing, self.dpi);
                            if ls < -0.01 {
                                para_y += ls;
                            }
                        }
                    }
                }
            }
            } // else (가로쓰기)

            // 셀 내 각주 참조 번호 윗첨자
            for para in &cell.paragraphs {
                self.add_footnote_superscripts(tree, &mut cell_node, para, styles);
            }

            // (b) 셀 테두리를 엣지 그리드에 수집
            if let Some(bs) = border_style {
                collect_cell_borders(
                    h_edges, v_edges,
                    c, r, cell.col_span as usize, cell.row_span as usize,
                    &bs.borders,
                );
            }

            table_node.children.push(cell_node);

            // (c) 셀 대각선 렌더링 (셀 콘텐츠 위에 그림)
            if let Some(bs) = border_style {
                table_node.children.extend(
                    render_cell_diagonal(tree, bs, cell_x, cell_y, cell_w, cell_h),
                );
            }
        }
    }

    pub(crate) fn calc_cell_controls_height(
        &self,
        cell: &crate::model::table::Cell,
        styles: &ResolvedStyleSet,
    ) -> f64 {
        let measurer = super::super::height_measurer::HeightMeasurer::new(self.dpi);
        measurer.cell_controls_height(&cell.paragraphs, styles, 0)
    }

    /// 중첩 표의 총 높이를 계산한다 (행 높이 합 + cell_spacing).
    /// MeasuredCell.line_heights에서 중첩 표가 추가 줄로 포함될 때의 높이와 일관되게 계산.
    pub(crate) fn calc_nested_table_height(
        &self,
        table: &crate::model::table::Table,
        styles: &ResolvedStyleSet,
    ) -> f64 {
        let col_count = table.col_count as usize;
        let row_count = table.row_count as usize;
        let row_heights = self.resolve_row_heights(table, col_count, row_count, None, styles);
        let cell_spacing = hwpunit_to_px(table.cell_spacing as i32, self.dpi);
        let om_top = hwpunit_to_px(table.outer_margin_top as i32, self.dpi);
        let om_bottom = hwpunit_to_px(table.outer_margin_bottom as i32, self.dpi);
        row_heights.iter().sum::<f64>() + cell_spacing * (row_count.saturating_sub(1) as f64)
            + om_top + om_bottom
    }

    /// 셀의 content_offset 이후 실제 남은 콘텐츠 높이를 계산한다.
    /// MeasuredCell과 동일한 높이 로직을 사용한다 (pagination 엔진이 MeasuredCell 기준으로
    /// content_offset을 산출하므로 동일 기준이어야 함).
    pub(crate) fn calc_cell_remaining_content_height(
        &self,
        cell: &crate::model::table::Cell,
        styles: &ResolvedStyleSet,
        content_offset: f64,
    ) -> f64 {
        // MeasuredCell과 동일한 높이 계산:
        // 각 줄 h+ls, 단 셀의 마지막 줄(마지막 문단의 마지막 줄)은 ls 제외
        let mut total = 0.0;
        let cell_para_count = cell.paragraphs.len();
        for (pidx, p) in cell.paragraphs.iter().enumerate() {
            let comp = compose_paragraph(p);
            let para_style = styles.para_styles.get(p.para_shape_id as usize);
            let is_last_para = pidx + 1 == cell_para_count;
            let spacing_before = if pidx > 0 {
                para_style.map(|s| s.spacing_before).unwrap_or(0.0)
            } else {
                0.0
            };
            let spacing_after = if !is_last_para {
                para_style.map(|s| s.spacing_after).unwrap_or(0.0)
            } else {
                0.0
            };
            if comp.lines.is_empty() {
                // 중첩 표 컨트롤 문단: 실제 중첩 표 높이로 계산
                let nested_h: f64 = p.controls.iter().map(|ctrl| {
                    if let Control::Table(t) = ctrl {
                        self.calc_nested_table_height(t, styles)
                    } else {
                        0.0
                    }
                }).sum();
                let h = if nested_h > 0.0 { nested_h } else { hwpunit_to_px(400, self.dpi) };
                total += spacing_before + h + spacing_after;
            } else {
                // 중첩 표가 있는 문단: LINE_SEG 높이와 실제 중첩 표 높이 중 큰 값 사용
                let has_table_in_para = p.controls.iter().any(|c| matches!(c, Control::Table(_)));
                let line_count = comp.lines.len();
                let line_based_h: f64 = comp.lines.iter().enumerate().map(|(li, line)| {
                    let h = hwpunit_to_px(line.line_height, self.dpi);
                    let is_cell_last_line = is_last_para && li + 1 == line_count;
                    let ls = if !is_cell_last_line {
                        hwpunit_to_px(line.line_spacing, self.dpi)
                    } else {
                        0.0
                    };
                    spacing_before * (if li == 0 { 1.0 } else { 0.0 })
                        + h + ls
                        + spacing_after * (if li + 1 == line_count { 1.0 } else { 0.0 })
                }).sum();
                if has_table_in_para {
                    let nested_h: f64 = p.controls.iter().map(|ctrl| {
                        if let Control::Table(t) = ctrl {
                            self.calc_nested_table_height(t, styles)
                        } else {
                            0.0
                        }
                    }).sum();
                    total += nested_h.max(line_based_h);
                } else {
                    total += line_based_h;
                }
            }
        }
        (total - content_offset).max(0.0)
    }

    /// 셀 내 문단 줄 높이로부터 content_offset/content_limit 기준 줄 범위를 계산한다.
    pub(crate) fn compute_cell_line_ranges(
        &self,
        cell: &crate::model::table::Cell,
        composed_paras: &[ComposedParagraph],
        content_offset: f64,
        content_limit: f64,
        styles: &ResolvedStyleSet,
    ) -> Vec<(usize, usize)> {
        // 셀 콘텐츠의 cumulative position(누적 px) 기반 가시성 결정.
        // - LINE_SEG.vpos 는 컬럼 리셋이 발생하므로 셀 시작부터의 누적 위치로 사용 불가 → line_height + line_spacing 누적 사용.
        // - content_offset > 0: [0, content_offset) 영역의 콘텐츠는 이전 페이지 → 스킵.
        // - content_limit > 0: [0, content_limit] 영역의 콘텐츠만 표시.
        // - 중첩 표(atomic) 문단은 분할 불가 — 경계를 걸치면 한쪽 페이지에만 렌더링.
        let mut result = Vec::with_capacity(composed_paras.len());
        let has_offset = content_offset > 0.0;
        let has_limit = content_limit > 0.0;
        let mut cum: f64 = 0.0;
        // [Task #431] content_limit 은 현재 페이지에서 표시할 상대 길이(px) 의미이므로
        // 절대 좌표(cum 기반)와 비교하려면 content_offset 을 더해 절대 끝 좌표로 변환한다.
        // (Task #362 의 도입 시점에 단위 mismatch 가 있었음 — content_offset >= content_limit
        // 케이스에서 셀 내 문단이 즉시 break 되어 빈 페이지로 출력되던 결함 정정.)
        let abs_limit = if has_limit { content_offset + content_limit } else { 0.0 };

        let total_paras = composed_paras.len();
        for (pi, (comp, para)) in composed_paras.iter().zip(cell.paragraphs.iter()).enumerate() {
            let para_style = styles.para_styles.get(para.para_shape_id as usize);
            let is_last_para = pi + 1 == total_paras;
            // MeasuredCell 규칙: 첫 문단은 spacing_before 없음, 마지막 문단은 spacing_after 없음
            let spacing_before = if pi > 0 { para_style.map(|s| s.spacing_before).unwrap_or(0.0) } else { 0.0 };
            let spacing_after = if !is_last_para { para_style.map(|s| s.spacing_after).unwrap_or(0.0) } else { 0.0 };
            let line_count = comp.lines.len();

            // 중첩 표 포함 문단(atomic) — line_count==0 또는 has_table_in_para
            let has_table_in_para = para.controls.iter().any(|c| matches!(c, Control::Table(_)));
            if line_count == 0 || has_table_in_para {
                let nested_h: f64 = para.controls.iter().map(|ctrl| {
                    if let Control::Table(t) = ctrl {
                        self.calc_nested_table_height(t, styles)
                    } else { 0.0 }
                }).sum();
                let para_h = if line_count == 0 {
                    let h = if nested_h > 0.0 { nested_h } else { hwpunit_to_px(400, self.dpi) };
                    spacing_before + h + spacing_after
                } else {
                    let line_based_h: f64 = comp.lines.iter().enumerate().map(|(li, line)| {
                        let h = hwpunit_to_px(line.line_height, self.dpi);
                        let ls = hwpunit_to_px(line.line_spacing, self.dpi);
                        let is_cell_last_line = is_last_para && li + 1 == line_count;
                        let mut lh = if !is_cell_last_line { h + ls } else { h };
                        if li == 0 { lh += spacing_before; }
                        if li == line_count - 1 { lh += spacing_after; }
                        lh
                    }).sum();
                    nested_h.max(line_based_h)
                };

                let para_start_pos = cum;
                let para_end_pos = cum + para_h;
                cum = para_end_pos;

                // 가시성 결정: atomic — 한쪽 페이지에만 렌더링.
                // - content_offset 영역 안에 끝나면(이전 페이지 전체 포함됨) → 스킵
                // - content_limit 영역을 끝점이 초과하면 → 다음 페이지로 미룸
                // - offset 경계를 걸치면 현재 페이지(continuation)에서 렌더링
                //
                // [Task #362] 한 페이지보다 큰 nested table 예외:
                // para_h 가 content_limit 자체를 초과하는 경우 (한 페이지에 어떻게 해도 못 들어감)
                // atomic 미루기 대신 visible 로 표시 (다음 페이지 PartialTable continuation 으로 분할).
                // v0.7.3 의 처리 시멘틱과 동일.
                let was_on_prev = has_offset && para_end_pos <= content_offset;
                let bigger_than_page = has_limit && para_h > content_limit;
                // [Task #431] abs_limit (= content_offset + content_limit) 와 비교 (단위 정합)
                let exceeds_limit = has_limit && para_end_pos > abs_limit && !bigger_than_page;
                let visible_count = if line_count == 0 { 0 } else { line_count };
                if was_on_prev || exceeds_limit {
                    // (n,n): 렌더 스킵 마커. line_count==0 이면 (0,0) 동일.
                    result.push((visible_count, visible_count));
                } else {
                    result.push((0, visible_count));
                }
                let _ = para_start_pos; // 추적 변수 (미사용 경고 회피)
                continue;
            }

            // 일반 문단: line 단위 누적 + 위치 기반 가시성
            let mut para_start = 0;
            let mut para_end = 0;
            let mut started = false;

            for (li, line) in comp.lines.iter().enumerate() {
                let h = hwpunit_to_px(line.line_height, self.dpi);
                let ls = hwpunit_to_px(line.line_spacing, self.dpi);
                let is_cell_last_line = is_last_para && li + 1 == line_count;
                let mut line_h = if !is_cell_last_line { h + ls } else { h };
                if li == 0 {
                    line_h += spacing_before;
                }
                if li == line_count - 1 {
                    line_h += spacing_after;
                }

                let line_end_pos = cum + line_h;

                if has_offset && line_end_pos <= content_offset {
                    // 이전 페이지에서 완전히 렌더링됨 → 스킵
                    cum = line_end_pos;
                    para_start = li + 1;
                    para_end = li + 1;
                    continue;
                }

                if has_limit && line_end_pos > abs_limit {
                    // [Task #431] abs_limit (= content_offset + content_limit) 와 비교 (단위 정합)
                    // limit 초과 → 이 줄과 이후 모든 콘텐츠 차단
                    break;
                }

                cum = line_end_pos;
                if !started {
                    started = true;
                    // para_start 는 첫 가시 줄의 인덱스에 고정됨 (위 루프에서 갱신됨)
                }
                para_end = li + 1;
            }

            if !started {
                // 한 줄도 렌더링 안 됨: 모두 offset 영역에 있거나 limit 초과
                // → 누적은 이미 라인별로 처리됨
            }

            result.push((para_start, para_end));
        }

        result
    }

    /// 줄 범위(line_ranges)에 해당하는 셀 콘텐츠의 실제 렌더링 높이를 계산한다.
    /// compute_cell_line_ranges()의 결과를 받아서, 렌더링될 줄들의 높이를 합산한다.
    /// MeasuredCell 규칙: 첫 문단 spacing_before 없음, 마지막 문단 spacing_after 없음,
    /// 셀 마지막 줄 line_spacing 제외.
    pub(crate) fn calc_visible_content_height_from_ranges(
        &self,
        composed_paras: &[ComposedParagraph],
        paragraphs: &[crate::model::paragraph::Paragraph],
        line_ranges: &[(usize, usize)],
        styles: &ResolvedStyleSet,
    ) -> f64 {
        self.calc_visible_content_height_from_ranges_with_offset(
            composed_paras, paragraphs, line_ranges, styles, 0.0,
        )
    }

    /// calc_visible_content_height_from_ranges 의 확장판 — split_start 의 content_offset 을 받아서
    /// 한 페이지보다 큰 nested table 의 잔여 높이를 정확히 계산한다.
    /// [Task #362] split_start 시 nested table 잔여 높이 누락으로 row 높이가 잘못 계산되는 결함 정정.
    pub(crate) fn calc_visible_content_height_from_ranges_with_offset(
        &self,
        composed_paras: &[ComposedParagraph],
        paragraphs: &[crate::model::paragraph::Paragraph],
        line_ranges: &[(usize, usize)],
        styles: &ResolvedStyleSet,
        content_offset: f64,
    ) -> f64 {
        let para_count = paragraphs.len();
        let mut total = 0.0;
        let mut cum_pos = 0.0f64;  // 누적 콘텐츠 위치 (compute_cell_line_ranges 와 동일)
        let first_visible_pi = line_ranges.iter().position(|&(s, e)| s < e);
        let _last_visible_pi = line_ranges.iter().rposition(|&(s, e)| s < e);
        for (pi, ((comp, para), &(start, end))) in composed_paras.iter()
            .zip(paragraphs.iter())
            .zip(line_ranges.iter())
            .enumerate()
        {
            let para_style = styles.para_styles.get(para.para_shape_id as usize);
            let is_last_para = pi + 1 == para_count;
            let line_count = comp.lines.len();
            let spacing_before = if pi > 0 { para_style.map(|s| s.spacing_before).unwrap_or(0.0) } else { 0.0 };
            let spacing_after = if !is_last_para { para_style.map(|s| s.spacing_after).unwrap_or(0.0) } else { 0.0 };
            let has_table_in_para = para.controls.iter().any(|c| matches!(c, Control::Table(_)));

            // [Task #362] nested table paragraph 의 실제 콘텐츠 높이
            // (compute_cell_line_ranges 와 동일한 시멘틱)
            let para_h = if line_count == 0 || has_table_in_para {
                let nested_h: f64 = para.controls.iter().map(|ctrl| {
                    if let Control::Table(t) = ctrl {
                        self.calc_nested_table_height(t, styles)
                    } else { 0.0 }
                }).sum();
                if line_count == 0 {
                    let h = if nested_h > 0.0 { nested_h } else { hwpunit_to_px(400, self.dpi) };
                    spacing_before + h + spacing_after
                } else {
                    let line_based_h: f64 = comp.lines.iter().enumerate().map(|(li, line)| {
                        let h = hwpunit_to_px(line.line_height, self.dpi);
                        let ls = hwpunit_to_px(line.line_spacing, self.dpi);
                        let is_cell_last_line = is_last_para && li + 1 == line_count;
                        let mut lh = if !is_cell_last_line { h + ls } else { h };
                        if li == 0 { lh += spacing_before; }
                        if li == line_count - 1 { lh += spacing_after; }
                        lh
                    }).sum();
                    nested_h.max(line_based_h)
                }
            } else {
                0.0  // 일반 line 단위 처리는 아래 분기에서
            };

            // nested table paragraph 처리
            if (line_count == 0 || has_table_in_para) && start < end {
                // [Task #362] 한 페이지보다 큰 nested table 분할: 시작 위치가 offset 이전이면
                // 잔여 = para_end_pos - max(content_offset, para_start_pos)
                let para_start_pos = cum_pos;
                let para_end_pos = cum_pos + para_h;
                if content_offset > 0.0 && para_start_pos < content_offset && para_end_pos > content_offset {
                    // 분할 케이스: offset 이후의 잔여만 누적
                    total += para_end_pos - content_offset;
                } else if content_offset > 0.0 && para_end_pos <= content_offset {
                    // 이전 페이지에서 다 표시됨
                } else {
                    // 전체 표시
                    total += para_h;
                }
                cum_pos = para_end_pos;
                continue;
            }

            if start >= end {
                // 보이지 않는 일반 paragraph: cum_pos 만 진행
                if has_table_in_para || line_count == 0 {
                    cum_pos += para_h;
                } else {
                    let line_based_h: f64 = comp.lines.iter().enumerate().map(|(li, line)| {
                        let h = hwpunit_to_px(line.line_height, self.dpi);
                        let ls = hwpunit_to_px(line.line_spacing, self.dpi);
                        let is_cell_last_line = is_last_para && li + 1 == line_count;
                        let mut lh = if !is_cell_last_line { h + ls } else { h };
                        if li == 0 { lh += spacing_before; }
                        if li == line_count - 1 { lh += spacing_after; }
                        lh
                    }).sum();
                    cum_pos += line_based_h;
                }
                continue;
            }

            let is_visible_first = Some(pi) == first_visible_pi;
            // spacing_before: 렌더링되는 첫 문단에서는 적용하지 않음
            if start == 0 && !is_visible_first {
                total += spacing_before;
            }
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
            // spacing_after: 마지막 문단에서는 적용하지 않음
            if end == comp.lines.len() && end > start && !is_last_para {
                total += spacing_after;
            }
            // cum_pos 갱신 (전체 paragraph 가 차지하는 위치)
            let line_based_h: f64 = comp.lines.iter().enumerate().map(|(li, line)| {
                let h = hwpunit_to_px(line.line_height, self.dpi);
                let ls = hwpunit_to_px(line.line_spacing, self.dpi);
                let is_cell_last_line = is_last_para && li + 1 == line_count;
                let mut lh = if !is_cell_last_line { h + ls } else { h };
                if li == 0 { lh += spacing_before; }
                if li == line_count - 1 { lh += spacing_after; }
                lh
            }).sum();
            cum_pos += line_based_h;
        }
        total
    }
}
