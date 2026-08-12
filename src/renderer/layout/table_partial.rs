//! 페이지 분할 표 레이아웃 (layout_partial_table)

use super::super::composer::{compose_paragraph, ComposedParagraph};
use super::super::float_placement::native_hwp5_stored_reset_fragment_paint_geometry;
use super::super::height_measurer::MeasuredTable;
use super::super::page_layout::LayoutRect;
use super::super::render_tree::*;
use super::super::style_resolver::ResolvedStyleSet;
use super::super::{hwpunit_to_px, ShapeStyle};
use super::border_rendering::{
    build_row_col_x, collect_cell_borders, render_edge_borders, render_transparent_borders,
};
use super::table_layout::{
    calc_nested_split_rows, effective_margin_left_line, extend_completed_nested_table_border_clips,
    NestedTableSplit,
};
use super::text_measurement::{estimate_text_width, resolved_to_text_style};
use super::utils::find_bin_data;
use super::{
    repeats_native_empty_host_rowbreak_fragment_margin, CellContext, CellPathEntry, LayoutEngine,
};
use crate::model::bin_data::BinDataContent;
use crate::model::control::Control;
use crate::model::paragraph::Paragraph;
use crate::model::shape::CaptionDirection;
use crate::model::style::{Alignment, BorderLine};

/// `layout_partial_table_resolved`가 표 자체와 분리해 사용하는 host 문맥.
///
/// 일반 페이지 item은 원본 문단/control에서 이 값을 만들고, 재귀 child cursor는
/// 임시 `Table::clone()` 없이 원본 중첩 표와 synthetic-equivalent 기본값을 넘긴다.
#[derive(Clone, Copy)]
struct PartialTableHostContext<'a> {
    paragraphs: &'a [Paragraph],
    para_index: usize,
    control_index: usize,
    repeat_fragment_outer_margin: bool,
    pre_emitted_host_height: f64,
    host_line_spacing: f64,
}

/// Returns the content table inside transparent, empty 1×1 wrapper tables.
///
/// HWPX can retain a shell table solely as the host for a nested table.  The
/// shell's one empty paragraph carries no independently visible content; its
/// row geometry is therefore the nested table's geometry.  Keep this narrow:
/// any text, an additional paragraph, or a non-1×1 grid makes the outer table
/// semantically observable and stops unwrapping.
fn transparent_nested_table(table: &crate::model::table::Table) -> &crate::model::table::Table {
    if table.row_count != 1 || table.col_count != 1 || table.cells.len() != 1 {
        return table;
    }

    let cell = &table.cells[0];
    if cell.paragraphs.len() != 1 {
        return table;
    }
    let para = &cell.paragraphs[0];
    if para
        .text
        .chars()
        .any(|ch| !ch.is_whitespace() && ch != '\r' && ch != '\n')
    {
        return table;
    }
    let Some(nested) = para.controls.iter().find_map(|control| match control {
        Control::Table(table) => Some(table.as_ref()),
        _ => None,
    }) else {
        return table;
    };

    transparent_nested_table(nested)
}

/// 분할 셀 조각에서 실제로 보이는 첫 줄의 저장 vpos를 찾는다.
///
/// `cell_line_ranges_from_cut`은 문단 중간 줄에서 시작할 수 있다. 문단 첫 줄을
/// 쓰면 이미 앞 조각에서 소비한 줄 높이를 다시 더해 다음 문단 스냅이 밀린다.
fn fragment_vpos_origin(
    cell: &crate::model::table::Cell,
    line_ranges: Option<&[(usize, usize)]>,
) -> i32 {
    line_ranges
        .and_then(|ranges| {
            ranges
                .iter()
                .position(|&(start, end)| start < end)
                .and_then(|para_idx| {
                    let (start_line, _) = ranges[para_idx];
                    cell.paragraphs.get(para_idx).and_then(|para| {
                        para.line_segs
                            .get(start_line)
                            // recompose된 문단처럼 저장 LINE_SEG가 줄 수보다 적으면
                            // 기존의 보수적 첫 세그먼트 폴백을 유지한다.
                            .or_else(|| para.line_segs.first())
                            .map(|seg| seg.vertical_pos)
                    })
                })
        })
        .unwrap_or(0)
        .max(0)
}

/// 셀의 실제 텍스트 하단 경계.
///
/// `text_y_start`에는 세로 정렬 offset이 포함되므로 여기에 전체 `cell_h`를 더하면
/// Center/Bottom 셀에서 물리 셀 하단을 넘는다.
fn cell_content_bottom(cell_y: f64, cell_h: f64, pad_bottom: f64) -> f64 {
    cell_y + cell_h - pad_bottom
}

/// [#4159] 종료 분할 셀의 clip이 재귀 중첩 표 전체 stroke를 포섭하도록 확장한다.
///
/// 재귀 표는 셀의 `inner_y`(top padding 뒤)에서 시작하지만 바깥 셀과 같은 fragment
/// 높이를 사용할 수 있다. 이때 중첩 표의 마지막 border만 padding만큼 셀 bbox 밖으로
/// 내려간다. 이어질 유닛이 없는 terminal 조각에서는 그 stroke도 현재 쪽 소속이므로
/// clip에 포함한다. 비종료 조각은 다음 쪽 콘텐츠 노출을 막기 위해 절대 확장하지 않는다.
fn expand_terminal_cell_clip_to_nested_table_descendants(
    cell_node: &mut RenderNode,
    terminal: bool,
) {
    if !terminal || !matches!(&cell_node.node_type, RenderNodeType::TableCell(cell) if cell.clip) {
        return;
    }

    fn subtree_bottom(node: &RenderNode) -> f64 {
        node.children
            .iter()
            .fold(node.bbox.y + node.bbox.height, |bottom, child| {
                bottom.max(subtree_bottom(child))
            })
    }

    fn nested_table_bottom(node: &RenderNode) -> Option<f64> {
        node.children.iter().fold(None, |bottom, child| {
            let candidate = if matches!(child.node_type, RenderNodeType::Table(_)) {
                Some(subtree_bottom(child))
            } else {
                nested_table_bottom(child)
            };
            match (bottom, candidate) {
                (Some(a), Some(b)) => Some(a.max(b)),
                (Some(a), None) => Some(a),
                (None, Some(b)) => Some(b),
                (None, None) => None,
            }
        })
    }

    let Some(content_bottom) = nested_table_bottom(cell_node) else {
        return;
    };
    let grown_height = content_bottom - cell_node.bbox.y;
    if grown_height > cell_node.bbox.height {
        cell_node.bbox.height = grown_height;
    }
}

/// 명시적 recursive RowCut으로 source-bounded 렌더를 끝낸 직계 child만 현재
/// clipped cell viewport에 포섭한다.
///
/// 일반 nonterminal cell을 확장하면 다음 쪽 scalar tail이 노출된다. 반면
/// `recursive_cut` 경로는 현재 쪽의 source start/end를 이미 제한하므로, 해당 호출이
/// 새로 추가한 table root bbox는 안전하게 현재 조각 소유로 볼 수 있다. child 내부를
/// 다시 순회하지 않는 이유는 그 안의 별도 clipped scalar tail을 섞지 않기 위해서다.
fn expand_cell_clip_to_new_source_bounded_children(
    cell_node: &mut RenderNode,
    first_new_child: usize,
) {
    if !matches!(&cell_node.node_type, RenderNodeType::TableCell(cell) if cell.clip) {
        return;
    }

    let current_bottom = cell_node.bbox.y + cell_node.bbox.height;
    let content_bottom = cell_node.children[first_new_child.min(cell_node.children.len())..]
        .iter()
        .filter(|child| child.visible)
        .map(|child| child.bbox.y + child.bbox.height)
        .fold(current_bottom, f64::max);
    let grown_height = content_bottom - cell_node.bbox.y;
    if grown_height > cell_node.bbox.height {
        cell_node.bbox.height = grown_height;
    }
}

// 표 수평 정렬 보조 타입은 table_layout.rs에 통합됨

/// [#4149] 셀 커서 fast path 프로브 — 부분 표 조각 레이아웃을 대상 셀 하나로 제한한다.
///
/// 계약: 방출되는 노드의 좌표는 전량 레이아웃과 완전 동일해야 한다 — 프로브는
/// "생략"만 하고 "변형"은 하지 않는다 (셀 방출 루프는 셀-간 캐리가 없음이 근거).
pub(crate) struct PartialTableCellProbe {
    /// 대상 셀 인덱스 — 이 셀만 방출한다.
    pub(crate) cell_idx: usize,
    /// 이 문단까지 방출 후 중단한다 (캐럿 문단 — 이후 문단은 캐럿 탐색에 불필요하고
    /// 앞 문단의 y 에도 영향이 없다).
    pub(crate) stop_after_para: usize,
    /// true 면 컷 창 문단(`window_paras`)만 순회·compose 한다. 사전 게이트가
    /// (a) 컷 존재, (b) shrink 조기탈출(다중줄 문단 존재), (c) effective Top 정렬을
    /// 증명한 경우에만 true 가 된다.
    pub(crate) windowed: bool,
    /// `windowed` 일 때 컷 창에 유닛이 있는 문단 범위 [lo, hi] (inclusive).
    /// 범위 밖 문단은 컷 창에 유닛이 없어 전량 레이아웃에서도 skip 됨이 증명된다.
    pub(crate) window_paras: (usize, usize),
}

/// [#4149] 셀 문단 compose 저장소 — windowed 프로브에서만 lazy.
///
/// Lazy 슬롯의 compose 결과는 Eager 경로와 동일한 변환 순서
/// (compose → recompose_for_cell_width → recompose_stored_single_line_if_overflowing)를
/// 문단 단위로 적용한다. windowed 프로브 게이트가 shrink 를 조기탈출로 증명하므로
/// Lazy 에서 inner_width 는 Eager 와 동일하다.
enum CellComposedStore {
    Eager(Vec<ComposedParagraph>),
    Lazy(Vec<Option<ComposedParagraph>>),
}

impl CellComposedStore {
    fn get(
        &mut self,
        cpi: usize,
        cell: &crate::model::table::Cell,
        inner_width: f64,
        styles: &ResolvedStyleSet,
    ) -> &ComposedParagraph {
        match self {
            CellComposedStore::Eager(v) => &v[cpi],
            CellComposedStore::Lazy(slots) => {
                if slots[cpi].is_none() {
                    let para = &cell.paragraphs[cpi];
                    let mut comp = compose_paragraph(para);
                    crate::renderer::composer::recompose_for_cell_width(
                        &mut comp,
                        para,
                        inner_width,
                        styles,
                    );
                    if cell.text_direction == 0 {
                        crate::renderer::composer::recompose_stored_single_line_if_overflowing(
                            &mut comp,
                            para,
                            inner_width,
                            styles,
                        );
                    }
                    slots[cpi] = Some(comp);
                }
                slots[cpi].as_ref().unwrap()
            }
        }
    }

    /// 전량 슬라이스가 필요한 경로(세로쓰기 등)를 위한 완전 구성.
    fn materialize(
        &mut self,
        cell: &crate::model::table::Cell,
        inner_width: f64,
        styles: &ResolvedStyleSet,
    ) {
        if matches!(self, CellComposedStore::Lazy(_)) {
            let mut v = Vec::with_capacity(cell.paragraphs.len());
            for cpi in 0..cell.paragraphs.len() {
                v.push(self.get(cpi, cell, inner_width, styles).clone());
            }
            *self = CellComposedStore::Eager(v);
        }
    }

    /// Eager 전제 슬라이스 접근 — windowed 프로브 게이트 밖에서만 호출된다.
    fn eager_slice(&self) -> &[ComposedParagraph] {
        match self {
            CellComposedStore::Eager(v) => v,
            CellComposedStore::Lazy(_) => {
                unreachable!("windowed 프로브에서 전량 compose 접근 금지")
            }
        }
    }
}

/// [#4149] 프로브 사전 계획 결과.
pub(crate) enum ProbeCutPlan {
    /// 프로브로 다룰 수 없는 조합 (rowspan 셀, 빈 창 등) — legacy 폴백.
    Unsupported,
    /// 이 조각에서 셀이 컷되지 않음 (전체 가시). 소형 셀 한정 전량 프로브 가능.
    Uncut,
    /// 컷 존재. `window_paras` = 창에 유닛이 있는 문단 범위 [lo, hi] (inclusive).
    Cut {
        window_paras: (usize, usize),
        /// 전량 레이아웃의 `cell_was_split` 이 true 임이 증명됨 (s>0 문단 존재
        /// 또는 창 밖 미가시 문단의 compose 줄 수 ≥ 1).
        split_proven: bool,
        /// 대상 문단이 창 안에 있는가 — 밖이면 이 페이지에 대상 문단 run 이 없다.
        target_in_window: bool,
    },
}

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

/// [#4128 추출] 행내 `row_span==1` 셀의 col 오름차순 컷 벡터 서수.
/// `advance_row_cut` 부기와 동일한 순서 (기존 인라인 식의 명명).
fn single_row_cut_index(
    table: &crate::model::table::Table,
    cell: &crate::model::table::Cell,
) -> usize {
    table
        .cells
        .iter()
        .filter(|c| c.row_span == 1 && c.row == cell.row && c.col < cell.col)
        .count()
}

/// [#4128 추출] 이 페이지 조각에서 `cell` 의 가시 유닛 창 `[su, eu)`.
/// `apply_start`/`apply_end` 는 셀이 분할 시작/끝 행(블록)에 걸렸는지 여부
/// (`is_split_start_row`/`is_split_end_row` 판정 결과). `units_len` 이 있으면
/// 그 범위로 클램프하고, 없으면 `usize::MAX` 센티널을 유지한다.
#[allow(clippy::too_many_arguments)]
fn cell_cut_window(
    table: &crate::model::table::Table,
    cell: &crate::model::table::Cell,
    is_block_split: bool,
    apply_start: bool,
    start_block: Option<(usize, usize)>,
    apply_end: bool,
    end_block: Option<(usize, usize)>,
    start_cut: &[usize],
    end_cut: &[usize],
    units_len: Option<usize>,
) -> (usize, usize) {
    let (su, eu) = if is_block_split {
        let su = match (apply_start, start_block) {
            (true, Some((bs, be))) => block_cut_index(table, bs, be, cell)
                .and_then(|i| start_cut.get(i).copied())
                .unwrap_or(0),
            _ => 0,
        };
        let eu = match (apply_end, end_block) {
            (true, Some((bs, be))) => block_cut_index(table, bs, be, cell)
                .and_then(|i| end_cut.get(i).copied())
                .unwrap_or(usize::MAX),
            _ => usize::MAX,
        };
        (su, eu)
    } else {
        let cut_idx = single_row_cut_index(table, cell);
        let su = if apply_start {
            start_cut.get(cut_idx).copied().unwrap_or(0)
        } else {
            0
        };
        let eu = if apply_end {
            end_cut.get(cut_idx).copied().unwrap_or(usize::MAX)
        } else {
            usize::MAX
        };
        (su, eu)
    };
    match units_len {
        Some(len) => {
            let su = su.min(len);
            (su, eu.clamp(su, len))
        }
        None => (su, eu),
    }
}

impl LayoutEngine {
    /// [#4128] 이 PartialTable 페이지 조각에 `cell` 의 대상 위치가 실제로 렌더되는가.
    /// pagination 메타데이터(행 범위 + 유닛 컷)와 memoize 된 `cell_units` 만 사용하며
    /// render tree 를 짓지 않는다. 분할 게이트 판정은 `layout_partial_table_cells`
    /// 의 셀 방출 판정과 동일 산식 — 드리프트 금지.
    ///
    /// `target`: `(cell_para_idx, target_line, at_line_start)`. `None` 은 셀 전체 질의
    /// (행/블록 겹침만 판정). 컷 경계(`ord == eu`)는 대상 offset 이 정확히 줄 시작일
    /// 때만 포함한다 — legacy 오름차순 스캔의 inclusive run 매치
    /// (`offset <= char_start + count`)가 이전 조각을 먼저 돌려주는 동작과 결과
    /// 페이지가 일치해야 하기 때문.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn partial_table_page_contains_cell_position(
        &self,
        table: &crate::model::table::Table,
        cell: &crate::model::table::Cell,
        start_row: usize,
        end_row: usize,
        start_cut: &[usize],
        end_cut: &[usize],
        is_block_split: bool,
        target: Option<(usize, usize, bool)>,
        styles: &ResolvedStyleSet,
    ) -> bool {
        let cell_row = cell.row as usize;
        let cell_end_row = cell_row + (cell.row_span as usize).max(1);
        // 행/블록 겹침 없음 → 이 조각에 셀 없음. 반복 헤더 사본은 원본 행 페이지가
        // legacy 첫-히트와 같으므로 후보에 넣지 않는다.
        if cell_row >= end_row || cell_end_row <= start_row {
            return false;
        }
        let Some((cell_para_idx, target_line, at_line_start)) = target else {
            return true;
        };
        // 분할 게이트 — layout_partial_table_cells 와 동일 판정
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
        if !(is_split_start_row || is_split_end_row) {
            return true; // 이 조각에서 셀이 컷되지 않음 → 전체 가시
        }
        // 비블록 컷 모델은 row_span==1 셀만 부기 — rowspan 걸침 셀은 보수적 포함
        // (straddle 높이 컷 경로는 페이지 후보를 좁힐 권위가 아니다).
        if !is_block_split && cell.row_span > 1 {
            return true;
        }
        let Some(ord) = self.cell_unit_ordinal_for(cell, table, styles, cell_para_idx, target_line)
        else {
            return true; // 유닛 매핑 실패(빈 셀 등) → 보수적 포함
        };
        let units_len = self.cell_units(cell, table, styles).len();
        let (su, eu) = cell_cut_window(
            table,
            cell,
            is_block_split,
            is_split_start_row,
            split_start_block,
            is_split_end_row,
            split_end_block,
            start_cut,
            end_cut,
            Some(units_len),
        );
        ord >= su && (ord < eu || (ord == eu && at_line_start))
    }

    /// [#4149] 커서 fast path 의 프로브 사전 계획 — 이 PartialTable 조각에서 `cell` 의
    /// 컷 창과 창 문단 범위를 계산한다. 분할 게이트 판정은
    /// `layout_partial_table_cells` 의 셀 방출 판정과 동일 산식 — 드리프트 금지.
    /// memoize 된 `cell_units` 만 사용하며 render tree 를 짓지 않는다.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn partial_table_cell_probe_plan(
        &self,
        table: &crate::model::table::Table,
        cell: &crate::model::table::Cell,
        start_row: usize,
        end_row: usize,
        start_cut: &[usize],
        end_cut: &[usize],
        is_block_split: bool,
        styles: &ResolvedStyleSet,
        target_para: usize,
    ) -> ProbeCutPlan {
        // rowspan 셀은 straddle 높이-컷 경로(is_rowbreak_straddle)가 얽혀 보수적 폴백.
        if cell.row_span > 1 {
            return ProbeCutPlan::Unsupported;
        }
        let cell_row = cell.row as usize;
        let cell_end_row = cell_row + (cell.row_span as usize).max(1);
        // 분할 게이트 — layout_partial_table_cells 와 동일 판정
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
        if !(is_split_start_row || is_split_end_row) {
            // row_span==1 이므로 is_rowbreak_straddle(rowspan>1 전제) 도달 불가 — 전체 가시.
            return ProbeCutPlan::Uncut;
        }
        let (su, eu) = cell_cut_window(
            table,
            cell,
            is_block_split,
            is_split_start_row,
            split_start_block,
            is_split_end_row,
            split_end_block,
            start_cut,
            end_cut,
            None,
        );
        let units = self.cell_units(cell, table, styles);
        let lo = su.min(units.len());
        let hi = eu.min(units.len()).max(lo);
        if lo >= hi {
            return ProbeCutPlan::Unsupported;
        }
        let mut pmin = usize::MAX;
        let mut pmax = 0usize;
        for u in &units[lo..hi] {
            pmin = pmin.min(u.para_idx);
            pmax = pmax.max(u.para_idx);
        }
        if pmin == usize::MAX || pmax >= cell.paragraphs.len() {
            return ProbeCutPlan::Unsupported;
        }
        // cell_was_split 증명 — 전량 레이아웃의 `s != 0 || e != total` 판정과 동치인
        // 충분조건만 취한다:
        //   (a) 어떤 문단의 가시 시작줄 s > 0, 또는
        //   (b) 창 밖 미가시 문단((0,0))의 compose 줄 수 ≥ 1 (recompose 는 줄을
        //       늘리기만 하므로 pre-recompose ≥ 1 ⇒ post ≥ 1 > 0 = e).
        let ranges = self.cell_line_ranges_from_cut(cell, table, styles, su, eu);
        let mut split_proven = ranges.iter().any(|&(s, _)| s > 0);
        if !split_proven {
            for (i, &(s, e)) in ranges.iter().enumerate() {
                if s == 0 && e == 0 && (i < pmin || i > pmax) {
                    if compose_paragraph(&cell.paragraphs[i]).lines.is_empty() {
                        continue;
                    }
                    split_proven = true;
                    break;
                }
            }
        }
        ProbeCutPlan::Cut {
            window_paras: (pmin, pmax),
            split_proven,
            target_in_window: (pmin..=pmax).contains(&target_para),
        }
    }

    /// [#4149] 표 서브트리(호스트 표 + 중첩 표/글상자/캡션)가 프로브를 차단하는
    /// 상태 의존 요소를 갖는지 — 표 포인터 키 메모.
    ///
    /// 차단 근거: (a) 개요/번호 문단은 auto counter 를 소비하는데 프로브는 이전
    /// 페이지 replay 없이 셀 하나만 레이아웃하므로 번호가 어긋난다.
    /// (b) AutoNumber/NewNumber 컨트롤·캡션 자동번호도 동일. (c) 묶음 개체는 내부
    /// 글상자 유무를 싸게 판정할 수 없어 보수적으로 차단.
    pub(crate) fn table_subtree_blocks_cursor_probe(
        &self,
        table: &crate::model::table::Table,
        styles: &ResolvedStyleSet,
    ) -> bool {
        fn paras_block(paras: &[Paragraph], styles: &ResolvedStyleSet) -> bool {
            use crate::model::style::HeadType;
            for p in paras {
                let head = styles
                    .para_styles
                    .get(p.para_shape_id as usize)
                    .map(|s| s.head_type)
                    .unwrap_or(HeadType::None);
                if matches!(head, HeadType::Outline | HeadType::Number) {
                    return true;
                }
                for c in &p.controls {
                    match c {
                        Control::AutoNumber(_) | Control::NewNumber(_) => return true,
                        Control::Table(t) => {
                            // 캡션은 존재 자체가 아니라 auto counter 소비 요소
                            // (개요/번호 헤드·AutoNumber)를 가질 때만 차단한다 —
                            // layout_caption 은 apply_auto_numbers_to_composed 로만
                            // counter 를 만진다.
                            if let Some(cap) = t.caption.as_ref() {
                                if paras_block(&cap.paragraphs, styles) {
                                    return true;
                                }
                            }
                            for cell in &t.cells {
                                if paras_block(&cell.paragraphs, styles) {
                                    return true;
                                }
                            }
                        }
                        Control::Shape(s) => {
                            use crate::model::shape::ShapeObject;
                            let drawing = match &**s {
                                ShapeObject::Rectangle(sh) => Some(&sh.drawing),
                                ShapeObject::Ellipse(sh) => Some(&sh.drawing),
                                ShapeObject::Polygon(sh) => Some(&sh.drawing),
                                ShapeObject::Curve(sh) => Some(&sh.drawing),
                                ShapeObject::Line(_) | ShapeObject::Arc(_) => None,
                                // Group/Chart/Ole 등은 내부 글상자·캡션을 싸게 확인할 수
                                // 없어 보수적으로 차단한다.
                                _ => return true,
                            };
                            if let Some(d) = drawing {
                                if let Some(cap) = d.caption.as_ref() {
                                    if paras_block(&cap.paragraphs, styles) {
                                        return true;
                                    }
                                }
                                if let Some(tb) = d.text_box.as_ref() {
                                    if paras_block(&tb.paragraphs, styles) {
                                        return true;
                                    }
                                }
                            }
                        }
                        Control::Picture(pic) => {
                            if let Some(cap) = pic.caption.as_ref() {
                                if paras_block(&cap.paragraphs, styles) {
                                    return true;
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            false
        }
        let key = table as *const crate::model::table::Table as usize;
        if let Some(&cached) = self.cursor_probe_block_cache.borrow().get(&key) {
            return cached;
        }
        let blocked = table
            .cells
            .iter()
            .any(|c| paras_block(&c.paragraphs, styles));
        self.cursor_probe_block_cache
            .borrow_mut()
            .insert(key, blocked);
        blocked
    }

    /// [#2029 추출] 부분 표의 셀 방출 루프 — 셀 geometry/배경/반복 헤더와 셀
    /// 문단 배치를 `table_node.children` 에 방출한다. 셀-간 캐리 없음(실측 muts 0,
    /// 외부 sink = table_node 단일) — 원본 무변경 이동.
    #[allow(clippy::too_many_arguments)]
    fn layout_partial_table_cells(
        &self,
        tree: &mut LayoutFrame,
        table_node: &mut RenderNode,
        table: &crate::model::table::Table,
        para_index: usize,
        control_index: usize,
        section_index: usize,
        styles: &ResolvedStyleSet,
        outline_numbering_id: u16,
        bin_data_content: &[BinDataContent],
        start_row: usize,
        end_row: usize,
        end_row_height_override: Option<f64>,
        is_continuation: bool,
        start_cut: &[usize],
        end_cut: &[usize],
        is_block_split: bool,
        cell_spacing: f64,
        col_count: usize,
        row_count: usize,
        table_x: f64,
        table_y: f64,
        row_heights: &[f64],
        resolved_row_heights: &[f64],
        row_col_x: &[Vec<f64>],
        header_rows: &[usize],
        render_rows: &[usize],
        render_row_y: &[f64],
        h_edges: &mut Vec<Vec<Option<BorderLine>>>,
        v_edges: &mut Vec<Vec<Option<BorderLine>>>,
        measured_table: Option<&MeasuredTable>,
        enclosing_cell_ctx: Option<&CellContext>,
        clamp_header_negative_para_offset: bool,
        probe: Option<&PartialTableCellProbe>,
    ) {
        for (cell_idx, cell) in table.cells.iter().enumerate() {
            // [#4149] 프로브: 대상 셀만 방출. 셀 방출 루프는 셀-간 캐리가 없어
            // (실측 muts 0, 외부 sink = table_node 단일) 생략이 대상 셀 좌표를 바꾸지 않는다.
            if probe.is_some_and(|p| p.cell_idx != cell_idx) {
                continue;
            }
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

            // [Task #1748] RowBreak 표에서 페이지 경계가 rowspan 블록 내부를 per-row
            // 분할할 때(#1022 경로), 컷 부기(start_cut/end_cut)는 컷 행의 row_span==1
            // 셀만 담아 경계에 걸친 rowspan 셀은 컷 없이 전체 렌더된다 — 컷 페이지에선
            // 셀 박스 아래로 흘러넘치고(+13px 잉크), 연속 페이지에선 처음부터
            // 재렌더(중복)된다. 높이 기반 유닛 컷으로 가시 줄 범위를 제한한다.
            let straddles_fragment_start = cell_row < start_row && cell_end_row > start_row;
            let straddles_fragment_end = cell_row < render_range_end
                && (cell_end_row > render_range_end
                    || (cell_end_row == render_range_end && !end_cut.is_empty()));
            let is_rowbreak_straddle = !is_block_split
                && !is_in_split_row
                && cell.row_span > 1
                && !is_repeated_header_cell
                && matches!(
                    table.page_break,
                    crate::model::table::TablePageBreak::RowBreak
                )
                && (straddles_fragment_start || straddles_fragment_end);
            // HWP5 저장 pagination 계약의 정확한 2행 rowspan/2문단 형상은 문단 하나가
            // 행 하나의 저장 owner다. 여기서 일반 높이 컷을 적용하면 첫 문단의 trailing
            // line/문단 간격이 첫 행보다 커져 양쪽 문단이 continuation에 재방출될 수
            // 있다(76076 p18→p19). HWP5-origin HWPX에도 같은 source owner를 보존하고,
            // 순수 HWPX·컷·중첩·다중줄 일반 rowspan에는 적용하지 않는다.
            let native_two_row_paragraph_owner_boundary = is_rowbreak_straddle
                && start_cut.is_empty()
                && end_cut.is_empty()
                && ((straddles_fragment_start && start_row == cell_row + 1)
                    || (straddles_fragment_end && render_range_end == cell_row + 1))
                && self.native_two_row_rowspan_paragraph_owner_boundary(cell, table, styles);

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
                    clip: is_in_split_row || is_rowbreak_straddle,
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

            // [#4149] windowed 프로브면 창 문단만 lazy compose. 그 외에는 종전과
            // 동일한 순서로 전량 compose → shrink → recompose.
            let probe_windowed = probe.is_some_and(|p| p.windowed);
            let mut composed_store: CellComposedStore;
            if probe_windowed {
                // shrink 생략 근거: 프로브 사전 게이트가 line_segs>=2 문단 존재를
                // 증명했고, shrunk_cell_horizontal_padding 은 그 경우 composed 를
                // 읽지 않고 패딩을 그대로 반환한다 (조기 탈출과 동일 결과).
                composed_store = CellComposedStore::Lazy(vec![None; cell.paragraphs.len()]);
            } else {
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

                let inner_width_for_recompose = (cell_w - pad_left - pad_right).max(0.0);
                // [Task #671] line_segs 비어 있는 셀 paragraph 의 단일 ComposedLine 압축
                // 결과를 셀 가용 너비 (inner_width) 에 맞춰 다중 ComposedLine 으로 재분할.
                for (cpi, para) in cell.paragraphs.iter().enumerate() {
                    if let Some(comp) = composed_paras.get_mut(cpi) {
                        crate::renderer::composer::recompose_for_cell_width(
                            comp,
                            para,
                            inner_width_for_recompose,
                            styles,
                        );
                        // [#2291] 부실 저장(ls==1·실폭 초과) 재분할 — 가로쓰기 셀 한정.
                        if cell.text_direction == 0 {
                            crate::renderer::composer::recompose_stored_single_line_if_overflowing(
                                comp,
                                para,
                                inner_width_for_recompose,
                                styles,
                            );
                        }
                    }
                }
                composed_store = CellComposedStore::Eager(composed_paras);
            }

            let inner_x = cell_x + pad_left;
            let inner_width = (cell_w - pad_left - pad_right).max(0.0);
            let inner_height = (cell_h - pad_top - pad_bottom).max(0.0);

            // 분할 행: [Task #993/#1025] start_cut/end_cut(유닛 컷)으로 표시할 줄 범위 계산.
            // 블록 분할이면 블록-셀 (row,col) 인덱스, 그 외는 행내 row_span==1 col 인덱스.
            let cut_units: Option<(usize, usize)> = if is_in_split_row {
                Some(cell_cut_window(
                    table,
                    cell,
                    is_block_split,
                    is_split_start_row,
                    split_start_block,
                    is_split_end_row,
                    split_end_block,
                    start_cut,
                    end_cut,
                    None,
                ))
            } else if native_two_row_paragraph_owner_boundary {
                let su = usize::from(straddles_fragment_start);
                let eu = if straddles_fragment_end {
                    1
                } else {
                    usize::MAX
                };
                Some((su, eu))
            } else if is_rowbreak_straddle {
                // [Task #1748] 높이 기반 유닛 컷. 이전 프래그먼트 소비 높이(prior_h)는
                // 2b 오버라이드와 동일한 식으로 재계산 — 온전 행은 컷 측정
                // (row_cut_content_height), 분할 행(start_row)은 start_cut 이전 유닛
                // 높이. 컷 페이지가 end_cut 으로 계산한 값과 같은 식이라 경계 유닛
                // 인덱스(컷 페이지 eu == 연속 페이지 su)가 산술적으로 일치한다.
                let mut prior_h = 0.0f64;
                if straddles_fragment_start {
                    for r in cell_row..start_row {
                        let has_single_row_cells = table
                            .cells
                            .iter()
                            .any(|c| c.row as usize == r && c.row_span == 1);
                        let h = if has_single_row_cells {
                            let h = self.row_cut_content_height(table, r, &[], &[], styles);
                            if h > 0.0 {
                                h
                            } else {
                                resolved_row_heights.get(r).copied().unwrap_or(0.0)
                            }
                        } else {
                            resolved_row_heights.get(r).copied().unwrap_or(0.0)
                        };
                        prior_h += h + cell_spacing;
                    }
                    if !start_cut.is_empty() {
                        prior_h +=
                            self.row_cut_content_height(table, start_row, &[], start_cut, styles);
                    }
                }
                let su = if prior_h > 0.0 {
                    self.cell_units_fitting_height(cell, table, styles, prior_h - pad_top)
                } else {
                    0
                };
                let eu = if straddles_fragment_end {
                    self.cell_units_fitting_height(cell, table, styles, prior_h + cell_h - pad_top)
                        .max(su)
                } else {
                    usize::MAX
                };
                Some((su, eu))
            } else {
                None
            };
            let line_ranges: Option<Vec<(usize, usize)>> = cut_units
                .map(|(su, eu)| self.cell_line_ranges_from_cut(cell, table, styles, su, eu));
            // 셀 내 텍스트 높이 (분할 행이면 줄 범위 내만 계산)
            // spacing_before: 셀 첫 문단 제외, spacing_after: 셀 마지막 문단 제외
            let split_para_count = cell.paragraphs.len();
            // [#4149] windowed 프로브: 아래에서 effective_align 이 Top 으로 확정되므로
            // (cell_was_split=true 사전 증명) total_content_height 는 미사용 — 0 고정.
            let total_content_height = if probe_windowed {
                0.0
            } else if let Some(ref ranges) = line_ranges {
                let mut total = 0.0;
                for (pi, ((comp, para), &(start, end))) in composed_store
                    .eager_slice()
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
                    if start < end {
                        total +=
                            self.paragraph_cell_non_inline_controls_flow_height(&para.controls);
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
                    let unit_content_h = self.cell_units_content_height(cell, table, styles);
                    let last_seg_end: i32 = cell
                        .paragraphs
                        .iter()
                        .flat_map(|p| p.line_segs.last())
                        .map(|s| s.vertical_pos + s.line_height)
                        .max()
                        .unwrap_or(0);
                    let vpos_h = hwpunit_to_px(last_seg_end, self.dpi);
                    let line_h = self.calc_composed_paras_content_height(
                        composed_store.eager_slice(),
                        &cell.paragraphs,
                        styles,
                    );
                    let nested_bottom =
                        self.calc_nested_controls_bottom_height(&cell.paragraphs, styles);
                    vpos_h
                        .max(line_h)
                        .max(nested_bottom)
                        .max(unit_content_h)
                        .max(self.calc_non_inline_controls_flow_height(&cell.paragraphs))
                        .max(self.calc_cell_wrap_objects_bottom_height(&cell.paragraphs))
                } else {
                    self.calc_composed_paras_content_height(
                        composed_store.eager_slice(),
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
            let cell_was_split = if probe_windowed {
                // [#4149] windowed 프로브 사전 게이트가 증명한 값 (s>0 문단 존재 또는
                // 창 밖 미가시 문단의 compose 줄 수 ≥ 1) — 전량 판정과 동치.
                true
            } else {
                cut_units.is_some_and(|(start_unit, end_unit)| {
                    let unit_len = self.cell_units(cell, table, styles).len();
                    start_unit > 0 || end_unit < unit_len
                }) || line_ranges.as_ref().is_some_and(|ranges| {
                    ranges.iter().enumerate().any(|(i, &(s, e))| {
                        let total = composed_store
                            .eager_slice()
                            .get(i)
                            .map(|c| c.lines.len())
                            .unwrap_or(0);
                        s != 0 || e != total
                    })
                })
            };
            // [#4042] 쪽 경계로 실제 잘리는 셀은 세로 가운데/아래 정렬이 성립하지 않는다.
            // 한컴 조판 규칙: 용지 시작 y 에서 아래로 흐르며 쪽 경계에 닿으면 자른다 —
            // 가시 슬라이스(inner_height)보다 콘텐츠가 큰 셀은 위에서부터 흘러야 하며,
            // 가운데 정렬은 다음 쪽으로 넘어갈 콘텐츠까지 세어 가시 조각을 셀 중앙으로
            // 밀어버린다. `cell_was_split`(문단 줄 컷)은 문단 줄은 온전하되 중첩 표 유닛이
            // 슬라이스를 넘어 잘리는 셀(42065 Row2: 셀 안 두 표가 쪽을 넘김)을 놓친다.
            // 그래서 전체 콘텐츠 높이(중첩 표 포함, cell_units 캐시 재사용 → O(1))가
            // 가시 슬라이스를 넘는지 직접 본다. 넘으면 이 조각은 컷 대상이므로 Top 앵커.
            // 넘지 않는 셀(예: #697 세로 병합 라벨)은 종전대로 valign 을 존중한다.
            // 다중열(col_count>1) 중첩 표를 담은 셀만 대상. 단일 1×1 중첩 표는 그
            // fragment 를 셀 중앙에 두는 것이 한컴 정답(#2195/#4058 76076, issue_2308 핀)
            // 이고, cell_units_content_height 는 1×1 표의 full 높이를 세어 슬라이스를
            // 넘는 것처럼 오판(false positive)하므로 제외한다. 42065 Row2 처럼 여러
            // 다중열 표가 쪽을 넘겨 흐르는 셀만 Top 앵커가 필요하다.
            let has_multicol_nested = cell.paragraphs.iter().any(|p| {
                p.controls
                    .iter()
                    .any(|c| matches!(c, Control::Table(t) if t.col_count > 1))
            });
            let cell_content_cut_by_slice = has_multicol_nested
                && self.cell_units_content_height(cell, table, styles) > inner_height + 0.5;
            let effective_align = if (is_in_split_row || is_rowbreak_straddle)
                && (cell_was_split || cell_content_cut_by_slice)
            {
                VerticalAlign::Top
            } else {
                cell.vertical_align
            };
            // [#3820 Stage 78] RowBreak 마지막 physical tail은 paginator가 정한
            // 정확한 높이(override)를 사용한다. 저장 LINE_SEG가 glyph em보다 작은
            // 경우, 그 raw 줄높이만으로 Center를 계산하면 한 줄짜리 텍스트가 tail의
            // 지나치게 아래로 내려간다. 실제 paint glyph가 차지하는 em은 Center
            // 정렬의 최소 콘텐츠 높이여야 한다. 이 보정은 end-tail을 실제로 소유한
            // 셀에만 적용해 일반 저장 줄높이/행 높이 해석에는 영향을 주지 않는다.
            let owns_end_tail = end_row_height_override.is_some()
                && end_row.checked_sub(1).is_some_and(|last_row| {
                    cell_row <= last_row && last_row < cell_row + cell.row_span as usize
                });
            let centered_content_height =
                if effective_align == VerticalAlign::Center && owns_end_tail {
                    let visual_height = line_ranges.as_ref().map(|ranges| {
                        let mut total = 0.0;
                        let para_count = cell.paragraphs.len();
                        for (pi, ((comp, para), &(start, end))) in composed_store
                            .eager_slice()
                            .iter()
                            .zip(cell.paragraphs.iter())
                            .zip(ranges.iter())
                            .enumerate()
                        {
                            let para_style = styles.para_styles.get(para.para_shape_id as usize);
                            let is_last_para = pi + 1 == para_count;
                            if start == 0 && end > 0 && pi > 0 {
                                total += para_style.map(|s| s.spacing_before).unwrap_or(0.0);
                            }
                            for li in start..end.min(comp.lines.len()) {
                                let line = &comp.lines[li];
                                let raw_height = hwpunit_to_px(line.line_height, self.dpi);
                                let glyph_em = line
                                    .runs
                                    .iter()
                                    .filter_map(|run| {
                                        styles
                                            .char_styles
                                            .get(run.char_style_id as usize)
                                            .map(|style| style.font_size)
                                    })
                                    .fold(0.0f64, f64::max);
                                let line_height = raw_height.max(glyph_em);
                                let is_cell_last_line = is_last_para && li + 1 == comp.lines.len();
                                total += line_height;
                                if !is_cell_last_line {
                                    total += hwpunit_to_px(line.line_spacing, self.dpi);
                                }
                            }
                            if end == comp.lines.len() && end > start && !is_last_para {
                                total += para_style.map(|s| s.spacing_after).unwrap_or(0.0);
                            }
                        }
                        total
                    });
                    visual_height
                        .map(|height| total_content_height.max(height))
                        .unwrap_or(total_content_height)
                } else {
                    total_content_height
                };
            let text_y_start = match effective_align {
                VerticalAlign::Top => cell_y + pad_top,
                VerticalAlign::Center => {
                    cell_y + pad_top + (inner_height - centered_content_height).max(0.0) / 2.0
                }
                VerticalAlign::Bottom => {
                    cell_y + pad_top + (inner_height - total_content_height).max(0.0)
                }
            };

            // 세로쓰기 셀: 별도 레이아웃 경로 (가로 레이아웃 루프 대신)
            if cell.text_direction != 0 {
                // [#4149] 프로브가 세로쓰기 셀을 대상으로 삼는 일은 게이트로 막지만,
                // 방어적으로 전량 구성 후 동일 경로를 태운다 (좌표 동일).
                composed_store.materialize(cell, inner_width, styles);
                let vert_inner_area = LayoutRect {
                    x: inner_x,
                    y: cell_y + pad_top,
                    width: inner_width,
                    height: inner_height,
                };
                self.layout_vertical_cell_text(
                    tree,
                    &mut cell_node,
                    composed_store.eager_slice(),
                    &cell.paragraphs,
                    styles,
                    &vert_inner_area,
                    cell.vertical_align,
                    cell.text_direction,
                    section_index,
                    Some((para_index, control_index)),
                    cell_idx,
                    enclosing_cell_ctx.cloned(),
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
                            &mut *h_edges,
                            &mut *v_edges,
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
                    // block table 문단은 cut에 선택돼도 visible text line이 없으면
                    // `(n,n)`으로 남는다. `(0,0)`은 미선택 문단과 구분할 수 없지만
                    // `n>0`은 cell unit 원장이 이 control 문단을 현재 조각에 넣었다는
                    // 증거다. 이를 무시하면 바로 앞 텍스트 문단을 셀의 마지막 문단으로
                    // 오판해 trailing line_spacing을 버린다(issue2007 p14: 780HU).
                    let selected_zero_width_table_fragment = s == e
                        && s > 0
                        && cell.paragraphs.get(i).is_some_and(|para| {
                            // treat-as-char 표도 빈 host paragraph에서는 CellUnit의
                            // mixed nested fragment로 페이지를 나눠 실제 block처럼
                            // 배치된다(issue2007 p12/p15). source cut이 선택한 `(n,n)`
                            // table owner라는 계약이 중요하며 TAC 속성은 제외 근거가 아니다.
                            para.controls
                                .iter()
                                .any(|control| matches!(control, Control::Table(_)))
                        });
                    if s < e || selected_zero_width_table_fragment {
                        last_idx = i;
                    }
                }
                last_idx
            } else {
                cell.paragraphs.len().saturating_sub(1)
            };

            let mut para_y = text_y_start;
            let mut has_preceding_text = false;
            // [#3637] 이 조각에서 **실제로 그려지는 첫 문단**의 vpos. 아래 중첩 표
            // 문단 스냅이 쓰는 조각 원점이다. `line_segs.first()` 를 그대로 쓰면 셀
            // 전체 좌표라 연속 조각에서 원점만큼 통째로 밀린다.
            let frag_vpos_origin = fragment_vpos_origin(cell, line_ranges.as_deref());
            let preserve_linear_single_cell_vpos = cut_units.is_some_and(|(su, _)| su == 0)
                && matches!(
                    table.page_break,
                    crate::model::table::TablePageBreak::RowBreak
                )
                && !table.common.treat_as_char
                && table.row_count == 1
                && table.col_count == 1
                && (table.common.vertical_offset as i32) == 0;
            let vpos_origin = if preserve_linear_single_cell_vpos {
                cell.paragraphs
                    .first()
                    .and_then(|p| p.line_segs.first().map(|seg| seg.vertical_pos))
                    .unwrap_or(0)
                    .max(0)
            } else {
                0
            };
            // [#4149] windowed 프로브: 컷 창에 유닛이 없는 문단은 아래 skip 판정의
            // 네 조건(line_ranges·mixed·nested·non-inline)이 모두 창 유닛에서만
            // 유도되므로 전량 레이아웃에서도 반드시 skip 된다 — 순회 자체를 생략한다.
            // stop_after_para 이후 문단은 캐럿 문단의 좌표에 영향이 없어 중단한다.
            let (loop_start, loop_end_excl) = match probe {
                Some(p) if p.windowed => {
                    let (lo, hi) = p.window_paras;
                    let end = hi
                        .saturating_add(1)
                        .min(cell.paragraphs.len())
                        .min(p.stop_after_para.saturating_add(1));
                    (lo.min(end), end)
                }
                Some(p) => (
                    0,
                    cell.paragraphs
                        .len()
                        .min(p.stop_after_para.saturating_add(1)),
                ),
                None => (0, cell.paragraphs.len()),
            };
            for cp_idx in loop_start..loop_end_excl {
                let para = &cell.paragraphs[cp_idx];
                // 분할 행이면 해당 문단의 줄 범위 적용
                let (start_line, end_line) = if let Some(ref ranges) = line_ranges {
                    if cp_idx < ranges.len() {
                        ranges[cp_idx]
                    } else {
                        (0, 0) // 범위 밖 문단은 렌더링하지 않음
                    }
                } else {
                    (
                        0,
                        composed_store
                            .get(cp_idx, cell, inner_width, styles)
                            .lines
                            .len(),
                    )
                };
                let mixed_nested_split = cut_units.and_then(|(su, eu)| {
                    self.mixed_nested_split_from_cut(cell, table, styles, su, eu, cp_idx)
                });
                let nested_cursor_split = cut_units.and_then(|(su, eu)| {
                    self.nested_table_split_from_cut_units(cell, table, styles, su, eu, cp_idx)
                });
                // [Task #1073] 이 문단이 per-중첩행 유닛으로 분해됐으면(가시 텍스트 없음 +
                // 단일 중첩 표 2행+) 컷에 들어온 유닛의 `nested_row` 에서 중첩 행 범위를
                // 얻어 NestedTableSplit 으로 넘긴다.
                //
                // 종전에는 "컷 유닛 인덱스 == 중첩행 번호" 라고 가정해 **셀**이 문단 1개일
                // 때만 이 경로를 썼다. 분해 조건은 문단 단위(cell_units)인데 게이트는 셀
                // 단위여서, 문단이 여럿인 셀은 아래 `available_h` 휴리스틱으로 폴백했고 그
                // 분기는 오프셋을 0.0 으로 고정하므로 연속 페이지가 행 0 부터 다시 그리고
                // 뒤 행이 어느 페이지에도 나오지 않았다.
                let nested_cut_rows: Option<(usize, usize)> = if nested_cursor_split.is_some() {
                    None
                } else {
                    cut_units.and_then(|(su, eu)| {
                        self.nested_row_range_from_cut_units(cell, table, styles, su, eu, cp_idx)
                    })
                };
                let visible_non_inline_controls = cut_units.is_some_and(|(su, eu)| {
                    self.cell_cut_contains_non_inline_control_units(
                        cell, table, styles, su, eu, cp_idx,
                    )
                });
                let has_table_ctrl = para.controls.iter().any(|c| matches!(c, Control::Table(_)));
                // [#3820 Stage 77] HWP5에는 내부 표 control만 있고 LINE_SEG가 전혀
                // 없는 셀 문단이 있다(76076 p35 row 6). 이 표가 들어 있는 outer
                // fragment가 아직 source cut을 쓰지 않는다면, `(0, 0)`은 비가시
                // 판정이 아니라 control-only 문단의 합성 결과다. normal-table path와
                // 같이 control을 배치해야 한다. cut fragment에서는 단위 소유권을
                // 유지해 다음 쪽 표를 앞쪽에 중복 방출하지 않는다.
                let uncut_control_only_nested_table = cut_units.is_none()
                    && has_table_ctrl
                    && para
                        .text
                        .chars()
                        .all(|ch| ch.is_whitespace() || ch == '\r' || ch == '\n');

                // [Task #993] 컷 범위 밖 문단은 이전/다음 페이지 소속 — 이 페이지에서
                // 스킵한다. cell_line_ranges_from_cut 이 가시 유닛만 범위에 넣으므로
                // (중첩 표/빈 문단 포함) start_line>=end_line 이면 비가시가 확정이다.
                // content_y_accum 은 가시 콘텐츠만 추적하므로 스킵 시 전진하지 않는다.
                if start_line >= end_line
                    && mixed_nested_split.is_none()
                    && nested_cursor_split.is_none()
                    && !visible_non_inline_controls
                    && !uncut_control_only_nested_table
                {
                    continue;
                }

                // [#4149] 가시 문단만 여기 도달 — lazy 슬롯은 이 시점에 compose 된다.
                let composed = composed_store.get(cp_idx, cell, inner_width, styles);

                if preserve_linear_single_cell_vpos {
                    let target_seg = para
                        .line_segs
                        .get(start_line)
                        .or_else(|| para.line_segs.first());
                    if let Some(seg) = target_seg {
                        let target_top =
                            hwpunit_to_px((seg.vertical_pos - vpos_origin).max(0), self.dpi);
                        let current_top = (para_y - text_y_start).max(0.0);
                        if target_top > current_top {
                            para_y += target_top - current_top;
                        }
                    }
                }

                // [#3637 진단] 조각 셀에서 실제로 배치되는 문단과 그 y. 컷 범위 밖
                // 문단이 예외 경로(중첩 표 보유 등)로 새는지 직접 본다. 동작 불변.
                if std::env::var("RHWP_DIAG_CELLPARA").is_ok() {
                    eprintln!(
                        "DIAG_CELLPARA pi={} cell=({},{}) cp={} lines={}..{} cut={:?} nested={:?} mixed={} nonline={} para_y={:.1} cell_bot={:.1}",
                        para_index,
                        cell.row,
                        cell.col,
                        cp_idx,
                        start_line,
                        end_line,
                        cut_units,
                        nested_cut_rows,
                        mixed_nested_split.is_some(),
                        visible_non_inline_controls,
                        para_y,
                        cell_content_bottom(cell_y, cell_h, pad_bottom),
                    );
                }
                let cell_context = if let Some(context) = enclosing_cell_ctx {
                    let mut context = context.clone();
                    if let Some(last) = context.path.last_mut() {
                        last.cell_index = cell_idx;
                        last.cell_para_index = cp_idx;
                        last.text_direction = cell.text_direction;
                    }
                    context
                } else {
                    CellContext {
                        parent_para_index: para_index,
                        path: vec![CellPathEntry {
                            control_index,
                            cell_index: cell_idx,
                            cell_para_index: cp_idx,
                            text_direction: cell.text_direction,
                        }],
                    }
                };
                let cell_context_opt = Some(cell_context.clone());

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
                // `layout_table`의 empty-TAC 경로와 같은 줄별 폭 계약을 partial
                // RowBreak fragment에도 쓴다. 빈 paragraph의 image controls는 source
                // char position이 같아도 HWP LINE_SEG가 각각의 flow slot을 보존한다.
                let tac_line_widths: Vec<f64> = {
                    let mut line_widths = vec![0.0f64; composed.lines.len().max(1)];
                    for ctrl in &para.controls {
                        let (is_tac, width) = match ctrl {
                            Control::Picture(pic) if pic.common.treat_as_char => {
                                (true, hwpunit_to_px(pic.common.width as i32, self.dpi))
                            }
                            Control::Shape(shape) if shape.common().treat_as_char => {
                                (true, hwpunit_to_px(shape.common().width as i32, self.dpi))
                            }
                            Control::Equation(eq) => {
                                (true, hwpunit_to_px(eq.common.width as i32, self.dpi))
                            }
                            Control::Table(table) if table.common.treat_as_char => (
                                true,
                                hwpunit_to_px(
                                    table.common.width as i32
                                        + table.outer_margin_left as i32
                                        + table.outer_margin_right as i32,
                                    self.dpi,
                                ),
                            ),
                            _ => (false, 0.0),
                        };
                        if !is_tac {
                            continue;
                        }
                        if composed.lines.len() <= 1 {
                            line_widths[0] += width;
                            continue;
                        }
                        if let Some(line_width) = line_widths.iter_mut().find(|line_width| {
                            **line_width == 0.0 || **line_width + width <= inner_width + 0.5
                        }) {
                            *line_width += width;
                        } else if let Some(last) = line_widths.last_mut() {
                            *last += width;
                        }
                    }
                    line_widths
                };

                // 표 컨트롤이 없는 문단: 텍스트 먼저, 컨트롤 나중 (기존 동작)
                // 표 컨트롤이 있는 문단: 문단 앞 간격 적용 → 표 먼저 배치 → 텍스트(엔터 등) 나중
                if !has_table_ctrl
                    || composed
                        .lines
                        .iter()
                        .any(|line| line.runs.iter().any(|run| !run.text.trim().is_empty()))
                {
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
                    // [Task #1728 v2] 셀-내 continuation 조각(cut su>0)의 첫 가시 문단
                    // (아직 텍스트 없음 + 문단 첫 줄부터 시작)은 셀-상단이라 layout_composed_paragraph
                    // 의 column-top 트림에 걸려 앞 간격이 사라진다. 한컴은 유지하므로 토글 on.
                    // 1×1 linear 셀(page-spanning 컨테이너, preserve_linear_single_cell_vpos
                    // 계열)의 continuation 은 자연 흐름으로 이미 정합하며 textbox/shape 를 품을 수
                    // 있어 spacing 추가 시 프레임 밖으로 밀린다(#issue_rowbreak_chart_overlap p17).
                    // 다행/다열 표의 거대 셀 intra-cell continuation 만 대상으로 한정한다.
                    let keep_spacing = cut_units.is_some_and(|(su, _)| su > 0)
                        && !has_preceding_text
                        && start_line == 0
                        && !(table.row_count == 1 && table.col_count == 1);
                    self.keep_continuation_column_top_spacing_before
                        .set(keep_spacing);
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
                    self.keep_continuation_column_top_spacing_before.set(false);

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
                    // Normal and partial table layout used to differ here: the normal
                    // path maps picture-only TACs to their saved LINE_SEG slots, while
                    // this fallback accumulated every picture on the first slot. Keep
                    // this state local to the empty-run fallback; text-bearing and
                    // same-line TAC handling stays on the original path.
                    let all_runs_empty = composed.lines.iter().all(|line| line.runs.is_empty());
                    let para_margin_left = styles
                        .para_styles
                        .get(para.para_shape_id as usize)
                        .map(|style| style.margin_left)
                        .unwrap_or(0.0);
                    let para_indent = styles
                        .para_styles
                        .get(para.para_shape_id as usize)
                        .map(|style| style.indent)
                        .unwrap_or(0.0);
                    let mut empty_tac_seq_index = 0usize;
                    let mut empty_tac_current_line = 0usize;
                    let first_tac_width = tac_line_widths
                        .first()
                        .copied()
                        .unwrap_or(total_inline_width);
                    let mut empty_tac_x = match para_alignment {
                        Alignment::Center | Alignment::Distribute => {
                            inner_area.x + (inner_area.width - first_tac_width).max(0.0) / 2.0
                        }
                        Alignment::Right => {
                            inner_area.x + (inner_area.width - first_tac_width).max(0.0)
                        }
                        _ => {
                            inner_area.x
                                + effective_margin_left_line(para_margin_left, para_indent, 0)
                        }
                    };
                    let mut empty_tac_y = para_y_before_compose;
                    let mut rendered_top_and_bottom_non_inline = false;

                    for (ctrl_idx, ctrl) in para.controls.iter().enumerate() {
                        match ctrl {
                            Control::Picture(pic) => {
                                let visible_non_inline_control =
                                    cut_units.map_or(true, |(su, eu)| {
                                        self.cell_cut_starts_non_inline_control(
                                            cell, table, styles, su, eu, cp_idx, ctrl_idx,
                                        )
                                    });
                                let fragment_owned_square_flow =
                                    self.profile.get().native_hwp5_layout()
                                        && cut_units.is_some()
                                        && visible_non_inline_control
                                        && pic.common.flow_with_text
                                        && matches!(
                                            pic.common.text_wrap,
                                            crate::model::shape::TextWrap::Square
                                        );
                                if !pic.common.treat_as_char
                                    && cut_units.is_some()
                                    && !visible_non_inline_control
                                {
                                    continue;
                                }
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
                                        if all_runs_empty && para.line_segs.len() > 1 {
                                            let target_line =
                                                empty_tac_seq_index.min(para.line_segs.len() - 1);
                                            empty_tac_seq_index += 1;
                                            if target_line > empty_tac_current_line {
                                                empty_tac_current_line = target_line;
                                                let line_width = tac_line_widths
                                                    .get(target_line)
                                                    .copied()
                                                    .unwrap_or(0.0);
                                                let line_margin = effective_margin_left_line(
                                                    para_margin_left,
                                                    para_indent,
                                                    target_line,
                                                );
                                                empty_tac_x = match para_alignment {
                                                    Alignment::Center | Alignment::Distribute => {
                                                        inner_area.x
                                                            + (inner_area.width - line_width)
                                                                .max(0.0)
                                                                / 2.0
                                                    }
                                                    Alignment::Right => {
                                                        inner_area.x
                                                            + (inner_area.width - line_width)
                                                                .max(0.0)
                                                    }
                                                    _ => inner_area.x + line_margin,
                                                };
                                                let first_vpos = para
                                                    .line_segs
                                                    .first()
                                                    .map(|line| line.vertical_pos)
                                                    .unwrap_or(0);
                                                if let Some(segment) =
                                                    para.line_segs.get(target_line)
                                                {
                                                    empty_tac_y = para_y_before_compose
                                                        + hwpunit_to_px(
                                                            segment.vertical_pos - first_vpos,
                                                            self.dpi,
                                                        );
                                                }
                                            }
                                            let pic_h =
                                                hwpunit_to_px(pic.common.height as i32, self.dpi);
                                            let clamped_w = pic_w.min(inner_area.width);
                                            let clamped_h = if pic_w > 0.0 {
                                                pic_h * (clamped_w / pic_w)
                                            } else {
                                                pic_h
                                            };
                                            let pic_area = LayoutRect {
                                                x: empty_tac_x,
                                                y: empty_tac_y,
                                                width: clamped_w,
                                                height: clamped_h,
                                            };
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
                                            empty_tac_x += clamped_w;
                                            continue;
                                        }
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
                                    let top_and_bottom_para = matches!(
                                        pic.common.text_wrap,
                                        crate::model::shape::TextWrap::TopAndBottom
                                    ) && matches!(
                                        pic.common.vert_rel_to,
                                        crate::model::shape::VertRelTo::Para
                                    );
                                    // #1921 p8: 빈 top-anchored 문단 안에 Square 부동 그림과
                                    // TAC 그림이 함께 있으면 compose가 TAC 높이만큼 para_y를
                                    // advance한다. Square 그림의 Para 기준은 advance 후 위치가
                                    // 아니라 그 문단이 시작한 physical cell content top이다.
                                    // vpos>0으로 실제로 밀린 빈 줄(#2226)과 일반 Square 셀은
                                    // 이 조건에 포함하지 않는다.
                                    let empty_top_anchored_square_with_inline_sibling = para
                                        .text
                                        .trim()
                                        .is_empty()
                                        && para
                                            .line_segs
                                            .first()
                                            .is_some_and(|seg| seg.vertical_pos == 0)
                                        && pic.common.flow_with_text
                                        && matches!(
                                            pic.common.text_wrap,
                                            crate::model::shape::TextWrap::Square
                                        )
                                        && para.controls.iter().any(|ctrl| {
                                            matches!(
                                                ctrl,
                                                Control::Picture(sibling) if sibling.common.treat_as_char
                                            )
                                        });
                                    let anchor_y = if empty_top_anchored_square_with_inline_sibling
                                    {
                                        cell_y + pad_top
                                    } else if top_and_bottom_para {
                                        if cut_units.is_some() && visible_non_inline_controls {
                                            // continuation 조각에 개체 flow 유닛이 실제 포함된 경우
                                            // 원본 line_seg vertical_pos 는 전체 셀 내부 좌표다. 그대로
                                            // 쓰면 다음 쪽 상단에 와야 할 그림이 조각 하단으로 밀린다.
                                            para_y_before_compose
                                        } else {
                                            para.line_segs
                                                .first()
                                                .filter(|seg| seg.vertical_pos >= 0)
                                                .map(|seg| {
                                                    cell_y
                                                        + pad_top
                                                        + hwpunit_to_px(seg.vertical_pos, self.dpi)
                                                })
                                                .unwrap_or(para_y_before_compose)
                                        }
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
                                    // [Issue #2071] 셀 vertical_align 존중 (table_layout.rs 동일 수정).
                                    // 한컴은 셀 앵커 자리차지 그림을 **셀 valign 으로만** 배치하고
                                    // 그림 자체 pos vert_align 은 무시한다. compute_object_position
                                    // 은 그림 pos vert_align 을 따르므로 콘텐츠 box·그림 높이 기준
                                    // 셀 valign 위치를 강제한다.
                                    let pic_y = if fragment_owned_square_flow {
                                        // p0처럼 같은 physical fragment가 여러 Square
                                        // control을 소유할 때, negative saved offset은
                                        // 이전 source ladder의 값이다. current fragment의
                                        // flow anchor를 다시 위로 끌어올리지 않는다.
                                        picture_anchor_y
                                    } else if top_and_bottom_para
                                        && pic.common.flow_with_text
                                        && !unrestricted_take_place_cell_float
                                    {
                                        let v_off = hwpunit_to_px(
                                            pic.common.vertical_offset as i32,
                                            self.dpi,
                                        );
                                        let content_top = cell_y + pad_top;
                                        match effective_align {
                                            VerticalAlign::Top => content_top + v_off,
                                            VerticalAlign::Center => {
                                                content_top + (inner_height - pic_h + v_off) / 2.0
                                            }
                                            VerticalAlign::Bottom => {
                                                content_top + inner_height - pic_h - v_off
                                            }
                                        }
                                    } else {
                                        pic_y
                                    };
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
                                            &mut *table_node,
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
                                    if matches!(
                                        pic.common.text_wrap,
                                        crate::model::shape::TextWrap::TopAndBottom
                                    ) {
                                        rendered_top_and_bottom_non_inline = true;
                                    } else if fragment_owned_square_flow {
                                        para_y +=
                                            self.cell_non_inline_control_flow_height(&pic.common);
                                    } else {
                                        para_y += self.non_inline_control_flow_height(&pic.common);
                                    }
                                }
                                has_preceding_text = true;
                            }
                            Control::Shape(shape) => {
                                // TextBox를 포함한 Shape는 한 control이 여러 physical
                                // fragment에 걸쳐 내부 문단을 이어 그릴 수 있다. Picture의
                                // entry-only owner 규칙을 Shape에 적용하면 뒤 fragment의
                                // 잔여 TextBox가 통째로 사라진다(rowbreak p17).
                                if !shape.common().treat_as_char
                                    && cut_units.is_some()
                                    && !visible_non_inline_controls
                                {
                                    continue;
                                }
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
                                    let mut shape_for_layout = shape.clone();
                                    if cut_units.is_some() && visible_non_inline_controls {
                                        shape_for_layout.common_mut().horizontal_offset = 0;
                                        shape_for_layout.common_mut().horz_align =
                                            crate::model::shape::HorzAlign::Center;
                                        if start_line >= end_line
                                            && matches!(
                                                shape.common().text_wrap,
                                                crate::model::shape::TextWrap::Square
                                                    | crate::model::shape::TextWrap::Tight
                                                    | crate::model::shape::TextWrap::Through
                                            )
                                            && matches!(
                                                shape.common().vert_rel_to,
                                                crate::model::shape::VertRelTo::Para
                                            )
                                            && (shape.common().vertical_offset as i32) > 0
                                        {
                                            shape_for_layout.common_mut().vertical_offset = 0;
                                        }
                                    }
                                    self.layout_cell_shape(
                                        tree,
                                        &mut cell_node,
                                        &shape_for_layout,
                                        &inner_area,
                                        shape_anchor_y,
                                        para_alignment,
                                        styles,
                                        bin_data_content,
                                        clamp_header_negative_para_offset,
                                        table_cell_ctx,
                                    );
                                    let is_top_and_bottom_shape = matches!(
                                        shape.common().text_wrap,
                                        crate::model::shape::TextWrap::TopAndBottom
                                    );
                                    let mut shape_flow_h =
                                        self.cell_non_inline_control_flow_height(shape.common());
                                    if is_top_and_bottom_shape {
                                        rendered_top_and_bottom_non_inline = true;
                                        shape_flow_h = 0.0;
                                    }
                                    if !is_top_and_bottom_shape && shape_flow_h <= 0.0 {
                                        shape_flow_h =
                                            if cut_units.is_some() && visible_non_inline_controls {
                                                hwpunit_to_px(
                                                    shape.common().height as i32,
                                                    self.dpi,
                                                ) + hwpunit_to_px(
                                                    (shape.common().vertical_offset as i32).max(0),
                                                    self.dpi,
                                                ) + hwpunit_to_px(
                                                    shape.common().margin.top as i32,
                                                    self.dpi,
                                                ) + hwpunit_to_px(
                                                    shape.common().margin.bottom as i32,
                                                    self.dpi,
                                                )
                                            } else {
                                                0.0
                                            };
                                    }
                                    para_y += shape_flow_h;
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
                                        script: eq.script.clone(),
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
                                            * self.render_table_width_scale(nested_table)
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
                                    let split_info = if let Some(split) =
                                        mixed_nested_split.as_ref()
                                    {
                                        Some(NestedTableSplit {
                                            start_row: split.start_row,
                                            end_row: split.end_row,
                                            visible_height: split.visible_height,
                                            flow_height: split.flow_height,
                                            offset_within_start: split.offset_within_start,
                                            content_offset: split.content_offset,
                                            force_source_start_cut: split.force_source_start_cut,
                                            replay_terminal_boundary_unit: split
                                                .replay_terminal_boundary_unit,
                                            terminal: split.terminal,
                                            recursive_cut: split.recursive_cut.clone(),
                                        })
                                    } else if let Some(split) = nested_cursor_split.as_ref() {
                                        Some(NestedTableSplit {
                                            start_row: split.start_row,
                                            end_row: split.end_row,
                                            visible_height: split.visible_height,
                                            flow_height: split.flow_height,
                                            offset_within_start: split.offset_within_start,
                                            content_offset: split.content_offset,
                                            force_source_start_cut: split.force_source_start_cut,
                                            replay_terminal_boundary_unit: split
                                                .replay_terminal_boundary_unit,
                                            terminal: split.terminal,
                                            recursive_cut: split.recursive_cut.clone(),
                                        })
                                    } else if let Some((row_lo, row_hi)) = nested_cut_rows {
                                        // [Task #1073] 페이지네이션 컷의 중첩행 범위로 직접
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
                                            true,
                                        );
                                        let ncs = hwpunit_to_px(
                                            nested_table.cell_spacing as i32,
                                            self.dpi,
                                        );
                                        let start_row = row_lo.min(nrow);
                                        let end_row = row_hi.min(nrow).max(start_row);
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
                                            flow_height: vis_h,
                                            content_offset: 0.0,
                                            force_source_start_cut: false,
                                            replay_terminal_boundary_unit: false,
                                            // [#3658] per-중첩행 컷 경로도 마지막 유닛까지
                                            // 포함한 컷(end_cut=[])이면 종료 조각이다.
                                            terminal: cut_units
                                                .is_some_and(|(_, eu)| eu == usize::MAX),
                                            offset_within_start: 0.0,
                                            recursive_cut: None,
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
                                            true,
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
                                            || s.offset_within_start > 0.5
                                            || s.visible_height + 0.5 < nested_h
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
                                    // [#4334] 이 재귀 중첩 표는 `table_meta: None` 이라
                                    // TableNode.para_index/control_index 가 항상 비었다 —
                                    // 방금 확장한 `nested_ctx` 에서 이 중첩 표 자신의
                                    // 좌표를 읽는다. `section_index` 는 이미 항상 채워지는데
                                    // para/control 만 비는 게 #4334 stage3 가 실측한 42개 중
                                    // 다수의 원인이었다.
                                    let derived_table_meta = nested_ctx
                                        .as_ref()
                                        .and_then(CellContext::nested_table_meta);
                                    let first_new_child = cell_node.children.len();
                                    let table_h_rendered = if let Some(recursive_cut) = split_info
                                        .as_ref()
                                        .and_then(|split| split.recursive_cut.as_ref())
                                    {
                                        // [#4069] 측정이 자식 표의 행·CellUnit 범위를
                                        // 재귀 투영한 경우 렌더도 같은 범위를 부분 표 경로에
                                        // 넘긴다. scalar y clip으로 표 전체를 매 쪽 재방출하면
                                        // vpos 리셋 프레임의 앞·뒤 문단이 서로 겹치므로, 동일
                                        // cursor가 선택한 문단/줄/자식 표만 방출해야 한다.
                                        // 이 cursor가 측정한 표는 문서 모델 안의 원본
                                        // `nested_table`이다. 매 페이지 clone을 만들면
                                        // `cell_units_cache`의 raw cell pointer가 allocator
                                        // 재사용으로 다른 clone을 가리킬 수 있다. 일반 wrapper가
                                        // 끝낸 table 해석 뒤의 구현을 원본 참조로 직접 호출한다.
                                        let rendered_bottom = self.layout_partial_table_resolved(
                                            tree,
                                            &mut cell_node,
                                            nested_table.as_ref(),
                                            PartialTableHostContext {
                                                paragraphs: &[],
                                                para_index: 0,
                                                control_index: 0,
                                                repeat_fragment_outer_margin: false,
                                                pre_emitted_host_height: 0.0,
                                                host_line_spacing: 0.0,
                                            },
                                            section_index,
                                            styles,
                                            outline_numbering_id,
                                            &ctrl_area,
                                            nested_y,
                                            bin_data_content,
                                            recursive_cut.start_row,
                                            recursive_cut.end_row,
                                            recursive_cut.start_row > 0
                                                || !recursive_cut.start_cut.is_empty(),
                                            &recursive_cut.start_cut,
                                            &recursive_cut.end_cut,
                                            recursive_cut.is_block_split,
                                            // recursive_cut의 행 cursor는 이미 이 호출의
                                            // `nested_table` 자신의 행 기준(#4069 재귀
                                            // 투영)이다. 투명 래퍼 벗기기는 별개 좌표계
                                            // 판정(#4326)이라 여기서는 항상 false.
                                            false,
                                            None,
                                            None,
                                            0.0,
                                            0.0,
                                            None,
                                            nested_ctx.as_ref(),
                                            clamp_header_negative_para_offset,
                                            // [#4149] 프로브는 최외곽 표에만 적용한다.
                                            None,
                                        );
                                        // [#3820/issue2007 p14] recursive cut은 현재 호출의
                                        // source 범위를 제한하지만 parent flow_height는 기존
                                        // 조각 높이를 유지한다. 새 child root만 clip에 포섭해
                                        // 현재 쪽 마지막 두 줄이 조상 clip에서 소실되지 않게
                                        // 하고 pagination cursor에는 영향을 주지 않는다.
                                        expand_cell_clip_to_new_source_bounded_children(
                                            &mut cell_node,
                                            first_new_child,
                                        );
                                        rendered_bottom - nested_y
                                    } else {
                                        self.layout_table(
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
                                            derived_table_meta,
                                            para_alignment,
                                            nested_ctx,
                                            0.0,
                                            0.0,
                                            None,
                                            split_ref,
                                            None,
                                            None,
                                            false,
                                            clamp_header_negative_para_offset,
                                            false,
                                        )
                                    };
                                    let visible_table_h = mixed_nested_split
                                        .as_ref()
                                        .map(|split| split.flow_height)
                                        .unwrap_or(table_h_rendered);
                                    para_y = nested_y + visible_table_h;
                                    has_preceding_text = true;
                                }
                            }
                            _ => {}
                        }
                    }
                    if rendered_top_and_bottom_non_inline {
                        para_y +=
                            self.paragraph_top_and_bottom_non_inline_flow_height(&para.controls);
                    }
                }

                if has_table_ctrl && mixed_nested_split.is_none() {
                    // LINE_SEG vpos 기반으로 para_y 보정.
                    let is_last_para = cp_idx + 1 == cell.paragraphs.len();
                    if !is_last_para {
                        if let Some(next_para) = cell.paragraphs.get(cp_idx + 1) {
                            if let Some(next_seg) = next_para.line_segs.first() {
                                // [#3637] `vertical_pos` 는 **셀 전체** 좌표라 조각 시작
                                // 유닛만큼의 원점이 빠져 있지 않다. 조각 후반부(start_cut 이
                                // 큰 연속 조각)에서 이 스냅은 문단을 셀 상자 밖으로 밀어내고,
                                // 그 글자는 어느 렌더 경로에도 보이지 않는다 — 텍스트 추출에만
                                // 남는다(156083443 보도자료 10쪽: cp=164 다음이 vpos=72846
                                // → y=1018.5, 셀 바닥 1005.1).
                                //
                                // 원점을 빼는 교정은 #3654 에서 실패했다. para_y 를 낮추면 그
                                // 값이 다시 컷 판정으로 되먹임되어 다른 문서에 새 넘침을
                                // 만든다. 이 스냅은 **밀어내기 전용**(`max`)이므로 셀 바닥으로
                                // 상한만 두면 밀림을 막으면서 기존 위치는 보존한다 — 스냅이
                                // 필요했던 쪽 안 문단은 상자 안이라 상한에 걸리지 않는다.
                                //
                                // 그래서 두 가지를 함께 건다.
                                //   ① 조각 원점(frag_vpos_origin)을 빼 조각-상대 좌표로
                                //   ② 그래도 남는 셀 내부 도약(#3654 가 걸린 자리, 소방방재
                                //      45,290 HU)에 대비해 셀 바닥으로 상한
                                // ①만으로는 #3654 처럼 도약 문서에서 여전히 밀려나고,
                                // ②만으로는 상한에서 멈춘 뒤 뒤 문단이 그 아래로 쌓인다.
                                let next_vpos_y = text_y_start
                                    + hwpunit_to_px(
                                        (next_seg.vertical_pos - frag_vpos_origin).max(0),
                                        self.dpi,
                                    );
                                let cell_content_bottom =
                                    cell_content_bottom(cell_y, cell_h, pad_bottom);
                                para_y = para_y.max(next_vpos_y.min(cell_content_bottom));
                            }
                        }
                    }
                }
            }

            // 각주 참조 번호
            for para in &cell.paragraphs {
                self.add_footnote_superscripts(tree, &mut cell_node, para, styles);
            }

            // [#4159] `end_cut=[]`인 종료 유닛 창은 이후 continuation이 없다. 재귀
            // 중첩 표의 bottom stroke가 top padding만큼 셀 clip을 넘는 경우에만
            // 현재 셀 bbox를 포섭 확장한다. `eu < usize::MAX` 비종료 조각은 보존한다.
            let terminal_cell_fragment =
                cut_units.is_some_and(|(_, end_unit)| end_unit == usize::MAX);
            expand_terminal_cell_clip_to_nested_table_descendants(
                &mut cell_node,
                terminal_cell_fragment,
            );

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
                        &mut *h_edges,
                        &mut *v_edges,
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
    }

    /// 표의 일부 행만 레이아웃한다 (페이지 분할).
    ///
    /// `start_row..end_row` 범위의 행만 렌더링한다.
    /// `is_continuation`이 true이고 repeat_header인 표면 행0(제목행)을 먼저 렌더링한다.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn layout_partial_table(
        &self,
        tree: &mut LayoutFrame,
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
        row_cursor_is_nested: bool,
        end_row_height_override: Option<f64>,
        start_row_height_override: Option<f64>,
        host_margin_left: f64,
        host_margin_right: f64,
        measured_table: Option<&MeasuredTable>,
        enclosing_cell_ctx: Option<&CellContext>,
        clamp_header_negative_para_offset: bool,
        probe: Option<&PartialTableCellProbe>,
    ) -> f64 {
        let para = match paragraphs.get(para_index) {
            Some(p) => p,
            None => return y_start,
        };
        let outer_table = match para.controls.get(control_index) {
            Some(Control::Table(t)) => t,
            _ => return y_start,
        };
        let repeat_fragment_outer_margin = repeats_native_empty_host_rowbreak_fragment_margin(
            self.profile.get().native_hwp5_layout(),
            paragraphs,
            para_index,
            control_index,
        );
        let pre_emitted_host_height = self
            .pre_emitted_host_heights
            .borrow()
            .get(&para_index)
            .copied()
            .unwrap_or(0.0);
        let host_line_spacing = para
            .line_segs
            .first()
            .map(|seg| hwpunit_to_px(seg.line_spacing, self.dpi))
            .unwrap_or(0.0);

        self.layout_partial_table_resolved(
            tree,
            col_node,
            outer_table.as_ref(),
            PartialTableHostContext {
                paragraphs,
                para_index,
                control_index,
                repeat_fragment_outer_margin,
                pre_emitted_host_height,
                host_line_spacing,
            },
            section_index,
            styles,
            outline_numbering_id,
            col_area,
            y_start,
            bin_data_content,
            start_row,
            end_row,
            is_continuation,
            start_cut,
            end_cut,
            is_block_split,
            row_cursor_is_nested,
            end_row_height_override,
            start_row_height_override,
            host_margin_left,
            host_margin_right,
            measured_table,
            enclosing_cell_ctx,
            clamp_header_negative_para_offset,
            probe,
        )
    }

    /// 이미 원본 표와 host 문맥이 해석된 부분 표 렌더 구현.
    ///
    /// 재귀 child cursor는 이 경로를 직접 호출해 문서 소유 `&Table`의 안정 주소를
    /// 유지한다. 그러면 `cell_units_cache`와 `table_nested_text_flag_cache`가 페이지마다
    /// 생성·폐기되는 clone의 재사용 주소를 잘못 적중하지 않는다.
    #[allow(clippy::too_many_arguments)]
    fn layout_partial_table_resolved(
        &self,
        tree: &mut LayoutFrame,
        col_node: &mut RenderNode,
        outer_table: &crate::model::table::Table,
        host: PartialTableHostContext<'_>,
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
        row_cursor_is_nested: bool,
        end_row_height_override: Option<f64>,
        start_row_height_override: Option<f64>,
        host_margin_left: f64,
        host_margin_right: f64,
        measured_table: Option<&MeasuredTable>,
        enclosing_cell_ctx: Option<&CellContext>,
        clamp_header_negative_para_offset: bool,
        probe: Option<&PartialTableCellProbe>,
    ) -> f64 {
        let PartialTableHostContext {
            paragraphs,
            para_index,
            control_index,
            repeat_fragment_outer_margin,
            pre_emitted_host_height,
            host_line_spacing,
        } = host;

        // [Issue #4326] Pagination can deliberately use the rows of a transparent
        // 1×1 wrapper's nested table.  The measured-table path has used that
        // effective table since the wrapper-unwrapping rule was introduced, but a
        // `PartialTable` used to identify its source by the outer control alone —
        // whether `start_row`/`end_row` addressed the outer table's own row domain
        // or the unwrapped nested table's was reconstructed here from
        // `end_row <= outer_table.row_count`. That inference collided with a
        // genuine single-row fragment of the *nested* table (`end_row == 1`),
        // which is indistinguishable by value from "the outer wrapper's own row 0"
        // (#4326: `margin_bottom` sweep flips 24/40 values into duplicate-printed,
        // body-overflowing table fragments). `row_cursor_is_nested` now carries
        // that coordinate-system decision from pagination as data, so this is a
        // direct dispatch rather than a second heuristic.
        //
        // A native HWP5 RowBreak wrapper owns its physical clip/frame only while
        // the partial cursor still names its own outer row (#1921 p16, #3637, HWP
        // 2020 p7); `row_cursor_is_nested == false` preserves that path unchanged.
        let table = if row_cursor_is_nested {
            transparent_nested_table(outer_table)
        } else {
            outer_table
        };

        if table.cells.is_empty() {
            return y_start;
        }

        // [#3820 Stage 120] A native-HWP5 empty-host 1x1 RowBreak table can store the
        // physical first-fragment height in `common.height`, then restart its cell LINE_SEG
        // ladder at zero for the successor page.  Keep this paint-only witness separate from
        // pagination: the PageItem cuts and logical consumed height remain authoritative.
        let stored_reset_paint_geometry =
            if enclosing_cell_ctx.is_none() && std::ptr::eq(table, outer_table) {
                paragraphs.get(para_index).and_then(|host_para| {
                    native_hwp5_stored_reset_fragment_paint_geometry(
                        self.profile.get().native_hwp5_layout(),
                        host_para,
                        table,
                        is_continuation,
                        start_cut,
                        end_cut,
                    )
                })
            } else {
                None
            };

        // 분할 표 첫 부분: vert_offset 적용 (자리차지 표의 세로 오프셋).
        // [Task #712] HwpUnit=u32 이라 `vertical_offset > 0` 는 음수 비트표현
        // (예: -1796 HU = 0xFFFFF8FC = 4294965500u32) 도 양수로 통과시켜
        // 후속 `as i32` 캐스트에서 음수가 적용 → 표가 위로 점프, 직전 인라인
        // 표 영역 침범. 비-Partial 경로(`table_layout.rs:1069+`)는 동일 분기에
        // `raw_y.max(y_start)` 클램프가 있어 음수 무력화. Partial 경로에는
        // 클램프가 없으므로 게이트를 signed 비교로 정정해 동등 효과.
        let vert_off_signed = table.common.vertical_offset as i32;
        let effective_vertical_offset = if !is_continuation
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
            // [#2015] host 텍스트가 pre-emit 된 경우, 진입 y_start 는 이미
            // para_start+host_h(host 텍스트 끝) 흐름 위치다. vert_offset 은 para_start 기준
            // 오프셋이므로 그대로 더하면 host_h 만큼 이중계상되어 표가 아래로 밀린다
            // (부동 RowBreak 표 91.2px 오버플로우). 표의 참 상단 = para_start+vert_off =
            // y_start+(vert_off−host_h). typeset 예산도 동일 감액을 적용한다.
            // host pre-emit 이 아니면 host_h=0 → 종전과 동일(회귀 없음).
            (hwpunit_to_px(vert_off_signed, self.dpi) - pre_emitted_host_height).max(0.0)
        } else {
            0.0
        };
        // [#2287 후속/1.hwpx p28] 같은 단의 직전 흐름 표와의 미세 겹침 방지
        // 안전망: 자리차지 표의 v_off/outer 흐름 미가산(#2097 반증 기록 축 —
        // 전면 가산은 82802 악화)으로 후속 TopAndBottom 표 조각의 typeset
        // 좌표가 직전 표 렌더 끝보다 소폭(6.4px) 이르게 잡히면 괘선이 겹쳐
        // 렌더된다. 흐름 표(vert=문단·비 TAC) 한정으로 직전 표 렌더 하단
        // 아래로 push-down — 겹침이 없으면 no-op.
        let is_para_flow_table = !table.common.treat_as_char
            && matches!(
                table.common.text_wrap,
                crate::model::shape::TextWrap::TopAndBottom
            )
            && matches!(
                table.common.vert_rel_to,
                crate::model::shape::VertRelTo::Para
            );
        let y_start = if is_para_flow_table {
            let prev_table_end = col_node
                .children
                .iter()
                .filter_map(|child| {
                    let RenderNodeType::Table(meta) = &child.node_type else {
                        return None;
                    };
                    let repeated_previous_bottom = if repeat_fragment_outer_margin {
                        meta.para_index
                            .zip(meta.control_index)
                            .and_then(|(pi, ci)| paragraphs.get(pi)?.controls.get(ci))
                            .and_then(|control| match control {
                                Control::Table(previous) => Some(hwpunit_to_px(
                                    previous.outer_margin_bottom as i32,
                                    self.dpi,
                                )),
                                _ => None,
                            })
                            .unwrap_or(0.0)
                    } else {
                        0.0
                    };
                    Some(child.bbox.y + child.bbox.height + repeated_previous_bottom)
                })
                .fold(f64::NEG_INFINITY, f64::max);
            if repeat_fragment_outer_margin {
                // The strict native-HWP shape uses the painted predecessor plus its trailing
                // margin as the flow base, then opens this fragment's top margin.  Apply the
                // positive object offset only to the first fragment.  This ordering restores the
                // 965HU p2 gap (283 previous-bottom + 283 current-top + 399 offset) and repeats
                // only 283HU at the p3 continuation top.
                let flow_base = if prev_table_end.is_finite() {
                    y_start.max(prev_table_end)
                } else {
                    y_start
                };
                flow_base
                    + hwpunit_to_px(table.outer_margin_top as i32, self.dpi)
                    + effective_vertical_offset
            } else if prev_table_end.is_finite()
                && y_start + effective_vertical_offset < prev_table_end - 0.5
            {
                prev_table_end
            } else {
                y_start + effective_vertical_offset
            }
        } else {
            y_start + effective_vertical_offset
        };

        let col_count = table.col_count as usize;
        let row_count = table.row_count as usize;
        let cell_spacing = hwpunit_to_px(table.cell_spacing as i32, self.dpi);

        // ── 1. 열 폭 계산 + 2. 행 높이 계산 (table_layout 공유 메서드) ──
        let col_widths = self.resolve_column_widths(table, col_count);
        let mut row_heights = self.resolve_row_heights(
            table,
            col_count,
            row_count,
            measured_table,
            styles,
            table.common.treat_as_char,
        );
        // [Task #1748] 컷 걸침 rowspan 셀의 이전 프래그먼트 소비 높이 재계산용 —
        // 2b 컷 오버라이드 이전의 원본 행 높이 (프래그먼트 무관 값).
        let resolved_row_heights = row_heights.clone();

        // ── 2b. 행 높이 오버라이드 (Task #993: 컷 기반) ──
        // 렌더 대상 모든 행의 높이를 페이지네이터와 동일한 컷 측정
        // (row_cut_content_height)으로 정정한다. 페이지네이터(typeset)와 렌더러가
        // 단일 측정 공간(advance_row_cut/cell_units)을 공유해야 분할 표가
        // 페이지를 넘지 않는다. 분할 행은 start_cut/end_cut 범위, 그 외 행은
        // 전체 콘텐츠. rowspan 연속 행(컷 0)은 resolve_row_heights 결과 유지.
        {
            let split_last_row = end_row.saturating_sub(1);
            let mut rows_to_set: std::collections::BTreeSet<usize> = (start_row..end_row).collect();
            // 연속분 머리행 반복 — start_row 이전의 반복 제목행도 렌더된다.
            // [Task #1716] 반복 대상은 표 상단의 연속 제목행 블록만(흩어진 하위 is_header 제외).
            // 페이지네이터(typeset.rs header_overhead)와 동일 leading_header_rows 사용 → 정합.
            if is_continuation && table.repeat_header && start_row > 0 {
                for r in table.leading_header_rows() {
                    if r < start_row {
                        rows_to_set.insert(r);
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
                    // [#2287/PR #2290 P1] 컷 블록 안의 rs=1 셀 없는 걸침-전용 행은
                    // 원본(resolve) 높이가 그대로 남아 rowspan 셀 bbox 가 컷과
                    // 무관하게 원본 크기(교육부 47×9 r3=2107px → 셀 2354.6px)로
                    // 유지됐다 — valign 이 콘텐츠를 셀 중앙(페이지 밖 y≈1259)으로
                    // 밀어 tail overflow 로 관측(리뷰 p26/p30). 컷 블록의 행높이는
                    // 아래 블록-합 보정이 권위이므로 여기서는 0 으로 둔다.
                    if rcells.is_empty() {
                        row_heights[r] = 0.0;
                        continue;
                    }
                    let mut per_start: Vec<usize> = Vec::with_capacity(rcells.len());
                    let mut per_end: Vec<usize> = Vec::with_capacity(rcells.len());
                    let mut has_visible_range = false;
                    let mut has_row_cut = false;
                    for c in &rcells {
                        let units = self.cell_units(c, table, styles);
                        let (su, eu) = cell_cut_window(
                            table,
                            c,
                            true,
                            in_start,
                            start_block,
                            in_end,
                            end_block,
                            start_cut,
                            end_cut,
                            Some(units.len()),
                        );
                        if eu > su {
                            has_visible_range = true;
                        }
                        if su > 0 || eu < units.len() {
                            has_row_cut = true;
                        }
                        per_start.push(su);
                        per_end.push(eu);
                    }
                    // [#2287/PR #2290 P1] 컷 블록(in_start/in_end) 안 행은 rs=1
                    // 셀 컷이 "전체 소비"(su=0, eu=len)여도 whole-row 경로의 선언
                    // 셀높이 max 를 타면 안 된다 — 블록 분할 중 행높이는 콘텐츠
                    // 기반이어야 하고, rowspan 가시분은 아래 블록-합 보정이 채운다
                    // (교육부 r3: rs=1 셀 2개 전체 소비 17.1px 인데 선언 max 로
                    // 2107.1 유지 → 셀 bbox 2354.6 → valign 이 페이지 밖으로).
                    let h = if !has_visible_range {
                        0.0
                    } else if has_row_cut || in_start || in_end {
                        self.row_cut_content_height(table, r, &per_start, &per_end, styles)
                    } else {
                        self.row_cut_content_height(table, r, &[], &[], styles)
                    };
                    if h > 0.0 {
                        row_heights[r] = h;
                    } else if has_row_cut {
                        // 컷 범위가 이 행에서 비가시(전부 다른 조각 소속)면 0.
                        row_heights[r] = 0.0;
                    }
                } else {
                    let su: &[usize] = if r == start_row { start_cut } else { &[] };
                    let eu: &[usize] = if r == split_last_row { end_cut } else { &[] };
                    // 기존 per-row 경로에서 rowspan 행은 기본적으로 atomic
                    // (resolve_row_heights) 유지. 단 RowBreak 의 큰 rowspan 블록 내부
                    // 행을 typeset 이 per-row cut 으로 분할한 split boundary 에서는
                    // 렌더러도 같은 cut 높이를 적용해야 한다.
                    let has_single_row_cells = table
                        .cells
                        .iter()
                        .any(|c| c.row as usize == r && c.row_span == 1);
                    if rowspan_touched && su.is_empty() && eu.is_empty() && !has_single_row_cells {
                        continue;
                    }
                    // [#2287 후속/1.hwpx p14] 컷 없는(whole-row) **순수 텍스트**
                    // 행은 재계산하지 않는다 — resolve_row_heights 가
                    // mt.row_heights(= typeset 조각 소비와 동일 측정 공간)를 이미
                    // 반영했는데, row_cut_content_height(whole-row)로 덮으면
                    // content(ls 계상 규칙 상이)가 선언 셀높이보다 커지는 행에서
                    // 렌더만 부풀어(85×3 표 41행 × +4.0px = +152px) typeset 소비
                    // 밖으로 조각 꼬리가 밀린다 — p14 QUR-001~005 행이 page frame
                    // 밖(y 1058~1164)으로 사라진 결함. 중첩 표 포함 행은 반대로
                    // mt 가 중첩 높이를 과소 계상해 재계산이 행 겹침을 막고
                    // 있으므로(rowbreak-problem-pages p7 pi=21 r2, 기존 회귀
                    // 테스트) 종전 재계산을 유지한다.
                    if su.is_empty() && eu.is_empty() && measured_table.is_some() {
                        let row_has_nested = table.cells.iter().any(|c| {
                            c.row as usize == r
                                && c.row_span == 1
                                && c.paragraphs.iter().any(|p| {
                                    p.controls.iter().any(|ct| matches!(ct, Control::Table(_)))
                                })
                        });
                        if !row_has_nested {
                            continue;
                        }
                    }
                    let h = self.row_cut_content_height(table, r, su, eu, styles);
                    if h > 0.0 {
                        row_heights[r] = h;
                    }
                }
            }
            // [#2287/PR #2290 P1] 블록-합 보정: 컷 블록에 걸친 rowspan 셀의 컷
            // 가시 높이(su..eu 유닛 합 + pad)가 rs=1 기반 행높이 합보다 크면
            // 블록 마지막 행에 차액을 가산한다 — rowspan 셀 bbox 가 컷 가시
            // 높이와 정합해야 클립/valign 이 컷 의미대로 동작한다 (typeset 의
            // consumed_height 와 동일 좌표계).
            if is_block_split {
                let mut blocks: Vec<(usize, usize)> = Vec::new();
                for b in [start_block, end_block].into_iter().flatten() {
                    if !blocks.contains(&b) {
                        blocks.push(b);
                    }
                }
                for (bs, be) in blocks {
                    let mut target = 0.0f64;
                    for c in table.cells.iter().filter(|c| {
                        c.row_span > 1 && (c.row as usize) >= bs && (c.row as usize) < be
                    }) {
                        let su = if start_block == Some((bs, be)) {
                            block_cut_index(table, bs, be, c)
                                .and_then(|i| start_cut.get(i).copied())
                                .unwrap_or(0)
                        } else {
                            0
                        };
                        let eu = if end_block == Some((bs, be)) {
                            block_cut_index(table, bs, be, c)
                                .and_then(|i| end_cut.get(i).copied())
                                .unwrap_or(usize::MAX)
                        } else {
                            usize::MAX
                        };
                        target = target.max(self.cell_cut_visible_height(c, table, styles, su, eu));
                    }
                    if target <= 0.0 {
                        continue;
                    }
                    let cur: f64 = (bs..be.min(row_count))
                        .map(|r| row_heights.get(r).copied().unwrap_or(0.0))
                        .sum();
                    if target > cur + 0.5 {
                        if let Some(last) = (bs..be.min(row_count)).next_back() {
                            row_heights[last] += target - cur;
                        }
                    }
                }
            }
        }

        // [#3820 Stage 76] RowBreak 표의 rowspan-연속 밴드에서 실제 셀 내용은
        // 현재 쪽에 모두 들어가지만, 원본 선언 행 높이만 남은 공간보다 큰 경우가
        // 있다. 페이지네이터는 다음 조각을 다음 행부터 재개하고 이 조각의 마지막
        // 행만 남은 물리 높이에 맞춰 소비한다. 렌더러도 같은 마지막 행의 **정확한
        // 물리 높이**를 적용해야 p35의 `주요내용`을 보인 뒤 p36을 다음 행에서
        // 시작한다. auto layout이 내용 한 줄(23px)만으로 행을 축소한 경우에는
        // `min`이 남은 75px band를 다시 버리므로, 여기서 limit은 상한이 아니라
        // fragment-local row height다.
        if let Some(limit) = start_row_height_override {
            if start_row < row_count {
                row_heights[start_row] = limit.max(0.0);
            }
        }
        if let Some(limit) = end_row_height_override {
            if let Some(last) = end_row.checked_sub(1).filter(|r| *r < row_count) {
                row_heights[last] = limit.max(0.0);
            }
        }

        // The first stored-reset fragment's composed cut includes the reset-preceding line's
        // trailing spacing.  That value remains the logical flow consumption, while the physical
        // row/cell clip and borders stop at the independently stored head height.  This is a
        // single-row predicate, so the delta can be restored exactly in the function return.
        let mut stored_reset_logical_height_delta = 0.0;
        if let Some(stored_height_hu) =
            stored_reset_paint_geometry.and_then(|geometry| geometry.first_fragment_height_hu)
        {
            if let Some(row_height) = row_heights.first_mut() {
                let stored_height = hwpunit_to_px(stored_height_hu, self.dpi);
                let paint_height = (*row_height).min(stored_height);
                stored_reset_logical_height_delta = (*row_height - paint_height).max(0.0);
                *row_height = paint_height;
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
            self.render_table_width_scale(table),
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
        // is_continuation && repeat_header → start_row 이전의 반복 제목행만 반복.
        // [Task #1716] 반복 대상은 표 상단의 연속 제목행 블록만(흩어진 하위 is_header 제외).
        // 페이지네이터(typeset.rs header_overhead)와 동일 leading_header_rows 사용 → 정합.
        let mut header_rows: Vec<usize> = Vec::new();
        if is_continuation && table.repeat_header && start_row > 0 {
            for r in table.leading_header_rows() {
                if r < start_row && r < row_count {
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

        // Outer margins are already present in the fragment's logical reservation.  Restore them
        // only on the table/cell/frame paint subtree, for both the first and successor fragment.
        let table_x = table_x
            + stored_reset_paint_geometry
                .map(|geometry| hwpunit_to_px(geometry.outer_left_hu, self.dpi))
                .unwrap_or(0.0);

        let table_y = if render_top_caption {
            y_start + caption_height + caption_spacing
        } else {
            y_start
        } + stored_reset_paint_geometry
            .map(|geometry| hwpunit_to_px(geometry.outer_top_hu, self.dpi))
            .unwrap_or(0.0);

        // ── 5. 표 노드 생성 ──
        // 재귀 부분 표는 합성 `(para=0, control=0)`으로 table 데이터를 조회하지만,
        // RenderNode provenance에는 원본 부모 셀 문단과 현재 표 control을 기록한다.
        // TextRun이 없는 빈 셀 hit-test는 Table/TableCell traversal context만 사용하므로
        // 이 metadata까지 실제 IR 경로여야 한다 (#4252).
        let (node_para_index, node_control_index) = enclosing_cell_ctx
            .and_then(|context| {
                let table_entry = context.path.last()?;
                let parent_entry = context.path.get(context.path.len().checked_sub(2)?)?;
                Some((parent_entry.cell_para_index, table_entry.control_index))
            })
            .unwrap_or((para_index, control_index));
        let table_id = tree.next_id();
        let mut table_node = RenderNode::new(
            table_id,
            RenderNodeType::Table(TableNode {
                row_count: table.row_count,
                col_count: table.col_count,
                border_fill_id: table.border_fill_id,
                section_index: Some(section_index),
                para_index: Some(node_para_index),
                control_index: Some(node_control_index),
                // [#4334] 표 분할 조각(partial table)도 원본 부모 셀 경로를 그대로
                // 옮겨 담는다 — provenance para/control 과 같은 근거(#4252).
                cell_context: enclosing_cell_ctx.cloned(),
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
        self.layout_partial_table_cells(
            tree,
            &mut table_node,
            table,
            para_index,
            control_index,
            section_index,
            styles,
            outline_numbering_id,
            bin_data_content,
            start_row,
            end_row,
            end_row_height_override,
            is_continuation,
            start_cut,
            end_cut,
            is_block_split,
            cell_spacing,
            col_count,
            row_count,
            table_x,
            table_y,
            &row_heights,
            &resolved_row_heights,
            &row_col_x,
            &header_rows,
            &render_rows,
            &render_row_y,
            &mut h_edges,
            &mut v_edges,
            measured_table,
            enclosing_cell_ctx,
            clamp_header_negative_para_offset,
            probe,
        );

        // 엣지 기반 테두리 렌더링
        let body_top_clip = (enclosing_cell_ctx.is_none()
            && self.is_body_flow_col_area(col_area)
            && (table_y - col_area.y).abs() <= 0.5)
            .then_some(col_area.y);
        table_node.children.extend(render_edge_borders(
            tree,
            &h_edges,
            &v_edges,
            &row_col_x,
            &grid_row_y,
            table_x,
            table_y,
            body_top_clip,
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

        // Partial-table cells receive their nested table edge nodes only
        // after the fragment cell loop. Preserve direct nested outer vertical
        // borders in the horizontal clip without widening the RowBreak
        // continuation viewport (issue2007 p2-p3).
        extend_completed_nested_table_border_clips(
            tree,
            &mut table_node,
            self.profile.get().native_hwp5_layout() || self.profile.get().hwp5_origin_hwpx(),
            self.profile.get().hwpx_container(),
        );

        // [Task #1860/#3820] 노드-자식 포섭 불변: 분할 표 조각의 셀 내 절대위치 shape
        // (as-char 텍스트박스/그림 등)가 유닛 기반 셀 높이를 초과해 그려지면 표 노드
        // bbox 가 자식을 clip 한다(page17 pi=28 텍스트박스 하단 잘림). 렌더 완료 후 모든
        // **가시** 자손의 최하단을 구해 표 노드 높이를 그만큼 확장한다(확장만, 축소 없음).
        //
        // 단, RowBreak 조각의 `TableCell { clip: true }` 아래 일반 흐름 자손은 그 물리
        // cell viewport 밖에서 다음/이전 쪽의 흐름을 보유할 수 있다. 그 invisible tail까지
        // Table bbox에 포함하면 body clip도 수천 px로 확대되어 Canvas/WASM replay가
        // 현재 쪽 밖의 내용을 paint 후보로 보게 된다(42065 p8 이후). 반면 직접 배치된
        // 도형은 clip cell의 현재 쪽 표시물일 수 있으므로, **현재 column body 안에서 끝나는
        // 경우에만** 그 도형 subtree를 계속 포함한다. 이 경계는 p17의 textbox-backed
        // rectangle은 보존하면서, 다른 문서의 다음 쪽 밖 도형을 table/body clip으로
        // 역류시키지 않는다.
        {
            let physical_page_bottom = col_area.y + col_area.height;
            fn descendant_bottom(node: &RenderNode, physical_page_bottom: f64) -> f64 {
                let mut b = node.bbox.y + node.bbox.height;
                if matches!(
                    node.node_type,
                    RenderNodeType::TableCell(TableCellNode { clip: true, .. })
                ) {
                    for child in &node.children {
                        let is_direct_drawing = matches!(
                            child.node_type,
                            RenderNodeType::Rectangle(_)
                                | RenderNodeType::Ellipse(_)
                                | RenderNodeType::Path(_)
                                | RenderNodeType::Image(_)
                                | RenderNodeType::Group(_)
                                | RenderNodeType::TextBox
                                | RenderNodeType::Equation(_)
                                | RenderNodeType::FormObject(_)
                                | RenderNodeType::Placeholder(_)
                                | RenderNodeType::RawSvg(_)
                        );
                        if is_direct_drawing {
                            let drawing_bottom = descendant_bottom(child, physical_page_bottom);
                            if drawing_bottom <= physical_page_bottom + 0.5 {
                                b = b.max(drawing_bottom);
                            }
                        }
                    }
                    return b;
                }
                for c in &node.children {
                    b = b.max(descendant_bottom(c, physical_page_bottom));
                }
                b
            }
            let content_bottom = table_node
                .children
                .iter()
                .map(|child| descendant_bottom(child, physical_page_bottom))
                .fold(table_node.bbox.y + table_node.bbox.height, f64::max);
            let grown = content_bottom - table_node.bbox.y;
            if grown > table_node.bbox.height {
                table_node.bbox.height = grown;
            }
        }

        col_node.children.push(table_node);

        // ── 캡션 렌더링 ──
        // cell_index = 65534: 캡션 식별 센티널 (셀 0과 구분)
        let cap_cell_ctx = Some(if let Some(context) = enclosing_cell_ctx {
            let mut context = context.clone();
            if let Some(last) = context.path.last_mut() {
                last.cell_index = 65534;
                last.cell_para_index = 0;
                last.text_direction = 0;
            }
            context
        } else {
            CellContext {
                parent_para_index: para_index,
                path: vec![CellPathEntry {
                    control_index,
                    cell_index: 65534,
                    cell_para_index: 0,
                    text_direction: 0,
                }],
            }
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
                    bin_data_content,
                    cap_cell_ctx.clone(),
                );
            }
        }
        if render_bottom_caption {
            if let Some(ref caption) = table.caption {
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
                    bin_data_content,
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
                    bin_data_content,
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
        // [#3637 진단] 조각 렌더 높이 vs 페이지네이터 컷 예산. 동작 불변.
        // TABLE_SPLIT_RESULT 의 consumed 와 짝지어 보면 두 공간의 발산이 보인다.
        if std::env::var("RHWP_DIAG_FRAG").is_ok() {
            eprintln!(
                "DIAG_FRAG pi={} ci={} rows={}..{} cont={} blk={} start_cut={:?} end_cut={:?} y_start={:.1} tbl_h={:.1} logical_delta={:.1} cap={:.1}",
                para_index,
                control_index,
                start_row,
                end_row,
                is_continuation,
                is_block_split,
                start_cut,
                end_cut,
                y_start,
                partial_table_height,
                stored_reset_logical_height_delta,
                caption_total,
            );
        }
        // Do not move subsequent flow or change PageItem ownership: Stage 120 changes only the
        // painted frame/clip.  The paginator consumed the original composed cut height.
        y_start + partial_table_height + stored_reset_logical_height_delta + caption_total
    }
}

#[cfg(test)]
mod tests {
    use super::{
        cell_content_bottom, expand_cell_clip_to_new_source_bounded_children,
        expand_terminal_cell_clip_to_nested_table_descendants, fragment_vpos_origin,
    };
    use crate::model::paragraph::{LineSeg, Paragraph};
    use crate::model::table::Cell;
    use crate::renderer::render_tree::{
        BoundingBox, LineNode, RenderNode, RenderNodeType, TableCellNode, TableNode,
    };

    fn paragraph_with_vpos(vposes: &[i32]) -> Paragraph {
        Paragraph {
            line_segs: vposes
                .iter()
                .map(|&vertical_pos| LineSeg {
                    vertical_pos,
                    ..Default::default()
                })
                .collect(),
            ..Default::default()
        }
    }

    #[test]
    fn split_cell_fragment_origin_uses_first_visible_line_not_paragraph_start() {
        let cell = Cell {
            paragraphs: vec![paragraph_with_vpos(&[4_000, 5_000, 6_000])],
            ..Default::default()
        };

        assert_eq!(
            fragment_vpos_origin(&cell, Some(&[(1, 3)])),
            5_000,
            "문단 중간에서 시작한 조각은 첫 줄 vpos를 다시 쓰면 안 된다"
        );
    }

    #[test]
    fn split_cell_snap_cap_uses_physical_content_bottom_not_valign_start() {
        // Center/Bottom valign의 text_y_start에는 이미 offset이 들어 있다. 물리 셀
        // 하단은 어떤 valign이든 cell_y + cell_h - pad_bottom으로 고정된다.
        assert_eq!(cell_content_bottom(100.0, 80.0, 7.0), 173.0);
    }

    fn clipped_cell_with_overflowing_nested_table() -> RenderNode {
        let mut cell = RenderNode::new(
            1,
            RenderNodeType::TableCell(TableCellNode {
                col: 0,
                row: 0,
                col_span: 1,
                row_span: 1,
                border_fill_id: 0,
                text_direction: 0,
                clip: true,
                model_cell_index: Some(0),
            }),
            BoundingBox::new(10.0, 20.0, 100.0, 80.0),
        );
        let mut table = RenderNode::new(
            2,
            RenderNodeType::Table(TableNode {
                row_count: 1,
                col_count: 1,
                border_fill_id: 0,
                section_index: None,
                para_index: None,
                control_index: None,
                cell_context: None,
            }),
            BoundingBox::new(15.0, 25.0, 90.0, 80.0),
        );
        table.children.push(RenderNode::new(
            3,
            RenderNodeType::Line(LineNode::new(15.0, 104.0, 105.0, 104.0, Default::default())),
            BoundingBox::new(15.0, 104.0, 90.0, 2.0),
        ));
        cell.children.push(table);
        cell
    }

    #[test]
    fn issue_4159_terminal_cell_clip_contains_nested_table_stroke() {
        let mut cell = clipped_cell_with_overflowing_nested_table();
        expand_terminal_cell_clip_to_nested_table_descendants(&mut cell, true);
        assert_eq!(cell.bbox.height, 86.0);
    }

    #[test]
    fn issue_4159_nonterminal_cell_clip_does_not_expose_nested_tail() {
        let mut cell = clipped_cell_with_overflowing_nested_table();
        expand_terminal_cell_clip_to_nested_table_descendants(&mut cell, false);
        assert_eq!(cell.bbox.height, 80.0);
    }

    #[test]
    fn issue_2007_recursive_cut_clip_only_contains_new_source_bounded_child() {
        let mut cell = clipped_cell_with_overflowing_nested_table();
        let first_new_child = cell.children.len();
        cell.children.push(RenderNode::new(
            4,
            RenderNodeType::Table(TableNode {
                row_count: 1,
                col_count: 1,
                border_fill_id: 0,
                section_index: None,
                para_index: None,
                control_index: None,
                cell_context: None,
            }),
            BoundingBox::new(15.0, 25.0, 90.0, 78.0),
        ));

        expand_cell_clip_to_new_source_bounded_children(&mut cell, first_new_child);

        assert_eq!(cell.bbox.height, 83.0);
    }
}
