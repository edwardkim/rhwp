//! 일반 행의 저장 프레임 선택과 통째 수용 Query. 실제 수용·컷 선택·이월은 부모 소유다.

use super::row::RowScanQuery;
use super::RowBlockQuery;
use crate::model::provenance::LayoutCompatibilityProfile;
use crate::renderer::layout::table_layout::RowCutResult;

pub(in crate::renderer::typeset) struct SourceFrameQuery<'a> {
    pub(in crate::renderer::typeset) row: &'a RowScanQuery<'a>,
    pub(in crate::renderer::typeset) row_start_cut: &'a [usize],
    pub(in crate::renderer::typeset) row_count: usize,
    pub(in crate::renderer::typeset) is_continuation: bool,
    pub(in crate::renderer::typeset) profile: &'a LayoutCompatibilityProfile,
}

/// 세 높이와 예산을 합치지 않는다. 실제로 앞서 수용한 높이/행 간격/현재 요구 높이다.
pub(in crate::renderer::typeset) struct WholeRowBudget {
    pub(in crate::renderer::typeset) consumed: f64,
    pub(in crate::renderer::typeset) cs_before: f64,
    pub(in crate::renderer::typeset) row_total: f64,
    pub(in crate::renderer::typeset) avail_for_rows: f64,
    pub(in crate::renderer::typeset) strict_painted_bottom_fit: bool,
    pub(in crate::renderer::typeset) source_first_fragment_overflow_allowance: f64,
    pub(in crate::renderer::typeset) source_first_fragment_row_end: Option<usize>,
}

pub(in crate::renderer::typeset) struct SourceFrameSelection {
    pub(in crate::renderer::typeset) terminal_response_before_empty_spacer: bool,
    pub(in crate::renderer::typeset) two_line_terminal_response_source_frame: Option<f64>,
    pub(in crate::renderer::typeset) stored_source_frame: Option<RowCutResult>,
    pub(in crate::renderer::typeset) terminal_source_frame: bool,
    pub(in crate::renderer::typeset) continued_source_frame: bool,
    pub(in crate::renderer::typeset) opening_source_frame: bool,
    pub(in crate::renderer::typeset) mid_source_frame: bool,
    pub(in crate::renderer::typeset) whole_row_fits: bool,
}

impl SourceFrameQuery<'_> {
    /// reflow/빈 행 조회는 선행 guard가 성립할 때만 호출한다. 컷을 복제하지 않는다.
    pub(in crate::renderer::typeset) fn resolve(
        &self,
        budget: WholeRowBudget,
        table_text_reflowed: impl Fn() -> bool,
        row_has_no_text_or_controls: impl Fn(usize) -> bool,
    ) -> SourceFrameSelection {
        let Self {
            row,
            row_start_cut,
            row_count,
            is_continuation,
            profile,
        } = *self;
        let RowScanQuery {
            rows,
            r,
            cursor_row,
            ..
        } = *row;
        let RowBlockQuery {
            layout_engine,
            mt,
            table,
            styles,
            rowspan_touched,
            ..
        } = *rows;
        let WholeRowBudget {
            consumed,
            cs_before,
            row_total,
            avail_for_rows,
            strict_painted_bottom_fit,
            source_first_fragment_overflow_allowance,
            source_first_fragment_row_end,
        } = budget;
        // The final visible response is followed by a row without text or
        // controls. Its stored row height is authoritative for whole-row ownership;
        // browser-composed height may be larger solely because of font
        // metrics and must not create a tail-only physical page.
        let terminal_response_before_empty_spacer = mt.allows_row_break_split()
            && r + 2 == row_count
            && row_start_cut.is_empty()
            && !rowspan_touched.get(r).copied().unwrap_or(true)
            && (row_has_no_text_or_controls(r + 1)
                || layout_engine.row_has_only_empty_spacer_units(table, r + 1, styles));
        let two_line_terminal_response_source_frame = (profile.hwpx_stored_layout()
            && r > cursor_row
            && terminal_response_before_empty_spacer
            && table.outer_margin_bottom > 0)
            .then(|| layout_engine.row_two_line_source_frame_height(table, r, styles))
            .flatten();
        // 저장 vpos reset은 같은 문단 안에서 양수 좌표가 0으로 되감긴 경우에만
        // 물리 조각 경계를 소유한다. 문단 전환의 0은 HWPX writer-local cursor라
        // 저장 프레임 컷으로 선택하지 않고 일반 행 용량 계산에 맡긴다.
        let stored_source_frame = (profile.hwpx_stored_layout()
            && !table.common.treat_as_char
            && mt.allows_row_break_split()
            && layout_engine.row_has_stored_vpos_frame_rewind(table, r)
            && layout_engine.row_has_single_visible_source_cell(table, r, styles)
            && !rowspan_touched.get(r).copied().unwrap_or(true))
        .then(|| layout_engine.stored_frame_cut_for_row(table, r, row_start_cut, styles))
        .flatten();
        let terminal_source_frame = profile.hwpx_stored_layout()
            && r + 2 == row_count
            && r > cursor_row
            && row_start_cut.is_empty()
            && layout_engine.row_has_stored_vpos_frame_rewind(table, r);
        let continued_source_frame =
            is_continuation && !row_start_cut.is_empty() && stored_source_frame.is_some();
        // direct HWPX의 첫 RowBreak 조각도 저장 LINE_SEG frame이 물리 owner를
        // 명시할 수 있다. 일반 capacity cut이 frame 끝 직전에서 멈추면 다음
        // fragment가 그 짧은 tail만 소유해 쪽 하나가 늘어난다. 이 경우에는
        // 기존 source-frame cut이 가진 정확한 CellUnit 끝을 첫 조각에도 쓴다.
        // 편집 뒤 reflow된 텍스트는 저장 좌표를 그대로 신뢰할 수 없으므로
        // continuation/terminal 경로와 달리 새 첫-fragment 경로에서는 제외한다.
        let opening_source_frame = !is_continuation
            && r == cursor_row
            && row_start_cut.is_empty()
            && !table_text_reflowed()
            && stored_source_frame.is_some();
        // [#5584 ②] 조각 **중간 행**(r > cursor_row)에서도 capacity cut 이 저장
        // 프레임 끝 직전에 멈추면 다음 fragment 가 짧은 tail(한 유닛)만 소유해
        // 쪽 하나가 늘어난다 — 3232693 p1: r=7 budget 153.9 에 141.3 소비 후
        // 다음 유닛(21.3)이 12.7px 부족으로 밀리고, p2 가 그 한 유닛만 담은 채
        // 저장 리셋에서 끊겨 5쪽(한글 4쪽). opening 경로와 같은 저장 계약이되,
        // 중간 행은 근소 초과(한 유닛 규모)일 때만 프레임 끝까지 당긴다 —
        // 상한은 아래 확장 지점에서 검사한다.
        let mid_source_frame = !is_continuation
            && r > cursor_row
            && row_start_cut.is_empty()
            && !table_text_reflowed()
            && stored_source_frame.is_some();
        let single_visible_source_frame = profile.hwpx_stored_layout()
            && stored_source_frame.is_some()
            && layout_engine.row_has_stored_vpos_frame_rewind(table, r)
            && layout_engine.row_has_single_visible_source_cell(table, r, styles);
        let strict_nonterminal_rounding_fit = strict_painted_bottom_fit
            && r + 1 < row_count
            && consumed + cs_before + row_total <= avail_for_rows + 0.5;
        let source_frame_whole_row_fits = source_first_fragment_overflow_allowance > 0.0
            && source_first_fragment_row_end == Some(r + 1)
            && consumed + cs_before + row_total
                <= avail_for_rows + source_first_fragment_overflow_allowance;
        // A direct HWPX row with one visible owner and a structural empty
        // partner has an explicit source fragment boundary.  Let the
        // row-cut walk retain it; ordinary and multi-owner rows keep the
        // measured whole-row fast path.
        let declared_source_frame = row_start_cut.is_empty()
            && !table_text_reflowed()
            && layout_engine.row_has_declared_stored_frame(table, r);
        let whole_row_fits = !declared_source_frame
            && ((!single_visible_source_frame
                && consumed + cs_before + row_total <= avail_for_rows)
                || strict_nonterminal_rounding_fit
                || source_frame_whole_row_fits);

        SourceFrameSelection {
            terminal_response_before_empty_spacer,
            two_line_terminal_response_source_frame,
            stored_source_frame,
            terminal_source_frame,
            continued_source_frame,
            opening_source_frame,
            mid_source_frame,
            whole_row_fits,
        }
    }
}
