//! 저장 프레임 꼬리 진입·후보 선택 Query. 경계/확장 계산은 extension, 상태 반영은 부모 소유다.

pub(in crate::renderer::typeset) mod extension;

use super::row::RowScanQuery;
use super::RowBlockQuery;
use crate::model::provenance::LayoutCompatibilityProfile;
use crate::renderer::layout::table_layout::RowCutResult;

pub(in crate::renderer::typeset) struct SourceTailQuery<'a> {
    pub(in crate::renderer::typeset) row: &'a RowScanQuery<'a>,
    pub(in crate::renderer::typeset) row_start_cut: &'a [usize],
    pub(in crate::renderer::typeset) profile: &'a LayoutCompatibilityProfile,
    pub(in crate::renderer::typeset) terminal_response_before_empty_spacer: bool,
    pub(in crate::renderer::typeset) terminal_source_frame: bool,
    pub(in crate::renderer::typeset) continued_source_frame: bool,
    pub(in crate::renderer::typeset) opening_source_frame: bool,
    pub(in crate::renderer::typeset) mid_source_frame: bool,
}

pub(in crate::renderer::typeset) struct SourceTailGate {
    pub(in crate::renderer::typeset) enabled: bool,
    pub(in crate::renderer::typeset) mid_frame_only: bool,
}

impl SourceTailQuery<'_> {
    /// 일반 예산 컷 이후 원래 reset/계약/중간 행/profile 순서로 판정한다.
    pub(in crate::renderer::typeset) fn gate(&self, res: &RowCutResult) -> SourceTailGate {
        let Self {
            row,
            row_start_cut,
            profile,
            terminal_response_before_empty_spacer,
            terminal_source_frame,
            continued_source_frame,
            opening_source_frame,
            mid_source_frame,
        } = *self;
        let RowScanQuery { rows, r, .. } = *row;
        let RowBlockQuery {
            layout_engine,
            table,
            styles,
            ..
        } = *rows;
        // A terminal paragraph tail must not cross the exact plain-text
        // reset where the ordinary capacity cut already stopped.  A row
        // may contain other `vpos=0` transitions for control-only
        // paragraphs; those are local layout coordinates and keep the
        // existing source-frame tail contract.
        let ordinary_cut_ends_at_plain_text_saved_reset = profile.hwp5_stored_pagination_layout()
            && !table.common.treat_as_char
            && terminal_response_before_empty_spacer
            && layout_engine.row_cut_ends_at_plain_text_saved_reset(
                table,
                r,
                row_start_cut,
                &res.end_cut,
                styles,
            );
        let source_frame_tail_contract = (terminal_response_before_empty_spacer
            && !ordinary_cut_ends_at_plain_text_saved_reset)
            || terminal_source_frame
            || continued_source_frame
            || opening_source_frame
            || mid_source_frame;
        // [#5584 ②] 중간 행 갈래만의 확장 상한 — 근소 부족(한 유닛 규모)일 때만.
        let mid_frame_only = mid_source_frame
            && !opening_source_frame
            && !terminal_source_frame
            && !continued_source_frame
            && !(terminal_response_before_empty_spacer
                && !ordinary_cut_ends_at_plain_text_saved_reset);

        // 일반 컷이 저장 쪽 되감김(다음 줄이 새 쪽 vpos 0)에서 끝났으면 그 자리가 원본
        // 프레임의 끝이다. 꼬리 확장이 되감김을 넘어 다음 프레임 내용을 이 쪽에 당겨 오면
        // 한/글이 다음 쪽에 둔 문단이 앞 쪽에 실린다(2025 행정업무운영 편람 hwp 291쪽 표
        // 칸 (4,2): 문단 2 는 저장 vpos 0 의 새 쪽 첫 문단, 정본도 292쪽 머리).
        let ordinary_cut_ends_at_stored_page_reset = res.consumed_height > 0.5
            && layout_engine.row_cut_ends_at_stored_page_reset(
                table,
                r,
                row_start_cut,
                &res.end_cut,
                styles,
            );
        let enabled = (profile.hwp5_stored_pagination_layout() || profile.hwpx_stored_layout())
            && !table.common.treat_as_char
            && !ordinary_cut_ends_at_stored_page_reset
            && source_frame_tail_contract;
        SourceTailGate {
            enabled,
            mid_frame_only,
        }
    }

    /// gate 안에서만 호출한다. 저장 컷을 이동하며 fallback은 원본처럼 지연 평가한다.
    pub(in crate::renderer::typeset) fn candidate(
        &self,
        res: &RowCutResult,
        stored_source_frame: Option<RowCutResult>,
    ) -> Option<RowCutResult> {
        let Self {
            row,
            row_start_cut,
            continued_source_frame,
            ..
        } = *self;
        let RowScanQuery { rows, r, .. } = *row;
        let RowBlockQuery {
            layout_engine,
            table,
            styles,
            ..
        } = *rows;
        if continued_source_frame || res.consumed_height <= 0.5 {
            // A numeric tail allowance used to make this 0px case
            // reach the first saved response line.  Select that exact
            // source unit instead, so a page-tail frame can begin
            // without guessing its pixel height.
            layout_engine.next_visible_unit_cut_for_row(
                table,
                r,
                row_start_cut,
                &res.end_cut,
                styles,
            )
        } else {
            stored_source_frame.or_else(|| {
                layout_engine.paragraph_tail_cut_for_row(
                    table,
                    r,
                    row_start_cut,
                    &res.end_cut,
                    styles,
                )
            })
        }
    }
}
