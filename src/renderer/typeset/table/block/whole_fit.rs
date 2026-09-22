//! Whole-table fit query. Reads the post-entry flow; does not place or advance it.
//! Existing source-frame and tolerance rules are preserved, not endorsed anew.

use crate::renderer::typeset::{
    controls, hwpunit_to_px, is_para_topbottom_float, is_synthetic_line_seg,
    line_seg_visible_bounds_px, native_hwp5_saved_rowbreak_tail_frame_matches,
    para_has_visible_text, rowbreak_table_has_internal_saved_vpos_reset, signed_hwpunit, table,
    table_declared_height_has_stored_cell_content_frame,
    table_declared_object_covers_cell_row_frames, PageItem, TypesetEngine, TypesetState,
};

use super::BlockTableInput;

pub(super) struct WholeFitInput {
    pub(super) next_starts_new_page: bool,
    pub(super) next_rewinds_after_table: bool,
    pub(super) host_spacing_total: f64,
    pub(super) table_total: f64,
    pub(super) available: f64,
    pub(super) declared_object_total: f64,
    pub(super) single_row_object_declared_fits_current: bool,
}

pub(super) struct WholeFit {
    pub(super) para_has_stored_line_seg: bool,
    pub(super) single_row_object_height_advance: Option<f64>,
    pub(super) fits_after_overlay_shapes: bool,
    pub(super) native_hwp5_rewinding_rowbreak_uses_painted_row_footprint: bool,
    pub(super) whole_fit_table_total: f64,
    pub(super) hwpx_noninline_tac_measured_fit: bool,
    pub(super) declared_table_whole_fits: bool,
    pub(super) saved_table_source_frame: Option<(f64, f64)>,
}

impl TypesetEngine {
    pub(super) fn query_whole_table_fit(
        &self,
        st: &TypesetState,
        input: BlockTableInput<'_>,
        fit: WholeFitInput,
    ) -> WholeFit {
        let BlockTableInput {
            para_idx,
            para,
            table,
            ft,
            fmt,
            mt,
            ..
        } = input;
        let WholeFitInput {
            next_starts_new_page,
            next_rewinds_after_table,
            host_spacing_total,
            table_total,
            available,
            declared_object_total,
            single_row_object_declared_fits_current,
        } = fit;
        let para_has_stored_line_seg = para.line_segs.iter().any(|ls| !is_synthetic_line_seg(ls));
        let single_row_object_height_fits_current = single_row_object_declared_fits_current;
        // HWP5-origin HWPX는 저장 object 높이 기준으로는 현재 쪽에 들어가지만,
        // cell 내용 측정치는 더 큰 1행 자리차지 표를 쪽 하단까지 차지한 것으로
        // 기록한다. native HWP는 RowBreak fragment의 실제 소비량을 따라야 하므로
        // 이 HWPX 호환 보정을 적용하지 않는다.
        let single_row_object_height_advance =
            (st.profile.hwp5_origin_hwpx() && single_row_object_height_fits_current).then(|| {
                if st.current_height + table_total > available {
                    (available - st.current_height).max(0.0)
                } else {
                    declared_object_total
                }
            });

        let current_column_has_only_overlay_shapes = st.current_height <= 0.5
            && st
                .current_items
                .iter()
                .all(|item| matches!(item, PageItem::Shape { .. }));
        let fits_after_overlay_shapes =
            current_column_has_only_overlay_shapes && table_total <= available + 12.0;
        // [#3820] native HWP5의 page-tail ordinary RowBreak 표는 whole-fit gate가
        // 저장 common.height(`table_total`)만 보면, renderer가 실제로 paint할 행
        // footprint보다 작게 판정해 footer 아래까지 행을 보존한다. source의 다음
        // 문단 vpos rewind가 physical fragment 경계를 명시하고, rowspan/cell-footnote가
        // 없는 ordinary-row 형상에서만 measured row footprint를 권위로 삼는다.
        // 일반 HWPX, page-top 표, rowspan 및 실제 intra-row cut은 기존 경로를 유지한다.
        let native_hwp5_rewinding_rowbreak_uses_painted_row_footprint = st.profile.hwp5_stored_pagination_layout()
                && !table.common.treat_as_char
                && is_para_topbottom_float(&table.common)
                && matches!(
                    table.page_break,
                    crate::model::table::TablePageBreak::RowBreak
                )
                && table.row_count > 1
                && ft.table_footnotes.is_empty()
                && st.current_height >= st.base_available_height() * 0.5
                && table.cells.iter().all(|cell| cell.row_span == 1)
                // The physical fragment boundary may be stored inside the last
                // cell's lineSeg sequence, not only at the following host
                // paragraph. Both are source-owned rewinds; ignoring the former
                // lets a declared whole-fit gate retain one painted row too many.
                && (next_rewinds_after_table
                    || rowbreak_table_has_internal_saved_vpos_reset(table));
        let measured_row_table_height = mt.as_ref().and_then(|measured| {
            (!measured.row_heights.is_empty()).then(|| {
                measured.row_heights.iter().sum::<f64>()
                    + measured.cell_spacing * measured.row_heights.len().saturating_sub(1) as f64
            })
        });
        let uses_painted_row_footprint_for_whole_fit =
            native_hwp5_rewinding_rowbreak_uses_painted_row_footprint
                && measured_row_table_height
                    .is_some_and(|height| height > ft.effective_height + 0.5);
        let whole_fit_table_total = if uses_painted_row_footprint_for_whole_fit {
            table_total.max(measured_row_table_height.unwrap_or(0.0) + host_spacing_total)
        } else {
            table_total
        };
        if std::env::var("RHWP_TABLE_DRIFT").is_ok() {
            eprintln!(
                "TABLE_PAINT_FOOTPRINT pi={} native={} rewind={} measured={:.1} effective={:.1} whole_fit={}",
                para_idx,
                st.profile.hwp5_stored_pagination_layout(),
                next_rewinds_after_table,
                measured_row_table_height.unwrap_or(0.0),
                ft.effective_height,
                uses_painted_row_footprint_for_whole_fit,
            );
        }
        // [#2097/#2105] 한글의 실제 행높이 합은 저장 선언 높이와 일치한다(1730000
        // 새만금 COM 3자 비교: 저장 910.5px = 한글 910.6px vs rhwp 실측 954.1px).
        // 셀 내용 실측 팽창으로 측정 fit 이 실패해도 선언 높이가 현재 쪽에 들어가면
        // 통째 배치해 마지막 행 sliver 여분 페이지를 막는다. 쪽나눔=None(#2097)은
        // 한글이 행 컷하지 않는 표, RowBreak(#2105, 19378753 밀양시 907.7px 선언
        // vs 955.9px 실측)는 "나눔 허용"이지 강제가 아니라 선언 fit 시 한글도 나누지
        // 않는다 — 선언이 fit 하지 않는 다쪽 표의 분할 의미론은 불변. CellBreak 는
        // 셀 중간 컷 의미론이 별개라 비대상. advance 는 측정 table_total 을 유지해
        // 같은 쪽 후속 겹침을 차단한다.
        // A RowBreak table on a fresh fragment has no preceding flow to
        // contradict its declared height. Mid-fragment, trust the declaration
        // only when the source host records the object's bottom inside the same
        // body frame. Measured overshoot buckets cannot distinguish a font-metric
        // drift from a real source-owned fragment boundary.
        let rowbreak_at_fragment_start = st.current_items.is_empty();
        let midpage_rowbreak_has_saved_object_bottom = para
            .line_segs
            .iter()
            .find(|ls| !is_synthetic_line_seg(ls))
            .is_some_and(|seg| {
                let base = st.vpos_page_base.unwrap_or(0);
                let v_off = signed_hwpunit(table.common.vertical_offset);
                let top_hu = seg
                    .vertical_pos
                    .saturating_add(v_off.max(0))
                    .saturating_sub(base);
                let bottom_hu =
                    top_hu.saturating_add(table.common.height.min(i32::MAX as u32) as i32);
                hwpunit_to_px(bottom_hu, self.dpi) <= available
            });
        let declared_fit_scope_ok = match table.page_break {
            crate::model::table::TablePageBreak::None => true,
            crate::model::table::TablePageBreak::RowBreak => {
                rowbreak_at_fragment_start || midpage_rowbreak_has_saved_object_bottom
            }
            crate::model::table::TablePageBreak::CellBreak => false,
        };
        // Declared cell boxes are trustworthy only when every text-bearing cell
        // has a stored lineSeg frame that fits inside its own declaration *and*
        // the table object frame owns the declared row geometry. A percentage/
        // cap cannot tell browser metric expansion from a genuinely taller
        // source row, and a cell-local frame alone cannot distinguish a stale
        // short table object from a source-owned RowBreak fragment.
        let declared_excess_has_source_frame =
            table_declared_height_has_stored_cell_content_frame(table, self.dpi)
                && (!matches!(
                    table.page_break,
                    crate::model::table::TablePageBreak::RowBreak
                ) || table_declared_object_covers_cell_row_frames(table, self.dpi));
        // HWPX에서는 `treatAsChar` bit만으로 inline 표가 되지 않는다. stored-layout
        // 문서의 `flowWithText=0` 표는 block table인데, raw bit를 그대로 사용하면
        // declared whole-fit에서 제외되어 generic row cut이 저장 row height를 다시
        // 팽창시키고, 실제로 들어가는 표까지 다음 쪽으로 조기 이월한다 (#3820 p144).
        // 이 좁은 경로는 쪽나눔=None·footnote 없음·실측 높이 fit을 함께 요구한다.
        // 진짜 inline TAC와 native HWP5의 기존 정책은 `uses_tac_table_flow`에 맡긴다.
        let hwpx_noninline_tac_measured_fit = self.profile.get().hwpx_stored_layout()
            && table.common.treat_as_char
            && !self.uses_tac_table_flow(table)
            && matches!(table.page_break, crate::model::table::TablePageBreak::None)
            && ft.table_footnotes.is_empty()
            && st.current_height + ft.effective_height <= available + 0.5;
        // [편집 세션] 셀 편집으로 실측이 선언을 넘게 자란 표는 선언 기준 whole-fit
        // 이 무의미하다 — 선언으로는 "들어간다"인데 실측은 본문 하단을 넘어,
        // 표가 앞 쪽에 잘린 채 남는다(셀 Enter 재현). 실측을 fit 기준으로 써서
        // 넘치면 이월·스캔 경로로 넘긴다.
        let session_grown_measured_fit = self.profile.get().session_edited()
            && ft.effective_height > declared_object_total + 8.0;
        let declared_fit_height = if hwpx_noninline_tac_measured_fit || session_grown_measured_fit {
            ft.effective_height
        } else {
            declared_object_total
        };
        // A stored RowBreak object frame can fit in the current body while
        // browser measurement places the table body a rounding-sized amount
        // below it. The declared frame remains the source ownership boundary in
        // this non-TAC, footnote-free shape for both HWP and HWPX; a genuinely
        // tall table still takes the row scanner because its measured body
        // exceeds this narrow 2px conversion bound.
        const NEAR_MEASURED_ROWBREAK_FIT_PX: f64 = 2.0;
        // A vertical merge beginning in the first logical row makes that row
        // and its successor an atomic stored band. Splitting before that band
        // would retain a border-only fragment even though the declared object
        // fits in the current body.
        let has_leading_rowspan_band = table
            .cells
            .iter()
            .any(|cell| cell.row == 0 && cell.row_span > 1);
        let near_measured_rowbreak_fits = !table.common.treat_as_char
            && matches!(
                table.page_break,
                crate::model::table::TablePageBreak::RowBreak
            )
            && ft.table_footnotes.is_empty()
            // HWPX stored-layout keeps a pagination frame independent from
            // the native HWP5 table declaration. Its near measured fit is a
            // converter provenance contract; native HWP5 and HWP5-origin HWPX
            // must additionally prove that the object frame owns all declared
            // row geometry (#5128 스펙 문서 표 174/193/203/284 통째 흡수 방지).
            && (!st.profile.hwp5_stored_pagination_layout()
                || declared_excess_has_source_frame
                || has_leading_rowspan_band)
            && declared_object_total > host_spacing_total
            && st.current_height + declared_object_total <= available
            && st.current_height + ft.effective_height
                <= available + NEAR_MEASURED_ROWBREAK_FIT_PX;
        // HWPX CELL(RowBreak) TAC 표는 일반적으로 선언 높이가 current fragment의
        // source-owned table frame이다. 단, 단일 빈 host의 유일한 non-synthetic
        // LINE_SEG가 다행 measured table보다 짧으면 그 line은 table band를 소유하지
        // 않는다. 이 예외만 declared-fit에서 제외해 measured table과 뒤 문단이
        // 같은 쪽에 겹치는 것을 막는다. 나머지 CELL/TAC 문서는 기존 declared-fit
        // 호환 경로를 유지한다.
        let hwpx_tac_cell_leftover_missing_owned_line = st.profile.hwpx_stored_layout()
            && table.common.treat_as_char
            && table.common.flow_with_text
            && matches!(
                table.page_break,
                crate::model::table::TablePageBreak::RowBreak
            )
            && para.controls.len() == 1
            && table.row_count == 3
            && table.col_count == 1
            && table.cells.len() == 3
            && !table.repeat_header
            && !para_has_visible_text(para)
            && fmt.line_heights.len() == 1
            && para.line_segs.len() == 1
            && para.line_segs.first().is_some_and(|seg| {
                !is_synthetic_line_seg(seg)
                    && hwpunit_to_px(seg.line_height, self.dpi) + 0.5 < ft.total_height
            });
        // [#6448] HWPX `pageBreak="CELL"`은 모델 RowBreak다. 글자처럼 취급 표는
        // 일반 declared-fit에서 제외되어 measured expansion으로 다음 쪽에 통째
        // 이월될 수 있다. leftover에 declaration이 들어가면 source frame을
        // 존중하되, 위의 누락 host-line 형상은 physical band 경로로 보낸다.
        let hwpx_tac_cell_leftover_declared_fits = st.profile.hwpx_stored_layout()
            && table.common.treat_as_char
            && matches!(
                table.page_break,
                crate::model::table::TablePageBreak::RowBreak
            )
            && ft.table_footnotes.is_empty()
            && declared_object_total > host_spacing_total
            && !st.current_items.is_empty()
            && st.current_height + declared_object_total <= available
            && !hwpx_tac_cell_leftover_missing_owned_line;
        let declared_table_whole_fits = near_measured_rowbreak_fits
            || hwpx_tac_cell_leftover_declared_fits
            || (!uses_painted_row_footprint_for_whole_fit
                && declared_fit_scope_ok
                // This HWPX compatibility route uses the measured table height,
                // not the declared object height. Requiring the declared object to
                // cover every cell row here turns a fitting flowWithText=0 table
                // into an intra-row fragment solely because its source declaration
                // is not the height authority for this profile.
                && (hwpx_noninline_tac_measured_fit || declared_excess_has_source_frame)
                && !ft.strict_following_plain_text_fit
                && (!table.common.treat_as_char || hwpx_noninline_tac_measured_fit)
                && declared_object_total > host_spacing_total
                && st.current_height + declared_fit_height <= available);
        // 빈 host의 단일 inline 표에서 저장 LineSeg 높이와 table common 높이가
        // 정확히 같으면, 그 LineSeg는 표의 실제 physical frame이다. 누적 측정이
        // source top을 지나쳤더라도 frame 전체가 현재 body 안에 있으면 source
        // frame이 generic table_total보다 page owner를 우선한다.
        let saved_single_inline_table_source_frame = (table.common.treat_as_char
            && table.row_count == 1
            && table.col_count == 1
            && table.cells.len() == 1
            && para.controls.len() == 1
            && !para_has_visible_text(para)
            && ft.table_footnotes.is_empty())
        .then(|| {
            let mut source_lines = para
                .line_segs
                .iter()
                .filter(|seg| !is_synthetic_line_seg(seg));
            let seg = source_lines.next()?;
            if source_lines.next().is_some()
                || seg.line_height != table.common.height.min(i32::MAX as u32) as i32
            {
                return None;
            }
            line_seg_visible_bounds_px(seg, st.vpos_page_base.unwrap_or(0), self.dpi)
        })
        .flatten()
        .filter(|(source_top, source_bottom)| {
            *source_top < st.current_height && *source_bottom <= available
        });
        // 다행 RowBreak 표는 common.height가 첫 fragment만 뜻할 수도 있다. cell
        // 내부 reset 없이 다음 host가 새 물리 page를 명시할 때만, object frame을
        // 현 page 전체를 소유한 frame으로 쓴다.
        let saved_rowbreak_object_frame = ((st.profile.hwpx_container()
            || st.profile.native_hwp5_layout())
            && !table.common.treat_as_char
            && matches!(
                table.page_break,
                crate::model::table::TablePageBreak::RowBreak
            )
            && table.row_count > 1
            && para.controls.len() == 1
            && !para_has_visible_text(para)
            && ft.table_footnotes.is_empty()
            && signed_hwpunit(table.common.vertical_offset) <= 0
            && next_starts_new_page
            && !rowbreak_table_has_internal_saved_vpos_reset(table))
        .then(|| {
            let mut source_lines = para
                .line_segs
                .iter()
                .filter(|seg| !is_synthetic_line_seg(seg));
            let seg = source_lines.next()?;
            (source_lines.next().is_none())
                .then(|| line_seg_visible_bounds_px(seg, st.vpos_page_base.unwrap_or(0), self.dpi))
                .flatten()
        })
        .flatten()
        .and_then(|(source_top, _)| {
            let source_bottom = source_top + declared_object_total - host_spacing_total;
            let source_frame_matches_profile = if st.profile.hwpx_container() {
                source_top < st.current_height && source_bottom <= available
            } else {
                native_hwp5_saved_rowbreak_tail_frame_matches(
                    source_top,
                    source_bottom,
                    st.current_height,
                    available,
                )
            };
            source_frame_matches_profile.then_some((source_top, source_bottom))
        });
        let saved_table_source_frame =
            saved_single_inline_table_source_frame.or(saved_rowbreak_object_frame);
        WholeFit {
            para_has_stored_line_seg,
            single_row_object_height_advance,
            fits_after_overlay_shapes,
            native_hwp5_rewinding_rowbreak_uses_painted_row_footprint,
            whole_fit_table_total,
            hwpx_noninline_tac_measured_fit,
            declared_table_whole_fits,
            saved_table_source_frame,
        }
    }
}
