//! A genuine first fragment owns its declared physical space, not the next row.

use crate::renderer::typeset::{BlockTableRowScan, TypesetEngine, TypesetState};

use super::{FragmentBudget, FragmentInput};

pub(super) struct StoredPartialFragment {
    pub(super) height: f64,
    pub(super) complete_last_row: Option<(usize, f64)>,
}

impl TypesetEngine {
    pub(super) fn stored_body_closing_fragment_extent(
        &self,
        st: &TypesetState,
        input: FragmentInput<'_>,
        budget: &FragmentBudget,
        scan: &BlockTableRowScan,
    ) -> Option<(f64, f64)> {
        if !input.start.is_continuation
            || !input.start.stored_fragment_closes_body
            || input.start.start_cut_is_block
            || input.row_cursor_is_nested
            || st.current_height > 0.5
            || st.current_zone_y_offset != 0.0
            || scan.end_row <= input.start.cursor_row
            || scan.split_block_start.is_some()
            || scan.split_end_limit <= 0.0
            || scan.split_end_cut.is_empty()
            || input.source.table.caption.is_some()
        {
            return None;
        }
        let row = scan.end_row - 1;
        let first = input.start.cursor_row;
        let engine = &input.prepared.layout_engine;
        let table = input.source.row_geometry_table;
        let styles = input.source.styles;
        let start = if row == first {
            input.start.start_cut.as_slice()
        } else {
            &[]
        };
        if !engine.row_has_original_plain_text_owners(table, row, &scan.split_end_cut)
            || !(engine.row_cut_ends_at_original_plain_text_reset(
                table,
                row,
                start,
                &scan.split_end_cut,
                styles,
            ) || engine.row_cut_ends_at_saved_zero_origin_frame(
                table,
                row,
                start,
                &scan.split_end_cut,
                styles,
            ))
        {
            return None;
        }
        let top = budget.fragment_placement.map_or(
            st.current_height + budget.host_before_overhead + budget.vert_offset_overhead,
            |placement| placement.table_top,
        );
        let bottom = budget.source_frame_bottom?;
        let height = bottom - top;
        let roundoff = f64::EPSILON * (height.abs() + budget.avail_for_rows.abs()).max(1.0) * 4.0;
        if height <= 0.0 || height > budget.avail_for_rows + budget.header_overhead + roundoff {
            return None;
        }
        let measured = input.source.measured_table;
        let before = if row == first {
            0.0
        } else {
            if !(first + 1..row)
                .all(|owner| engine.whole_fragment_row_uses_measured_height(table, owner))
            {
                return None;
            }
            let opening = if let Some(height) = input.start.start_row_height_override {
                height
            } else if input.start.start_cut.is_empty() {
                *measured.row_heights.get(first)?
            } else {
                if !engine.row_has_original_plain_text_owners(table, first, &input.start.start_cut)
                {
                    return None;
                }
                engine.row_cut_content_height(table, first, &input.start.start_cut, &[], styles)
            };
            opening
                + measured
                    .row_heights
                    .get(first + 1..row)?
                    .iter()
                    .sum::<f64>()
                + measured.cell_spacing * (row - first) as f64
        } + budget.header_overhead;
        let last = height - before;
        let selected = scan.end_row_height_override.unwrap_or(scan.split_end_limit);
        (last >= selected).then_some((height, last))
    }

    pub(super) fn stored_partial_fragment_complete_extent(
        &self,
        input: FragmentInput<'_>,
        budget: &FragmentBudget,
        scan: &BlockTableRowScan,
    ) -> Option<(f64, f64)> {
        let height = budget.stored_partial_frame_height?;
        if input.start.is_continuation
            || input.start.cursor_row != 0
            || !input.start.start_cut.is_empty()
            || scan.end_row == 0
            || scan.end_row >= input.prepared.row_count
            || height <= scan.consumed + budget.header_overhead + 0.5
        {
            return None;
        }
        // Content ownership is already final. The authenticated object box
        // also owns its unused band; preserve the selected row minimum rather
        // than inferring another row or shortening an accepted line advance.
        let (before_last, selected_last) = if let Some(block_start) = scan.split_block_start {
            self.stored_complete_block_prefix(input, scan, block_start, height)?
        } else {
            let row = scan.end_row - 1;
            let selected_last = if scan.split_end_cut.is_empty() && scan.split_end_limit <= 0.0 {
                scan.end_row_height_override
                    .unwrap_or(*input.prepared.whole_row_fit_heights.get(row)?)
            } else {
                // 미완료 내용 컷도 독립적으로 인증한 첫 물리 상자의 빈 띠를 소유한다.
                // 실제 일반 행 소유자와 저장 경계를 확인하고, 스캔이 선택한
                // 유닛을 늘리지 않은 채 같은 행 상자의 물리 점유만 확장한다.
                let table = input.source.row_geometry_table;
                let engine = &input.prepared.layout_engine;
                let styles = input.source.styles;
                if scan.split_end_limit <= 0.0
                    || scan.split_end_cut.is_empty()
                    || !engine.row_has_original_plain_text_owners(table, row, &scan.split_end_cut)
                    || !(engine.row_cut_ends_at_original_plain_text_reset(
                        table,
                        row,
                        &[],
                        &scan.split_end_cut,
                        styles,
                    ) || engine.row_cut_ends_at_saved_zero_origin_frame(
                        table,
                        row,
                        &[],
                        &scan.split_end_cut,
                        styles,
                    ))
                {
                    return None;
                }
                scan.end_row_height_override.unwrap_or(scan.split_end_limit)
            };
            let before_last = if !scan.split_end_cut.is_empty() {
                let engine = &input.prepared.layout_engine;
                let table = input.source.row_geometry_table;
                if !(0..row)
                    .all(|owner| engine.whole_fragment_row_uses_measured_height(table, owner))
                {
                    return None;
                }
                // 앞의 완전 행은 측정과 paint가 함께 쓰는 실제 물리 상자다.
                // 내용 스캔에 들어간 줄 전진/패딩을 이 상자에 다시 합산하지 않는다.
                let measured = input.source.measured_table;
                measured.row_heights.get(..row)?.iter().sum::<f64>()
                    + measured.cell_spacing * row as f64
            } else {
                scan.consumed - selected_last
            };
            (before_last, selected_last)
        };
        let last_height = height - budget.header_overhead - before_last;
        (before_last >= 0.0 && last_height >= selected_last).then_some((height, last_height))
    }

    fn stored_complete_block_prefix(
        &self,
        input: FragmentInput<'_>,
        scan: &BlockTableRowScan,
        block_start: usize,
        height: f64,
    ) -> Option<(f64, f64)> {
        let table = input.source.row_geometry_table;
        let styles = input.source.styles;
        let engine = &input.prepared.layout_engine;
        let measured = input.source.measured_table;
        let (first, end, _) = measured.row_block_for(block_start);
        if first != block_start || scan.end_row <= first || scan.end_row >= end {
            return None;
        }
        let source_height = engine.stored_complete_block_opening_frame_height(
            table,
            input.source.paragraph,
            scan.end_row,
            &scan.split_end_cut,
            styles,
        )?;
        if height != source_height {
            return None;
        }
        // A block cut can encode a complete physical row without an empty
        // vector. Verify the exact measured-row owner used by paint; the unit
        // walk's consumed height alone does not include this row's blank band.
        if !(0..scan.end_row).all(|row| {
            if row < first {
                engine.whole_fragment_row_uses_measured_height(table, row)
            } else {
                engine.row_block_cut_uses_measured_height(
                    table,
                    (first, end),
                    row,
                    &[],
                    &scan.split_end_cut,
                    styles,
                )
            }
        }) {
            return None;
        }
        let last = scan.end_row - 1;
        let before = measured.row_heights.get(..last)?.iter().sum::<f64>()
            + measured.cell_spacing * last as f64;
        let selected = scan
            .end_row_height_override
            .unwrap_or(*measured.row_heights.get(last)?);
        Some((before, selected))
    }

    pub(super) fn stored_partial_fragment_height(
        &self,
        st: &TypesetState,
        input: FragmentInput<'_>,
        saved_frame: Option<(f64, f64)>,
        declared_height_is_credible: bool,
        available: f64,
    ) -> Option<StoredPartialFragment> {
        let source = input.source;
        if !(st.profile.hwp5_stored_pagination_layout() || st.profile.hwpx_stored_layout())
            || st.profile.session_edited()
            || source.paragraph.stored_text_partition_is_dirty()
            || source.paragraph.cell_format_vpos_dirty
            || self.render_normalization.table_text_reflowed(source.table)
            || !input
                .prepared
                .layout_engine
                .stored_row_boxes_exceed_first_frame(source.row_geometry_table)
        {
            return None;
        }
        // The existing saved-frame producer authenticates the host origin and
        // available body space. The raw row lower bound independently proves
        // this object box is only a fragment. Its units still need a real cut.
        let height = saved_frame?.0.min(available);
        let mut before = 0.0;
        let mut complete_last_row = None;
        for (row, row_height) in input.prepared.cut_row_heights.iter().enumerate() {
            if row > 0 {
                before += input.prepared.cell_spacing;
            }
            if before + row_height > height {
                let remaining = height - before;
                // A declared row can include unused space. All of its units
                // may fit in the source remainder even when that box does not.
                // The complete-row minimum covers owners starting in this row.
                // A spanning owner can still owe units when its span ends here;
                // absence of a span after the cut does not prove completion.
                let active: Vec<_> = source
                    .row_geometry_table
                    .cells
                    .iter()
                    .filter(|cell| {
                        usize::from(cell.row) <= row
                            && row < usize::from(cell.row) + usize::from(cell.row_span)
                    })
                    .collect();
                let ordinary_owners = !active.is_empty()
                    && active
                        .iter()
                        .all(|cell| usize::from(cell.row) == row && cell.row_span == 1);
                // A Top-aligned plain row still owns its declared blank tail.
                // Keep its original box so the normal RowCut can carry that
                // space after all content is consumed. Pictures and aligned
                // source boxes retain their complete measured-frame contract.
                let ordinary_blank_tail = ordinary_owners
                    && crate::renderer::typeset::table::scan::row::ordinary_band_row_shape(
                        source.row_geometry_table,
                        row,
                        *row_height,
                        self.dpi,
                    )
                    && active
                        .iter()
                        .all(|cell| cell.vertical_align == crate::model::table::VerticalAlign::Top);
                let complete_owners = if ordinary_owners {
                    !ordinary_blank_tail
                } else {
                    !active.is_empty()
                        && input
                            .prepared
                            .layout_engine
                            .stored_complete_source_row_content_fits(
                                source.row_geometry_table,
                                row,
                                input.start.cursor_row,
                                &input.prepared.cut_row_heights,
                                remaining,
                                source.styles,
                            )
                };
                let content = input
                    .prepared
                    .layout_engine
                    .row_complete_cut_content_height(source.row_geometry_table, row, source.styles);
                // 저장 HU에서 정확히 닫히는 상자도 내용+안여백과 앞 행 차감의
                // f64 합산 순서가 다르면 끝자리만 초과할 수 있다. 실제 공간
                // 초과를 수용하지 않고, 같은 프레임 산술의 roundoff만 구분한다.
                let roundoff = f64::EPSILON
                    * (content.abs() + height.abs() + before.abs()).max(1.0)
                    * (row + active.len() + 3) as f64;
                let fits = content <= remaining
                    || (content.is_finite()
                        && remaining.is_finite()
                        && content - remaining <= roundoff);
                // 완전 행의 압축에는 기존 행 경계의 신뢰 근거를 유지한다.
                // 행 내부 조각의 엄격한 상한은 저장 앵커/본문 예산에서 인증하고,
                // 빈 띠의 소유는 스캔 뒤 실제 원본 부분 컷에서 따로 확인한다.
                if declared_height_is_credible && complete_owners && fits {
                    complete_last_row = Some((row, remaining));
                }
                break;
            }
            before += row_height;
        }
        Some(StoredPartialFragment {
            height,
            complete_last_row,
        })
    }
}
