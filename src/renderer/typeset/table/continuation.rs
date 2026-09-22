//! 표 재개 커서·shadow flow의 소유 타입. 재개 전이는 이 모듈에서 관리한다.

mod fragment;
mod job;
mod step;

use crate::renderer::typeset::{
    paragraph, row_geometry_table, table, FootnoteFragment, MeasuredTable, Paragraph,
    ResolvedStyleSet, TableCellFootnote, TypesetState,
};

/// [#2424] block-table continuation loop의 재개 지점.
///
/// 행과 셀별 절대 unit cut, rowspan block cut 여부, continuation 여부를 owned state로
/// 묶어 fragment budget 경계에서 그대로 보존한다.
#[derive(Debug, Clone, Default, PartialEq)]
pub(in crate::renderer::typeset) struct TableContinuationCursor {
    pub(in crate::renderer::typeset) row: usize,
    pub(in crate::renderer::typeset) start_cut: Vec<usize>,
    pub(in crate::renderer::typeset) start_cut_is_block: bool,
    /// 앞 조각에서 셀 내용은 전부 소비됐지만 그 행의 빈 하단 밴드는 다음
    /// 조각에 남는 경우의 물리 높이. `start_cut`이 내용을 숨기고 이 값은
    /// 테두리/그리드만 이어 준다.
    pub(in crate::renderer::typeset) start_row_height_override: Option<f64>,
    pub(in crate::renderer::typeset) is_continuation: bool,
    pub(in crate::renderer::typeset) fragments_emitted: usize,
    /// fragment queue에서 이미 page에 등록한 표 각주 수.
    pub(in crate::renderer::typeset) next_table_footnote: usize,
    /// 앞 표 fragment에 번호가 찍힌 table-cell 각주의 번호 없는 tail. 다음 physical
    /// fragment의 FootnoteArea 첫 항목으로 먼저 등록해야 source 순서가 보존된다.
    pub(in crate::renderer::typeset) pending_table_footnote_fragment:
        Option<PendingTableFootnoteFragment>,
}

/// RowBreak 표 셀 각주가 HWP 저장 vpos reset에서 물리 page를 넘을 때의 tail 정보.
///
/// 본문 각주와 달리 표 fragment가 먼저 확정된 뒤 footnote queue가 처리되므로, 첫
/// fragment를 등록한 시점에는 아직 다음 page가 없다. source index와 fragment만
/// cursor에 보존해 다음 table fragment가 끝난 뒤 같은 순서로 등록한다.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::renderer::typeset) struct PendingTableFootnoteFragment {
    pub(in crate::renderer::typeset) note_index: usize,
    pub(in crate::renderer::typeset) fragment: FootnoteFragment,
}

impl TableContinuationCursor {
    pub(in crate::renderer::typeset) fn skip_consumed_row(&mut self) {
        self.row = self.row.saturating_add(1);
        self.start_cut.clear();
        self.start_cut_is_block = false;
        self.start_row_height_override = None;
        self.is_continuation = true;
    }

    pub(in crate::renderer::typeset) fn advance(
        &mut self,
        end_row: usize,
        split_block_start: Option<usize>,
        cut: Vec<usize>,
        has_intra_row_split: bool,
    ) {
        self.start_row_height_override = None;
        if has_intra_row_split {
            self.row = split_block_start.unwrap_or(end_row.saturating_sub(1));
            self.start_cut = cut;
            self.start_cut_is_block = split_block_start.is_some();
        } else {
            self.row = end_row;
            self.start_cut.clear();
            self.start_cut_is_block = false;
        }
        self.is_continuation = true;
        self.fragments_emitted = self.fragments_emitted.saturating_add(1);
    }

    pub(in crate::renderer::typeset) fn finish(
        &mut self,
        row_count: usize,
        emitted_fragment: bool,
    ) {
        self.row = row_count;
        self.start_cut.clear();
        self.start_cut_is_block = false;
        self.start_row_height_override = None;
        if emitted_fragment {
            self.fragments_emitted = self.fragments_emitted.saturating_add(1);
        }
    }
}

/// [#2424] continuation loop 진입 전에 한번 계산하는 owned 준비 상태.
pub(in crate::renderer::typeset) struct BlockTableContinuationPreparedState {
    /// 현재 host frame에서 확정한 좌표. 다른 단으로 진행하면 앵커 거리는 소진된다.
    pub(in crate::renderer::typeset) host_placement:
        Option<crate::renderer::float_placement::ParagraphFloatPlacement>,
    pub(in crate::renderer::typeset) host_frame: (usize, u16, u64),
    pub(in crate::renderer::typeset) row_count: usize,
    pub(in crate::renderer::typeset) cell_spacing: f64,
    pub(in crate::renderer::typeset) can_intra_split: bool,
    pub(in crate::renderer::typeset) base_available: f64,
    pub(in crate::renderer::typeset) table_available: f64,
    pub(in crate::renderer::typeset) layout_engine: crate::renderer::layout::LayoutEngine,
    pub(in crate::renderer::typeset) rowspan_touched: Vec<bool>,
    pub(in crate::renderer::typeset) cut_row_heights: Vec<f64>,
    /// 행 전체가 fragment에 남는지 판정할 때의 paint footprint. RowBreak의 실제
    /// intra-row cut 계산은 `cut_row_heights`를 계속 사용한다.
    pub(in crate::renderer::typeset) whole_row_fit_heights: Vec<f64>,
    /// native HWP5 rewind 표의 첫 whole-row fragment가 footer 경계에 남겨야 하는
    /// paint-local slack. continuation과 intra-row cut에는 적용하지 않는다.
    pub(in crate::renderer::typeset) first_fragment_painted_row_footer_guard: f64,
    pub(in crate::renderer::typeset) caption_is_top: bool,
    pub(in crate::renderer::typeset) caption_overhead: f64,
    pub(in crate::renderer::typeset) total_rows_height: f64,
    pub(in crate::renderer::typeset) total_footnote_height: f64,
    pub(in crate::renderer::typeset) queue_table_footnotes: bool,
    pub(in crate::renderer::typeset) table_footnotes: Vec<TableCellFootnote>,
    pub(in crate::renderer::typeset) footnote_margin: f64,
    pub(in crate::renderer::typeset) host_spacing_total: f64,
    pub(in crate::renderer::typeset) host_spacing_before: f64,
    pub(in crate::renderer::typeset) host_spacing_after_only: f64,
    /// 마지막 RowBreak child 뒤의 저장 empty-host line spacing. 첫 anchor
    /// fragment가 아니라 terminal continuation 뒤에서 한 번만 소비한다.
    pub(in crate::renderer::typeset) terminal_nested_child_host_line_spacing: f64,
    pub(in crate::renderer::typeset) strict_following_plain_text_fit: bool,
    pub(in crate::renderer::typeset) budget_para_start_height: f64,
    /// native HWP5 RowBreak 표가 기존 FootnoteArea 직전까지의 물리 경계를
    /// 사용해도 되는 것으로 조판 전에 확인됐을 때의 첫 fragment 절대 경계.
    /// 일반 표에는 `None`으로 기존 보수 budget을 유지한다.
    pub(in crate::renderer::typeset) first_fragment_actual_footnote_boundary: Option<f64>,
    /// 다음 host의 양수 vpos rewind가 현재 RowBreak 표의 continuation source
    /// page를 가리키는지 여부. page-top reset은 표 종료이므로 포함하지 않는다.
    pub(in crate::renderer::typeset) source_next_positive_rewind: bool,
    /// Paint와 공유하는 저장 첫 조각 원점(단 위쪽 기준).
    pub(in crate::renderer::typeset) first_fragment_saved_offset: Option<f64>,
    /// Both stored fragment heights independently prove this whole-row boundary.
    pub(in crate::renderer::typeset) source_cellbreak_row_end: Option<usize>,
    /// 고정 선언 높이보다 실측 내용이 크게 넘치는 native HWP5 RowBreak 표가 마지막
    /// continuation fragment에서 URL 각주를 붙일 때의 실제 경계 완화 여부.
    pub(in crate::renderer::typeset) relax_terminal_table_footnote_fit: bool,
}

/// [#2424] 한 continuation iteration이 caller-controlled step에 돌려주는 진행 상태.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::renderer::typeset) enum TableContinuationIteration {
    Skipped,
    Emitted,
    Complete,
}

/// [#2424] 각 step이 document/measurement cache에서 다시 빌릴 불변 입력.
/// context에는 borrow를 저장하지 않아 이후 WASM 호출 사이 소유를 막지 않는다.
#[derive(Clone, Copy)]
pub(in crate::renderer::typeset) struct BlockTableContinuationSource<'a> {
    pub(in crate::renderer::typeset) para_index: usize,
    pub(in crate::renderer::typeset) control_index: usize,
    pub(in crate::renderer::typeset) paragraph: &'a Paragraph,
    /// 페이지 항목과 부동 배치 속성은 바깥 표의 것으로 보존한다. 다만 빈 1×1
    /// 래퍼는 측정기와 렌더러가 내부 표를 직접 쓰므로, 행 컷 계산도 같은 유효
    /// 표를 사용해야 `MeasuredTable`의 행 수와 컷 대상 행 수가 일치한다.
    pub(in crate::renderer::typeset) table: &'a crate::model::table::Table,
    pub(in crate::renderer::typeset) row_geometry_table: &'a crate::model::table::Table,
    pub(in crate::renderer::typeset) measured_table: &'a MeasuredTable,
    pub(in crate::renderer::typeset) styles: &'a ResolvedStyleSet,
}

/// [#2424] fragment-budget drain의 caller-owned 제어 상태.
/// cursor, budget/step 통계, 비가변 준비값과 shadow page-flow state를 소유한다.
pub(in crate::renderer::typeset) struct BlockTableContinuationContext {
    pub(in crate::renderer::typeset) cursor: TableContinuationCursor,
    pub(in crate::renderer::typeset) fragment_budget: usize,
    pub(in crate::renderer::typeset) steps_completed: usize,
    pub(in crate::renderer::typeset) prepared: BlockTableContinuationPreparedState,
    pub(in crate::renderer::typeset) flow_state: TypesetState,
}

/// [#2424] WASM 호출 사이에 보존되는 단일 대형 표 continuation 작업.
///
/// 문서/측정 캐시의 borrow는 저장하지 않고, 매 step마다 좌표로 다시 해석한다.
/// 공개 pagination은 작업 완료 전까지 이 shadow flow state와 분리되어 있다.
pub(crate) struct ResumableTablePaginationJob {
    pub(in crate::renderer::typeset) context: BlockTableContinuationContext,
    pub(in crate::renderer::typeset) paragraph_index: usize,
    pub(in crate::renderer::typeset) control_index: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ResumablePaginationStep {
    pub(crate) fragments_processed: usize,
    pub(crate) complete: bool,
}

impl BlockTableContinuationContext {
    pub(in crate::renderer::typeset) fn new(
        fragment_budget: usize,
        prepared: BlockTableContinuationPreparedState,
        flow_state: TypesetState,
    ) -> Self {
        Self {
            cursor: TableContinuationCursor::default(),
            fragment_budget: fragment_budget.max(1),
            steps_completed: 0,
            prepared,
            flow_state,
        }
    }

    pub(in crate::renderer::typeset) fn step<F>(&mut self, mut next: F)
    where
        F: FnMut(
            &BlockTableContinuationPreparedState,
            &mut TypesetState,
            &mut TableContinuationCursor,
        ) -> TableContinuationIteration,
    {
        if self.is_complete() {
            return;
        }
        self.steps_completed = self.steps_completed.saturating_add(1);
        let step_start_fragments = self.cursor.fragments_emitted;
        loop {
            match next(&self.prepared, &mut self.flow_state, &mut self.cursor) {
                TableContinuationIteration::Skipped => {
                    if self.cursor.row >= self.prepared.row_count {
                        return;
                    }
                }
                TableContinuationIteration::Complete => return,
                TableContinuationIteration::Emitted => {
                    if self.cursor.row >= self.prepared.row_count
                        || self
                            .cursor
                            .fragments_emitted
                            .saturating_sub(step_start_fragments)
                            >= self.fragment_budget
                    {
                        return;
                    }
                }
            }
        }
    }

    pub(in crate::renderer::typeset) fn is_complete(&self) -> bool {
        self.cursor.row >= self.prepared.row_count
    }

    pub(in crate::renderer::typeset) fn into_flow_state(self) -> TypesetState {
        self.flow_state
    }
}
