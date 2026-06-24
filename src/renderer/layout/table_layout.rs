//! 표 레이아웃 (layout_table + 셀 높이/줄범위 계산)

use super::super::composer::{compose_paragraph, ComposedLine, ComposedParagraph};
use super::super::height_measurer::MeasuredTable;
use super::super::page_layout::LayoutRect;
use super::super::render_tree::*;
use super::super::style_resolver::{ResolvedBorderStyle, ResolvedStyleSet};
use crate::model::bin_data::BinDataContent;
use crate::model::control::Control;
use crate::model::paragraph::Paragraph;
use crate::model::style::{Alignment, BorderLine};
use crate::model::table::VerticalAlign;

/// [Task #548] paragraph 의 line N 에 적용되는 effective margin_left.
/// paragraph_layout.rs 의 line_indent 산식과 동일 (단일 룰).
/// - positive indent: line 0 에만 +indent 적용 (첫줄 들여쓰기)
/// - negative indent (hanging): line N≥1 에 +|indent| 적용
/// - indent=0: 모든 line 에 margin_left 만 적용
fn effective_margin_left_line(margin_left: f64, indent: f64, line_n: usize) -> f64 {
    let line_indent = if indent > 0.0 {
        if line_n == 0 {
            indent
        } else {
            0.0
        }
    } else if indent < 0.0 {
        if line_n == 0 {
            0.0
        } else {
            indent.abs()
        }
    } else {
        0.0
    };
    margin_left + line_indent
}

use super::super::composer::effective_text_for_metrics;
use super::super::{hwpunit_to_px, ShapeStyle};
use super::border_rendering::{
    build_row_col_x, collect_cell_borders, create_border_line_nodes, render_cell_diagonal,
    render_edge_borders, render_transparent_borders,
};
use super::text_measurement::{estimate_text_width, resolved_to_text_style};
use super::utils::find_bin_data;
use super::{CellContext, CellPathEntry, LayoutEngine};

// 표 수평 정렬: model::shape 타입 사용
use crate::model::shape::{CommonObjAttr, HorzAlign, HorzRelTo, TextWrap, VertRelTo};

fn build_col_row_y_from_cell_heights(
    table: &crate::model::table::Table,
    row_heights: &[f64],
    row_y: &[f64],
    col_count: usize,
    row_count: usize,
    cell_spacing: f64,
    dpi: f64,
) -> Vec<Vec<f64>> {
    let mut cell_height_grid = vec![vec![None::<f64>; row_count]; col_count];
    for (cell_idx, cell) in table.cells.iter().enumerate() {
        if cell.row_span == 1
            && cell.col_span == 1
            && cell.height < 0x8000_0000
            && (cell.col as usize) < col_count
            && (cell.row as usize) < row_count
        {
            let render_height = table
                .local_resize_cell_heights
                .iter()
                .find(|(idx, _)| *idx == cell_idx)
                .map(|(_, height)| *height)
                .unwrap_or(cell.height);
            cell_height_grid[cell.col as usize][cell.row as usize] =
                Some(hwpunit_to_px(render_height as i32, dpi));
        }
    }

    let fallback_h = hwpunit_to_px(400, dpi);
    let target_total = if table.common.height > 0 {
        hwpunit_to_px(table.common.height as i32, dpi)
            + cell_spacing * row_count.saturating_sub(1) as f64
    } else {
        row_y.last().copied().unwrap_or(0.0)
    };
    let mut col_row_y = vec![vec![0.0f64; row_count + 1]; col_count];
    for c in 0..col_count {
        let col_idx = c as u16;
        if !table.local_resize_cols.contains(&col_idx) {
            col_row_y[c].clone_from_slice(row_y);
            continue;
        }
        for r in 0..row_count {
            let h = cell_height_grid[c][r]
                .or_else(|| row_heights.get(r).copied())
                .unwrap_or(fallback_h);
            col_row_y[c][r + 1] =
                col_row_y[c][r] + h + if r + 1 < row_count { cell_spacing } else { 0.0 };
        }
        // 저장 파일의 cell.height는 표 전체 높이와 맞지 않는 보조값일 수 있다.
        // 열별 누적 높이가 표 외곽과 맞을 때만 독립 horizontal segment로 해석한다.
        if (col_row_y[c][row_count] - target_total).abs() > 0.5 && row_y.len() == row_count + 1 {
            col_row_y[c].clone_from_slice(row_y);
        }
    }
    col_row_y
}

fn has_independent_col_row_y(col_row_y: &[Vec<f64>], row_y: &[f64]) -> bool {
    col_row_y.iter().any(|cy| {
        cy.iter()
            .zip(row_y.iter())
            .any(|(a, b)| (a - b).abs() > 0.01)
    })
}

fn render_cell_box_borders(
    tree: &mut PageRenderTree,
    bs: &ResolvedBorderStyle,
    x: f64,
    y: f64,
    w: f64,
    h: f64,
) -> Vec<RenderNode> {
    let mut nodes = Vec::new();
    nodes.extend(create_border_line_nodes(
        tree,
        &bs.borders[2],
        x,
        y,
        x + w,
        y,
    ));
    nodes.extend(create_border_line_nodes(
        tree,
        &bs.borders[3],
        x,
        y + h,
        x + w,
        y + h,
    ));
    nodes.extend(create_border_line_nodes(
        tree,
        &bs.borders[0],
        x,
        y,
        x,
        y + h,
    ));
    nodes.extend(create_border_line_nodes(
        tree,
        &bs.borders[1],
        x + w,
        y,
        x + w,
        y + h,
    ));
    nodes
}

/// [Task #993] 분할 표 행 컷 — 행에 속한 셀(col 오름차순)별 "소비한 콘텐츠 유닛 수".
/// 빈 Vec = 처음부터(아무것도 소비 안 함).
pub(crate) type RowCut = Vec<usize>;

/// [Task #993] `advance_row_cut` 결과.
#[derive(Debug, Clone)]
pub(crate) struct RowCutResult {
    /// 셀별 소비 유닛 수 (전진 후).
    pub end_cut: RowCut,
    /// 어느 셀이든 vpos 리셋(hard break)에서 멈췄는가.
    pub hit_hard_break: bool,
    /// 모든 셀이 모든 유닛을 소비했는가.
    pub fully_consumed: bool,
    /// 이 프래그먼트의 콘텐츠 높이 (셀별 표시 높이의 최댓값, 패딩 제외).
    pub consumed_height: f64,
}

/// [Task #993] 한 셀의 콘텐츠 유닛 — 합성 줄 1개 또는 중첩 표 atom 1개.
pub(super) struct CellUnit {
    /// 유닛 높이 (px).
    height: f64,
    /// 이 유닛 앞에 vpos 리셋(셀 내부 페이지 분할)이 있는가.
    hard_break_before: bool,
    /// 이 유닛이 속한 문단 인덱스 (셀 내).
    para_idx: usize,
    /// 이 유닛이 visible 일 때 기여하는 문단 내 줄 범위 `[vis_start, vis_end)`.
    /// 텍스트 줄 유닛 = `(li, li+1)`, 중첩/빈 atom = `(0, line_count.max(1))`.
    vis_start: usize,
    vis_end: usize,
    /// [Task #1073] 이 유닛이 중첩 표의 한 행을 표현하면 그 행 인덱스. 텍스트/일반 유닛은 None.
    /// 분할 행에서 컷 → `NestedTableSplit`(중첩행 범위) 매핑에 사용.
    nested_row: Option<usize>,
}

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
        return NestedTableSplit {
            start_row: 0,
            end_row: 0,
            visible_height: 0.0,
            offset_within_start: 0.0,
        };
    }

    // row_y 누적 배열 (layout_table과 동일 방식)
    let mut row_y = vec![0.0f64; row_count + 1];
    for i in 0..row_count {
        row_y[i + 1] =
            row_y[i] + row_heights[i] + if i + 1 < row_count { cell_spacing } else { 0.0 };
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

    NestedTableSplit {
        start_row,
        end_row,
        visible_height,
        offset_within_start: 0.0,
    }
}

impl LayoutEngine {
    /// 셀 안 비-TAC 자리차지 개체가 표 흐름에 요구하는 세로 범위.
    ///
    /// 한컴의 `쪽 영역 안으로 제한`은 세로 기준이 문단일 때 개체를 쪽 영역 안에
    /// 남기도록 흐름 높이에 반영된다. 반대로 제한이 꺼진 문단 기준 floating
    /// 개체는 표 행 높이를 밀지 않는다.
    pub(crate) fn non_inline_control_flow_height(&self, common: &CommonObjAttr) -> f64 {
        if common.treat_as_char || !matches!(common.text_wrap, TextWrap::TopAndBottom) {
            return 0.0;
        }
        let object_height = hwpunit_to_px(common.height as i32, self.dpi);
        if matches!(common.vert_rel_to, VertRelTo::Para) {
            if common.flow_with_text {
                hwpunit_to_px((common.vertical_offset as i32).max(0), self.dpi) + object_height
            } else {
                0.0
            }
        } else {
            object_height
        }
    }

    pub(crate) fn calc_non_inline_controls_flow_height(&self, paragraphs: &[Paragraph]) -> f64 {
        paragraphs
            .iter()
            .flat_map(|p| p.controls.iter())
            .map(|ctrl| match ctrl {
                Control::Picture(pic) => self.non_inline_control_flow_height(&pic.common),
                Control::Shape(shape) => self.non_inline_control_flow_height(shape.common()),
                _ => 0.0,
            })
            .sum()
    }

    fn cell_wrap_object_visual_bottom(&self, common: &CommonObjAttr) -> f64 {
        if common.treat_as_char {
            return 0.0;
        }
        if !matches!(
            common.text_wrap,
            TextWrap::Square | TextWrap::Tight | TextWrap::Through
        ) {
            return 0.0;
        }

        let object_height = hwpunit_to_px(common.height as i32, self.dpi);
        let top_offset = if matches!(common.vert_rel_to, VertRelTo::Para) {
            hwpunit_to_px((common.vertical_offset as i32).max(0), self.dpi)
        } else {
            0.0
        };
        top_offset + object_height
    }

    pub(crate) fn calc_cell_wrap_objects_bottom_height(&self, paragraphs: &[Paragraph]) -> f64 {
        paragraphs
            .iter()
            .map(|p| {
                let para_top = p
                    .line_segs
                    .first()
                    .map(|s| hwpunit_to_px(s.vertical_pos, self.dpi))
                    .unwrap_or(0.0);
                let object_bottom = p
                    .controls
                    .iter()
                    .map(|ctrl| match ctrl {
                        Control::Picture(pic) => self.cell_wrap_object_visual_bottom(&pic.common),
                        Control::Shape(shape) => {
                            self.cell_wrap_object_visual_bottom(shape.common())
                        }
                        _ => 0.0,
                    })
                    .fold(0.0f64, f64::max);
                if object_bottom > 0.0 {
                    para_top + object_bottom
                } else {
                    0.0
                }
            })
            .fold(0.0f64, f64::max)
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn layout_table(
        &self,
        tree: &mut PageRenderTree,
        col_node: &mut RenderNode,
        table: &crate::model::table::Table,
        section_index: usize,
        styles: &ResolvedStyleSet,
        outline_numbering_id: u16,
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
        clamp_header_negative_para_offset: bool,
    ) -> f64 {
        if table.cells.is_empty() {
            if depth == 0 {
                return y_start;
            } else {
                return 0.0;
            }
        }
        let header_footer_padding_compat = matches!(
            col_node.node_type,
            RenderNodeType::Header | RenderNodeType::Footer | RenderNodeType::MasterPage
        );
        // 1x1 래퍼 표 감지: 외곽 표를 무시하고 내부 표를 직접 렌더링.
        // (Task #688) 셀 paragraphs 가 2개 이상이면 첫 nested 표만 unwrap 시 나머지
        // paragraph 의 nested 표가 누락되므로 paragraphs.len() == 1 가드를 둔다.
        // controls.len() == 1 가드는 두지 않는다 — exam_social.hwp pi=15 (PR #681)
        // 처럼 정렬 마커 등 다른 control 이 동거하는 케이스에서 unwrap + 외곽선 분기를
        // 모두 보존해야 하므로 find_map 으로 첫 nested table 만 추출한다.
        if table.row_count == 1 && table.col_count == 1 && table.cells.len() == 1 {
            let cell = &table.cells[0];
            if cell.paragraphs.len() == 1 {
                let p = &cell.paragraphs[0];
                let has_visible_text = p
                    .text
                    .chars()
                    .any(|ch| !ch.is_whitespace() && ch != '\r' && ch != '\n');
                if !has_visible_text {
                    if let Some(nested) = p.controls.iter().find_map(|c| {
                        if let Control::Table(t) = c {
                            Some(t.as_ref())
                        } else {
                            None
                        }
                    }) {
                        // [Task: nested-table-border] 자료 박스 외곽 테두리 추가:
                        // 외부 1x1 표가 wrapper 라도 padding + border_fill 에 테두리선이
                        // 정의된 경우 (자료 박스 외곽), 외곽 4개 라인을 별도 추가하여 시각 정합.
                        // 외곽 박스의 size 는 nested layout 의 실제 결과 (y_end - y_start) 와
                        // nested 표의 측정 width 를 사용하여 내부 표 영역과 정확히 정합.
                        // (exam_social.hwp pi=15 4번 자료 박스: 외부 1x1 padding=(850,850,850,850)
                        //  border_fill_id=6, 내부 6x3 대화체 셀.)
                        let outer_y = y_start;
                        let outer_border_meta = if depth == 0 {
                            let has_outer_padding = cell.padding.left != 0
                                || cell.padding.right != 0
                                || cell.padding.top != 0
                                || cell.padding.bottom != 0;
                            if has_outer_padding {
                                // border_fill_id 는 1-based(borderFillIDRef), border_styles 는
                                // 0-based Vec 이므로 -1 변환한다. (일반 셀/표/zone lookup 과 동일)
                                if let Some(bs) = styles
                                    .border_styles
                                    .get((cell.border_fill_id as usize).saturating_sub(1))
                                {
                                    let any_border = bs.borders.iter().any(|b| {
                                        b.line_type != crate::model::style::BorderLineType::None
                                    });
                                    if any_border {
                                        Some(bs.borders)
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        } else {
                            None
                        };

                        // nested 표 위치/size 미리 결정 (nested layout 의 위치 결정 logic 동일)
                        let pw_now = self.current_paper_width.get();
                        let paper_w = if pw_now > 0.0 { Some(pw_now) } else { None };
                        let nested_w = hwpunit_to_px(nested.common.width as i32, self.dpi);
                        let outer_w_for_box = nested_w;
                        let outer_x_for_box = self.compute_table_x_position(
                            nested,
                            nested_w,
                            col_area,
                            depth,
                            host_alignment,
                            host_margin_left,
                            host_margin_right,
                            inline_x_override,
                            paper_w,
                        );

                        let y_end = self.layout_table(
                            tree,
                            col_node,
                            nested,
                            section_index,
                            styles,
                            outline_numbering_id,
                            col_area,
                            y_start,
                            bin_data_content,
                            None,
                            depth,
                            table_meta,
                            host_alignment,
                            enclosing_cell_ctx,
                            host_margin_left,
                            host_margin_right,
                            inline_x_override,
                            nested_split,
                            para_y,
                            clamp_header_negative_para_offset,
                        );

                        if let Some(bs_borders) = outer_border_meta {
                            let outer_h_actual = (y_end - outer_y).max(0.0);
                            if outer_h_actual > 0.0 {
                                use super::border_rendering::create_border_line_nodes;
                                // 좌
                                col_node.children.extend(create_border_line_nodes(
                                    tree,
                                    &bs_borders[0],
                                    outer_x_for_box,
                                    outer_y,
                                    outer_x_for_box,
                                    outer_y + outer_h_actual,
                                ));
                                // 우
                                col_node.children.extend(create_border_line_nodes(
                                    tree,
                                    &bs_borders[1],
                                    outer_x_for_box + outer_w_for_box,
                                    outer_y,
                                    outer_x_for_box + outer_w_for_box,
                                    outer_y + outer_h_actual,
                                ));
                                // 상
                                col_node.children.extend(create_border_line_nodes(
                                    tree,
                                    &bs_borders[2],
                                    outer_x_for_box,
                                    outer_y,
                                    outer_x_for_box + outer_w_for_box,
                                    outer_y,
                                ));
                                // 하
                                col_node.children.extend(create_border_line_nodes(
                                    tree,
                                    &bs_borders[3],
                                    outer_x_for_box,
                                    outer_y + outer_h_actual,
                                    outer_x_for_box + outer_w_for_box,
                                    outer_y + outer_h_actual,
                                ));
                            }
                        }
                        return y_end;
                    }
                }
            }
        }

        let col_count = table.col_count as usize;
        let row_count = table.row_count as usize;
        let cell_spacing = hwpunit_to_px(table.cell_spacing as i32, self.dpi);

        // ── 1. 열 폭 + 행 높이 계산 ──
        let col_widths = self.resolve_column_widths(table, col_count);
        let row_heights =
            self.resolve_row_heights(table, col_count, row_count, measured_table, styles);

        // ── 2. 누적 위치 계산 ──
        let mut col_x = vec![0.0f64; col_count + 1];
        for i in 0..col_count {
            col_x[i + 1] =
                col_x[i] + col_widths[i] + if i + 1 < col_count { cell_spacing } else { 0.0 };
        }
        let mut row_y = vec![0.0f64; row_count + 1];
        for i in 0..row_count {
            row_y[i + 1] =
                row_y[i] + row_heights[i] + if i + 1 < row_count { cell_spacing } else { 0.0 };
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

        let row_col_x = build_row_col_x(
            table,
            &col_widths,
            col_count,
            row_count,
            cell_spacing,
            self.dpi,
        );
        let independent_col_row_y = if split_row_range.is_none() && !table.common.treat_as_char {
            let col_row_y = build_col_row_y_from_cell_heights(
                table,
                &row_heights,
                &row_y,
                col_count,
                row_count,
                cell_spacing,
                self.dpi,
            );
            if has_independent_col_row_y(&col_row_y, &row_y) {
                Some(col_row_y)
            } else {
                None
            }
        } else {
            None
        };

        let table_width = row_col_x
            .iter()
            .map(|rx| rx.last().copied().unwrap_or(0.0))
            .fold(col_x.last().copied().unwrap_or(0.0), f64::max);
        let table_height = if let Some(col_row_y) = independent_col_row_y.as_ref() {
            col_row_y
                .iter()
                .filter_map(|cy| cy.last().copied())
                .fold(row_y.last().copied().unwrap_or(0.0), f64::max)
        } else if let Some((_, er)) = split_row_range {
            row_y[er].max(0.0)
        } else {
            row_y.last().copied().unwrap_or(0.0)
        };

        // ── 3. 위치 결정 ──
        let pw = self.current_paper_width.get();
        let paper_w = if pw > 0.0 { Some(pw) } else { None };
        let mut table_x = self.compute_table_x_position(
            table,
            table_width,
            col_area,
            depth,
            host_alignment,
            host_margin_left,
            host_margin_right,
            inline_x_override,
            paper_w,
        );

        let (caption_height, caption_spacing) = if depth == 0 {
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

        // Left 캡션: 표를 캡션 크기만큼 오른쪽으로 이동
        if depth == 0 {
            if let Some(ref cap) = table.caption {
                if matches!(cap.direction, crate::model::shape::CaptionDirection::Left) {
                    let cap_w = hwpunit_to_px(cap.width as i32, self.dpi);
                    table_x += cap_w + caption_spacing;
                }
            }
        }

        let table_text_wrap = if depth == 0 {
            table.common.text_wrap
        } else {
            crate::model::shape::TextWrap::Square
        };
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
                table,
                table_height,
                y_start,
                col_area,
                depth,
                caption_height,
                caption_spacing,
                para_y,
            ) - split_y_offset
        };
        let inline_table_flow_y_shift = if inline_x_override.is_some() {
            para_y
                .map(|anchor_y| (table_y - anchor_y).max(0.0))
                .unwrap_or(0.0)
        } else {
            0.0
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
                    tree,
                    &mut table_node,
                    Some(tbl_bs),
                    table_x,
                    table_y,
                    table_width,
                    table_height,
                    bin_data_content,
                );
            }
        }

        // ── 4-2. cellzone 배경 렌더링 (zone 전체 영역에 한 번) ──
        for zone in &table.zones {
            if zone.border_fill_id == 0 {
                continue;
            }
            let zone_idx = (zone.border_fill_id as usize).saturating_sub(1);
            if let Some(zone_bs) = styles.border_styles.get(zone_idx) {
                // zone 영역의 좌표 계산
                let sc = zone.start_col as usize;
                let ec = (zone.end_col as usize + 1).min(col_count);
                let sr = zone.start_row as usize;
                let er = (zone.end_row as usize + 1).min(row_count);
                if sc < col_count && sr < row_count {
                    let zone_x = table_x
                        + row_col_x
                            .get(sr)
                            .and_then(|r| r.get(sc))
                            .copied()
                            .unwrap_or(0.0);
                    let zone_y = table_y + row_y.get(sr).copied().unwrap_or(0.0);
                    let zone_x_end = table_x
                        + row_col_x
                            .get(sr)
                            .and_then(|r| {
                                if ec < r.len() {
                                    Some(r[ec])
                                } else {
                                    r.last().map(|&last_x| {
                                        // 마지막 열 끝 = 마지막 열 시작 + 해당 셀 너비
                                        let last_col = r.len() - 1;
                                        table
                                            .cells
                                            .iter()
                                            .find(|c| {
                                                c.row as usize == sr && c.col as usize == last_col
                                            })
                                            .map(|c| {
                                                last_x + hwpunit_to_px(c.width as i32, self.dpi)
                                            })
                                            .unwrap_or(last_x)
                                    })
                                }
                            })
                            .unwrap_or(0.0);
                    let zone_y_end = table_y
                        + row_y.get(er).copied().unwrap_or_else(|| {
                            // 마지막 행 끝 = 마지막 행 시작 + 해당 행 높이
                            row_y.get(er - 1).copied().unwrap_or(0.0)
                                + table
                                    .row_sizes
                                    .get(er - 1)
                                    .map(|&h| hwpunit_to_px(h as i32, self.dpi))
                                    .unwrap_or(0.0)
                        });
                    let zone_w = (zone_x_end - zone_x).max(0.0);
                    let zone_h = (zone_y_end - zone_y).max(0.0);
                    // [Task #429] 단색/패턴/그라데이션 + 이미지 채우기 (zone 의 별도 image fill 처리는
                    // render_cell_background 가 통합 처리하므로 제거)
                    self.render_cell_background(
                        tree,
                        &mut table_node,
                        Some(zone_bs),
                        zone_x,
                        zone_y,
                        zone_w,
                        zone_h,
                        bin_data_content,
                    );
                }
            }
        }

        // ── 5. 셀 레이아웃 ──
        let mut h_edges: Vec<Vec<Option<BorderLine>>> = vec![vec![None; col_count]; row_count + 1];
        let mut v_edges: Vec<Vec<Option<BorderLine>>> = vec![vec![None; row_count]; col_count + 1];

        self.layout_table_cells(
            tree,
            &mut table_node,
            table,
            section_index,
            styles,
            outline_numbering_id,
            col_area,
            bin_data_content,
            depth,
            table_meta,
            enclosing_cell_ctx,
            &row_col_x,
            &row_y,
            independent_col_row_y.as_deref(),
            col_count,
            row_count,
            table_x,
            table_y,
            &mut h_edges,
            &mut v_edges,
            split_row_range,
            row_y_shift,
            clamp_header_negative_para_offset,
            inline_table_flow_y_shift,
            header_footer_padding_compat,
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
                    if c >= col_count || r >= row_count {
                        continue;
                    }
                    let ec = (c + cell.col_span as usize).min(col_count);
                    let er = (r + cell.row_span as usize).min(row_count);
                    // 상단
                    if r == 0 {
                        for cc in c..ec {
                            h_covered[0][cc] = true;
                        }
                    }
                    // 하단
                    if er == row_count {
                        for cc in c..ec {
                            h_covered[row_count][cc] = true;
                        }
                    }
                    // 좌측
                    if c == 0 {
                        for rr in r..er {
                            v_covered[0][rr] = true;
                        }
                    }
                    // 우측
                    if ec == col_count {
                        for rr in r..er {
                            v_covered[col_count][rr] = true;
                        }
                    }
                }

                // 셀이 커버하지 않는 외곽 엣지에만 fallback 적용
                for c in 0..col_count {
                    if h_edges[0][c].is_none() && !h_covered[0][c] {
                        let b = &borders[2];
                        if !matches!(b.line_type, crate::model::style::BorderLineType::None) {
                            h_edges[0][c] = Some(*b);
                        }
                    }
                    if h_edges[row_count][c].is_none() && !h_covered[row_count][c] {
                        let b = &borders[3];
                        if !matches!(b.line_type, crate::model::style::BorderLineType::None) {
                            h_edges[row_count][c] = Some(*b);
                        }
                    }
                }
                for r in 0..row_count {
                    if v_edges[0][r].is_none() && !v_covered[0][r] {
                        let b = &borders[0];
                        if !matches!(b.line_type, crate::model::style::BorderLineType::None) {
                            v_edges[0][r] = Some(*b);
                        }
                    }
                    if v_edges[col_count][r].is_none() && !v_covered[col_count][r] {
                        let b = &borders[1];
                        if !matches!(b.line_type, crate::model::style::BorderLineType::None) {
                            v_edges[col_count][r] = Some(*b);
                        }
                    }
                }
            }
        }

        // ── 6. 테두리 렌더링 ──
        if independent_col_row_y.is_none() {
            table_node.children.extend(render_edge_borders(
                tree, &h_edges, &v_edges, &row_col_x, &row_y, table_x, table_y,
            ));
            if self.show_transparent_borders.get() {
                table_node.children.extend(render_transparent_borders(
                    tree, &h_edges, &v_edges, &row_col_x, &row_y, table_x, table_y,
                ));
            }
        }

        col_node.children.push(table_node);

        // ── 7. 캡션 렌더링 ──
        if depth == 0 {
            if let Some(ref caption) = table.caption {
                use crate::model::shape::{CaptionDirection, CaptionVertAlign};
                let (cap_x, cap_w, cap_y) = match caption.direction {
                    CaptionDirection::Top => (table_x, table_width, y_start),
                    CaptionDirection::Bottom => (
                        table_x,
                        table_width,
                        table_y + table_height + caption_spacing,
                    ),
                    CaptionDirection::Left | CaptionDirection::Right => {
                        let cw = hwpunit_to_px(caption.width as i32, self.dpi);
                        let cx = if caption.direction == CaptionDirection::Left {
                            table_x - cw - caption_spacing
                        } else {
                            table_x + table_width + caption_spacing
                        };
                        let cy = match caption.vert_align {
                            CaptionVertAlign::Top => table_y,
                            CaptionVertAlign::Center => {
                                table_y + (table_height - caption_height).max(0.0) / 2.0
                            }
                            CaptionVertAlign::Bottom => {
                                table_y + (table_height - caption_height).max(0.0)
                            }
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
                    tree,
                    col_node,
                    caption,
                    styles,
                    col_area,
                    cap_x,
                    cap_w,
                    cap_y,
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
                matches!(
                    c.direction,
                    CaptionDirection::Left | CaptionDirection::Right
                )
            });
            let caption_extra = if is_lr_cap {
                0.0
            } else {
                caption_height
                    + if caption_height > 0.0 {
                        caption_spacing
                    } else {
                        0.0
                    }
            };
            if matches!(
                table_text_wrap,
                crate::model::shape::TextWrap::BehindText
                    | crate::model::shape::TextWrap::InFrontOfText
            ) {
                // 글뒤로/글앞으로: y_offset 변경 없음
                y_start
            } else if matches!(table_text_wrap, crate::model::shape::TextWrap::TopAndBottom)
                && !table.common.treat_as_char
            {
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
        let inferred_local_resize_rows = table.inferred_local_resize_rows();
        let mut col_widths = vec![0.0f64; col_count];
        for cell in &table.cells {
            if table.local_resize_rows.contains(&cell.row)
                || inferred_local_resize_rows.contains(&cell.row)
            {
                continue;
            }
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
                if table.local_resize_rows.contains(&cell.row)
                    || inferred_local_resize_rows.contains(&cell.row)
                {
                    continue;
                }
                let c = cell.col as usize;
                let span = cell.col_span as usize;
                if span > 1 && c + span <= col_count {
                    let total_w = hwpunit_to_px(cell.width as i32, self.dpi);
                    if let Some(existing) = constraints.iter_mut().find(|x| x.0 == c && x.1 == span)
                    {
                        if total_w > existing.2 {
                            existing.2 = total_w;
                        }
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
                    let unknown_cols: Vec<usize> =
                        (c..c + span).filter(|&i| col_widths[i] == 0.0).collect();
                    if unknown_cols.len() == 1 {
                        let remaining = (total_w - known_sum).max(0.0);
                        col_widths[unknown_cols[0]] = remaining;
                        progress = true;
                    }
                }
                if !progress {
                    break;
                }
            }

            for &(c, span, total_w) in &constraints {
                let known_sum: f64 = (c..c + span).map(|i| col_widths[i]).sum();
                let unknown_cols: Vec<usize> =
                    (c..c + span).filter(|&i| col_widths[i] == 0.0).collect();
                if !unknown_cols.is_empty() {
                    let remaining = (total_w - known_sum).max(0.0);
                    let per_col = remaining / unknown_cols.len() as f64;
                    for i in unknown_cols {
                        col_widths[i] = per_col;
                    }
                }
            }

            // 병합 셀 제약이 이미 값이 있는 열들로만 구성되어도 총합이 더 클 수 있다.
            // 한컴은 이 경우 뒤쪽 열을 확장해 병합 셀 폭을 만족시킨다.
            for &(c, span, total_w) in &constraints {
                let known_sum: f64 = (c..c + span).map(|i| col_widths[i]).sum();
                let deficit = total_w - known_sum;
                if deficit > 0.5 {
                    let target_col = c + span - 1;
                    if target_col < col_widths.len() {
                        col_widths[target_col] += deficit;
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
        let target_width = if table.common.width > 0 {
            hwpunit_to_px(table.common.width as i32, self.dpi)
        } else {
            0.0
        };
        if target_width > 0.0 {
            let current: f64 = col_widths.iter().sum();
            let residual = target_width - current;
            if residual > 0.5 {
                if let Some(last) = col_widths.last_mut() {
                    *last += residual;
                }
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
        self.resolve_row_heights_with_common_fit(
            table,
            col_count,
            row_count,
            measured_table,
            styles,
            true,
        )
    }

    fn resolve_row_heights_for_content(
        &self,
        table: &crate::model::table::Table,
        col_count: usize,
        row_count: usize,
        measured_table: Option<&MeasuredTable>,
        styles: &ResolvedStyleSet,
    ) -> Vec<f64> {
        self.resolve_row_heights_with_common_fit(
            table,
            col_count,
            row_count,
            measured_table,
            styles,
            false,
        )
    }

    fn resolve_row_heights_with_common_fit(
        &self,
        table: &crate::model::table::Table,
        col_count: usize,
        row_count: usize,
        measured_table: Option<&MeasuredTable>,
        styles: &ResolvedStyleSet,
        fit_common_height: bool,
    ) -> Vec<f64> {
        if let Some(mt) = measured_table {
            let mut rh = mt.row_heights.clone();
            rh.resize(row_count, hwpunit_to_px(400, self.dpi));
            if fit_common_height {
                self.fit_row_heights_to_common_height(table, &mut rh);
            }
            return rh;
        }

        // 1단계: row_span==1인 셀에서 개별 행 높이 추출
        let mut row_heights = vec![0.0f64; row_count];
        for cell in &table.cells {
            if table.local_resize_cols.contains(&cell.col) {
                continue;
            }
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
            if table.local_resize_cols.contains(&cell.col) {
                continue;
            }
            if cell.row_span == 1 && (cell.row as usize) < row_count {
                let r = cell.row as usize;
                let (pad_left, pad_right, pad_top, pad_bottom) =
                    self.resolve_cell_padding(cell, table);

                let content_height = if cell.text_direction != 0 {
                    // 세로쓰기: line_seg.segment_width가 열의 세로 길이
                    self.calc_vertical_cell_content_height(&cell.paragraphs)
                } else {
                    let cell_w_px = hwpunit_to_px(cell.width as i32, self.dpi);
                    let inner_width = (cell_w_px - pad_left - pad_right).max(0.0);
                    self.calc_cell_paragraphs_content_height(&cell.paragraphs, styles, inner_width)
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
                if table.local_resize_cols.contains(&cell.col) {
                    continue;
                }
                let r = cell.row as usize;
                let span = cell.row_span as usize;
                if span > 1 && r + span <= row_count && cell.height < 0x80000000 {
                    let total_h = hwpunit_to_px(cell.height as i32, self.dpi);
                    if let Some(existing) = constraints.iter_mut().find(|x| x.0 == r && x.1 == span)
                    {
                        if total_h > existing.2 {
                            existing.2 = total_h;
                        }
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
                    let unknown_rows: Vec<usize> =
                        (r..r + span).filter(|&i| row_heights[i] == 0.0).collect();
                    if unknown_rows.len() == 1 {
                        let remaining = (total_h - known_sum).max(0.0);
                        row_heights[unknown_rows[0]] = remaining;
                        progress = true;
                    }
                }
                if !progress {
                    break;
                }
            }
            for &(r, span, total_h) in &constraints {
                let known_sum: f64 = (r..r + span).map(|i| row_heights[i]).sum();
                let unknown_rows: Vec<usize> =
                    (r..r + span).filter(|&i| row_heights[i] == 0.0).collect();
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
            if table.local_resize_cols.contains(&cell.col) {
                continue;
            }
            let r = cell.row as usize;
            let span = cell.row_span as usize;
            if span > 1 && r + span <= row_count {
                let (pad_left, pad_right, pad_top, pad_bottom) =
                    self.resolve_cell_padding(cell, table);
                let cell_w_px = hwpunit_to_px(cell.width as i32, self.dpi);
                let inner_width = (cell_w_px - pad_left - pad_right).max(0.0);
                let content_height =
                    self.calc_cell_paragraphs_content_height(&cell.paragraphs, styles, inner_width);
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
        if fit_common_height {
            self.fit_row_heights_to_common_height(table, &mut row_heights);
        }
        row_heights
    }

    fn fit_row_heights_to_common_height(
        &self,
        table: &crate::model::table::Table,
        row_heights: &mut [f64],
    ) {
        if row_heights.is_empty() {
            return;
        }
        let target_height = if table.common.height > 0 {
            hwpunit_to_px(table.common.height as i32, self.dpi)
        } else {
            0.0
        };
        if target_height > 0.0 {
            let current: f64 = row_heights.iter().sum();
            let residual = target_height - current;
            if residual > 0.5 {
                if let Some(last) = row_heights.last_mut() {
                    *last += residual;
                }
            }
        }
    }

    /// 셀 문단들의 콘텐츠 높이 합산 (spacing + line_height + line_spacing)
    pub(crate) fn calc_cell_paragraphs_content_height(
        &self,
        paragraphs: &[Paragraph],
        styles: &ResolvedStyleSet,
        cell_inner_width_px: f64,
    ) -> f64 {
        let cell_para_count = paragraphs.len();
        let line_based_height: f64 = paragraphs
            .iter()
            .enumerate()
            .map(|(pidx, p)| {
                let mut comp = compose_paragraph(p);
                // [Task #671] line_segs 비어 있는 셀 paragraph 의 단일 ComposedLine
                // 압축 결과를 셀 가용 너비에 맞춰 다중 ComposedLine 으로 재분할.
                // 측정/렌더링 일관성 보장 (table_layout.rs:1226 의 렌더링 경로와 동일).
                crate::renderer::composer::recompose_for_cell_width(
                    &mut comp,
                    p,
                    cell_inner_width_px,
                    styles,
                );
                self.calc_para_lines_height(
                    &comp.lines,
                    self.is_hwp3_variant.get() && p.line_segs.is_empty() && !p.text.is_empty(),
                    pidx,
                    cell_para_count,
                    styles.para_styles.get(p.para_shape_id as usize),
                    styles,
                )
            })
            .sum();
        line_based_height
            .max(self.calc_nested_controls_bottom_height(paragraphs, styles))
            .max(self.calc_non_inline_controls_flow_height(paragraphs))
            .max(self.calc_cell_wrap_objects_bottom_height(paragraphs))
    }

    /// pre-composed 문단들의 콘텐츠 높이 합산 (compose 생략)
    pub(crate) fn calc_composed_paras_content_height(
        &self,
        composed_paras: &[ComposedParagraph],
        paragraphs: &[Paragraph],
        styles: &ResolvedStyleSet,
    ) -> f64 {
        let cell_para_count = paragraphs.len();
        composed_paras
            .iter()
            .zip(paragraphs.iter())
            .enumerate()
            .map(|(pidx, (comp, para))| {
                self.calc_para_lines_height(
                    &comp.lines,
                    self.is_hwp3_variant.get()
                        && para.line_segs.is_empty()
                        && !para.text.is_empty(),
                    pidx,
                    cell_para_count,
                    styles.para_styles.get(para.para_shape_id as usize),
                    styles,
                )
            })
            .sum()
    }

    /// 단일 문단의 줄 높이 합산 (공통 로직)
    ///
    /// [Task #674] line_height 측정에 corrected_line_height 보정 적용.
    /// line_segs 부재 paragraph 의 fallback line_height (400 HU = 5.33 px) 가
    /// max_fs 보다 작은 경우 ParaShape 의 line_spacing_type + line_spacing 으로
    /// 보정. height_measurer.rs:570-587 와 동일 로직 — 측정/layout 일관성 보장.
    fn calc_para_lines_height(
        &self,
        lines: &[crate::renderer::composer::ComposedLine],
        hwp3_variant_synthetic: bool,
        pidx: usize,
        total_para_count: usize,
        para_style: Option<&crate::renderer::style_resolver::ResolvedParaStyle>,
        styles: &ResolvedStyleSet,
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
            let cell_ls_val = para_style.map(|s| s.line_spacing).unwrap_or(160.0);
            let cell_ls_type = para_style
                .map(|s| s.line_spacing_type)
                .unwrap_or(crate::model::style::LineSpacingType::Percent);
            let line_count = lines.len();
            let lines_total: f64 = lines
                .iter()
                .enumerate()
                .map(|(i, line)| {
                    let raw_lh = hwpunit_to_px(line.line_height, self.dpi);
                    let max_fs = line
                        .runs
                        .iter()
                        .map(|r| {
                            styles
                                .char_styles
                                .get(r.char_style_id as usize)
                                .map(|cs| cs.font_size)
                                .unwrap_or(0.0)
                        })
                        .fold(0.0f64, f64::max);
                    let h = crate::renderer::corrected_line_height_for_variant_synthetic(
                        raw_lh,
                        max_fs,
                        cell_ls_type,
                        cell_ls_val,
                        hwp3_variant_synthetic,
                    );
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
    fn calc_vertical_cell_content_height(&self, paragraphs: &[Paragraph]) -> f64 {
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

    /// 셀 패딩 계산
    pub(crate) fn resolve_cell_padding(
        &self,
        cell: &crate::model::table::Cell,
        table: &crate::model::table::Table,
    ) -> (f64, f64, f64, f64) {
        self.resolve_cell_padding_for_context(cell, table, false)
    }

    fn resolve_cell_padding_for_context(
        &self,
        cell: &crate::model::table::Cell,
        table: &crate::model::table::Table,
        allow_saved_small_cell_margin: bool,
    ) -> (f64, f64, f64, f64) {
        // HWP 스펙: aim(apply_inner_margin)=true → cell.padding,
        //           aim=false → table.padding 우선.
        // 한컴은 aim=false일 때 cell.padding 원값을 파일에 보존하더라도 렌더에는 쓰지 않는다.
        // aim=true에서는 0mm도 사용자가 지정한 셀 고유 안 여백으로 존중한다.
        let use_cell_left = Self::should_use_cell_padding_axis_for_context(
            cell,
            cell.padding.left,
            table.padding.left,
            allow_saved_small_cell_margin,
        );
        let use_cell_right = Self::should_use_cell_padding_axis_for_context(
            cell,
            cell.padding.right,
            table.padding.right,
            allow_saved_small_cell_margin,
        );
        let use_cell_top = Self::should_use_cell_padding_axis_for_context(
            cell,
            cell.padding.top,
            table.padding.top,
            allow_saved_small_cell_margin,
        );
        let use_cell_bottom = Self::should_use_cell_padding_axis_for_context(
            cell,
            cell.padding.bottom,
            table.padding.bottom,
            allow_saved_small_cell_margin,
        );

        let pad_left = if use_cell_left {
            hwpunit_to_px(cell.padding.left as i32, self.dpi)
        } else {
            hwpunit_to_px(table.padding.left as i32, self.dpi)
        };
        let pad_right = if use_cell_right {
            hwpunit_to_px(cell.padding.right as i32, self.dpi)
        } else {
            hwpunit_to_px(table.padding.right as i32, self.dpi)
        };
        let pad_top = if use_cell_top {
            hwpunit_to_px(cell.padding.top as i32, self.dpi)
        } else {
            hwpunit_to_px(table.padding.top as i32, self.dpi)
        };
        let pad_bottom = if use_cell_bottom {
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

    fn should_use_cell_padding_axis_for_context(
        cell: &crate::model::table::Cell,
        cell_padding: i16,
        table_padding: i16,
        allow_saved_small_cell_margin: bool,
    ) -> bool {
        if cell.apply_inner_margin {
            return cell_padding != 0;
        }

        if cell_padding <= table_padding {
            return false;
        }

        // 오래된 HWP/HWPX에는 hasMargin=0이어도 셀별 안여백 보존값이 렌더링에
        // 필요한 경우가 있다(KTX 목차, exam_kor 보기 박스 등). 다만 1443 샘플처럼
        // 사용자가 10mm급 명시 여백을 껐다가 저장한 값은 한컴이 렌더링에 쓰지 않는다.
        if !allow_saved_small_cell_margin && cell_padding >= 2500 {
            return false;
        }

        true
    }

    /// 셀 텍스트가 오버플로우할 때 좌우 패딩을 축소하여 공간을 확보한다.
    /// composed 문단의 각 줄 텍스트 폭을 측정하여 최대값이 가용 폭을 초과하면
    /// 패딩을 비례 축소한다 (최소 1px 보장).
    ///
    /// [Task #617] 다중 줄(2 줄 이상) 단락이 있는 셀은 HWP 가 가용 폭에 자간을
    /// 분배·줄바꿈을 확정한 상태이므로 padding 을 보존한다 (자연 폭 추정으로
    /// 다시 깎으면 본문이 테두리에 닿는 시각 오류 발생 — exam_kor.hwp
    /// 16/27/36번 보기 박스). 단일 줄 셀(좁은 수치 셀에서 오버플로우 가능성
    /// 있음) 은 종전 휴리스틱으로 보호한다.
    pub(crate) fn shrink_cell_padding_for_overflow(
        &self,
        pad_left: f64,
        pad_right: f64,
        cell_w: f64,
        composed_paras: &[ComposedParagraph],
        paragraphs: &[Paragraph],
        styles: &ResolvedStyleSet,
        preserve_cell_padding: bool,
    ) -> (f64, f64) {
        if preserve_cell_padding {
            return (pad_left, pad_right);
        }

        // [Task #617] 다중 줄(2 줄 이상) 단락이 line_segs 로 분배 완료된 경우,
        // HWP 가 가용 폭에 맞춰 자간을 분배하고 줄바꿈을 확정한 상태이므로
        // 자연 폭 추정으로 다시 깎으면 오버 페인팅. 단일 줄 셀(좁은 수치 셀
        // 등에서 오버플로우 가능성 있음) 은 종전 휴리스틱으로 보호한다.
        let any_multiline_distributed = paragraphs.iter().any(|p| p.line_segs.len() >= 2);
        if any_multiline_distributed {
            return (pad_left, pad_right);
        }

        let mut max_line_w = 0.0f64;
        for comp in composed_paras {
            for line in &comp.lines {
                let mut w = 0.0;
                for run in &line.runs {
                    let mut ts = resolved_to_text_style(styles, run.char_style_id, run.lang_index);
                    if run.char_overlap.is_some() {
                        let fs = if ts.font_size > 0.0 {
                            ts.font_size
                        } else {
                            12.0
                        };
                        let chars: Vec<char> = run.text.chars().collect();
                        w += fs
                            * crate::renderer::composer::char_overlap_advance_units(&chars) as f64;
                        continue;
                    }
                    // 자연 폭 측정: 음수 자간을 제거하여 글리프가 서로 겹치지 않는 최소 폭을 얻음
                    if ts.letter_spacing < 0.0 {
                        ts.letter_spacing = 0.0;
                    }
                    // [Task #555] PUA 옛한글 변환 후 자모 시퀀스 폭 사용.
                    // (estimate_text_width 는 ts.ratio 를 자체 반영함.)
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
        cell_x: f64,
        cell_y: f64,
        cell_w: f64,
        cell_h: f64,
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
            if let Some(img_content) =
                crate::renderer::layout::find_bin_data(bin_data_content, img_fill.bin_data_id)
            {
                let img_id = tree.next_id();
                let img_node = RenderNode::new(
                    img_id,
                    RenderNodeType::Image(ImageNode {
                        fill_mode: Some(img_fill.fill_mode),
                        brightness: img_fill.brightness,
                        contrast: img_fill.contrast,
                        effect: img_fill.effect,
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
                Alignment::Center | Alignment::Distribute => {
                    ref_x + (ref_w - table_width).max(0.0) / 2.0
                }
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
                    if body.2 > 0.0 {
                        (body.0, body.2)
                    } else {
                        (col_area.x, col_area.width)
                    }
                }
                HorzRelTo::Para => (
                    col_area.x + host_margin_left,
                    col_area.width - host_margin_left,
                ),
                _ => (col_area.x, col_area.width),
            };
            match horz_align {
                HorzAlign::Left | HorzAlign::Inside => ref_x + h_offset,
                HorzAlign::Center => ref_x + (ref_w - table_width).max(0.0) / 2.0 + h_offset,
                // Task #347: picture_footnote.rs:185와 동일하게 - h_offset (오른쪽 끝에서 안쪽으로 오프셋).
                HorzAlign::Right | HorzAlign::Outside => {
                    ref_x + (ref_w - table_width).max(0.0) - h_offset
                }
            }
        } else {
            // 중첩 표: outer_margin_left 적용 + host_alignment에 따라 셀 내에서 정렬
            let om_left = hwpunit_to_px(table.outer_margin_left as i32, self.dpi);
            let area_x = col_area.x + om_left;
            let area_w = (col_area.width - om_left).max(0.0);
            match host_alignment {
                Alignment::Center | Alignment::Distribute => {
                    area_x + (area_w - table_width).max(0.0) / 2.0
                }
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
        let table_text_wrap = if depth == 0 {
            table.common.text_wrap
        } else {
            crate::model::shape::TextWrap::Square
        };

        if depth == 0
            && !table_treat_as_char
            && matches!(
                table_text_wrap,
                crate::model::shape::TextWrap::TopAndBottom
                    | crate::model::shape::TextWrap::BehindText
                    | crate::model::shape::TextWrap::InFrontOfText
            )
        {
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
                    if body.3 > 0.0 {
                        (body.1, body.3)
                    } else {
                        (col_area.y, col_area.height)
                    }
                }
                crate::model::shape::VertRelTo::Para => {
                    (anchor_y, col_area.height - (anchor_y - col_area.y).max(0.0))
                }
                crate::model::shape::VertRelTo::Paper => (0.0, page_h_approx),
            };
            // Top 캡션: 표 위치를 캡션 높이만큼 아래로 이동
            let caption_top_offset = if let Some(ref cap) = table.caption {
                use crate::model::shape::CaptionDirection;
                if matches!(cap.direction, CaptionDirection::Top) {
                    caption_height
                        + if caption_height > 0.0 {
                            caption_spacing
                        } else {
                            0.0
                        }
                } else {
                    0.0
                }
            } else {
                0.0
            };
            let vert_align = table.common.vert_align;
            // [Task #898] Paper-relative 표는 v_offset 이 외곽 박스 (outer_margin 포함) 기준이므로
            // 가시 표 상단 = v_offset + outer_margin_top. 한컴 PDF (exam_math.hwp 바탕쪽 쪽번호 박스) 정합.
            let om_top_px = if matches!(vert_rel_to, crate::model::shape::VertRelTo::Paper) {
                hwpunit_to_px(table.outer_margin_top as i32, self.dpi)
            } else {
                0.0
            };
            let om_bottom_px = if matches!(vert_rel_to, crate::model::shape::VertRelTo::Paper) {
                hwpunit_to_px(table.outer_margin_bottom as i32, self.dpi)
            } else {
                0.0
            };
            let raw_y = match vert_align {
                crate::model::shape::VertAlign::Top | crate::model::shape::VertAlign::Inside => {
                    ref_y + v_offset + caption_top_offset + om_top_px
                }
                crate::model::shape::VertAlign::Center => {
                    ref_y + (ref_h - table_height) / 2.0 + v_offset + caption_top_offset
                }
                crate::model::shape::VertAlign::Bottom
                | crate::model::shape::VertAlign::Outside => {
                    ref_y + ref_h - table_height - v_offset + caption_top_offset - om_bottom_px
                }
            };
            // Para 기준 + bit 13: 본문 영역으로 제한
            // 앞선 표/텍스트가 차지한 영역(y_start) 아래로 밀어내고, 본문 영역 내로 클램핑
            // Task #347: TopAndBottom 만 y_start 이하로 밀어냄. 글뒤로(BehindText) /
            // 글앞으로(InFrontOfText) 표는 절대 위치 오버레이이므로 push-down 미적용.
            if matches!(vert_rel_to, crate::model::shape::VertRelTo::Para) {
                let body_top = col_area.y;
                let body_bottom = col_area.y + col_area.height - table_height;
                let pushed =
                    if matches!(table_text_wrap, crate::model::shape::TextWrap::TopAndBottom) {
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
            } else {
                0.0
            };
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
        outline_numbering_id: u16,
        col_area: &LayoutRect,
        bin_data_content: &[BinDataContent],
        depth: usize,
        table_meta: Option<(usize, usize)>,
        enclosing_cell_ctx: Option<CellContext>,
        row_col_x: &[Vec<f64>],
        row_y: &[f64],
        independent_col_row_y: Option<&[Vec<f64>]>,
        col_count: usize,
        row_count: usize,
        table_x: f64,
        table_y: f64,
        h_edges: &mut Vec<Vec<Option<BorderLine>>>,
        v_edges: &mut Vec<Vec<Option<BorderLine>>>,
        row_filter: Option<(usize, usize)>,
        row_y_shift: f64,
        clamp_header_negative_para_offset: bool,
        inline_table_flow_y_shift: f64,
        header_footer_padding_compat: bool,
    ) {
        let mut independent_border_nodes: Vec<RenderNode> = Vec::new();
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
            let cell_col_y = independent_col_row_y.and_then(|col_y| col_y.get(c));
            // row_y는 이미 시프트된 상태이므로 음수일 수 있음 (start_row 이전 행).
            // 독립 셀 높이가 있는 표는 해당 열의 누적 y를 사용한다.
            let raw_cell_y = table_y
                + cell_col_y
                    .and_then(|cy| cy.get(r).copied())
                    .unwrap_or(row_y[r]);
            let cell_y = if row_filter.is_some() {
                raw_cell_y.max(table_y)
            } else {
                raw_cell_y
            };
            let end_col = (c + cell.col_span as usize).min(col_count);
            let end_row = (r + cell.row_span as usize).min(row_count);
            let cell_w = row_col_x[r][end_col] - row_col_x[r][c];
            let raw_cell_h = cell_col_y
                .and_then(|cy| {
                    let start = cy.get(r).copied()?;
                    let end = cy.get(end_row).copied()?;
                    Some(end - start)
                })
                .unwrap_or_else(|| row_y[end_row] - row_y[r]);
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

            // 셀 패딩 (cell.padding이 0이면 table.padding fallback)
            let (mut pad_left, mut pad_right, pad_top, pad_bottom) =
                self.resolve_cell_padding_for_context(cell, table, header_footer_padding_compat);

            let mut composed_paras: Vec<_> = cell
                .paragraphs
                .iter()
                .map(|p| compose_paragraph(p))
                .collect();

            // [Task #1073] 중첩 표 분할 연속 페이지(row_filter sr>0)에서 분할 시작 행보다
            // 먼저 시작한 rowspan 셀(r < sr)은 라벨이 이전 페이지에 이미 렌더됨 → 연속
            // 페이지에선 공란(영역/배경만, 텍스트 미렌더). 외부 표 advance_row_block_cut 의
            // rs>1 라벨 공란 정합. row_filter 는 중첩 표 분할 전용(외부 표는 별도 경로).
            if let Some((sr, _)) = row_filter {
                if sr > 0 && r < sr {
                    composed_paras.clear();
                }
            }

            // 텍스트 오버플로우 시 좌우 패딩 축소.
            // 1443 셀 안여백 샘플처럼 큰 명시 좌우 여백은 한컴과 같이 보존하되,
            // 기존 문서의 1~4mm급 일반 셀 여백은 종전 오버플로우 방어를 유지한다.
            let preserve_explicit_horizontal_padding =
                cell.apply_inner_margin && cell.padding.left.max(cell.padding.right) >= 1700;
            let (new_pl, new_pr) = self.shrink_cell_padding_for_overflow(
                pad_left,
                pad_right,
                cell_w,
                &composed_paras,
                &cell.paragraphs,
                styles,
                preserve_explicit_horizontal_padding,
            );
            pad_left = new_pl;
            pad_right = new_pr;

            let inner_x = cell_x + pad_left;
            let inner_width = (cell_w - pad_left - pad_right).max(0.0);
            let inner_height = (cell_h - pad_top - pad_bottom).max(0.0);

            // [Task #671] line_segs 비어 있는 셀 paragraph 의 단일 ComposedLine 압축
            // 결과를 셀 가용 너비 (inner_width) 에 맞춰 다중 ComposedLine 으로 재분할.
            // 한컴이 PARA_LINE_SEG 를 인코딩하지 않은 케이스 (samples/계획서.hwp) 의
            // 줄겹침 시각 결함 정정. 정상 line_segs 인코딩된 paragraph 는 무영향.
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

            // AutoNumber(Page) 치환: 셀 내 쪽번호 필드를 현재 페이지 번호로 변환
            let current_pn = self.current_page_number.get();
            if current_pn > 0 {
                for (cpi, para) in cell.paragraphs.iter().enumerate() {
                    if para.controls.iter().any(|c| {
                        matches!(c, Control::AutoNumber(an)
                            if an.number_type == crate::model::control::AutoNumberType::Page)
                    }) {
                        if let Some(comp) = composed_paras.get_mut(cpi) {
                            self.substitute_page_auto_numbers_in_composed(para, comp, current_pn);
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
                    &composed_paras,
                    &cell.paragraphs,
                    styles,
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
                                    text_height += self.non_inline_control_flow_height(&pic.common);
                                }
                            }
                            Control::Shape(shape) => {
                                let shape_h = hwpunit_to_px(shape.common().height as i32, self.dpi);
                                if shape.common().treat_as_char {
                                    if shape_h > max_inline_height {
                                        max_inline_height = shape_h;
                                    }
                                } else {
                                    text_height +=
                                        self.non_inline_control_flow_height(shape.common());
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

                let nested_bottom =
                    self.calc_nested_controls_bottom_height(&cell.paragraphs, styles);
                let wrap_object_bottom =
                    self.calc_cell_wrap_objects_bottom_height(&cell.paragraphs);
                composed_height
                    .max(vpos_height)
                    .max(nested_bottom)
                    .max(wrap_object_bottom)
            };

            // 수직 정렬 (분할 표에서는 Top 강제 — 보이는 영역이 전체 셀보다 작음)
            let effective_valign = if row_filter.is_some() {
                VerticalAlign::Top
            } else {
                cell.vertical_align
            };
            // Task #347: HWP는 LineSeg.vertical_pos에 첫 줄의 절대 위치(셀 내부 컨텐츠 상단부터)
            // 를 기록한다. 다만 이 값을 모든 vertical_align에 곧바로 적용하면 Center/Bottom
            // 지정 셀도 Top처럼 배치된다. vpos 앵커링은 Top 셀의 세부 줄 위치 보정으로만
            // 사용하고, Center/Bottom은 전체 콘텐츠 높이 기반의 기존 정렬 계산을 유지한다.
            // 단, line_segs가 비어있는 Top 케이스는 기존 폴백 유지.
            // [Task #362] 셀 안에 nested table 이 있는 경우 vpos 적용 제외.
            // nested table 케이스에서 LineSeg.vpos 가 셀 콘텐츠 시작 오프셋 의미가 아니라
            // 셀 안의 누적 위치로 사용되어, vpos 를 추가하면 콘텐츠가 표 높이를 초과하여 클립 발생.
            // (kps-ai p56 case: 외부 셀 vpos=2000HU 가 추가되어 19.5px 클립.)
            let has_nested_table = cell
                .paragraphs
                .iter()
                .any(|p| p.controls.iter().any(|c| matches!(c, Control::Table(_))));
            let first_line_vpos = cell
                .paragraphs
                .first()
                .and_then(|p| p.line_segs.first())
                .map(|ls| hwpunit_to_px(ls.vertical_pos, self.dpi));
            let use_top_vpos_anchor = matches!(effective_valign, VerticalAlign::Top);
            let text_y_start = if use_top_vpos_anchor
                && !has_nested_table
                && first_line_vpos.filter(|&v| v > 0.0).is_some()
            {
                // vpos는 셀 컨텐츠 상단(=cell_y+pad_top)으로부터의 첫 줄 top y 오프셋
                cell_y + pad_top + first_line_vpos.unwrap()
            } else {
                match effective_valign {
                    VerticalAlign::Top => cell_y + pad_top,
                    VerticalAlign::Center => {
                        let mechanical_offset =
                            (inner_height - total_content_height).max(0.0) / 2.0;
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
                    tree,
                    &mut cell_node,
                    &composed_paras,
                    &cell.paragraphs,
                    styles,
                    &vert_inner_area,
                    cell.vertical_align,
                    cell.text_direction,
                    section_index,
                    table_meta,
                    cell_idx,
                    enclosing_cell_ctx.clone(),
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
                for (cp_idx, (composed, para)) in composed_paras
                    .iter()
                    .zip(cell.paragraphs.iter())
                    .enumerate()
                {
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

                    let has_table_ctrl =
                        para.controls.iter().any(|c| matches!(c, Control::Table(_)));
                    // [Task #573] inline TAC 표(treat_as_char=true) 와 block 표(treat_as_char=false)
                    // 를 분리. 인라인 TAC 표가 있는 셀 paragraph 의 surrounding text (예: "ㄷ. ",
                    // "이다.") 가 layout_composed_paragraph 호출 미진입으로 미렌더되던 결함 정정.
                    // block 표는 별도 layout_table 호출로 배치되므로 텍스트 흐름 외부 — 기존
                    // ELSE 분기 로직 유지. inline TAC 표는 layout_composed_paragraph 의 run_tacs
                    // 에서 텍스트와 함께 배치되어야 함.
                    let has_block_table_ctrl = para
                        .controls
                        .iter()
                        .any(|c| matches!(c, Control::Table(t) if !t.common.treat_as_char));

                    // HWP/HWPX가 셀 내부 문단의 LINE_SEG.vpos를 제공하는 경우에는
                    // 누적 y 대신 그 절대 위치를 우선한다. 조직도형 표처럼 셀 하나에
                    // 여러 짧은 문단이 있고 paraPr spacing/lineSpacing이 함께 지정된
                    // 문서는 한컴이 각 문단 top을 vpos로 고정해 둔다. 누적 y만 쓰면
                    // spacing_before가 중복되거나 음수 line_spacing이 누적되어 줄 위치가
                    // 점점 어긋난다.
                    if use_top_vpos_anchor && !has_nested_table {
                        if let Some(first_seg) = para.line_segs.first() {
                            if first_seg.vertical_pos >= 0 {
                                let spacing_before = styles
                                    .para_styles
                                    .get(para.para_shape_id as usize)
                                    .map(|s| s.spacing_before)
                                    .unwrap_or(0.0);
                                let anchored_y = cell_y
                                    + pad_top
                                    + hwpunit_to_px(first_seg.vertical_pos, self.dpi);
                                // layout_composed_paragraph()가 spacing_before를 더하므로
                                // 호출 전에 그 값을 빼서 최종 line top이 vpos와 일치하게 한다.
                                para_y = anchored_y - spacing_before;
                            }
                        }
                    }

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
                            if !is_tac {
                                continue;
                            }
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
                    let total_inline_width: f64 =
                        tac_line_widths.iter().cloned().fold(0.0f64, f64::max);

                    if !has_block_table_ctrl {
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
                        let numbered_comp = if end_line > 0 {
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
                            0,
                            end_line,
                            section_index,
                            cp_idx,
                            cell_context.clone(),
                            !use_top_vpos_anchor,
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
                        // (leading spacing은 LINE_SEG vpos에 이미 반영되어 있음)
                    }

                    let para_alignment = styles
                        .para_styles
                        .get(para.para_shape_id as usize)
                        .map(|s| s.alignment)
                        .unwrap_or(Alignment::Left);
                    // [Task #548] paragraph margin_left + first-line indent 를 inline shape
                    // 위치에 반영. paragraph_layout 텍스트 경로와 동일한 effective_margin_left
                    // 산식을 적용해 텍스트와 shape 위치 일관성 보장.
                    let para_margin_left_px = styles
                        .para_styles
                        .get(para.para_shape_id as usize)
                        .map(|s| s.margin_left)
                        .unwrap_or(0.0);
                    let para_indent_px = styles
                        .para_styles
                        .get(para.para_shape_id as usize)
                        .map(|s| s.indent)
                        .unwrap_or(0.0);

                    let mut prev_tac_text_pos: usize = 0;
                    // LINE_SEG 기반 줄별 TAC 이미지 배치를 위한 상태
                    // 빈 문단(runs 없음)에서 TAC 컨트롤을 LINE_SEG에 순서대로 매핑
                    let all_runs_empty = composed.lines.iter().all(|l| l.runs.is_empty());
                    let mut tac_seq_index: usize = 0; // TAC 컨트롤 순번 (빈 문단용)
                    let mut current_tac_line: usize = 0;
                    let mut inline_x = {
                        let line_w = tac_line_widths
                            .first()
                            .copied()
                            .unwrap_or(total_inline_width);
                        let line_margin =
                            effective_margin_left_line(para_margin_left_px, para_indent_px, 0);
                        match para_alignment {
                            Alignment::Center | Alignment::Distribute => {
                                inner_area.x + (inner_area.width - line_w).max(0.0) / 2.0
                            }
                            Alignment::Right => inner_area.x + (inner_area.width - line_w).max(0.0),
                            _ => inner_area.x + line_margin,
                        }
                    };
                    let mut tac_img_y = para_y_before_compose;

                    for (ctrl_idx, ctrl) in para.controls.iter().enumerate() {
                        match ctrl {
                            Control::Picture(pic) => {
                                if pic.common.treat_as_char {
                                    let pic_w = hwpunit_to_px(pic.common.width as i32, self.dpi);
                                    // [Task #928] paragraph_layout 이 inline picture 를 emit 한
                                    // 경우 set_inline_shape_position 을 호출하므로 (paragraph_layout.rs
                                    // 라인 2019-2022), 본 가드는 inline_shape_position 등록 여부로
                                    // 판정한다. 기존 tac_controls + line_chars 기반 가드는 boundary
                                    // 케이스 (abs_pos == line_chars) 를 빠뜨려 exam_kor 5p ㉢
                                    // 그림 중복 emit 회귀가 있었다.
                                    let will_render_inline = tree
                                        .get_inline_shape_position(
                                            section_index,
                                            cp_idx,
                                            ctrl_idx,
                                            cell_context.as_ref(),
                                        )
                                        .is_some();
                                    if !will_render_inline {
                                        // LINE_SEG 기반 줄 판별
                                        let target_line = if all_runs_empty
                                            && para.line_segs.len() > 1
                                        {
                                            // 빈 문단: TAC 순번으로 LINE_SEG에 1:1 매핑
                                            let li = tac_seq_index.min(para.line_segs.len() - 1);
                                            tac_seq_index += 1;
                                            li
                                        } else {
                                            // 텍스트 있는 문단: char position으로 줄 판별
                                            composed
                                                .tac_controls
                                                .iter()
                                                .find(|&&(_, _, ci)| ci == ctrl_idx)
                                                .map(|&(abs_pos, _, _)| {
                                                    composed
                                                        .lines
                                                        .iter()
                                                        .enumerate()
                                                        .rev()
                                                        .find(|(_, line)| {
                                                            abs_pos >= line.char_start
                                                        })
                                                        .map(|(li, _)| li)
                                                        .unwrap_or(0)
                                                })
                                                .unwrap_or(0)
                                        };

                                        if target_line > current_tac_line {
                                            // 줄이 바뀜: inline_x 리셋, y를 LINE_SEG vpos 기준으로 이동
                                            current_tac_line = target_line;
                                            let line_w = tac_line_widths
                                                .get(target_line)
                                                .copied()
                                                .unwrap_or(0.0);
                                            // [Task #548] target_line 의 effective_margin_left 적용
                                            let line_margin = effective_margin_left_line(
                                                para_margin_left_px,
                                                para_indent_px,
                                                target_line,
                                            );
                                            inline_x = match para_alignment {
                                                Alignment::Center | Alignment::Distribute => {
                                                    inner_area.x
                                                        + (inner_area.width - line_w).max(0.0) / 2.0
                                                }
                                                Alignment::Right => {
                                                    inner_area.x
                                                        + (inner_area.width - line_w).max(0.0)
                                                }
                                                _ => inner_area.x + line_margin,
                                            };
                                            if let Some(seg) = para.line_segs.get(target_line) {
                                                // [Task #520 / #624 복원] LineSeg.vertical_pos 는 셀 origin 기준 절대값.
                                                // para_y_before_compose 에 이미 ls[0].vpos 가 누적되어 있어
                                                // 상대 오프셋(seg.vpos - ls[0].vpos)만 더해야 이중 합산을 피한다.
                                                let first_vpos = para
                                                    .line_segs
                                                    .first()
                                                    .map(|f| f.vertical_pos)
                                                    .unwrap_or(0);
                                                tac_img_y = para_y_before_compose
                                                    + hwpunit_to_px(
                                                        seg.vertical_pos - first_vpos,
                                                        self.dpi,
                                                    );
                                            }
                                        }

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
                                            y: tac_img_y,
                                            width: clamped_w,
                                            height: clamped_h,
                                        };
                                        // [Task #1151 v4] 셀 안 inline picture (tac=true):
                                        // outer paragraph idx + inner picture ctrl idx +
                                        // cell_ctx 전달 → ImageNode cell_index + cursor_rect
                                        // hit-test 정합.
                                        self.layout_picture(
                                            tree,
                                            &mut cell_node,
                                            pic,
                                            &pic_area,
                                            bin_data_content,
                                            Alignment::Left,
                                            Some(section_index),
                                            cell_context.as_ref().map(|c| c.parent_para_index),
                                            Some(ctrl_idx),
                                            cell_context.as_ref(),
                                        );
                                        inline_x += clamped_w;
                                        continue;
                                    }
                                    inline_x += pic_w;
                                } else {
                                    // 비-인라인(자리차지/글뒤로/글앞으로) 이미지:
                                    // 본문배치 속성(가로/세로 기준, 정렬, 오프셋) 적용
                                    let pic_w = hwpunit_to_px(pic.common.width as i32, self.dpi);
                                    let pic_h = hwpunit_to_px(pic.common.height as i32, self.dpi);
                                    // [Task #577] TopAndBottom + vert_rel_to=Para 인 셀 내부 이미지는
                                    // anchor 라인이 이미지에 의해 displaced 되므로, layout_composed_paragraph
                                    // 가 advance 시킨 para_y 가 아닌 anchor 시점(para_y_before_compose)을 기준
                                    // 으로 해야 cell-clip 영역 내부에 정확히 배치된다. (exam_science 2번 보기 ⑤
                                    // 등 5개 이미지에서 line_height(약 15.32px) 만큼 아래로 밀려 잘림.)
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
                                    let unrestricted_take_place_cell_float = !pic
                                        .common
                                        .flow_with_text
                                        && matches!(pic.common.text_wrap, TextWrap::TopAndBottom)
                                        && matches!(pic.common.vert_rel_to, VertRelTo::Para);
                                    let detached_from_inline_table_flow = inline_table_flow_y_shift
                                        > 0.0
                                        && unrestricted_take_place_cell_float;
                                    let picture_anchor_y = if detached_from_inline_table_flow {
                                        anchor_y - inline_table_flow_y_shift - row_y[r].max(0.0)
                                    } else if unrestricted_take_place_cell_float {
                                        // 한컴의 셀 내부 자리차지 그림은 제한이 꺼지면
                                        // offset 지점에 그림 하단이 걸리도록 위로 빠진다.
                                        // compute_object_position 이 아래에서 vOffset 을
                                        // 다시 더하므로 여기서는 미리 vOffset+높이를 뺀다.
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
                                    // [Task #1151 v4] 셀 안 non-inline picture (tac=false 자리차지 등):
                                    // outer paragraph idx + inner picture ctrl idx +
                                    // cell_ctx 전달.
                                    if detached_from_inline_table_flow
                                        || unrestricted_take_place_cell_float
                                    {
                                        self.layout_picture(
                                            tree,
                                            table_node,
                                            &pic_for_layout,
                                            &pic_area,
                                            bin_data_content,
                                            Alignment::Left,
                                            Some(section_index),
                                            cell_context.as_ref().map(|c| c.parent_para_index),
                                            Some(ctrl_idx),
                                            cell_context.as_ref(),
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
                                            cell_context.as_ref().map(|c| c.parent_para_index),
                                            Some(ctrl_idx),
                                            cell_context.as_ref(),
                                        );
                                    }
                                    para_y += self.non_inline_control_flow_height(&pic.common);
                                }
                                has_preceding_text = true;
                            }
                            Control::Shape(shape) => {
                                if shape.common().treat_as_char {
                                    let shape_w =
                                        hwpunit_to_px(shape.common().width as i32, self.dpi);
                                    // [Task #928] paragraph_layout 의 run_tacs 처리 (라인 2026-2034)
                                    // 가 inline Shape 위치를 set_inline_shape_position 으로 등록
                                    // 하므로, 본 가드는 등록 여부로 판정한다. Picture 분기와 동일
                                    // 패턴이며 boundary 케이스에 안전.
                                    let will_render_inline = tree
                                        .get_inline_shape_position(
                                            section_index,
                                            cp_idx,
                                            ctrl_idx,
                                            cell_context.as_ref(),
                                        )
                                        .is_some();
                                    // [Task #500] Picture 분기와 정합: target_line 산출 + 줄 변경 시
                                    // inline_x/tac_img_y 리셋. multi-line paragraph 에서 사각형이
                                    // ls[1]+ 에 있을 때 paragraph 첫 줄 좌표가 잘못 사용되던 결함 정정.
                                    let target_line = if all_runs_empty && para.line_segs.len() > 1
                                    {
                                        let li = tac_seq_index.min(para.line_segs.len() - 1);
                                        tac_seq_index += 1;
                                        li
                                    } else {
                                        composed
                                            .tac_controls
                                            .iter()
                                            .find(|&&(_, _, ci)| ci == ctrl_idx)
                                            .map(|&(abs_pos, _, _)| {
                                                composed
                                                    .lines
                                                    .iter()
                                                    .enumerate()
                                                    .rev()
                                                    .find(|(_, line)| abs_pos >= line.char_start)
                                                    .map(|(li, _)| li)
                                                    .unwrap_or(0)
                                            })
                                            .unwrap_or(0)
                                    };
                                    if target_line > current_tac_line {
                                        current_tac_line = target_line;
                                        let line_w = tac_line_widths
                                            .get(target_line)
                                            .copied()
                                            .unwrap_or(0.0);
                                        // [Task #548] target_line 의 effective_margin_left 적용
                                        let line_margin = effective_margin_left_line(
                                            para_margin_left_px,
                                            para_indent_px,
                                            target_line,
                                        );
                                        inline_x = match para_alignment {
                                            Alignment::Center | Alignment::Distribute => {
                                                inner_area.x
                                                    + (inner_area.width - line_w).max(0.0) / 2.0
                                            }
                                            Alignment::Right => {
                                                inner_area.x + (inner_area.width - line_w).max(0.0)
                                            }
                                            _ => inner_area.x + line_margin,
                                        };
                                        if let Some(seg) = para.line_segs.get(target_line) {
                                            // [Task #520] LineSeg.vertical_pos 는 셀 origin 기준 절대값.
                                            // para_y_before_compose 에 이미 ls[0].vpos 가 누적되어 있어
                                            // 상대 오프셋만 더해야 한다 (Picture 분기와 동일).
                                            let first_vpos = para
                                                .line_segs
                                                .first()
                                                .map(|f| f.vertical_pos)
                                                .unwrap_or(0);
                                            tac_img_y = para_y_before_compose
                                                + hwpunit_to_px(
                                                    seg.vertical_pos - first_vpos,
                                                    self.dpi,
                                                );
                                        }
                                    }
                                    if !will_render_inline {
                                        // Shape 앞의 텍스트 너비 계산: tac_controls에서 이 Shape의 text_pos와
                                        // 이전 Shape의 text_pos 차이에 해당하는 텍스트 너비를 inline_x에 반영
                                        if let Some(&(tac_pos, _, _)) = composed
                                            .tac_controls
                                            .iter()
                                            .find(|&&(_, _, ci)| ci == ctrl_idx)
                                        {
                                            // [Task #495] 가드: 사각형이 paragraph 첫 줄(ls[0]) 범위 안에 있을 때만
                                            // text_before 추출/발행. multi-line paragraph 에서 사각형이 ls[1]+ 에
                                            // 있는 경우 composed.lines.first() 만 보던 기존 코드는 첫 줄 전체
                                            // 텍스트를 잘못 추출해 paragraph_layout 결과와 중복 발행했음.
                                            let in_first_line = composed
                                                .lines
                                                .first()
                                                .map(|line| {
                                                    let line_chars: usize = line
                                                        .runs
                                                        .iter()
                                                        .map(|r| r.text.chars().count())
                                                        .sum();
                                                    tac_pos >= line.char_start
                                                        && tac_pos < line.char_start + line_chars
                                                })
                                                .unwrap_or(false);
                                            // 이 Shape 앞에 아직 inline_x에 반영되지 않은 텍스트가 있는지 계산
                                            let text_before: String = if in_first_line {
                                                composed
                                                    .lines
                                                    .first()
                                                    .map(|line| {
                                                        let mut chars_so_far = 0usize;
                                                        let mut result = String::new();
                                                        for run in &line.runs {
                                                            for ch in run.text.chars() {
                                                                if chars_so_far >= prev_tac_text_pos
                                                                    && chars_so_far < tac_pos
                                                                {
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
                                                let char_style_id = composed
                                                    .lines
                                                    .first()
                                                    .and_then(|l| l.runs.first())
                                                    .map(|r| r.char_style_id)
                                                    .unwrap_or(0);
                                                let lang_index = composed
                                                    .lines
                                                    .first()
                                                    .and_then(|l| l.runs.first())
                                                    .map(|r| r.lang_index)
                                                    .unwrap_or(0);
                                                let ts = resolved_to_text_style(
                                                    styles,
                                                    char_style_id,
                                                    lang_index,
                                                );
                                                // [Task #555] PUA 옛한글 char 은 자모 시퀀스로 변환 후 폭 측정.
                                                let text_before_metrics: String = {
                                                    use super::super::pua_oldhangul::map_pua_old_hangul;
                                                    text_before
                                                        .chars()
                                                        .flat_map(|ch| {
                                                            if let Some(jamos) =
                                                                map_pua_old_hangul(ch)
                                                            {
                                                                jamos
                                                                    .iter()
                                                                    .copied()
                                                                    .collect::<Vec<_>>()
                                                            } else {
                                                                vec![ch]
                                                            }
                                                        })
                                                        .collect()
                                                };
                                                let text_w =
                                                    estimate_text_width(&text_before_metrics, &ts);
                                                let text_font_size = ts.font_size;
                                                // 텍스트 렌더링: Shape 사이에 배치
                                                // 텍스트 y를 Shape 하단 baseline에 맞춤
                                                // (Shape 높이 - 폰트 줄 높이)만큼 아래로 이동
                                                let text_baseline = text_font_size * 0.85;
                                                let font_line_h = text_font_size * 1.2;
                                                // 인접 Shape의 높이를 사용하여 텍스트 y를 baseline 정렬
                                                let adjacent_shape_h = para
                                                    .controls
                                                    .iter()
                                                    .find_map(|c| {
                                                        if let Control::Shape(s) = c {
                                                            if s.common().treat_as_char {
                                                                Some(hwpunit_to_px(
                                                                    s.common().height as i32,
                                                                    self.dpi,
                                                                ))
                                                            } else {
                                                                None
                                                            }
                                                        } else {
                                                            None
                                                        }
                                                    })
                                                    .unwrap_or(0.0);
                                                let text_y = para_y_before_compose
                                                    + (adjacent_shape_h - font_line_h).max(0.0);
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
                                                    BoundingBox::new(
                                                        inline_x,
                                                        text_y,
                                                        text_w,
                                                        font_line_h,
                                                    ),
                                                );
                                                cell_node.children.push(text_node);
                                                inline_x += text_w;
                                            }
                                            prev_tac_text_pos = tac_pos;
                                        }
                                    }
                                    // [Task #520 / #624 복원] target_line 기반 tac_img_y 사용 (Picture 분기와 동일).
                                    // para_y_before_compose 사용 시 multi-line paragraph 의 ls[1]+ inline TAC Shape 가
                                    // 항상 line 0 좌표에 떨어져 본문 텍스트와 겹친다 (exam_science p2 7번 글상자 ㉠).
                                    // [Task #928] will_render_inline=true 인 경우 paragraph_layout 이
                                    // 등록한 inline_shape_position 좌표를 사용해 도형 위치를
                                    // run_tacs split 에서 reserve 한 gap 과 정확히 정합시킨다.
                                    let (shape_x, shape_y) = if will_render_inline {
                                        tree.get_inline_shape_position(
                                            section_index,
                                            cp_idx,
                                            ctrl_idx,
                                            cell_context.as_ref(),
                                        )
                                        .unwrap_or((inline_x, tac_img_y))
                                    } else {
                                        (inline_x, tac_img_y)
                                    };
                                    let shape_area = LayoutRect {
                                        x: shape_x,
                                        y: shape_y,
                                        width: shape_w,
                                        height: inner_area.height,
                                    };
                                    // [Task #1138] 셀 컨텍스트 (section, outer_para, outer_table_ctrl, cell, cell_para, inner_ctrl)
                                    let table_cell_ctx = table_meta.map(|(opi, otci)| {
                                        (section_index, opi, otci, cell_idx, cp_idx, ctrl_idx)
                                    });
                                    self.layout_cell_shape(
                                        tree,
                                        &mut cell_node,
                                        shape,
                                        &shape_area,
                                        shape_y,
                                        Alignment::Left,
                                        styles,
                                        bin_data_content,
                                        clamp_header_negative_para_offset,
                                        table_cell_ctx,
                                    );
                                    inline_x += shape_w;
                                } else {
                                    let shape_anchor_y = if matches!(
                                        shape.common().vert_rel_to,
                                        crate::model::shape::VertRelTo::Para
                                    ) {
                                        para_y_before_compose
                                    } else {
                                        para_y
                                    };
                                    // [Task #1138] 셀 컨텍스트
                                    let table_cell_ctx = table_meta.map(|(opi, otci)| {
                                        (section_index, opi, otci, cell_idx, cp_idx, ctrl_idx)
                                    });
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
                                // 수식 컨트롤: 글자처럼 인라인 배치
                                let eq_w = hwpunit_to_px(eq.common.width as i32, self.dpi);

                                // 수식이 텍스트 run 사이에 인라인으로 배치되는 경우
                                // layout_composed_paragraph에서 이미 렌더링됨 → 건너뛰기
                                let has_text_in_para =
                                    para.text.chars().any(|c| c > '\u{001F}' && c != '\u{FFFC}');
                                // 빈 runs 셀 + TAC 수식: paragraph_layout(Task #287 경로)이 이미
                                // 렌더 후 set_inline_shape_position 호출. 중복 emit 방지(Issue #301).
                                let already_rendered_inline = tree
                                    .get_inline_shape_position(
                                        section_index,
                                        cp_idx,
                                        ctrl_idx,
                                        cell_context.as_ref(),
                                    )
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

                                    let tokens =
                                        super::super::equation::tokenizer::tokenize(&eq.script);
                                    let ast = super::super::equation::parser::EqParser::new(tokens)
                                        .parse();
                                    let font_size_px = hwpunit_to_px(eq.font_size as i32, self.dpi);
                                    let layout_box =
                                        super::super::equation::layout::EqLayout::new(font_size_px)
                                            .layout(&ast);
                                    let color_str =
                                        super::super::equation::svg_render::eq_color_to_svg(
                                            eq.color,
                                        );
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
                                            para_index: table_meta.map(|(pi, _)| pi),
                                            control_index: Some(ctrl_idx),
                                            cell_index: Some(cell_idx),
                                            cell_para_index: Some(cp_idx),
                                            note_ref: None,
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
                                    // [Task #573] layout_composed_paragraph 의 run_tacs 가
                                    // 인라인 TAC 표를 이미 렌더하고 set_inline_shape_position
                                    // 등록했다면 중복 emit 방지 (Equation 의 L1800 가드와 동일 패턴).
                                    let already_rendered_inline = tree
                                        .get_inline_shape_position(
                                            section_index,
                                            cp_idx,
                                            ctrl_idx,
                                            cell_context.as_ref(),
                                        )
                                        .is_some();
                                    let tac_w =
                                        hwpunit_to_px(nested_table.common.width as i32, self.dpi);
                                    if already_rendered_inline {
                                        inline_x += tac_w;
                                    } else {
                                        // [Task #1195] 표 앞에 텍스트(공백 등)가 선행하면, 한컴은
                                        // 그 textRun 너비 다음에 표를 놓되 잔여 너비가 부족하면
                                        // 다음 줄(line feed)에 조판한다. 즉 표는 문단 첫 줄이 아니라
                                        // 표가 속한 line_seg(표 앞 빈 줄 다음)에 위치한다.
                                        // 이미지 TAC 분기(L2231)와 동일하게 para_y_before_compose 에
                                        // (표 line_seg.vpos − 첫 line_seg.vpos) 상대 오프셋을 더한다.
                                        // (para_y_before_compose 에 이미 ls[0].vpos 가 누적되어 있음.)
                                        let table_anchor_y =
                                            if has_preceding_text && para.line_segs.len() > 1 {
                                                let first_vpos = para
                                                    .line_segs
                                                    .first()
                                                    .map(|f| f.vertical_pos)
                                                    .unwrap_or(0);
                                                let tbl_vpos = para
                                                    .line_segs
                                                    .last()
                                                    .map(|s| s.vertical_pos)
                                                    .unwrap_or(first_vpos);
                                                para_y_before_compose
                                                    + hwpunit_to_px(tbl_vpos - first_vpos, self.dpi)
                                            } else {
                                                para_y_before_compose
                                            };
                                        let ctrl_area = LayoutRect {
                                            x: inline_x,
                                            y: table_anchor_y,
                                            width: tac_w,
                                            height: (inner_area.height
                                                - (table_anchor_y - inner_area.y))
                                                .max(0.0),
                                        };
                                        let table_h = self.layout_table(
                                            tree,
                                            &mut cell_node,
                                            nested_table,
                                            section_index,
                                            styles,
                                            outline_numbering_id,
                                            &ctrl_area,
                                            table_anchor_y,
                                            bin_data_content,
                                            None,
                                            depth + 1,
                                            None,
                                            para_alignment,
                                            nested_ctx,
                                            0.0,
                                            0.0,
                                            Some(inline_x),
                                            None,
                                            None,
                                            clamp_header_negative_para_offset,
                                        );
                                        inline_x += tac_w;
                                        // para_y는 TAC 표 높이만큼 갱신 (같은 문단 내 다음 표도 같은 y)
                                        let new_bottom = para_y_before_compose + table_h;
                                        if new_bottom > para_y {
                                            para_y = new_bottom;
                                        }
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
                                                        styles,
                                                        run.char_style_id,
                                                        run.lang_index,
                                                    );
                                                    // [Task #555] PUA 옛한글 변환 후 자모 시퀀스 폭.
                                                    text_w += estimate_text_width(
                                                        effective_text_for_metrics(run),
                                                        &ts,
                                                    );
                                                }
                                            }
                                        }
                                        text_w
                                    } else {
                                        0.0
                                    };
                                    // TAC 표 앞 텍스트 렌더링 (문단부호 등 표시용)
                                    if tac_text_offset > 0.0 {
                                        let line_h = composed
                                            .lines
                                            .first()
                                            .map(|l| hwpunit_to_px(l.line_height, self.dpi))
                                            .unwrap_or(12.0);
                                        let baseline = line_h * 0.85;
                                        let line_id = tree.next_id();
                                        let mut line_node = RenderNode::new(
                                            line_id,
                                            RenderNodeType::TextLine(TextLineNode::new(
                                                line_h, baseline,
                                            )),
                                            BoundingBox::new(
                                                inner_area.x,
                                                nested_y,
                                                tac_text_offset,
                                                line_h,
                                            ),
                                        );
                                        let mut run_x = inner_area.x;
                                        for line in &composed.lines {
                                            for run in &line.runs {
                                                if run.text.is_empty() {
                                                    continue;
                                                }
                                                let ts = resolved_to_text_style(
                                                    styles,
                                                    run.char_style_id,
                                                    run.lang_index,
                                                );
                                                // [Task #555] PUA 옛한글 변환 후 자모 시퀀스 폭.
                                                let run_w = estimate_text_width(
                                                    effective_text_for_metrics(run),
                                                    &ts,
                                                );
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
                                                    BoundingBox::new(
                                                        run_x, nested_y, run_w, line_h,
                                                    ),
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
                                        height: (inner_area.height - (nested_y - inner_area.y))
                                            .max(0.0),
                                    };
                                    let table_h = self.layout_table(
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
                                        depth + 1,
                                        None,
                                        para_alignment,
                                        nested_ctx,
                                        0.0,
                                        0.0,
                                        None,
                                        None,
                                        None,
                                        clamp_header_negative_para_offset,
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
                        let total_text_chars = composed
                            .lines
                            .first()
                            .map(|line| {
                                line.runs
                                    .iter()
                                    .map(|r| r.text.chars().count())
                                    .sum::<usize>()
                            })
                            .unwrap_or(0);
                        if prev_tac_text_pos < total_text_chars {
                            let remaining_text: String = composed
                                .lines
                                .first()
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
                                let char_style_id = composed
                                    .lines
                                    .first()
                                    .and_then(|l| l.runs.last())
                                    .map(|r| r.char_style_id)
                                    .unwrap_or(0);
                                let lang_index = composed
                                    .lines
                                    .first()
                                    .and_then(|l| l.runs.last())
                                    .map(|r| r.lang_index)
                                    .unwrap_or(0);
                                let ts = resolved_to_text_style(styles, char_style_id, lang_index);
                                // [Task #555] PUA 옛한글 char 은 자모 시퀀스로 변환 후 폭 측정.
                                let remaining_metrics: String = {
                                    use super::super::pua_oldhangul::map_pua_old_hangul;
                                    remaining_trimmed
                                        .chars()
                                        .flat_map(|ch| {
                                            if let Some(jamos) = map_pua_old_hangul(ch) {
                                                jamos.iter().copied().collect::<Vec<_>>()
                                            } else {
                                                vec![ch]
                                            }
                                        })
                                        .collect()
                                };
                                let text_w = estimate_text_width(&remaining_metrics, &ts);
                                let text_baseline = ts.font_size * 0.85;
                                let text_h = ts.font_size * 1.2;
                                // 마지막 Shape 높이 기준으로 텍스트 y 계산
                                let last_shape_h = para
                                    .controls
                                    .iter()
                                    .rev()
                                    .find_map(|c| {
                                        if let Control::Shape(s) = c {
                                            if s.common().treat_as_char {
                                                Some(hwpunit_to_px(
                                                    s.common().height as i32,
                                                    self.dpi,
                                                ))
                                            } else {
                                                None
                                            }
                                        } else {
                                            None
                                        }
                                    })
                                    .unwrap_or(0.0);
                                let text_y =
                                    para_y_before_compose + (last_shape_h - text_h).max(0.0);
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
                                    let next_vpos_y = text_y_start
                                        + hwpunit_to_px(next_seg.vertical_pos, self.dpi);
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

            // (b) 셀 테두리를 수집한다. 열별 높이가 다른 표는 row_y 격자로
            // 테두리를 그릴 수 없으므로 셀 bbox 기준 라인을 별도로 생성한다.
            if let Some(bs) = border_style {
                if independent_col_row_y.is_some() {
                    independent_border_nodes.extend(render_cell_box_borders(
                        tree, bs, cell_x, cell_y, cell_w, cell_h,
                    ));
                } else {
                    collect_cell_borders(
                        h_edges,
                        v_edges,
                        c,
                        r,
                        cell.col_span as usize,
                        cell.row_span as usize,
                        &bs.borders,
                    );
                }
            }

            table_node.children.push(cell_node);

            // (c) 셀 대각선 렌더링 (셀 콘텐츠 위에 그림)
            if let Some(bs) = border_style {
                table_node.children.extend(render_cell_diagonal(
                    tree, bs, cell_x, cell_y, cell_w, cell_h,
                ));
            }
        }
        if !independent_border_nodes.is_empty() {
            table_node.children.extend(independent_border_nodes);
        }
    }

    pub(crate) fn calc_cell_controls_height(
        &self,
        cell: &crate::model::table::Cell,
        styles: &ResolvedStyleSet,
    ) -> f64 {
        let measurer = super::super::height_measurer::HeightMeasurer::new(self.dpi)
            .with_hwp3_variant(self.is_hwp3_variant.get());
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
        row_heights.iter().sum::<f64>()
            + cell_spacing * (row_count.saturating_sub(1) as f64)
            + om_top
            + om_bottom
    }

    /// 셀 내 중첩 표가 실제로 차지하는 하단 위치를 계산한다.
    ///
    /// 일부 HWP/HWPX는 중첩 표 문단의 LINE_SEG.line_height에 내부 표의 실제
    /// 높이를 반영하지 않는다. 렌더링/측정은 해당 문단의 vertical_pos에 중첩 표
    /// 측정 높이를 더한 값을 셀 콘텐츠 끝점 후보로 사용한다.
    pub(crate) fn calc_nested_controls_bottom_height(
        &self,
        paragraphs: &[Paragraph],
        styles: &ResolvedStyleSet,
    ) -> f64 {
        paragraphs
            .iter()
            .map(|p| {
                let nested_h: f64 = p
                    .controls
                    .iter()
                    .map(|ctrl| {
                        if let Control::Table(t) = ctrl {
                            self.calc_nested_table_height(t, styles)
                        } else {
                            0.0
                        }
                    })
                    .sum();
                if nested_h <= 0.0 {
                    0.0
                } else {
                    let para_top = p
                        .line_segs
                        .first()
                        .map(|s| hwpunit_to_px(s.vertical_pos, self.dpi))
                        .unwrap_or(0.0);
                    para_top + nested_h
                }
            })
            .fold(0.0f64, f64::max)
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
                let nested_h: f64 = p
                    .controls
                    .iter()
                    .map(|ctrl| {
                        if let Control::Table(t) = ctrl {
                            self.calc_nested_table_height(t, styles)
                        } else {
                            0.0
                        }
                    })
                    .sum();
                let h = if nested_h > 0.0 {
                    nested_h
                } else {
                    hwpunit_to_px(400, self.dpi)
                };
                total += spacing_before + h + spacing_after;
            } else {
                // 중첩 표가 있는 문단: LINE_SEG 높이와 실제 중첩 표 높이 중 큰 값 사용
                let has_table_in_para = p.controls.iter().any(|c| matches!(c, Control::Table(_)));
                let line_count = comp.lines.len();
                let line_based_h: f64 = comp
                    .lines
                    .iter()
                    .enumerate()
                    .map(|(li, line)| {
                        let h = hwpunit_to_px(line.line_height, self.dpi);
                        let is_cell_last_line = is_last_para && li + 1 == line_count;
                        let ls = if !is_cell_last_line {
                            hwpunit_to_px(line.line_spacing, self.dpi)
                        } else {
                            0.0
                        };
                        spacing_before * (if li == 0 { 1.0 } else { 0.0 })
                            + h
                            + ls
                            + spacing_after * (if li + 1 == line_count { 1.0 } else { 0.0 })
                    })
                    .sum();
                if has_table_in_para {
                    let nested_h: f64 = p
                        .controls
                        .iter()
                        .map(|ctrl| {
                            if let Control::Table(t) = ctrl {
                                self.calc_nested_table_height(t, styles)
                            } else {
                                0.0
                            }
                        })
                        .sum();
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
        let has_offset = content_offset > 0.0;
        let has_limit = content_limit > 0.0;

        // [Task #991] 분할 시작/중간 페이지(has_offset)의 줄 컷을 독립 재계산하지
        // 않고, 끝 페이지 패스(prefix 패스)에서 유도한다.
        //
        // 끝 페이지(`!has_offset`)와 시작 페이지가 분할 경계를 각자 계산하면,
        // `limit_reached` 전파(Task #485)·vpos 리셋 컷(Task #697)·vpos 동기화
        // (Task #700)가 두 경로에서 다르게 작동해 줄이 중복되거나 누락된다.
        // 모든 컷을 동일한 prefix 패스(`cell_line_prefix_counts`)로 통일하면,
        // - 시작 줄 = budget `content_offset` 안에 들어가는 prefix 줄 수
        // - 끝 줄   = budget `content_offset + content_limit` 안의 prefix 줄 수
        //   (limit 없으면 문단 전체)
        // 가 되어, 끝 페이지 포함분과 정확히 상보가 된다(중복·누락 불가).
        if has_offset {
            let skip = self.cell_line_prefix_counts(cell, composed_paras, content_offset, styles);
            let keep: Vec<usize> = if has_limit {
                self.cell_line_prefix_counts(
                    cell,
                    composed_paras,
                    content_offset + content_limit,
                    styles,
                )
            } else {
                composed_paras.iter().map(|c| c.lines.len()).collect()
            };
            return skip
                .iter()
                .zip(keep.iter())
                .map(|(&s, &e)| (s, e.max(s)))
                .collect();
        }

        let mut result = Vec::with_capacity(composed_paras.len());
        let mut cum: f64 = 0.0;
        // [Task #431] content_limit 은 현재 페이지에서 표시할 상대 길이(px) 의미이므로
        // 절대 좌표(cum 기반)와 비교하려면 content_offset 을 더해 절대 끝 좌표로 변환한다.
        // (Task #362 의 도입 시점에 단위 mismatch 가 있었음 — content_offset >= content_limit
        // 케이스에서 셀 내 문단이 즉시 break 되어 빈 페이지로 출력되던 결함 정정.)
        // [Task #656] abs_limit 그대로 사용 (epsilon 제거).
        // - Task #485 의 SPLIT_LIMIT_EPSILON = 2.0px 휴리스틱 마진은 typeset/layout 의
        //   trail_ls 비교 모델 어긋남을 흡수하던 임시방편이었음.
        // - 본질 정정: break 비교 시 마지막 visible 줄의 trail_ls 제외 (line_break_pos = cum + h).
        //   typeset 의 split_end_limit = avail_content 추정과 layout 의 셀 마지막 줄 trail_ls
        //   미렌더 모델 (is_cell_last_line) 과 일관 → epsilon 마진 없이 폰트 무관하게 정합.
        let abs_limit = if has_limit {
            content_offset + content_limit
        } else {
            0.0
        };

        // [Task #485 Bug-1] abs_limit 도달 후 렌더 차단 플래그.
        // 이전엔 inner break 만 빠져나와 다음 단락에서 같은 cum 으로 재평가 → 셀 마지막 단락(line_spacing 제외로 line_h 작아짐)이
        // abs_limit 안에 fit 하여 통과하는 out-of-order 결함 발생. 한 번 도달하면 이후 단락 모두 미렌더로 처리.
        let mut limit_reached = false;

        let total_paras = composed_paras.len();
        // [Task #700] 셀별 가드용 — 셀 첫 paragraph 의 LINE_SEG[0].vpos 가 0 이어야 한컴 정상 인코딩.
        let cell_first_vpos = cell
            .paragraphs
            .first()
            .and_then(|p| p.line_segs.first().map(|s| s.vertical_pos))
            .unwrap_or(-1);

        for (pi, (comp, para)) in composed_paras
            .iter()
            .zip(cell.paragraphs.iter())
            .enumerate()
        {
            // [Task #700] paragraph 진입 시 cum 을 LINE_SEG.vpos 절대값으로 동기화.
            // 한컴은 셀 콘텐츠 위치를 LINE_SEG.vpos 단위로 인코딩 (paragraph 사이 spacing 도 vpos
            // 차분에 흡수). rhwp 의 line_height + line_spacing + spacing_before/after 누적은
            // 한컴 vpos 단위와 ~수십 px 어긋나, split_end content_limit (한컴 vpos 단위) 와 비교 시
            // cut 위치가 어긋나는 회귀 (예: inner-table-01 cell[11] p[17] 까지 cut 해야 하는데
            // p[19] 까지 visible 처리). cum 을 vpos 절대값으로 동기화하여 한컴 정합화.
            //
            // [Task #697] 또한 한컴은 셀 내부 페이지 분할 위치에서 LINE_SEG.vpos 를 0 으로 리셋한
            // 인코딩을 사용 (예: cell[11] p[20] vpos=0). vpos 리셋 검출 시 cum 을 abs_limit 까지
            // 강제 진행시켜 후속 paragraph 들이 limit 초과로 cut.
            //
            // 가드:
            // - cell_first_vpos == 0 — 한컴 정상 인코딩 케이스만 (다른 케이스 회피, 회귀 방지)
            // - target_cum > cum — cum 만 전진 허용 (감소 금지, line metric 가 vpos 보다 큰 paragraph
            //   영향 차단)
            // - 차분 누적 (delta) 대신 절대 동기화 — paragraph 사이 spacing mismatch 누적으로 인한
            //   회귀 (form-002 등) 회피.
            if pi > 0 && cell_first_vpos == 0 {
                let prev_para = &cell.paragraphs[pi - 1];
                let prev_end_vpos = prev_para
                    .line_segs
                    .last()
                    .map(|s| s.vertical_pos + s.line_height)
                    .unwrap_or(-1);
                let cur_first_vpos = para.line_segs.first().map(|s| s.vertical_pos).unwrap_or(-1);
                if cur_first_vpos >= 0 && prev_end_vpos > 0 {
                    if cur_first_vpos < prev_end_vpos {
                        // vpos 리셋 — page-break 신호
                        if has_limit && cum < abs_limit {
                            cum = abs_limit;
                        }
                    } else {
                        // 정상 누적 — cum 을 vpos 절대값으로 동기화 (전진만)
                        let target_cum = hwpunit_to_px(cur_first_vpos, self.dpi);
                        if target_cum > cum {
                            cum = target_cum;
                        }
                    }
                }
            }

            let para_style = styles.para_styles.get(para.para_shape_id as usize);
            let is_last_para = pi + 1 == total_paras;
            // MeasuredCell 규칙: 첫 문단은 spacing_before 없음, 마지막 문단은 spacing_after 없음
            let spacing_before = if pi > 0 {
                para_style.map(|s| s.spacing_before).unwrap_or(0.0)
            } else {
                0.0
            };
            let spacing_after = if !is_last_para {
                para_style.map(|s| s.spacing_after).unwrap_or(0.0)
            } else {
                0.0
            };
            let line_count = comp.lines.len();

            // [Task #485 Bug-1] 한도 초과 후 후속 단락은 강제 미렌더 (시각 순서 보존).
            if limit_reached {
                let visible_count = if line_count == 0 { 0 } else { line_count };
                result.push((visible_count, visible_count));
                continue;
            }

            // 중첩 표 포함 문단(atomic) — line_count==0 또는 has_table_in_para
            let has_table_in_para = para.controls.iter().any(|c| matches!(c, Control::Table(_)));
            if line_count == 0 || has_table_in_para {
                let nested_h: f64 = para
                    .controls
                    .iter()
                    .map(|ctrl| {
                        if let Control::Table(t) = ctrl {
                            self.calc_nested_table_height(t, styles)
                        } else {
                            0.0
                        }
                    })
                    .sum();
                let para_h = if line_count == 0 {
                    let h = if nested_h > 0.0 {
                        nested_h
                    } else {
                        hwpunit_to_px(400, self.dpi)
                    };
                    spacing_before + h + spacing_after
                } else {
                    let line_based_h: f64 = comp
                        .lines
                        .iter()
                        .enumerate()
                        .map(|(li, line)| {
                            let h = hwpunit_to_px(line.line_height, self.dpi);
                            let ls = hwpunit_to_px(line.line_spacing, self.dpi);
                            let is_cell_last_line = is_last_para && li + 1 == line_count;
                            let mut lh = if !is_cell_last_line { h + ls } else { h };
                            if li == 0 {
                                lh += spacing_before;
                            }
                            if li == line_count - 1 {
                                lh += spacing_after;
                            }
                            lh
                        })
                        .sum();
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
                // [Task #656] epsilon 제거 — atomic 단락은 단일 단위로 visible/skip 결정
                let exceeds_limit = has_limit && para_end_pos > abs_limit && !bigger_than_page;
                let visible_count = if line_count == 0 { 0 } else { line_count };
                if was_on_prev || exceeds_limit {
                    // (n,n): 렌더 스킵 마커. line_count==0 이면 (0,0) 동일.
                    result.push((visible_count, visible_count));
                    // [Task #485 Bug-1] limit 초과 단락 발생 시 후속 단락 차단.
                    if exceeds_limit {
                        limit_reached = true;
                    }
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

                // [Task #656] break 비교 시 마지막 visible 줄의 trail_ls 제외.
                // - cum 누적은 line_h (h+ls) 그대로 (이전 줄들의 ls 는 다음 줄 직전 spacing 이므로 렌더)
                // - break 비교는 line_break_pos = cum + h (이 줄의 ls 제외) 로 비교
                //   → 이 줄이 visible 시 마지막 줄이면 trail_ls 미렌더 영역, abs_limit 안에 들어감
                // typeset 의 split_end_limit = avail_content 추정과 정합. 셀
                // is_cell_last_line 분기의 trail_ls 미렌더 모델과 동일 본질.
                // (Task #485 의 epsilon 휴리스틱 본질 정정 — 휴리스틱 마진 없이 일관된 모델, 폰트 무관.)
                let line_break_pos = cum + h;
                if has_limit && line_break_pos > abs_limit {
                    // [Task #485 Bug-1] outer 루프도 차단 — 후속 단락의 작은 line_h slip 방지.
                    limit_reached = true;
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

    /// [Task #991] 셀 콘텐츠를 누적하며 예산 `budget_px` 안에 들어가는 문단별 prefix
    /// 줄 수를 반환한다.
    ///
    /// 끝 페이지 패스(`compute_cell_line_ranges` 를 `offset=0, limit=budget` 로 호출)의
    /// 결과에서 추출한다. `offset=0` 이므로 재귀 호출은 `has_offset=false` 경로(끝 페이지
    /// 로직)를 타며 더 이상 재귀하지 않는다.
    ///
    /// 끝 페이지 결과 `(s, e)`:
    /// - `s == 0`: `e` 가 budget 안에 들어간 prefix 가시 줄 수.
    /// - `s != 0`: 한도 초과 스킵 마커 → prefix 0줄.
    fn cell_line_prefix_counts(
        &self,
        cell: &crate::model::table::Cell,
        composed_paras: &[ComposedParagraph],
        budget_px: f64,
        styles: &ResolvedStyleSet,
    ) -> Vec<usize> {
        let ranges = self.compute_cell_line_ranges(cell, composed_paras, 0.0, budget_px, styles);
        ranges
            .iter()
            .map(|&(s, e)| if s == 0 { e } else { 0 })
            .collect()
    }

    /// [Task #993] 한 셀의 콘텐츠를 "유닛" 시퀀스로 평탄화한다.
    ///
    /// 유닛 1개 = 합성 줄 1개 또는 중첩 표 atom 1개(중첩 표 문단 = 유닛 1개,
    /// 분할 불가). 유닛 높이는 `compute_cell_line_ranges`/`calc_visible_content_*`
    /// 의 줄 높이 계산과 동일 규칙(줄 h+ls, 셀 마지막 줄 ls 제외, 문단 첫·마지막
    /// 줄에 spacing_before/after). `hard_break_before` = 이 유닛 앞에 HWP vpos
    /// 리셋(셀 내부 페이지 분할, `[Task #697]`)이 있는가.
    pub(super) fn cell_units(
        &self,
        cell: &crate::model::table::Cell,
        table: &crate::model::table::Table,
        styles: &ResolvedStyleSet,
    ) -> Vec<CellUnit> {
        let (pad_left, pad_right, pad_top, pad_bottom) = self.resolve_cell_padding(cell, table);
        let cell_w = if cell.width < 0x8000_0000 {
            hwpunit_to_px(cell.width as i32, self.dpi)
        } else {
            0.0
        };
        let inner_width = (cell_w - pad_left - pad_right).max(0.0);
        // [Task #700] vpos 동기화 가드와 동일 — 한컴 정상 인코딩(첫 문단 vpos=0) 한정.
        let cell_first_vpos = cell
            .paragraphs
            .first()
            .and_then(|p| p.line_segs.first().map(|s| s.vertical_pos))
            .unwrap_or(-1);
        let para_count = cell.paragraphs.len();
        let mut units: Vec<CellUnit> = Vec::new();
        for (pi, p) in cell.paragraphs.iter().enumerate() {
            let mut comp = compose_paragraph(p);
            crate::renderer::composer::recompose_for_cell_width(&mut comp, p, inner_width, styles);
            let para_style = styles.para_styles.get(p.para_shape_id as usize);
            let is_last_para = pi + 1 == para_count;
            let spacing_before = if pi > 0 {
                para_style.map(|s| s.spacing_before).unwrap_or(0.0)
            } else {
                0.0
            };
            let spacing_after = if !is_last_para {
                para_style.map(|s| s.spacing_after).unwrap_or(0.0)
            } else {
                0.0
            };
            // vpos 리셋 검출: 직전 문단 끝보다 현재 문단 시작 vpos 가 작으면 리셋.
            let reset_before = if pi > 0 && cell_first_vpos == 0 {
                let prev = &cell.paragraphs[pi - 1];
                let prev_end = prev
                    .line_segs
                    .last()
                    .map(|s| s.vertical_pos + s.line_height)
                    .unwrap_or(-1);
                let cur_first = p.line_segs.first().map(|s| s.vertical_pos).unwrap_or(-1);
                cur_first >= 0 && prev_end > 0 && cur_first < prev_end
            } else {
                false
            };
            let line_reset_before = |li: usize| -> bool {
                if li == 0 {
                    return reset_before;
                }
                if cell_first_vpos != 0 {
                    return false;
                }
                let Some(prev) = p.line_segs.get(li - 1) else {
                    return false;
                };
                let Some(cur) = p.line_segs.get(li) else {
                    return false;
                };
                let prev_end = prev.vertical_pos + prev.line_height;
                cur.vertical_pos >= 0 && prev_end > 0 && cur.vertical_pos < prev_end
            };
            // [Task #993] 줄 높이는 렌더러(layout_composed_paragraph)와 동일하게
            // corrected_line_height 를 적용한다 — raw line_height 가 폰트보다
            // 작은 폴백 케이스에서 렌더러가 키운 높이를 컷 측정이 따라가지
            // 못하면 분할 표가 페이지를 넘는다(측정 공간 불일치).
            let corrected_h = |line: &ComposedLine| -> f64 {
                let raw_lh = hwpunit_to_px(line.line_height, self.dpi);
                match para_style {
                    Some(ps) => {
                        let max_fs = line
                            .runs
                            .iter()
                            .map(|r| {
                                let ts = super::text_measurement::resolved_to_text_style(
                                    styles,
                                    r.char_style_id,
                                    r.lang_index,
                                );
                                if ts.font_size > 0.0 {
                                    ts.font_size
                                } else {
                                    12.0
                                }
                            })
                            .fold(0.0f64, f64::max);
                        crate::renderer::corrected_line_height_for_variant_synthetic(
                            raw_lh,
                            max_fs,
                            ps.line_spacing_type,
                            ps.line_spacing,
                            self.is_hwp3_variant.get()
                                && p.line_segs.is_empty()
                                && !p.text.is_empty(),
                        )
                    }
                    None => raw_lh,
                }
            };
            let has_table_in_para = p.controls.iter().any(|c| matches!(c, Control::Table(_)));
            let line_count = comp.lines.len();
            // [Task #1073] 텍스트 없는 문단(가시 텍스트 없음 — 합성 줄은 placeholder)에 단일
            // 중첩 표가 있고 그 표가 2행 이상이면 per-중첩행 유닛으로 분해 — advance_row_cut 가
            // 중첩 표 행 경계에서 페이지 분할할 수 있게 한다. whole-row 높이 합은
            // calc_nested_table_height 와 정확히 일치(드리프트 0):
            // Σ row_h + cs*(n-1) + om_top + om_bottom + spacing.
            // 2단계+ 중첩/텍스트 동거 문단은 아래 atom 폴백 유지(범위 외).
            if has_table_in_para && p.text.trim().is_empty() {
                let nested_tables: Vec<&crate::model::table::Table> = p
                    .controls
                    .iter()
                    .filter_map(|c| match c {
                        Control::Table(t) => Some(t.as_ref()),
                        _ => None,
                    })
                    .collect();
                if nested_tables.len() == 1 && nested_tables[0].row_count >= 2 {
                    let nt = nested_tables[0];
                    let ncol = nt.col_count as usize;
                    let nrow = nt.row_count as usize;
                    // 분할 컷은 저장된 표 높이보다 실제 콘텐츠 높이를 기준으로 잡아야
                    // page-larger 중첩 표가 한컴처럼 행 단위로 이어진다.
                    let rhs = self.resolve_row_heights_for_content(nt, ncol, nrow, None, styles);
                    let ncs = hwpunit_to_px(nt.cell_spacing as i32, self.dpi);
                    let om_top = hwpunit_to_px(nt.outer_margin_top as i32, self.dpi);
                    let om_bot = hwpunit_to_px(nt.outer_margin_bottom as i32, self.dpi);
                    for (ri, rh) in rhs.iter().enumerate() {
                        let mut uh = *rh;
                        if ri + 1 < nrow {
                            uh += ncs;
                        }
                        if ri == 0 {
                            uh += om_top + spacing_before;
                        }
                        if ri + 1 == nrow {
                            uh += om_bot + spacing_after;
                        }
                        units.push(CellUnit {
                            height: uh,
                            hard_break_before: reset_before && ri == 0,
                            para_idx: pi,
                            vis_start: 0,
                            vis_end: line_count.max(1),
                            nested_row: Some(ri),
                        });
                    }
                    continue;
                }
            }
            if line_count == 0 || has_table_in_para {
                // 중첩 표/빈 문단 — atomic 유닛 1개.
                let nested_h: f64 = p
                    .controls
                    .iter()
                    .map(|ctrl| {
                        if let Control::Table(t) = ctrl {
                            self.calc_nested_table_height(t, styles)
                        } else {
                            0.0
                        }
                    })
                    .sum();
                let para_h = if line_count == 0 {
                    let h = if nested_h > 0.0 {
                        nested_h
                    } else {
                        hwpunit_to_px(400, self.dpi)
                    };
                    spacing_before + h + spacing_after
                } else {
                    let line_based_h: f64 = comp
                        .lines
                        .iter()
                        .enumerate()
                        .map(|(li, line)| {
                            let h = corrected_h(line);
                            let ls = hwpunit_to_px(line.line_spacing, self.dpi);
                            let is_cell_last_line = is_last_para && li + 1 == line_count;
                            // [Task #1022/#1086] trailing ls 규칙 — HeightMeasurer 와
                            // 정합. CellBreak/TAC 표는 기존 trailing geometry 를 보존하고,
                            // block RowBreak 표는 렌더 가시 높이처럼 셀 마지막 줄
                            // trailing 을 제외해 행 fit 을 맞춘다.
                            let is_block_rowbreak = matches!(
                                table.page_break,
                                crate::model::table::TablePageBreak::RowBreak
                            ) && !table.common.treat_as_char;
                            let include_trailing_ls = !is_cell_last_line || para_count > 1;
                            let include_trailing_ls =
                                include_trailing_ls && (!is_cell_last_line || !is_block_rowbreak);
                            let mut lh = if include_trailing_ls { h + ls } else { h };
                            if li == 0 {
                                lh += spacing_before;
                            }
                            if li == line_count - 1 {
                                lh += spacing_after;
                            }
                            lh
                        })
                        .sum();
                    nested_h.max(line_based_h)
                };
                units.push(CellUnit {
                    height: para_h,
                    hard_break_before: reset_before,
                    para_idx: pi,
                    vis_start: 0,
                    vis_end: line_count.max(1),
                    nested_row: None,
                });
            } else {
                // 일반 텍스트 문단 — 합성 줄마다 유닛 1개.
                for (li, line) in comp.lines.iter().enumerate() {
                    let h = corrected_h(line);
                    let ls = hwpunit_to_px(line.line_spacing, self.dpi);
                    let is_cell_last_line = is_last_para && li + 1 == line_count;
                    let is_block_rowbreak = matches!(
                        table.page_break,
                        crate::model::table::TablePageBreak::RowBreak
                    ) && !table.common.treat_as_char;
                    let include_trailing_ls = !is_cell_last_line || para_count > 1;
                    let include_trailing_ls =
                        include_trailing_ls && (!is_cell_last_line || !is_block_rowbreak);
                    let mut lh = if include_trailing_ls { h + ls } else { h };
                    if li == 0 {
                        lh += spacing_before;
                    }
                    if li == line_count - 1 {
                        lh += spacing_after;
                    }
                    units.push(CellUnit {
                        height: lh,
                        hard_break_before: line_reset_before(li),
                        para_idx: pi,
                        vis_start: li,
                        vis_end: li + 1,
                        nested_row: None,
                    });
                }
            }
        }

        // [Task #1022] 비인라인 Picture/Shape(wrap=TopAndBottom) — LINE_SEG.lh 에
        // 미포함이므로 HeightMeasurer 와 동일하게 cell_units 끝에 별도 가산.
        // 분할 가능하도록 ~16px 단위로 쪼개되, 가시 줄은 없다(filler).
        {
            let mut non_inline_h = 0.0f64;
            for para in &cell.paragraphs {
                for ctrl in &para.controls {
                    match ctrl {
                        Control::Picture(pic) => {
                            non_inline_h += self.non_inline_control_flow_height(&pic.common);
                        }
                        crate::model::control::Control::Shape(shape) => {
                            non_inline_h += self.non_inline_control_flow_height(shape.common());
                        }
                        _ => {}
                    }
                }
            }
            if non_inline_h > 0.5 {
                let last_para = para_count.saturating_sub(1);
                const FILLER_UNIT_PX: f64 = 16.0;
                let mut remaining = non_inline_h;
                while remaining > 0.5 {
                    let h = remaining.min(FILLER_UNIT_PX);
                    units.push(CellUnit {
                        height: h,
                        hard_break_before: false,
                        para_idx: last_para,
                        vis_start: 0,
                        vis_end: 0,
                        nested_row: None,
                    });
                    remaining -= h;
                }
            }
        }
        let _ = (pad_top, pad_bottom); // [Task #1022] cell.height 필러 제거 — row_cut_content_height 가 셀별 max(cell.height, content+pad) 로 행 단계에서 정합.
        units
    }

    /// [Task #993] 분할 표 행 컷을 전진시킨다 — 분할 표 페이지네이션의 단일 권위 함수.
    ///
    /// `start_cut`(이전 페이지까지 셀별 소비 유닛 수)에서 시작해, 각 셀을 공통
    /// 높이 예산 `avail_height` 안에서 동시 전진시킨다. 어느 유닛도 `avail_height`
    /// 안에 안 들어가면 진행 보장을 위해 셀당 유닛 1개는 강제 소비한다. vpos
    /// 리셋(hard break)을 만나면 그 셀은 거기서 정지한다.
    ///
    /// 페이지네이터(분할 판정)와 렌더러(가시 범위)가 모두 이 함수를 호출하므로
    /// 두 경로의 컷이 정의상 일치한다.
    pub(crate) fn advance_row_cut(
        &self,
        table: &crate::model::table::Table,
        row: usize,
        start_cut: &[usize],
        avail_height: f64,
        styles: &ResolvedStyleSet,
    ) -> RowCutResult {
        let mut row_cells: Vec<&crate::model::table::Cell> = table
            .cells
            .iter()
            .filter(|c| c.row as usize == row && c.row_span == 1)
            .collect();
        row_cells.sort_by_key(|c| c.col);

        let mut end_cut: RowCut = Vec::with_capacity(row_cells.len());
        let mut hit_hard_break = false;
        let mut fully_consumed = true;
        let mut consumed_height = 0.0f64;
        let rewind_internal_hard_break_orphan = Self::row_has_prior_rowspan_cover(table, row);
        for (i, cell) in row_cells.iter().enumerate() {
            let units = self.cell_units(cell, table, styles);
            let start = start_cut.get(i).copied().unwrap_or(0).min(units.len());
            let mut j = start;
            let mut h = 0.0f64;
            while j < units.len() {
                let u = &units[j];
                // 시작 유닛(j==start)은 항상 소비 — 진행 보장.
                if j > start && u.hard_break_before {
                    if rewind_internal_hard_break_orphan {
                        Self::rewind_rowbreak_orphan_before_hard_break(
                            table, &units, start, &mut j, &mut h,
                        );
                    }
                    hit_hard_break = true;
                    break;
                }
                if j > start && h + u.height > avail_height {
                    break;
                }
                h += u.height;
                j += 1;
            }
            if j < units.len() {
                fully_consumed = false;
            }
            if h > consumed_height {
                consumed_height = h;
            }
            end_cut.push(j);
        }
        RowCutResult {
            end_cut,
            hit_hard_break,
            fully_consumed,
            consumed_height,
        }
    }

    /// [Task #1025] 행블록 컷 — rowspan(rs>1) 셀로 묶인 연속 행 블록 `[b_start, b_end)`
    /// 의 셀을 `(row, col)` 안정 순서로 순회하며 CellUnit(줄/중첩 atom) 단위로 진행한다.
    /// `advance_row_cut` 의 블록 일반화: 블록을 걸친 rs>1 셀 + 블록 내 각 행의 셀을 모두
    /// 포함한다. rs>1 라벨 셀은 첫 조각(start_cut 비었을 때)에서 전량 소비되고, 연속
    /// 조각에선 시작 인덱스가 이미 끝이라 0 유닛 진행 → 렌더 공란(한컴 정답).
    /// 거대 `row_span==1` 셀은 줄 단위로 페이지 경계까지 채우고 잔여를 다음 조각으로 넘긴다.
    ///
    /// 셀 순서·인덱스는 `row_block_content_height` / 렌더러와 공유하는 단일 정의다.
    /// 단일 비-rowspan 행(`b_end==b_start+1`, 블록 내 rs>1 셀 없음)에서는
    /// `advance_row_cut` 과 동일 결과를 낸다(회귀 0).
    pub(crate) fn advance_row_block_cut(
        &self,
        table: &crate::model::table::Table,
        b_start: usize,
        b_end: usize,
        start_cut: &[usize],
        avail_height: f64,
        styles: &ResolvedStyleSet,
    ) -> RowCutResult {
        let mut cells = Self::row_block_cells(table, b_start, b_end);
        // 안정 순서: (row, col) 오름차순.
        cells.sort_by_key(|c| (c.row, c.col));

        let mut end_cut: RowCut = Vec::with_capacity(cells.len());
        let mut hit_hard_break = false;
        let mut fully_consumed = true;
        let mut consumed_height = 0.0f64;
        for (i, cell) in cells.iter().enumerate() {
            let units = self.cell_units(cell, table, styles);
            let start = start_cut.get(i).copied().unwrap_or(0).min(units.len());
            let mut j = start;
            let mut h = 0.0f64;
            while j < units.len() {
                let u = &units[j];
                // 시작 유닛(j==start)은 항상 소비 — 진행 보장.
                if j > start && u.hard_break_before {
                    Self::rewind_rowbreak_orphan_before_hard_break(
                        table, &units, start, &mut j, &mut h,
                    );
                    hit_hard_break = true;
                    break;
                }
                if j > start && h + u.height > avail_height {
                    break;
                }
                h += u.height;
                j += 1;
            }
            if j < units.len() {
                fully_consumed = false;
            }
            if h > consumed_height {
                consumed_height = h;
            }
            end_cut.push(j);
        }
        RowCutResult {
            end_cut,
            hit_hard_break,
            fully_consumed,
            consumed_height,
        }
    }

    /// RowBreak rowspan 블록에서 셀의 행 시작 y를 반영해 컷을 전진시킨다.
    ///
    /// 일반 `advance_row_block_cut`은 블록 안의 모든 셀에 같은 예산을 주기 때문에,
    /// 위쪽 큰 셀이 페이지 경계에서 잘릴 때 아래 행의 짧은 셀까지 먼저 소비할 수 있다.
    /// 이 함수는 행별 top offset을 빼고 남은 예산으로 셀을 전진시켜 같은 블록 안의
    /// 아래 행 내용이 한컴처럼 다음 조각에 남도록 한다.
    pub(crate) fn advance_row_block_cut_with_row_offsets(
        &self,
        table: &crate::model::table::Table,
        b_start: usize,
        b_end: usize,
        start_cut: &[usize],
        avail_height: f64,
        row_offsets: &[f64],
        styles: &ResolvedStyleSet,
    ) -> RowCutResult {
        let mut cells = Self::row_block_cells(table, b_start, b_end);
        cells.sort_by_key(|c| (c.row, c.col));

        let mut end_cut: RowCut = Vec::with_capacity(cells.len());
        let mut hit_hard_break = false;
        let mut fully_consumed = true;
        let mut consumed_height = 0.0f64;
        for (i, cell) in cells.iter().enumerate() {
            let units = self.cell_units(cell, table, styles);
            let start = start_cut.get(i).copied().unwrap_or(0).min(units.len());
            let cell_row = cell.row as usize;
            let row_offset = cell_row
                .checked_sub(b_start)
                .and_then(|idx| row_offsets.get(idx))
                .copied()
                .unwrap_or(0.0);
            let cell_budget = (avail_height - row_offset).max(0.0);
            let allow_force_progress = row_offset <= 0.5;
            let mut j = start;
            let mut h = 0.0f64;
            while j < units.len() {
                let u = &units[j];
                if j > start && u.hard_break_before {
                    Self::rewind_rowbreak_orphan_before_hard_break(
                        table, &units, start, &mut j, &mut h,
                    );
                    hit_hard_break = true;
                    break;
                }
                if j > start && h + u.height > cell_budget {
                    break;
                }
                if j == start && !allow_force_progress && h + u.height > cell_budget {
                    break;
                }
                h += u.height;
                j += 1;
            }
            if j < units.len() {
                fully_consumed = false;
            }
            if h > 0.0 {
                consumed_height = consumed_height.max(row_offset + h);
            }
            end_cut.push(j);
        }
        RowCutResult {
            end_cut,
            hit_hard_break,
            fully_consumed,
            consumed_height,
        }
    }

    fn rewind_rowbreak_orphan_before_hard_break(
        table: &crate::model::table::Table,
        units: &[CellUnit],
        start: usize,
        j: &mut usize,
        h: &mut f64,
    ) {
        if !matches!(
            table.page_break,
            crate::model::table::TablePageBreak::RowBreak
        ) || *j <= start + 1
        {
            return;
        }

        let hard_break_unit = &units[*j];
        let prev = &units[*j - 1];
        if prev.para_idx == hard_break_unit.para_idx {
            *h -= prev.height;
            *j -= 1;
        }
    }

    fn row_has_prior_rowspan_cover(table: &crate::model::table::Table, row: usize) -> bool {
        table.cells.iter().any(|cell| {
            let start = cell.row as usize;
            let end = start + (cell.row_span as usize).max(1);
            cell.row_span > 1 && start < row && row < end
        })
    }

    /// RowBreak 표의 rowspan 블록 중 셀 내부 HWP page reset 이 처음 나타나는 셀의
    /// 시작 행을 찾는다. 단순 rowspan 라벨 표는 기존 행 경계 분할을 유지한다.
    pub(crate) fn row_block_first_internal_hard_break_row(
        &self,
        table: &crate::model::table::Table,
        b_start: usize,
        b_end: usize,
        styles: &ResolvedStyleSet,
    ) -> Option<usize> {
        Self::row_block_cells(table, b_start, b_end)
            .iter()
            .filter_map(|cell| {
                let has_hard_break = self
                    .cell_units(cell, table, styles)
                    .iter()
                    .enumerate()
                    .any(|(i, unit)| i > 0 && unit.hard_break_before);
                has_hard_break.then_some(cell.row as usize)
            })
            .min()
    }

    /// RowBreak 표의 rowspan 블록 중 셀 내부 HWP page reset 이 있는 블록만
    /// 블록 컷 대상으로 삼기 위한 가드.
    pub(crate) fn row_block_has_internal_hard_break(
        &self,
        table: &crate::model::table::Table,
        b_start: usize,
        b_end: usize,
        styles: &ResolvedStyleSet,
    ) -> bool {
        self.row_block_first_internal_hard_break_row(table, b_start, b_end, styles)
            .is_some()
    }

    /// [Task #1025] 행블록 `[b_start, b_end)` 와 교차하는 셀(rs>1 포함)을 모은다.
    /// `advance_row_block_cut` / `row_block_content_height` / 렌더러 공유 — 순서는
    /// 호출부에서 `(row, col)` 로 정렬한다.
    pub(crate) fn row_block_cells<'a>(
        table: &'a crate::model::table::Table,
        b_start: usize,
        b_end: usize,
    ) -> Vec<&'a crate::model::table::Cell> {
        table
            .cells
            .iter()
            .filter(|c| {
                let cr = c.row as usize;
                let ce = cr + (c.row_span as usize).max(1);
                cr < b_end && ce > b_start
            })
            .collect()
    }

    /// [Task #1025] 행블록 컷 범위 `[start_cut, end_cut)` 의 블록 표시 높이(패딩 포함).
    /// 블록 셀별 `content_in_cut + pad`, 블록 max. `advance_row_block_cut` 과 동일한
    /// `(row, col)` 셀 순서를 사용한다.
    pub(crate) fn row_block_content_height(
        &self,
        table: &crate::model::table::Table,
        b_start: usize,
        b_end: usize,
        start_cut: &[usize],
        end_cut: &[usize],
        styles: &ResolvedStyleSet,
    ) -> f64 {
        let mut cells = Self::row_block_cells(table, b_start, b_end);
        cells.sort_by_key(|c| (c.row, c.col));
        let mut max_h = 0.0f64;
        for (i, cell) in cells.iter().enumerate() {
            let units = self.cell_units(cell, table, styles);
            let su = start_cut.get(i).copied().unwrap_or(0).min(units.len());
            let eu = end_cut
                .get(i)
                .copied()
                .unwrap_or(units.len())
                .clamp(su, units.len());
            let content: f64 = units[su..eu].iter().map(|u| u.height).sum();
            let (_, _, pad_top, pad_bottom) = self.resolve_cell_padding(cell, table);
            let h = content + pad_top + pad_bottom;
            if h > max_h {
                max_h = h;
            }
        }
        max_h
    }

    /// 블록 컷 벡터를 특정 행의 per-row 컷으로 변환해 해당 행 표시 높이를 계산한다.
    pub(crate) fn row_block_cut_row_content_height(
        &self,
        table: &crate::model::table::Table,
        b_start: usize,
        b_end: usize,
        row: usize,
        start_cut: &[usize],
        end_cut: &[usize],
        styles: &ResolvedStyleSet,
    ) -> f64 {
        let mut block_cells = Self::row_block_cells(table, b_start, b_end);
        block_cells.sort_by_key(|c| (c.row, c.col));

        let mut row_cells: Vec<&crate::model::table::Cell> = table
            .cells
            .iter()
            .filter(|c| c.row as usize == row && c.row_span == 1)
            .collect();
        row_cells.sort_by_key(|c| c.col);

        if row_cells.is_empty() {
            return 0.0;
        }

        let mut per_start = Vec::with_capacity(row_cells.len());
        let mut per_end = Vec::with_capacity(row_cells.len());
        let mut has_visible_range = false;
        let mut has_row_cut = false;
        for cell in row_cells {
            let block_idx = block_cells
                .iter()
                .position(|c| c.row == cell.row && c.col == cell.col);
            let units = self.cell_units(cell, table, styles);
            let su = block_idx
                .and_then(|idx| start_cut.get(idx).copied())
                .unwrap_or(0)
                .min(units.len());
            let eu = block_idx
                .and_then(|idx| end_cut.get(idx).copied())
                .unwrap_or(units.len())
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

        if !has_visible_range {
            return 0.0;
        }

        if has_row_cut {
            self.row_cut_content_height(table, row, &per_start, &per_end, styles)
        } else {
            self.row_cut_content_height(table, row, &[], &[], styles)
        }
    }

    /// [Task #993] 한 셀의 유닛 범위 `[start_unit, end_unit)`를 문단별 줄 범위로
    /// 변환한다. `layout_partial_table`이 `RowCut`으로 가시 범위를 렌더할 때
    /// 사용 — 결과는 종전 `compute_cell_line_ranges`와 같은
    /// `Vec<(start_line, end_line)>` 형식(문단마다 1개, 미가시 문단은 `(0,0)`).
    pub(crate) fn cell_line_ranges_from_cut(
        &self,
        cell: &crate::model::table::Cell,
        table: &crate::model::table::Table,
        styles: &ResolvedStyleSet,
        start_unit: usize,
        end_unit: usize,
    ) -> Vec<(usize, usize)> {
        let units = self.cell_units(cell, table, styles);
        let mut ranges = vec![(0usize, 0usize); cell.paragraphs.len()];
        let mut seen = vec![false; cell.paragraphs.len()];
        let lo = start_unit.min(units.len());
        let hi = end_unit.min(units.len());
        for u in units.iter().take(hi).skip(lo) {
            if u.para_idx >= ranges.len() {
                continue;
            }
            if !seen[u.para_idx] {
                ranges[u.para_idx] = (u.vis_start, u.vis_end);
                seen[u.para_idx] = true;
            } else {
                let r = &mut ranges[u.para_idx];
                r.0 = r.0.min(u.vis_start);
                r.1 = r.1.max(u.vis_end);
            }
        }
        ranges
    }

    /// RowBreak 분할의 컷 범위에 실제 보이는 내용이 남아 있는지 확인한다.
    ///
    /// 마지막 continuation 에 빈 문단/패딩만 남은 조각은 한컴 PDF에서 별도 페이지를
    /// 만들지 않는 경우가 있어, 페이지네이터가 terminal sliver 를 걸러낼 때 사용한다.
    pub(crate) fn row_cut_range_has_visible_content(
        &self,
        table: &crate::model::table::Table,
        row: usize,
        start_cut: &[usize],
        end_cut: &[usize],
        styles: &ResolvedStyleSet,
    ) -> bool {
        let mut row_cells: Vec<&crate::model::table::Cell> = table
            .cells
            .iter()
            .filter(|c| c.row as usize == row && c.row_span == 1)
            .collect();
        row_cells.sort_by_key(|c| c.col);

        for (i, cell) in row_cells.iter().enumerate() {
            let units = self.cell_units(cell, table, styles);
            let su = start_cut.get(i).copied().unwrap_or(0).min(units.len());
            let eu = end_cut
                .get(i)
                .copied()
                .unwrap_or(units.len())
                .clamp(su, units.len());
            if units[su..eu]
                .iter()
                .any(|unit| Self::cell_unit_has_visible_content(cell, unit))
            {
                return true;
            }
        }

        false
    }

    fn cell_unit_has_visible_content(cell: &crate::model::table::Cell, unit: &CellUnit) -> bool {
        if unit.nested_row.is_some() {
            return true;
        }

        let Some(para) = cell.paragraphs.get(unit.para_idx) else {
            return false;
        };
        !para.text.trim().is_empty() || !para.controls.is_empty()
    }

    /// [Task #993 / #1022] 분할 행에서 컷 범위 `[start_cut, end_cut)` 사이의
    /// **행 총 높이**(패딩 포함)를 반환한다. HeightMeasurer 와 정합 — 셀별로
    /// `max(cell.height, content + pad_cell)` 를 산출해 행 max.
    ///
    /// - 분할 아닌 행(start_cut/end_cut 모두 빈 Vec): `max(cell.height,
    ///   content+pad_cell)` per cell, row max.
    /// - 분할 행(컷 범위 일부): `content_in_range + pad_cell` per cell, row max.
    ///   분할 시 cell.height 강제는 적용하지 않는다(콘텐츠가 부분이므로).
    ///
    /// 셀 인덱스는 `advance_row_cut` 과 동일하게 `row_span==1` 셀을 col
    /// 오름차순 정렬한 순서다.
    pub(crate) fn row_cut_content_height(
        &self,
        table: &crate::model::table::Table,
        row: usize,
        start_cut: &[usize],
        end_cut: &[usize],
        styles: &ResolvedStyleSet,
    ) -> f64 {
        let mut row_cells: Vec<&crate::model::table::Cell> = table
            .cells
            .iter()
            .filter(|c| c.row as usize == row && c.row_span == 1)
            .collect();
        row_cells.sort_by_key(|c| c.col);
        let is_whole_row = start_cut.is_empty() && end_cut.is_empty();
        let mut max_h = 0.0f64;
        for (i, cell) in row_cells.iter().enumerate() {
            let units = self.cell_units(cell, table, styles);
            let su = start_cut.get(i).copied().unwrap_or(0).min(units.len());
            let eu = end_cut
                .get(i)
                .copied()
                .unwrap_or(units.len())
                .clamp(su, units.len());
            let content: f64 = units[su..eu].iter().map(|u| u.height).sum();
            let (_, _, pad_top, pad_bottom) = self.resolve_cell_padding(cell, table);
            let pad_cell = pad_top + pad_bottom;
            let cell_h_px = if cell.height < 0x8000_0000 {
                hwpunit_to_px(cell.height as i32, self.dpi)
            } else {
                0.0
            };
            let h = if is_whole_row {
                // HeightMeasurer required_height + row 단계 1 cell.height max 정합.
                (content + pad_cell).max(cell_h_px)
            } else {
                // 분할 행 — cell.height 강제 없음.
                content + pad_cell
            };
            if h > max_h {
                max_h = h;
            }
        }
        max_h
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
            composed_paras,
            paragraphs,
            line_ranges,
            styles,
            0.0,
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
        let mut cum_pos = 0.0f64; // 누적 콘텐츠 위치 (compute_cell_line_ranges 와 동일)
        let first_visible_pi = line_ranges.iter().position(|&(s, e)| s < e);
        let _last_visible_pi = line_ranges.iter().rposition(|&(s, e)| s < e);
        for (pi, ((comp, para), &(start, end))) in composed_paras
            .iter()
            .zip(paragraphs.iter())
            .zip(line_ranges.iter())
            .enumerate()
        {
            let para_style = styles.para_styles.get(para.para_shape_id as usize);
            let is_last_para = pi + 1 == para_count;
            let line_count = comp.lines.len();
            let spacing_before = if pi > 0 {
                para_style.map(|s| s.spacing_before).unwrap_or(0.0)
            } else {
                0.0
            };
            let spacing_after = if !is_last_para {
                para_style.map(|s| s.spacing_after).unwrap_or(0.0)
            } else {
                0.0
            };
            let has_table_in_para = para.controls.iter().any(|c| matches!(c, Control::Table(_)));

            // [Task #362] nested table paragraph 의 실제 콘텐츠 높이
            // (compute_cell_line_ranges 와 동일한 시멘틱)
            let para_h = if line_count == 0 || has_table_in_para {
                let nested_h: f64 = para
                    .controls
                    .iter()
                    .map(|ctrl| {
                        if let Control::Table(t) = ctrl {
                            self.calc_nested_table_height(t, styles)
                        } else {
                            0.0
                        }
                    })
                    .sum();
                if line_count == 0 {
                    let h = if nested_h > 0.0 {
                        nested_h
                    } else {
                        hwpunit_to_px(400, self.dpi)
                    };
                    spacing_before + h + spacing_after
                } else {
                    let line_based_h: f64 = comp
                        .lines
                        .iter()
                        .enumerate()
                        .map(|(li, line)| {
                            let h = hwpunit_to_px(line.line_height, self.dpi);
                            let ls = hwpunit_to_px(line.line_spacing, self.dpi);
                            let is_cell_last_line = is_last_para && li + 1 == line_count;
                            let mut lh = if !is_cell_last_line { h + ls } else { h };
                            if li == 0 {
                                lh += spacing_before;
                            }
                            if li == line_count - 1 {
                                lh += spacing_after;
                            }
                            lh
                        })
                        .sum();
                    nested_h.max(line_based_h)
                }
            } else {
                0.0 // 일반 line 단위 처리는 아래 분기에서
            };

            // nested table paragraph 처리
            if (line_count == 0 || has_table_in_para) && start < end {
                // [Task #362] 한 페이지보다 큰 nested table 분할: 시작 위치가 offset 이전이면
                // 잔여 = para_end_pos - max(content_offset, para_start_pos)
                let para_start_pos = cum_pos;
                let para_end_pos = cum_pos + para_h;
                if content_offset > 0.0
                    && para_start_pos < content_offset
                    && para_end_pos > content_offset
                {
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
                    let line_based_h: f64 = comp
                        .lines
                        .iter()
                        .enumerate()
                        .map(|(li, line)| {
                            let h = hwpunit_to_px(line.line_height, self.dpi);
                            let ls = hwpunit_to_px(line.line_spacing, self.dpi);
                            let is_cell_last_line = is_last_para && li + 1 == line_count;
                            let mut lh = if !is_cell_last_line { h + ls } else { h };
                            if li == 0 {
                                lh += spacing_before;
                            }
                            if li == line_count - 1 {
                                lh += spacing_after;
                            }
                            lh
                        })
                        .sum();
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
            let line_based_h: f64 = comp
                .lines
                .iter()
                .enumerate()
                .map(|(li, line)| {
                    let h = hwpunit_to_px(line.line_height, self.dpi);
                    let ls = hwpunit_to_px(line.line_spacing, self.dpi);
                    let is_cell_last_line = is_last_para && li + 1 == line_count;
                    let mut lh = if !is_cell_last_line { h + ls } else { h };
                    if li == 0 {
                        lh += spacing_before;
                    }
                    if li == line_count - 1 {
                        lh += spacing_after;
                    }
                    lh
                })
                .sum();
            cum_pos += line_based_h;
        }
        total
    }
}

#[cfg(test)]
mod row_cut_tests {
    use super::LayoutEngine;
    use crate::model::paragraph::{LineSeg, Paragraph};
    use crate::model::table::{Cell, Table};
    use crate::renderer::composer::{ComposedLine, ComposedParagraph, ComposedTextRun};
    use crate::renderer::style_resolver::ResolvedStyleSet;

    /// line_height=1200 HU (=16 px @96dpi), line_spacing=0 인 N줄 텍스트 문단.
    /// vpos 는 vpos_start 부터 1200 HU 간격.
    fn text_para(n_lines: usize, vpos_start: i32) -> Paragraph {
        Paragraph {
            line_segs: (0..n_lines)
                .map(|i| LineSeg {
                    vertical_pos: vpos_start + i as i32 * 1200,
                    line_height: 1200,
                    line_spacing: 0,
                    ..Default::default()
                })
                .collect(),
            ..Default::default()
        }
    }

    fn cell(row: u16, col: u16, paragraphs: Vec<Paragraph>) -> Cell {
        Cell {
            row,
            col,
            row_span: 1,
            col_span: 1,
            width: 10000,
            paragraphs,
            ..Default::default()
        }
    }

    fn table(cells: Vec<Cell>) -> Table {
        let row_count = cells.iter().map(|c| c.row + 1).max().unwrap_or(1);
        let col_count = cells.iter().map(|c| c.col + 1).max().unwrap_or(1);
        Table {
            row_count,
            col_count,
            cells,
            ..Default::default()
        }
    }

    fn rowbreak_table(cells: Vec<Cell>) -> Table {
        Table {
            page_break: crate::model::table::TablePageBreak::RowBreak,
            ..table(cells)
        }
    }

    fn composed_text(text: &str) -> ComposedParagraph {
        ComposedParagraph {
            lines: vec![ComposedLine {
                runs: vec![ComposedTextRun {
                    text: text.to_string(),
                    ..Default::default()
                }],
                line_height: 1000,
                baseline_distance: 850,
                segment_width: 1000,
                column_start: 0,
                line_spacing: 0,
                has_line_break: false,
                char_start: 0,
            }],
            para_style_id: 0,
            inline_controls: Vec::new(),
            numbering_text: None,
            tac_controls: Vec::new(),
            footnote_positions: Vec::new(),
            tab_extended: Vec::new(),
        }
    }

    #[test]
    fn test_shrink_cell_padding_preserves_explicit_cell_margin() {
        let eng = LayoutEngine::new(96.0);
        let styles = ResolvedStyleSet::default();
        let composed = vec![composed_text("12345678901234567890")];
        let paragraphs = vec![Paragraph::default()];

        let shrunk = eng.shrink_cell_padding_for_overflow(
            20.0,
            20.0,
            30.0,
            &composed,
            &paragraphs,
            &styles,
            false,
        );
        assert!(
            shrunk.0 < 20.0 || shrunk.1 < 20.0,
            "일반 셀의 기존 오버플로우 방어는 유지되어야 함: {shrunk:?}"
        );

        let preserved = eng.shrink_cell_padding_for_overflow(
            20.0,
            20.0,
            30.0,
            &composed,
            &paragraphs,
            &styles,
            true,
        );
        assert_eq!(
            preserved,
            (20.0, 20.0),
            "안 여백 지정 셀은 한컴처럼 입력한 좌우 여백을 렌더링에서도 보존해야 함"
        );
    }

    #[test]
    fn test_advance_row_cut_basic_split() {
        // 1행 1셀, 6줄(각 16px). avail=50 → 3줄(48px) 소비, 4번째(64px)는 초과.
        let eng = LayoutEngine::new(96.0);
        let styles = ResolvedStyleSet::default();
        let t = table(vec![cell(0, 0, vec![text_para(6, 0)])]);
        let r = eng.advance_row_cut(&t, 0, &[], 50.0, &styles);
        assert_eq!(r.end_cut, vec![3]);
        assert!(!r.fully_consumed);
        assert!(!r.hit_hard_break);
        assert!((r.consumed_height - 48.0).abs() < 0.5);
    }

    #[test]
    fn test_advance_row_cut_fully_consumed() {
        let eng = LayoutEngine::new(96.0);
        let styles = ResolvedStyleSet::default();
        let t = table(vec![cell(0, 0, vec![text_para(6, 0)])]);
        let r = eng.advance_row_cut(&t, 0, &[], 500.0, &styles);
        assert_eq!(r.end_cut, vec![6]);
        assert!(r.fully_consumed);
    }

    #[test]
    fn test_advance_row_cut_force_progress() {
        // avail 이 한 줄(16px)보다 작아도 시작 유닛 1개는 강제 소비 — 무한 루프 방지.
        let eng = LayoutEngine::new(96.0);
        let styles = ResolvedStyleSet::default();
        let t = table(vec![cell(0, 0, vec![text_para(6, 0)])]);
        let r = eng.advance_row_cut(&t, 0, &[], 5.0, &styles);
        assert_eq!(r.end_cut, vec![1]);
        assert!(!r.fully_consumed);
    }

    #[test]
    fn test_advance_row_cut_vpos_reset_hard_break() {
        // 문단0(3줄 vpos 0..2400) + 문단1(2줄 vpos 1000..) — 문단1 시작 vpos 가
        // 문단0 끝(3600)보다 작아 vpos 리셋 → 문단1 앞에서 강제 분할.
        let eng = LayoutEngine::new(96.0);
        let styles = ResolvedStyleSet::default();
        let t = table(vec![cell(0, 0, vec![text_para(3, 0), text_para(2, 1000)])]);
        // avail 충분해도 리셋에서 정지.
        let r = eng.advance_row_cut(&t, 0, &[], 1000.0, &styles);
        assert_eq!(r.end_cut, vec![3]);
        assert!(r.hit_hard_break);
        assert!(!r.fully_consumed);
        // 다음 프래그먼트: 리셋 지점부터 재개 — 시작 유닛은 리셋이어도 소비.
        let r2 = eng.advance_row_cut(&t, 0, &r.end_cut, 1000.0, &styles);
        assert_eq!(r2.end_cut, vec![5]);
        assert!(r2.fully_consumed);
    }

    #[test]
    fn test_advance_row_cut_rowbreak_rewinds_internal_hard_break_orphan() {
        let eng = LayoutEngine::new(96.0);
        let styles = ResolvedStyleSet::default();
        let internal_reset = Paragraph {
            line_segs: vec![
                LineSeg {
                    vertical_pos: 0,
                    line_height: 1200,
                    line_spacing: 0,
                    ..Default::default()
                },
                LineSeg {
                    vertical_pos: 0,
                    line_height: 1200,
                    line_spacing: 0,
                    ..Default::default()
                },
            ],
            ..Default::default()
        };
        let t = rowbreak_table(vec![
            rscell(0, 0, 2, vec![text_para(1, 0)]),
            cell(
                1,
                1,
                vec![text_para(1, 0), text_para(1, 1200), internal_reset],
            ),
        ]);

        let r = eng.advance_row_cut(&t, 1, &[], 1000.0, &styles);

        assert_eq!(r.end_cut, vec![2]);
        assert!(r.hit_hard_break);
        assert!(!r.fully_consumed);
    }

    #[test]
    fn test_advance_row_cut_multi_cell() {
        // 1행 2셀: 셀0=3줄, 셀1=6줄. avail 충분 → 각 셀 전부 소비,
        // consumed_height = 두 셀 표시 높이의 최댓값(셀1, 96px).
        let eng = LayoutEngine::new(96.0);
        let styles = ResolvedStyleSet::default();
        let t = table(vec![
            cell(0, 0, vec![text_para(3, 0)]),
            cell(0, 1, vec![text_para(6, 0)]),
        ]);
        let r = eng.advance_row_cut(&t, 0, &[], 500.0, &styles);
        assert_eq!(r.end_cut, vec![3, 6]);
        assert!(r.fully_consumed);
        assert!((r.consumed_height - 96.0).abs() < 0.5);
    }

    fn rscell(row: u16, col: u16, row_span: u16, paragraphs: Vec<Paragraph>) -> Cell {
        Cell {
            row,
            col,
            row_span,
            col_span: 1,
            width: 10000,
            paragraphs,
            ..Default::default()
        }
    }

    /// [Task #1025] 단일 비-rowspan 행에서 advance_row_block_cut == advance_row_cut (회귀 0).
    #[test]
    fn test_block_cut_single_row_parity() {
        let eng = LayoutEngine::new(96.0);
        let styles = ResolvedStyleSet::default();
        let t = table(vec![
            cell(0, 0, vec![text_para(3, 0)]),
            cell(0, 1, vec![text_para(6, 0)]),
        ]);
        for avail in [50.0, 96.0, 500.0, 5.0] {
            let a = eng.advance_row_cut(&t, 0, &[], avail, &styles);
            let b = eng.advance_row_block_cut(&t, 0, 1, &[], avail, &styles);
            assert_eq!(a.end_cut, b.end_cut, "avail={avail}");
            assert_eq!(a.fully_consumed, b.fully_consumed, "avail={avail}");
            assert_eq!(a.hit_hard_break, b.hit_hard_break, "avail={avail}");
            assert!(
                (a.consumed_height - b.consumed_height).abs() < 0.5,
                "avail={avail}"
            );
        }
    }

    /// [Task #1025] rowspan 블록(rows 0-1)에서 거대 row_span==1 셀이 줄 단위로 분할.
    /// cell[label] r=0 rs=2(2줄), cell[a] r=0(2줄), cell[big] r=1(10줄).
    /// avail=80px(=5줄): 첫 조각은 라벨2 + a2 + big5 까지, big 잔여 5줄은 다음 조각.
    #[test]
    fn test_block_cut_rowspan_giant_split() {
        let eng = LayoutEngine::new(96.0);
        let styles = ResolvedStyleSet::default();
        let t = table(vec![
            rscell(0, 0, 2, vec![text_para(2, 0)]), // 라벨 (rows 0-1 걸침)
            cell(0, 1, vec![text_para(2, 0)]),      // row 0 일반 셀
            cell(1, 1, vec![text_para(10, 0)]),     // row 1 거대 셀 (10줄=160px)
        ]);
        // 셀 순서 (row,col): [ (0,0)라벨, (0,1)a, (1,1)big ]
        let first = eng.advance_row_block_cut(&t, 0, 2, &[], 80.0, &styles);
        // 라벨 2줄 전량, a 2줄 전량, big 5줄(80px) 까지.
        assert_eq!(first.end_cut, vec![2, 2, 5], "first: {:?}", first.end_cut);
        assert!(!first.fully_consumed);
        // 연속 조각: 라벨/a 는 이미 전량(공란), big 잔여 5줄.
        let cont = eng.advance_row_block_cut(&t, 0, 2, &first.end_cut, 500.0, &styles);
        assert_eq!(cont.end_cut, vec![2, 2, 10], "cont: {:?}", cont.end_cut);
        assert!(cont.fully_consumed);
    }
}
