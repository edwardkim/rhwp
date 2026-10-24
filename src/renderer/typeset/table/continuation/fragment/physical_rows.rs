//! Stored rowspan minimums consume physical boxes accepted by earlier fragments.

use crate::renderer::typeset::{
    hwpunit_to_px, BlockTableContinuationPreparedState, BlockTableContinuationSource,
    TypesetEngine, TypesetState,
};

use super::super::TableContinuationCursor;
use super::FragmentInput;

pub(super) struct StoredFragmentEnd<'a> {
    pub(super) row: usize,
    pub(super) complete_cut: bool,
    pub(super) block_start: Option<usize>,
    pub(super) cut: &'a [usize],
    pub(super) first_height: Option<(usize, f64)>,
    pub(super) last_height: Option<f64>,
    pub(super) restart_frame_closes_body: bool,
}

pub(super) fn declared_row_height(table: &crate::model::table::Table, row: usize, dpi: f64) -> f64 {
    table
        .cells
        .iter()
        .filter(|cell| usize::from(cell.row) == row && cell.row_span <= 1)
        .map(|cell| hwpunit_to_px(cell.height as i32, dpi))
        .fold(0.0f64, f64::max)
}

/// 행의 칸들이 글줄 유닛 밖의 개체(표·도형·그림·수식·양식)를 품지 않는가. 컷까지의 내용 높이를 글줄로
/// 잴 수 있는 행에만 걸침 몫 규칙을 쓴다.
pub(super) fn row_cells_hold_only_text_lines(
    table: &crate::model::table::Table,
    row: usize,
) -> bool {
    use crate::model::control::Control;
    table
        .cells
        .iter()
        .filter(|cell| usize::from(cell.row) == row)
        .flat_map(|cell| cell.paragraphs.iter())
        .flat_map(|paragraph| paragraph.controls.iter())
        .all(|control| {
            !matches!(
                control,
                Control::Table(_)
                    | Control::Shape(_)
                    | Control::Picture(_)
                    | Control::Equation(_)
                    | Control::Form(_)
            )
        })
}

impl TypesetEngine {
    pub(super) fn stored_header_fragment_placement(
        &self,
        st: &TypesetState,
        input: FragmentInput<'_>,
        outer_top: f64,
    ) -> Option<crate::renderer::float_placement::ParagraphFloatPlacement> {
        if !input.start.is_continuation || !self.stored_header_physical_rows(st, input.source) {
            return None;
        }
        let top = st.current_height + outer_top;
        Some(crate::renderer::float_placement::ParagraphFloatPlacement {
            flow: crate::renderer::float_placement::ParagraphFloatFlow::NextLine,
            anchor_y: st.current_height,
            stored_host_origin: None,
            stored_successor_line_origin: None,
            table_left: None,
            table_top: top,
            occupied_bottom: top,
        })
    }

    pub(super) fn stored_header_fragment_row_budget(
        &self,
        st: &TypesetState,
        input: FragmentInput<'_>,
        available: f64,
        outer_top: f64,
    ) -> f64 {
        if !input.start.is_continuation || !self.stored_header_physical_rows(st, input.source) {
            return available;
        }
        // Paint closes a page-filling fragment at this same physical boundary.
        // Its accepted row boxes must exclude the unpainted bottom band before
        // a later closing span subtracts those boxes from its declared height.
        let bottom = crate::renderer::float_placement::single_cell_page_fragment_bottom(
            input.source.table,
            st.layout.body_area.height,
            self.dpi,
        );
        available.min(
            (bottom - st.current_zone_y_offset - st.layout.pagination_tolerance_px - outer_top)
                .max(0.0),
        )
    }

    pub(super) fn stored_header_physical_rows(
        &self,
        st: &TypesetState,
        source: BlockTableContinuationSource<'_>,
    ) -> bool {
        crate::renderer::float_placement::stored_repeated_header_rowbreak_outer_frame(
            st.profile,
            source.paragraph,
            source.table,
        ) && !self.render_normalization.table_text_reflowed(source.table)
    }

    /// 쪽 끝 내용 컷에 걸리는 병합 칸의 최소 높이 중 현재 조각이 소유하는 잔여다.
    /// 앞서 수용한 물리 행 상자와 현재 조각을 차감한다. 후속 행으로 이어지는 칸은
    /// 같은 행의 남은 내용과 뒤 행의 높이도 예약한다.
    /// 한/글은 걸침 칸의 총높이를 쪽을 넘어서도 지키며, 그 몫이 있으면 쪽 끝의 빈 띠를
    /// 이 행 조각이 차지한다(`21761835` 2쪽 행 20: 칸 (20,0) rs=5 252px, 한/글 2020 PDF
    /// 행 20 첫 조각 59px = 내용 51px + 본문 끝까지의 띠). 몫이 없으면 띠는 비어 남는다.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn crossing_span_page_end_band(
        &self,
        st: &TypesetState,
        cursor: &TableContinuationCursor,
        input: FragmentInput<'_>,
        cut: (usize, &[usize], &[usize]),
        current: f64,
        band: f64,
        fragment_rows: &[f64],
    ) -> Option<f64> {
        let (row, _, end_cut) = cut;
        let source = input.source;
        let table = source.row_geometry_table;
        let layout = &input.prepared.layout_engine;
        if band <= 0.5
            || table.common.treat_as_char
            || table.page_break != crate::model::table::TablePageBreak::RowBreak
            || !(self.stored_header_physical_rows(st, source)
                || layout.has_stored_continuous_spans(table))
        {
            return None;
        }
        if !row_cells_hold_only_text_lines(table, row) {
            return self.legacy_crossing_span_page_end_band(
                st,
                cursor,
                input,
                cut,
                current,
                band,
                fragment_rows,
            );
        }
        let cs = input.prepared.cell_spacing;
        // 한/글은 쪽 끝에서 자른 행의 이 쪽 상자를 max(컷까지 내용, 행 선언 높이, 걸침 칸마다 «선언 높이 −
        // 이 칸의 앞 행 상자»)로 닫고, 비끝 조각 닫는 선(`band` 상한)을 넘지 않는다. 앞 행 상자를 알 수 없는
        // 칸(시작 행이 이어받은 컷)은 판정에서 뺀다. 1613000 구역 1 끝 행 실측(한/글 2020): 139쪽 29.91
        // (걸침 칸 «7. ACAS» 68.37 − 행 48 38.40), 141쪽 17.12(선언), 167쪽 38.40(컷 내용 2줄), 152·154쪽 닫는 선.
        let remaining = layout.row_cut_content_height(table, row, end_cut, &[], source.styles);
        let whole_row = layout.row_cut_content_height(table, row, &[], &[], source.styles);
        // 나뉜 행 자신의 선언·온전한 높이도 쪽을 넘어 보존된다. 두 조각 내용 합이 그보다 작으면
        // 그 차이가 이 조각의 띠로 쓰일 수 있다(1342000 81·82쪽 행 14: 88 + 280 ≈ 366.9px).
        // 측정 행 높이에는 두 조각 사이의 줄간격·여백이 겹쳐 들어가므로 선언 높이로 잰다(1613000 2줄 행:
        // 측정 38.43 − 17.09 − 17.09 = 4.24 는 선언 여유가 아니다).
        let declared_whole = declared_row_height(table, row, self.dpi);
        let row_slack = declared_whole.max(0.0) - current - remaining;
        let _ = whole_row;
        let declared_row = declared_whole;
        let first_row = input.start.cursor_row;
        let span_share = table
            .cells
            .iter()
            .filter(|cell| {
                let first = usize::from(cell.row);
                cell.row_span > 1
                    && cell.height > 0
                    && cell.height < 0x8000_0000
                    && first <= row
                    && row + 1 < first + usize::from(cell.row_span)
            })
            .filter_map(|cell| {
                let first = usize::from(cell.row);
                // 이어받은 시작 행은 앞 조각과 이 조각에 걸친 한 행이다 — 그 행의 측정 높이를 앞 행 상자로 센다.
                let prefix = (first..row).try_fold(0.0, |sum, prior| {
                    let height = if prior < first_row {
                        cursor.accepted_row_boxes.get(prior).copied().flatten()?
                    } else if prior == first_row && !input.start.start_cut.is_empty() {
                        *source.measured_table.row_heights.get(prior)?
                    } else {
                        *fragment_rows.get(prior)?
                    };
                    Some(sum + height + cs)
                })?;
                Some(hwpunit_to_px(cell.height as i32, self.dpi) - prefix)
            })
            .fold(f64::NEG_INFINITY, f64::max);
        // 앞 행 상자를 아는 걸침 칸이 하나도 없으면 몫을 정할 수 없다 — 종전처럼 쪽 끝까지 준다.
        if span_share == f64::NEG_INFINITY {
            return Some(band);
        }
        let row_box = current
            .max(declared_row)
            .max(span_share)
            .max(current + row_slack);
        let slack = row_box - current;
        (slack > 0.5).then(|| band.min(slack))
    }

    /// 개체를 품은 행의 종전 띠 몫: 걸침 칸마다 «선언 − 앞 행 − 끝 행 하한 − 뒤 행 내용» 의 최솟값과 나뉜 행의
    /// 선언 보존 몫 중 큰 쪽(21761835 2쪽·1342000 81쪽 근거).
    #[allow(clippy::too_many_arguments)]
    fn legacy_crossing_span_page_end_band(
        &self,
        st: &TypesetState,
        cursor: &TableContinuationCursor,
        input: FragmentInput<'_>,
        cut: (usize, &[usize], &[usize]),
        current: f64,
        band: f64,
        fragment_rows: &[f64],
    ) -> Option<f64> {
        let (row, start_cut, end_cut) = cut;
        let source = input.source;
        let table = source.row_geometry_table;
        let layout = &input.prepared.layout_engine;
        let cs = input.prepared.cell_spacing;
        // 쪽 끝 행 자체도 선언·온전한 내용 높이를 하한으로 가진다(두 조각의 합이 그보다
        // 작으면 남은 조각이 그 차이를 받는다).
        let remaining = layout.row_cut_content_height(table, row, end_cut, &[], source.styles);
        let whole_row = layout.row_cut_content_height(table, row, &[], &[], source.styles);
        let row_floor = (current + remaining).max(whole_row);
        // 나뉜 행 자신의 선언·온전한 높이도 쪽을 넘어 보존된다. 두 조각 내용 합이 그보다 작으면
        // 그 차이가 이 조각의 띠로 쓰일 수 있다(1342000 81·82쪽 행 14: 88 + 280 ≈ 366.9px).
        let row_slack = whole_row - current - remaining;
        // 원본이 확인한 쪽 재시작에서는 남은 내용이 다음 물리 프레임의
        // 소유자다. 그 몫을 현재 병합 종료 상자의 빈 띠로 먼저 쓰지 않는다.
        let stored_frame_end =
            layout.row_cut_ends_at_stored_page_reset(table, row, start_cut, end_cut, source.styles);
        let first_row = input.start.cursor_row;
        let slack = table
            .cells
            .iter()
            .filter(|cell| {
                cell.row_span > 1
                    && cell.height > 0
                    && cell.height < 0x8000_0000
                    && crate::renderer::layout::cell_crosses_fragment_end(cell, row + 1, end_cut)
            })
            .map(|cell| {
                let first = usize::from(cell.row);
                let end = first + usize::from(cell.row_span);
                // 이 칸의 앞 행은 실제로 수용한 상자만 센다. 시작 행이 이어받은 컷이면
                // 그 앞 조각 몫을 알 수 없으므로 판정하지 않는다.
                let prefix = (first..row).try_fold(0.0, |sum, prior| {
                    let height = if prior < first_row {
                        cursor.accepted_row_boxes.get(prior).copied().flatten()?
                    } else if prior == first_row && !input.start.start_cut.is_empty() {
                        return None;
                    } else {
                        *fragment_rows.get(prior)?
                    };
                    Some(sum + height + cs)
                })?;
                let after = (row + 1..end)
                    .map(|later| {
                        layout.row_cut_content_height(table, later, &[], &[], source.styles) + cs
                    })
                    .sum::<f64>();
                // 이 행에서 끝나는 병합 칸의 잔여 최소 높이는 현재 조각에 속한다.
                // 일반 셀의 뒤쪽 내용 컷은 별도로 공간을 점유하며, 그 미래 점유를
                // 현재 병합 칸의 남은 최소 높이에서 미리 공제하면 종료 상자가 줄어든다.
                let owned_row_floor = if end == row + 1 && !stored_frame_end {
                    current
                } else {
                    row_floor
                };
                let slack = hwpunit_to_px(cell.height as i32, self.dpi)
                    - prefix - owned_row_floor - after;
                if std::env::var("RHWP_DIAG_SCAN").is_ok() {
                    eprintln!(
                        "DIAG_SPAN_BAND_OWNER sec={} pi={} row={} first={} end={} declared={:.6} prefix={:.6} current={:.6} row_floor={:.6} after={:.6} slack={:.6}",
                        st.section_index, source.para_index, row, first, end,
                        hwpunit_to_px(cell.height as i32, self.dpi), prefix,
                        current, owned_row_floor, after, slack,
                    );
                }
                Some(slack)
            })
            .try_fold(f64::INFINITY, |least, slack| Some(least.min(slack?)))
            .filter(|slack| slack.is_finite())
            .unwrap_or(f64::NEG_INFINITY);
        if std::env::var("RHWP_DIAG_SCAN").is_ok() {
            eprintln!(
                "DIAG_SPAN_BAND_RESULT sec={} pi={} row={} current={:.6} remaining={:.6} whole={:.6} row_slack={:.6} span_slack={:.6} available={:.6}",
                st.section_index, source.para_index, row, current, remaining,
                whole_row, row_slack, slack, band,
            );
            eprintln!(
                "DIAG_SPAN_BAND_BOUNDARY sec={} pi={} row={} cut={:?} original_reset={} zero_frame={} stored_reset={}",
                st.section_index, source.para_index, row, end_cut,
                layout.row_cut_ends_at_original_plain_text_reset(table, row, &[], end_cut, source.styles),
                layout.row_cut_ends_at_saved_zero_origin_frame(table, row, &[], end_cut, source.styles),
                layout.row_cut_ends_at_stored_page_reset(table, row, &[], end_cut, source.styles),
            );
        }
        let slack = slack.max(row_slack);
        (slack > 0.5).then(|| band.min(slack))
    }

    pub(super) fn stored_fragment_row_plan(
        &self,
        st: &TypesetState,
        prepared: &BlockTableContinuationPreparedState,
        cursor: &TableContinuationCursor,
        source: BlockTableContinuationSource<'_>,
        physical_start_cut: &[usize],
    ) -> Option<Vec<f64>> {
        let table = source.row_geometry_table;
        let layout = &prepared.layout_engine;
        if let Some(frame) = cursor
            .stored_block_frame
            .as_ref()
            .filter(|frame| cursor.start_cut_is_block && cursor.row == frame.start)
        {
            let rows = layout.stored_block_remaining_rows(
                table,
                (frame.start, frame.end),
                &frame.rows,
                frame.accepted,
                &cursor.start_cut,
                source.styles,
            );
            let mut heights = source.measured_table.row_heights.clone();
            heights[frame.start..frame.end].copy_from_slice(&rows);
            return Some(heights);
        }
        if cursor.start_cut_is_block {
            let (first, last, _) = source.measured_table.row_block_for(cursor.row);
            let end = layout.row_block_cut_complete_prefix_end(
                table,
                (first, last),
                &cursor.start_cut,
                source.styles,
            )?;
            // 내용 컷만으로 빈 물리 공간까지 끝났다고 추정하지 않는다.
            // 앞 emit이 실제로 수용한 온전한 행 상자가 있는 prefix만 제외한다.
            if first != cursor.row
                || !(first..end).all(|row| {
                    cursor
                        .accepted_row_boxes
                        .get(row)
                        .copied()
                        .flatten()
                        .is_some_and(|height| height.is_finite() && height > 0.0)
                })
            {
                return None;
            }
            let mut heights = source.measured_table.row_heights.clone();
            heights[first..end].fill(0.0);
            return Some(heights);
        }
        let keeps_source_boxes = self.stored_header_physical_rows(st, source)
            || prepared
                .layout_engine
                .has_stored_continuous_spans(source.row_geometry_table);
        if !keeps_source_boxes
            || (!cursor.accepted_row_boxes.iter().any(Option::is_some)
                && cursor.pending_row_box.is_none())
        {
            return None;
        }
        // 이 높이는 앞 emit이 생산해 시작 행 스캔과 paint에 그대로 넘긴
        // 물리 상자다. 앞 쪽 상자와 합쳐 이후 병합 종료 행의 요구를 잰다.
        let resumed_row_box = cursor.stored_restart_row_box.and_then(|(row, before)| {
            (row == cursor.row && !cursor.start_cut.is_empty())
                .then_some(cursor.start_row_height_override?)
                .map(|current| before + current)
        });
        // 일반 칸을 모두 소비했어도 걸침 칸의 남은 유닛 때문에 커서는 같은
        // 행에 머물 수 있다. 이때 앞 조각이 이미 수용한 행 상자를 병합 최소
        // 높이의 차감 원장에 포함하고, 미소비 일반 행의 추정값과 구분한다.
        let completed_start_row_box = (!cursor.start_cut.is_empty()
            && prepared.layout_engine.row_cut_consumes_all_ordinary_units(
                table,
                cursor.row,
                &cursor.start_cut,
                source.styles,
            ))
        .then(|| cursor.accepted_row_boxes.get(cursor.row).copied().flatten())
        .flatten();
        // 앞 조각이 저장 쪽 재시작 컷에서 쪽 끝 띠까지 수용한 행은 그 상자에 이 조각의 몫
        // (남은 내용과, 행의 온전한 높이에서 앞 상자를 뺀 값 중 큰 것)을 더한 것이 그 행의
        // 물리 상자다. 걸침의 끝 행은 그만큼을 뺀 선언 몫을 받는다.
        let span_carry_row_box = cursor
            .stored_reset_band_row_box
            .filter(|(row, _)| *row == cursor.row && !cursor.start_cut.is_empty())
            .map(|(row, before)| {
                let remaining = prepared.layout_engine.row_cut_content_height(
                    table,
                    row,
                    &cursor.start_cut,
                    &[],
                    source.styles,
                );
                let whole_row = prepared.layout_engine.row_cut_content_height(
                    table,
                    row,
                    &[],
                    &[],
                    source.styles,
                );
                before + remaining.max(whole_row - before)
            });
        let pending_row_box = cursor.pending_row_box.and_then(|(row, before)| {
            (row == cursor.row && !cursor.start_cut.is_empty()).then(|| {
                let content = prepared.layout_engine.row_cut_content_height(
                    table,
                    row,
                    &cursor.start_cut,
                    &[],
                    source.styles,
                );
                before
                    + cursor
                        .start_row_height_override
                        .map_or(content, |height| content.max(height))
            })
        });
        let start_row_box = pending_row_box
            .or(resumed_row_box)
            .or(completed_start_row_box)
            .or(span_carry_row_box);
        // A cut's start owner subtracts every preceding physical row of its
        // spanning cell. Do not mix accepted boxes with original estimates for
        // unknown partial rows: that can repeat already consumed source units.
        let prefix_known = |first: usize, end: usize| {
            (first..end).all(|row| {
                (row == cursor.row && start_row_box.is_some())
                    || cursor
                        .accepted_row_boxes
                        .get(row)
                        .is_some_and(Option::is_some)
            })
        };
        if table.cells.iter().any(|cell| {
            let first = usize::from(cell.row);
            first < cursor.row
                && cursor.row < first + usize::from(cell.row_span)
                && !prefix_known(first, cursor.row)
        }) {
            return None;
        }
        let original = &source.measured_table.row_heights;
        let mut heights = original.clone();
        for (row, accepted) in cursor
            .accepted_row_boxes
            .iter()
            .enumerate()
            .take(cursor.row)
        {
            if let (Some(height), Some(target)) = (accepted, heights.get_mut(row)) {
                *target = *height;
            }
        }
        if let Some(height) = start_row_box {
            heights[cursor.row] = height;
        }
        let accepted_prefix_end = cursor.row + usize::from(start_row_box.is_some());
        // 내용 컷 또는 일반 셀이 없는 잔여 상자만 현재 행의 부분 소유다.
        // 온전한 다음 행에도 이전 행의 빈 띠가 전달될 수 있으므로 override의
        // 존재만으로 건너뛰지 않는다. 이 행의 최종 요구를 뒤 병합 행도 소비한다.
        let first_row_is_partial = !cursor.start_cut.is_empty()
            || crate::renderer::layout::LayoutEngine::carried_span_row_physical_height(
                table,
                cursor.row,
                &cursor.start_cut,
                cursor.start_cut_is_block,
                cursor.start_row_height_override,
            )
            .is_some();
        let first_whole_row = cursor.row + usize::from(first_row_is_partial);
        // 원장의 시작 행은 이전 조각까지 합한 온전한 높이다. 현재 조각의
        // 미소비 줄 요구에서는 같은 컷으로 다시 연 실제 시작 상자만 뺀다.
        let fragment_start_height =
            (!cursor.start_cut_is_block && first_row_is_partial).then(|| {
                layout.ordinary_rowbreak_fragment_heights(
                    table,
                    &heights,
                    cursor.row,
                    (cursor.row + 1).min(heights.len()),
                    physical_start_cut,
                    &[],
                    true,
                    cursor.start_row_height_override,
                    None,
                    &[],
                    &cursor.straddle_page_bands,
                    true,
                    source.styles,
                )[cursor.row]
            });
        for row in first_whole_row..heights.len() {
            // Only a closing span whose preceding physical boxes changed can
            // invalidate the globally measured remainder. Other rows keep their
            // original measurement, including content-driven growth.
            let closing: Vec<_> = table
                .cells
                .iter()
                .filter(|cell| {
                    cell.row_span > 1
                        && cell.height < 0x8000_0000
                        && cell.row as usize + cell.row_span as usize == row + 1
                        && prefix_known(usize::from(cell.row), accepted_prefix_end.min(row))
                })
                .collect();
            // 앞 조각이 수용한 상자뿐 아니라 이 루프가 방금 다시 푼 행도 걸침의 남은
            // 요구를 바꾼다.
            if !closing.iter().any(|cell| {
                let first = cell.row as usize;
                (first..row).any(|prior| (heights[prior] - original[prior]).abs() > 0.5)
            }) {
                continue;
            }
            // Whole-row content and single-row declarations provide the floor.
            // A row with no single-row owner is the unknown row of the spans that
            // close on it: its height is what those spans still require after the
            // accepted prefix. 한/글은 걸침 칸의 총높이를 쪽을 넘어서도 지킨다 —
            // `21761835` 표: 칸 (6,0) rs=5 462.6px 가 1쪽에서 449.1px 를 썼으므로
            // 2쪽 행 10 은 13.6px, 행 10~11 칸(105.9px)의 행 11 은 92.3px 다
            // (한/글 2020 PDF 2쪽 행 10~11 106px, 행 17 138px).
            let has_single_row_owner = table
                .cells
                .iter()
                .any(|cell| cell.row as usize == row && cell.row_span == 1);
            let mut floor = if has_single_row_owner {
                prepared
                    .layout_engine
                    .row_cut_content_height(table, row, &[], &[], source.styles)
            } else {
                0.0
            };
            // 단일 행 셀이 없어도 이어받은 병합 셀의 빈 문단이 줄 상자를
            // 소유할 수 있다. paint의 공통 요구를 같은 컷/셀 원장으로 소비한다.
            if !cursor.start_cut_is_block {
                if let Some(demand) = layout.straddle_continuation_demand(
                    table,
                    row,
                    cursor.row,
                    physical_start_cut,
                    false,
                    cursor.start_row_height_override,
                    &heights,
                    &cursor.straddle_page_bands,
                    source.styles,
                    (row + 1, true),
                ) {
                    let mut prefix = heights[cursor.row..row].iter().sum::<f64>()
                        + prepared.cell_spacing * (row - cursor.row) as f64;
                    if row > cursor.row {
                        if let Some(first) = fragment_start_height {
                            prefix += first - heights[cursor.row];
                        }
                    }
                    floor = floor.max((demand.required() - prefix).max(0.0));
                }
            }
            let required = closing.iter().fold(floor, |height, cell| {
                let first = cell.row as usize;
                let used = heights[first..row].iter().sum::<f64>()
                    + prepared.cell_spacing * (row - first) as f64;
                height.max(hwpunit_to_px(cell.height as i32, self.dpi) - used)
            });
            heights[row] = required;
        }
        if std::env::var("RHWP_TABLE_DRIFT").is_ok() {
            eprintln!(
                "TABLE_FRAGMENT_ROWS: pi={} cursor={} original={:?} accepted={:?} resolved={:?}",
                source.para_index, cursor.row, original, cursor.accepted_row_boxes, heights
            );
        }
        Some(heights)
    }

    pub(super) fn record_stored_fragment_rows(
        &self,
        st: &TypesetState,
        cursor: &mut TableContinuationCursor,
        input: FragmentInput<'_>,
        end: StoredFragmentEnd<'_>,
    ) {
        let complete_block_prefix =
            end.block_start
                .zip(end.last_height)
                .is_some_and(|(first, height)| {
                    let (_, last, _) = input.source.measured_table.row_block_for(first);
                    height.is_finite()
                        && height > 0.0
                        && input
                            .prepared
                            .layout_engine
                            .row_block_cut_complete_prefix_end(
                                input.source.row_geometry_table,
                                (first, last),
                                end.cut,
                                input.source.styles,
                            )
                            == Some(end.row)
                });
        let keeps_source_boxes = self.stored_header_physical_rows(st, input.source)
            || input
                .prepared
                .layout_engine
                .has_stored_continuous_spans(input.source.row_geometry_table);
        if (!keeps_source_boxes && !complete_block_prefix) || input.start.start_cut_is_block {
            return;
        }
        let heights = input
            .fragment_row_heights
            .unwrap_or(&input.source.measured_table.row_heights);
        let first = input.start.cursor_row
            + usize::from(
                !input.start.start_cut.is_empty()
                    || input.start.start_row_height_override.is_some(),
            );
        // A partial end block can contain several rows. None of those boxes
        // is a whole-row measurement. Without an explicit closing height the
        // final row can also fold during paint; its physical box is unknown.
        let whole_end = if complete_block_prefix {
            end.row
        } else {
            end.block_start.unwrap_or_else(|| {
                if end.complete_cut && end.last_height.is_some() {
                    end.row
                } else {
                    end.row.saturating_sub(1)
                }
            })
        };
        cursor.accepted_row_boxes.resize(heights.len(), None);
        // 부분 상자만으로 완료 행이라고 선언하지 않는다. 다음 행으로 실제
        // 전진한 일반 스캔의 첫 상자가 있어야 앞 조각의 합을 확정한다.
        if let Some((row, before)) = cursor.pending_row_box {
            if row == input.start.cursor_row
                && !input.start.start_cut.is_empty()
                && (row + 1 < end.row || row + 1 == end.row && end.complete_cut)
            {
                if let Some((_, current)) = end
                    .first_height
                    .filter(|(owner, height)| *owner == row && height.is_finite() && *height >= 0.0)
                {
                    let current = if row + 1 == end.row {
                        end.last_height.unwrap_or(current)
                    } else {
                        current
                    };
                    cursor.accepted_row_boxes[row] = Some(before + current);
                    if std::env::var("RHWP_TABLE_DRIFT").is_ok() {
                        eprintln!("TABLE_CLOSED_PARTIAL_ROW: pi={} row={} before={:.6} current={:.6} total={:.6}",
                            input.source.para_index, row, before, current, before + current);
                    }
                    cursor.pending_row_box = None;
                    cursor.stored_restart_row_box = None;
                }
            }
        }
        if let Some((row, before)) = cursor.stored_restart_row_box {
            if row == input.start.cursor_row && row + 1 < end.row {
                if let Some(current) = input.start.start_row_height_override {
                    cursor.accepted_row_boxes[row] = Some(before + current);
                    cursor.stored_restart_row_box = None;
                }
            }
        }
        // 아직 끝내지 않은 행의 컷은 완료 행으로 기록하지 않는다. 원본
        // 본문 프레임에서 실제 확정한 상자만 다음 같은 행에 전달한다.
        if end.restart_frame_closes_body && !end.complete_cut && end.block_start.is_none() {
            if let (Some(row), Some(height)) = (end.row.checked_sub(1), end.last_height) {
                let before = cursor
                    .stored_restart_row_box
                    .filter(|&(owner, _)| owner == row)
                    .map_or(0.0, |(_, height)| height);
                cursor.stored_restart_row_box = Some((row, before + height));
            }
        }
        // 같은 행을 다시 나눴으면 수용한 상자만 더한다. 내용 컷은 별도로
        // 이어받으며, 이 누적을 블록 높이나 원본 행 추정값으로 채우지 않는다.
        if !end.complete_cut && end.block_start.is_none() && !end.cut.is_empty() {
            if let (Some(row), Some(height)) = (end.row.checked_sub(1), end.last_height) {
                if height.is_finite() && height >= 0.0 {
                    let before = cursor
                        .pending_row_box
                        .filter(|(owner, _)| *owner == row)
                        .map_or(0.0, |(_, height)| height);
                    cursor.pending_row_box = Some((row, before + height));
                    if std::env::var("RHWP_TABLE_DRIFT").is_ok() {
                        eprintln!("TABLE_PENDING_ROW: pi={} row={} before={:.6} current={:.6} total={:.6}",
                            input.source.para_index, row, before, height, before + height);
                    }
                }
            }
        }
        if first >= whole_end {
            return;
        }
        if std::env::var("RHWP_TABLE_DRIFT").is_ok() {
            eprintln!(
                "TABLE_ACCEPTED_ROWS: pi={} rows={}..{} last={:?} measured={:?}",
                input.source.para_index, first, whole_end, end.last_height, heights
            );
        }
        for (row, &height) in heights.iter().enumerate().take(whole_end).skip(first) {
            cursor.accepted_row_boxes[row] = Some(if row + 1 == end.row {
                end.last_height.unwrap_or(height)
            } else {
                height
            });
        }
    }
}
