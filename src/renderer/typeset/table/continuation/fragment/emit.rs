//! 수용한 표 조각, 대기 중인 주석, 커서 이동을 원래 순서대로 반영한다.

use crate::renderer::typeset::{
    hwpunit_to_px, is_synthetic_line_seg, row_geometry_table, table, BlockTableRowScan, PageItem,
    TableContinuationIteration, TypesetEngine, TypesetState, VisibleFloatExclusion,
    MIN_TOP_KEEP_PX,
};

use super::super::TableContinuationCursor;
use super::{FragmentBudget, FragmentInput, FragmentProfile};

impl TypesetEngine {
    pub(super) fn emit_table_fragment(
        &self,
        st: &mut TypesetState,
        continuation: &mut TableContinuationCursor,
        input: FragmentInput<'_>,
        budget: &FragmentBudget,
        scan: BlockTableRowScan,
    ) -> TableContinuationIteration {
        let para_idx = input.source.para_index;
        let ctrl_idx = input.source.control_index;
        let table = input.source.table;
        let row_geometry_table = input.source.row_geometry_table;
        let mt = input.source.measured_table;
        let styles = input.source.styles;
        let row_count = input.prepared.row_count;
        let can_intra_split = input.prepared.can_intra_split;
        let layout_engine = &input.prepared.layout_engine;
        let cut_row_h = input
            .start
            .relieved_cut_row_heights
            .as_deref()
            .or(input.fragment_row_heights)
            .unwrap_or(&input.prepared.cut_row_heights);
        let straddle_row_relief = &input.start.straddle_row_relief;
        let straddle_page_bands = &input.start.straddle_page_bands;
        let caption_is_top = input.prepared.caption_is_top;
        let caption_overhead = input.prepared.caption_overhead;
        let queue_table_footnotes = input.prepared.queue_table_footnotes;
        let table_footnotes = &input.prepared.table_footnotes;
        let host_spacing_total = input.prepared.host_spacing_total;
        let terminal_host_spacing = input.prepared.terminal_host_spacing;
        let relax_terminal_table_footnote_fit = input.prepared.relax_terminal_table_footnote_fit;
        let cursor_row = input.start.cursor_row;
        let is_continuation = input.start.is_continuation;
        let start_cut_is_block = input.start.start_cut_is_block;
        let start_row_height_override = input.start.start_row_height_override;
        let start_cut = &input.start.start_cut;
        let row_cursor_is_nested = input.row_cursor_is_nested;
        let FragmentBudget {
            caption_extra,
            host_before_overhead,
            mut terminal_outer_bottom_overhead,
            fragment_outer_bottom_overhead,
            vert_offset_overhead,
            page_avail,
            mut fragment_placement,
            header_overhead,
            avail_for_rows,
            single_cell_fragment_shape,
            single_cell_box_height,
            saved_first_fragment_source_frame,
            source_first_fragment_row_end,
            ..
        } = *budget;
        let stored_complete_extent = self
            .stored_partial_fragment_complete_extent(input, budget, &scan)
            .or_else(|| self.stored_body_closing_fragment_extent(st, input, budget, &scan));
        let BlockTableRowScan {
            source_frame_band,
            consumed,
            mut end_row,
            split_block_start,
            split_end_cut,
            split_end_limit,
            mut end_row_height_override,
        } = scan;
        let source_first_row_frame =
            layout_engine.saved_first_row_opening_frame(table, styles, st.layout.body_area.height);
        let source_opening_row_frame = (!is_continuation
            && cursor_row == 0
            && start_cut.is_empty()
            && end_row == 1
            && split_block_start.is_none())
        .then_some(source_first_row_frame.as_ref())
        .flatten()
        .filter(|(cut, height, _)| {
            cut == &split_end_cut && end_row_height_override == Some(*height)
        });
        // 컷은 내용을 소비하지만 셀의 선언 최소 높이는 빈 물리 공간도 소유한다.
        // 앞 내용 컷으로 높이를 추정하지 않고 실제로 수용한 상자를 이어받는다.
        let stored_row_frame = end_row.checked_sub(1).and_then(|row| {
            if split_end_limit <= 0.0
                || split_end_cut.is_empty()
                || split_block_start.is_some()
                || source_opening_row_frame.is_some()
            {
                return None;
            }
            // 프레임 소유는 배치와 같은 예약 전 원본 본문 높이를 사용한다.
            // 각주와 영역은 아래의 수용 예산을 제한하며 원본 최소 높이를 없애지 않는다.
            let declared = layout_engine
                .stored_full_width_row_declared_height(
                    table,
                    row,
                    styles,
                    st.layout.body_area.height,
                )
                .or_else(|| {
                    (row == 0
                        && continuation
                            .stored_row_box_sum
                            .is_some_and(|(owner, _)| owner == row))
                    .then_some(source_first_row_frame.as_ref())
                    .flatten()
                    .map(|(_, opening, remaining)| opening + remaining)
                })?;
            let used = continuation
                .stored_row_box_sum
                .filter(|(owner, _)| *owner == row)
                .map_or(0.0, |(_, height)| height);
            let row_start = if row == cursor_row {
                start_cut.as_slice()
            } else {
                &[]
            };
            let current_content =
                layout_engine.row_cut_content_height(table, row, row_start, &split_end_cut, styles);
            let accepted_scan_height = end_row_height_override.unwrap_or(current_content);
            let before_row = (consumed - accepted_scan_height).max(0.0);
            let available = (avail_for_rows - before_row).max(0.0);
            // 스캐너의 덮어쓴 높이는 수용한 내용 컷만 나타낼 수 있다. 이는 하한이며
            // 저장 행의 빈 물리 띠까지 예약했다는 증거가 아니다. 조각의 가용 공간에
            // 그 띠를 배정하고 실제 상자를 원장에 남긴다.
            // 뒤 내용 컷은 앞쪽에 이미 수용한 물리 상자가 아니다. 특히 저장 쪽의
            // 재시작은 높은 원본 프레임 안에 가운데 정렬 그림을 남길 수 있다.
            // 미래 내용 높이 대신 실제로 배치한 상자만 차감한다.
            let source_space = (declared - used).max(0.0).min(available);
            let height = accepted_scan_height.max(current_content).max(source_space);
            Some((row, used, height, accepted_scan_height))
        });
        // 행 컷이 소비한 내용과 header의 요구 높이. 저장 상자의 빈 밴드는
        // 아래에서 별도로 물리 점유에 포함하며 컷 유닛을 더 소비하지 않는다.
        let mut partial_height: f64 =
            consumed + header_overhead + hwpunit_to_px(table.outer_cell_spacing_hu(), self.dpi);
        if let Some((height, last_height)) = stored_complete_extent {
            partial_height = height;
            end_row_height_override = Some(last_height);
        }
        if let Some((_, _, height, current_content)) = stored_row_frame {
            if height > current_content + 0.5 {
                end_row_height_override = Some(height);
                partial_height += height - current_content;
            }
        }
        if std::env::var("RHWP_DIAG_SCAN").is_ok() {
            eprintln!(
                "DIAG_SCAN FRAGMENT rows={}..{} continuation={} block={:?} cut={:?} consumed={:.3} frame={:?} override={:?}",
                cursor_row, end_row, is_continuation, split_block_start, split_end_cut,
                consumed, stored_row_frame, end_row_height_override
            );
        }
        // 원본 셀의 저장 쪽 0에서 재개하는 새 단은 실제 좌표축을 가진다.
        // 후속 TAC가 이 증거를 잃고 누적 좌표를 임의의 0 기준으로 읽지 않게 한다.
        let resumed_stored_page_frame = is_continuation
            && st.current_items.is_empty()
            && st.current_height == 0.0
            && !start_cut_is_block
            && !row_cursor_is_nested
            && start_cut.len() == 1
            && start_cut[0] > 0
            && table.row_count == 1
            && table.col_count == 1
            && table.cells.len() == 1
            && !st.profile.session_edited()
            && (st.profile.hwpx_stored_layout() || st.profile.hwp5_stored_pagination_layout())
            && !self.render_normalization.table_text_reflowed(table)
            && table_footnotes.is_empty()
            && layout_engine.cell_unit_stored_page_frame_origin(
                &table.cells[0],
                table,
                styles,
                start_cut[0],
            ) == Some(0);
        if resumed_stored_page_frame {
            st.record_vpos_page_origin(Some(0));
            st.record_vpos_origin_provenance(true);
        }
        // 종료 조각의 빈 저장 밴드도 같은 물리 행 높이로 예약·배치한다.
        // 실제 컷이 새 원본 쪽 프레임에서 시작한 경우에만 후속 저장 하단을 쓴다.
        let single_cell_closing_frame_bottom = (is_continuation
            && cursor_row == 0
            && end_row == 1
            && split_end_cut.is_empty()
            && start_cut.len() == 1
            && start_cut[0] > 0
            && !st.profile.session_edited()
            && (st.profile.hwpx_stored_layout() || st.profile.hwp5_stored_pagination_layout())
            && !self.render_normalization.table_text_reflowed(table)
            && table_footnotes.is_empty()
            && table.cells.first().is_some_and(|cell| {
                layout_engine.cell_unit_opens_stored_page_frame(cell, table, styles, start_cut[0])
            }))
        .then(|| {
            input
                .source
                .paragraphs_all
                .get(para_idx + 1)
                .and_then(|next| {
                    crate::renderer::float_placement::stored_single_cell_closing_frame_bottom(
                        input.source.paragraph,
                        next,
                        table,
                        self.dpi,
                    )
                })
        })
        .flatten();
        // 저장 하단은 단 기준 원점이다. 지금 조각의 위 여백·원점과
        // 반복 머리행은 같은 상자 안에 있으며 행 높이로 중복 예약하지 않는다.
        let fragment_top = st.current_height + host_before_overhead + vert_offset_overhead;
        if let Some(height) = single_cell_closing_frame_bottom
            .map(|bottom| bottom - fragment_top)
            .filter(|height| {
                *height >= partial_height && *height <= avail_for_rows + header_overhead
            })
        {
            end_row_height_override = Some(height - header_overhead);
            partial_height = height;
        }
        // #7095의 동일한 쪽 하단 상자를 저장 rowspan 경계에도 적용한다.
        // 내용 컷과 물리 빈 밴드를 분리하며 다음 조각은 실제 남은 행에서 재개한다.
        let stored_rowspan_frame = (is_continuation
            && start_cut.is_empty()
            && end_row_height_override.is_none()
            && header_overhead == 0.0)
            .then(|| {
                let frame_height =
                    crate::renderer::float_placement::single_cell_page_fragment_bottom(
                        table,
                        st.available_height(),
                        self.dpi,
                    ) - st.current_height
                        - host_before_overhead
                        - vert_offset_overhead;
                (frame_height >= partial_height && frame_height <= avail_for_rows)
                    .then(|| {
                        layout_engine.stored_rowspan_page_frame(
                            table,
                            cursor_row..end_row,
                            split_block_start,
                            &split_end_cut,
                            styles,
                            (frame_height, start_row_height_override),
                            mt,
                        )
                    })
                    .flatten()
                    .map(|frame| (frame, frame_height))
            })
            .flatten();
        if let Some(((last_height, _, _, _), frame_height)) = &stored_rowspan_frame {
            end_row_height_override = Some(*last_height);
            partial_height = *frame_height;
        }

        // 저장 첫 조각의 상자는 내용 컷만으로 표현되지 않는 빈 하단 밴드도 소유한다.
        // 뒤 조각의 유닛은 그대로 남기며, 이 밴드를 내용 tail에서 차감하지 않는다.
        let saved_block_opening_frame = (!is_continuation
            && cursor_row == 0
            && start_cut.is_empty())
        .then(|| split_block_start.filter(|row| *row + 1 == end_row))
        .flatten()
        .and_then(|row| {
            layout_engine.saved_block_reset_opening_frame_height(table, row, &split_end_cut, styles)
        });
        let saved_opening_frame = layout_engine
            .saved_multirow_opening_frame_height(
                table,
                cursor_row,
                end_row,
                start_cut,
                &split_end_cut,
                styles,
            )
            .or(saved_block_opening_frame);
        // 마지막 행의 빈 물리 밴드는 다음 문단 원점과 전체 저장 행합으로 입증한다.
        // 첫 프레임과 종료 프레임은 같은 선언 공간을 나누며 내용 컷은 그대로 보존한다.
        let saved_closing_frame = (!st.profile.session_edited()
            && (st.profile.hwpx_stored_layout() || st.profile.hwp5_stored_pagination_layout())
            && !self.render_normalization.table_text_reflowed(table)
            && end_row == row_count
            && table_footnotes.is_empty())
        .then(|| {
            input
                .source
                .paragraphs_all
                .get(para_idx + 1)
                .and_then(|next| {
                    crate::renderer::float_placement::stored_rowbreak_closing_frame_height(
                        input.source.paragraph,
                        next,
                        table,
                        self.dpi,
                    )
                })
        })
        .flatten();
        // 종료 행의 원본 프레임은 현재 컷의 내용 높이와 독립적으로 닫힌다.
        // 마지막 한 행의 이어받기일 때 동일 높이를 예약과 paint에 전달한다.
        if is_continuation
            && cursor_row + 1 == end_row
            && split_end_cut.is_empty()
            && start_cut.iter().any(|&cut| cut > 0)
        {
            if let Some(height) = saved_closing_frame
                .filter(|height| *height >= partial_height && *height <= avail_for_rows)
            {
                end_row_height_override = Some(height);
                partial_height = height;
            }
        }
        // 원본 누적 좌표가 닫는 첫 물리 프레임은 내용 컷과 별도로 소유한다.
        // 첫 조각과 이어받기 조각에 같은 행 높이 경계를 전달한다.
        let cumulative_opening_frame = (!is_continuation
            && cursor_row == 0
            && start_cut.is_empty()
            && !split_end_cut.is_empty()
            && split_end_limit > 0.0
            && end_row_height_override.is_none()
            && split_block_start.is_none()
            && table_footnotes.is_empty()
            && !st.profile.session_edited()
            && (st.profile.hwpx_stored_layout() || st.profile.hwp5_stored_pagination_layout())
            && !self.render_normalization.table_text_reflowed(table)
            && std::ptr::eq(table, row_geometry_table))
        .then(|| {
            input.source.paragraphs_all.get(para_idx + 1).and_then(|next| {
                crate::renderer::float_placement::stored_cumulative_rowbreak_opening_frame_height(
                    input.source.paragraph,
                    next,
                    table,
                    self.dpi,
                )
            })
        })
        .flatten();
        if let Some(frame_height) = cumulative_opening_frame {
            let before_last = cut_row_h
                .iter()
                .take(end_row.saturating_sub(1))
                .sum::<f64>()
                + mt.cell_spacing * end_row.saturating_sub(2) as f64;
            let last_height = frame_height - before_last;
            if last_height >= split_end_limit - 0.5
                && last_height < cut_row_h[end_row - 1]
                && frame_height <= avail_for_rows + header_overhead
            {
                end_row_height_override = Some(last_height);
                partial_height = frame_height;
            }
        }
        let mut source_frame_trailing_trim_applied = false;
        // 저장 첫 프레임의 마지막 글줄 뒤 간격은 다음 물리 쪽에 속한다.
        // 컷 유닛은 그대로 두고, 모든 셀의 가시 내용이 저장 상자에 들어갈 때만
        // 마지막 행의 그리기 높이를 원본 프레임에 맞춘다.
        if !is_continuation
            && cursor_row == 0
            && start_cut.is_empty()
            && !split_end_cut.is_empty()
            && end_row_height_override.is_none()
            && table_footnotes.is_empty()
            && !st.profile.session_edited()
            && !self.render_normalization.table_text_reflowed(table)
            && std::ptr::eq(table, row_geometry_table)
            && source_first_fragment_row_end == Some(end_row)
        {
            if let (Some(block_start), Some((frame_height, _))) =
                (split_block_start, saved_first_fragment_source_frame)
            {
                if frame_height < partial_height
                    && frame_height <= avail_for_rows
                    && layout_engine.saved_first_frame_block_cut_fits(
                        table,
                        block_start,
                        end_row,
                        &split_end_cut,
                        cut_row_h,
                        mt.cell_spacing,
                        frame_height,
                        styles,
                    )
                {
                    let before_last = cut_row_h
                        .iter()
                        .take(end_row.saturating_sub(1))
                        .sum::<f64>()
                        + mt.cell_spacing * end_row.saturating_sub(2) as f64;
                    let last_height = frame_height - before_last;
                    if last_height > 0.0 {
                        end_row_height_override = Some(last_height);
                        partial_height = frame_height;
                        source_frame_trailing_trim_applied = true;
                    }
                }
            }
        }
        // 문단 내부 원점0만으로는 쪽 경계를 입증하지 못한다. 일반 저장 리셋이 없으면
        // 호스트 뒤 원문의 쪽 원점 되감김으로 닫힌 두 줄 프레임을 확인한다.
        let opening_frame_has_source_boundary =
            layout_engine.row_cut_ends_at_original_plain_text_reset(
                table,
                end_row.saturating_sub(1),
                start_cut,
                &split_end_cut,
                styles,
            ) || layout_engine.row_cut_ends_at_native_saved_page_frame(
                table,
                end_row.saturating_sub(1),
                start_cut,
                &split_end_cut,
                styles,
            ) || layout_engine.row_cut_starts_intra_paragraph_stored_frame(
                table,
                end_row.saturating_sub(1),
                &split_end_cut,
                styles,
            ) || input
                .source
                .paragraphs_all
                .get(para_idx + 1)
                .is_some_and(|next| {
                    let host = input.source.paragraph;
                    matches!((host.line_segs.as_slice(), next.line_segs.as_slice()), ([a], [b])
                    if !host.stored_text_partition_is_dirty()
                        && !next.stored_text_partition_is_dirty()
                        && !host.cell_format_vpos_dirty && !next.cell_format_vpos_dirty
                        && (a.tag | b.tag)
                            & crate::model::paragraph::LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0
                        && b.vertical_pos >= 0 && b.vertical_pos < a.vertical_pos)
                });
        // 본문을 닫는 원본 상자는 폭0 앵커의 구역 설정과 무관하게
        // 첫 물리 프레임과 이어받는 행의 선언 잔여를 함께 소유한다.
        let body_filling_source_frame =
            crate::renderer::float_placement::stored_body_filling_rowbreak_frame(
                input.source.paragraph,
                table,
                st.layout.body_area.height,
                self.dpi,
                st.profile.hwpx_stored_layout() || st.profile.hwp5_stored_pagination_layout(),
                st.profile.session_edited(),
            );
        // 선언된 셀 최소 높이는 대체값이며, 저장된 첫 개체 프레임과 구분한다.
        // 유효성이 확인된 프레임으로 예약과 실제 그리기의 대체값을 함께 교체한다.
        // 스캔이 소유한 상자나 확인된 완전한 점유 범위는 자체 소유권을 유지하므로 여기서 교체하지 않는다.
        let declared_row_minimum_owns_override = stored_row_frame.is_some()
            && stored_complete_extent.is_none()
            && !source_frame_trailing_trim_applied;
        // 선택한 컷이 들어가더라도 대체 최소 높이는 저장 프레임보다 클 수 있다.
        // 프레임은 스캔이 수용한 내용 요구 높이와 비교한다. 대체 상자를 새 내용 요구 높이로 사용하지 않는다.
        let opening_frame_required_height = if declared_row_minimum_owns_override {
            consumed + header_overhead
        } else {
            partial_height
        };
        let first_fragment_blank_band = !is_continuation
            && (opening_frame_has_source_boundary
                || saved_block_opening_frame.is_some()
                || body_filling_source_frame)
            && (split_block_start.is_none() || saved_block_opening_frame.is_some())
            && (end_row_height_override.is_none() || declared_row_minimum_owns_override)
            && std::ptr::eq(table, row_geometry_table)
            && (crate::renderer::float_placement::object_only_saved_table_anchor(
                input.source.paragraph,
                table,
            ) || body_filling_source_frame
                || saved_closing_frame.is_some()
                || saved_block_opening_frame.is_some()
                || (crate::renderer::float_placement::stored_rowbreak_outer_frame_host(
                    input.source.paragraph,
                    table,
                ) && layout_engine.row_cut_starts_intra_paragraph_stored_frame(
                    table,
                    end_row - 1,
                    &split_end_cut,
                    styles,
                )))
            && saved_opening_frame.is_some_and(|frame_height| {
                (frame_height >= opening_frame_required_height
                    || saved_block_opening_frame.is_some())
                    && frame_height <= avail_for_rows + header_overhead
            });
        // 빈 밴드가 저장 되감김 앞 줄간격을 뺀 만큼만 생긴 경우인가. 그 밴드는 원래 컷이
        // 차지하던 간격이라, 이어받기로 넘길 때 갈린 칸의 남은 내용을 함께 확인한다.
        let blank_band_within_reset_trim = first_fragment_blank_band
            && saved_opening_frame.is_some_and(|frame_height| {
                let trim = end_row.checked_sub(1).map_or(0.0, |row| {
                    layout_engine.row_cut_stored_reset_trim(
                        table,
                        row,
                        if row == cursor_row {
                            start_cut.as_slice()
                        } else {
                            &[]
                        },
                        &split_end_cut,
                        styles,
                    )
                });
                trim > 0.5 && frame_height <= partial_height + trim + 0.5
            });
        if first_fragment_blank_band {
            let frame_height = saved_opening_frame.expect("accepted saved opening frame");
            let before_last = cut_row_h
                .iter()
                .take(end_row.saturating_sub(1))
                .sum::<f64>()
                + mt.cell_spacing * end_row.saturating_sub(2) as f64;
            end_row_height_override = Some((frame_height - before_last).max(0.0));
            partial_height = frame_height;
        }
        // 첫 조각 빈 밴드가 이미 저장 프레임으로 닫았으면 그 경로가 상자와 이어받기를 소유한다.
        if !is_continuation
            && !first_fragment_blank_band
            && cursor_row == 0
            && start_cut.is_empty()
            && !split_end_cut.is_empty()
            && end_row_height_override.is_none()
            && table_footnotes.is_empty()
            && !st.profile.session_edited()
            && !self.render_normalization.table_text_reflowed(table)
            && std::ptr::eq(table, row_geometry_table)
        {
            // 저장 되감김으로 끝나는 첫 조각은 한/글이 저장한 첫 프레임(`common.height`)이 그
            // 조각의 물리 상자다. 컷 예산은 되감김 앞 줄간격을 빼고 잡지만, 상자는 그 프레임을
            // 그대로 차지한다 — 렌더도 같은 프레임을 그린다(hwpctl 문단 176: 프레임 105.05,
            // 간격을 뺀 내용 102.4; 편람 부록 행 5: 프레임 550.5, 정본 표 아래 괘선 644.7).
            if split_block_start.is_none() {
                // 1×1 표는 저장 표 높이가 곧 첫 조각 프레임이다(#7095 첫 조각 상자 계약).
                let first_frame = saved_first_fragment_source_frame
                    .map(|(height, _)| height)
                    .or_else(|| {
                        (table.row_count == 1 && table.col_count == 1 && table.cells.len() == 1)
                            .then(|| hwpunit_to_px(table.common.height as i32, self.dpi))
                    });
                if let Some(frame_height) = first_frame {
                    let last_row = end_row.saturating_sub(1);
                    // 저장 프레임이 컷 상자보다 큰 몫은 컷이 뺀 되감김 앞 줄간격 안이어야 한다.
                    // 그보다 크면 프레임은 이 컷 하나의 상자가 아니다(hwpctl 문단 176: 차 2.6 ≤
                    // 줄간격, 편람 부록 행 5: 차 4.6 ≤ 6.3).
                    let reset_trim = layout_engine.row_cut_stored_reset_trim(
                        table,
                        last_row,
                        if last_row == cursor_row {
                            start_cut.as_slice()
                        } else {
                            &[]
                        },
                        &split_end_cut,
                        styles,
                    );
                    // 본문을 채우는 원본 프레임은 첫 조각 빈 밴드 경로가 상자와 이어받기를 함께
                    // 소유한다(form-002 hwpx 7→8쪽: 이어받는 행 상자 = 저장 행 − 첫 프레임).
                    // 같은 저장 구조면 원본 형식과 무관하다(form-002 hwp·hwpx 쌍둥이 정본이 같다).
                    if !body_filling_source_frame
                        && frame_height > partial_height + 0.5
                        && frame_height <= partial_height + reset_trim + 0.5
                        && frame_height <= avail_for_rows + 0.5
                        && layout_engine.row_cut_ends_at_stored_page_reset(
                            table,
                            last_row,
                            if last_row == cursor_row {
                                start_cut.as_slice()
                            } else {
                                &[]
                            },
                            &split_end_cut,
                            styles,
                        )
                    {
                        let before_last = cut_row_h.iter().take(last_row).sum::<f64>()
                            + mt.cell_spacing * last_row.saturating_sub(1) as f64;
                        let last_height = frame_height - before_last;
                        if last_height > 0.0 {
                            end_row_height_override = Some(last_height);
                            partial_height = frame_height;
                            source_frame_trailing_trim_applied = true;
                        }
                    }
                }
            }
        }
        if (stored_complete_extent.is_some() || first_fragment_blank_band)
            && fragment_placement.is_none()
        {
            // 유효성이 확인된 첫 개체 상자는 원점과 사용하지 않은 띠를 함께 소유한다.
            // 표 앞에 제목 줄이 있는 경우에도 같은 소유 관계를 적용한다.
            let top =
                st.current_height + caption_extra + host_before_overhead + vert_offset_overhead;
            fragment_placement = Some(crate::renderer::float_placement::ParagraphFloatPlacement {
                flow: crate::renderer::float_placement::ParagraphFloatFlow::NextLine,
                anchor_y: st.current_height,
                stored_host_origin: None,
                stored_successor_line_origin: None,
                table_left: None,
                table_top: top,
                occupied_bottom: top,
            });
        }
        // 종료 조각의 실제 프레임과 뒤 저장 줄이 아래 여백을 닫으면 동일한
        // 배치 계획으로 예약과 paint 흐름을 함께 전진시킨다.
        if is_continuation
            && end_row >= row_count
            && split_end_limit == 0.0
            && fragment_placement.is_none()
            // [#5585] 끝 조각 뒤 흐름을 중첩 자식의 저장 빈 Enter(`terminal_host_spacing`)가
            // 소유하면 그 간격이 이미 표 아래를 닫는다. 바깥 아래 여백을 따로 열면 둘을
            // 겹쳐 더한다 — 한글 2024 PDF 86712 26쪽: 조각 아래 괘선 145.76px 뒤 다음 표
            // 193.87px 은 Enter 간격만 더한 위치다(여백까지 더하면 +1.9px).
            && terminal_host_spacing <= 0.0
            && (st.profile.hwpx_stored_layout() || st.profile.hwp5_stored_pagination_layout())
            && !st.profile.session_edited()
            && st.col_count == 1
            && !self.render_normalization.table_text_reflowed(table)
        {
            let top = st.current_height + host_before_overhead + vert_offset_overhead;
            if let Some(margin) = input
                .source
                .paragraphs_all
                .get(para_idx + 1)
                .and_then(|next| {
                    crate::renderer::float_placement::stored_terminal_rowbreak_outer_margin_px(
                        input.source.paragraph,
                        next,
                        table,
                        top + partial_height,
                        self.dpi,
                    )
                })
                .filter(|margin| top + partial_height + margin <= st.base_available_height())
            {
                terminal_outer_bottom_overhead = terminal_outer_bottom_overhead.max(margin);
                fragment_placement =
                    Some(crate::renderer::float_placement::ParagraphFloatPlacement {
                        flow: crate::renderer::float_placement::ParagraphFloatFlow::NextLine,
                        anchor_y: st.current_height,
                        stored_host_origin: None,
                        stored_successor_line_origin: Some(top + partial_height + margin),
                        table_left: None,
                        table_top: top,
                        occupied_bottom: top + partial_height + margin,
                    });
            }
        }
        // 원본 첫 상자가 마지막 줄의 뒤 간격 안에서 끝나면 모든 컷 유닛을 보존하고
        // 예약과 paint가 같은 선언 물리 프레임을 소비한다.
        let two_frame_successor_origin =
            input
                .source
                .paragraphs_all
                .get(para_idx + 1)
                .and_then(|next| {
                    crate::renderer::float_placement::stored_two_frame_successor_origin_hu(
                        table, next,
                    )
                });
        let original_single_cell_opening_frame = !is_continuation
            && cursor_row == 0
            && start_cut.is_empty()
            && end_row == 1
            && row_count == 1
            && split_end_cut.len() == 1
            && !st.profile.session_edited()
            && (st.profile.hwpx_stored_layout() || st.profile.hwp5_stored_pagination_layout())
            && !self.render_normalization.table_text_reflowed(table)
            && table_footnotes.is_empty()
            && two_frame_successor_origin.is_some()
            && input
                .source
                .paragraphs_all
                .get(para_idx + 1)
                .is_some_and(|next| {
                    crate::renderer::float_placement::stored_single_cell_closing_frame_height(
                        input.source.paragraph,
                        next,
                        table,
                        self.dpi,
                    )
                    .is_some()
                })
            && table.cells.first().is_some_and(|cell| {
                layout_engine.cell_unit_opens_stored_page_frame(
                    cell,
                    table,
                    styles,
                    split_end_cut[0],
                )
            })
            && hwpunit_to_px(table.common.height as i32, self.dpi) >= partial_height
            && hwpunit_to_px(table.common.height as i32, self.dpi) <= avail_for_rows;
        let declared_opening_frame = (single_cell_fragment_shape
            && two_frame_successor_origin.is_some()
            && !is_continuation
            && cursor_row == 0
            && start_cut.is_empty()
            && end_row == row_count
            && split_end_cut.len() == 1
            && split_end_limit > 0.0
            && end_row_height_override.is_none()
            && !st.profile.session_edited()
            && !self.render_normalization.table_text_reflowed(table)
            && table_footnotes.is_empty()
            && layout_engine.stored_cut_closes_declared_opening_frame(
                &table.cells[0],
                table,
                styles,
                split_end_cut[0],
            ))
            || original_single_cell_opening_frame;
        if declared_opening_frame {
            partial_height = hwpunit_to_px(table.common.height as i32, self.dpi);
            end_row_height_override = Some(partial_height);
        }
        // 종료 상자의 마지막 유닛 뒤 빈 공간은 누적 상자와 잔여의 합이 cellSz를 닫고
        // 다음 원본 줄이 전체 바깥 상자를 닫을 때만 수용한다. 같은 계획이 후속 간격을 소유한다.
        if single_cell_fragment_shape
            && two_frame_successor_origin.is_some()
            && is_continuation
            && end_row >= row_count
            && split_end_limit == 0.0
            && end_row_height_override.is_none()
            && continuation.single_cell_box_sum_px > 0.0
            && st.profile.hwp5_stored_pagination_layout()
            && !st.profile.session_edited()
            && st.col_count == 1
            && !self.render_normalization.table_text_reflowed(table)
            && table.caption.is_none()
            && fragment_placement.is_none()
        {
            let cell = &table.cells[0];
            let successor = input.source.paragraphs_all.get(para_idx + 1);
            let closure = successor.and_then(|next| {
                let [line] = next.line_segs.as_slice() else {
                    return None;
                };
                if next.stored_text_partition_is_dirty()
                    || is_synthetic_line_seg(line)
                    || !next.controls.is_empty()
                    || !next.text.trim().is_empty()
                    || line.vertical_pos <= 0
                    || cell.height > i32::MAX as u32
                    || !start_cut.first().is_some_and(|&start| {
                        layout_engine.stored_cut_closes_declared_opening_frame(
                            cell,
                            table,
                            input.source.styles,
                            start,
                        )
                    })
                {
                    return None;
                }
                let top = st.current_height + host_before_overhead + vert_offset_overhead;
                let bottom_margin = hwpunit_to_px(i32::from(table.outer_margin_bottom), self.dpi);
                let next_origin = hwpunit_to_px(line.vertical_pos, self.dpi);
                let physical_height = next_origin - top - bottom_margin;
                let remaining = hwpunit_to_px(cell.height as i32, self.dpi)
                    - continuation.single_cell_box_sum_px;
                (physical_height >= partial_height
                    && next_origin <= st.available_height()
                    && (physical_height - remaining).abs() <= 2.0 * self.dpi / 7200.0)
                    .then_some((top, physical_height, bottom_margin, next_origin))
            });
            if let Some((top, height, margin, next_origin)) = closure {
                partial_height = height;
                end_row_height_override = Some(height);
                terminal_outer_bottom_overhead = margin;
                fragment_placement =
                    Some(crate::renderer::float_placement::ParagraphFloatPlacement {
                        flow: crate::renderer::float_placement::ParagraphFloatFlow::NextLine,
                        anchor_y: st.current_height,
                        stored_host_origin: None,
                        stored_successor_line_origin: Some(next_origin),
                        table_left: None,
                        table_top: top,
                        occupied_bottom: next_origin,
                    });
            }
        }
        // 끝 조각의 원본 조회는 스캔 이후 여백을 예약할 수 있다. 최종 수용한
        // 예약에서 아직 소유되지 않은 흐름의 남은 몫을 계산한다.
        // 끝 조각의 물리 상자와 아래 바깥여백은 같은 영역을 소유한다.
        // 엄격 예산·캡션 등이 이미 예약한 몫을 다시 더하지 않는다.
        let terminal_flow_outer_bottom =
            (input.prepared.terminal_outer_bottom_flow - terminal_outer_bottom_overhead).max(0.0);
        let host_spacing_after_only =
            input.prepared.host_spacing_after_only + terminal_flow_outer_bottom;
        let commit_fragment = |st: &mut TypesetState, owner_height: f64, terminal: bool| {
            if let Some(mut placement) = fragment_placement {
                // 끝이 아닌 컷은 다음 내용을 소유할 수 없다. 끝 조각의 원본
                // 원점은 유지하되, 간격을 소비하는 경로에서는 그 원점이
                // 최종 수용된 물리 하단과 일치해야 한다.
                if !terminal {
                    placement.stored_successor_line_origin = None;
                }
                placement.occupied_bottom = placement.table_top
                    + owner_height
                    + if terminal {
                        terminal_outer_bottom_overhead + terminal_flow_outer_bottom
                    } else {
                        // 실제로 예약한 조각 뒤 여백만 후속 흐름에 전달한다.
                        // 바깥 위 여백의 재개가 아래 여백 재개까지 뜻하지는 않는다.
                        fragment_outer_bottom_overhead
                    };
                if terminal
                    && terminal_outer_bottom_overhead > 0.0
                    && st.prefilled_line_prefixes.contains_key(&(para_idx + 1))
                {
                    placement.flow = crate::renderer::float_placement::ParagraphFloatFlow::NextLine;
                }
                st.record_paragraph_float_placement((para_idx, ctrl_idx), placement);
                st.align_flow_to(
                    placement.occupied_bottom
                        + if terminal {
                            input.prepared.host_spacing_after_only
                        } else {
                            0.0
                        },
                );
                if terminal {
                    st.add_visible_float_exclusion(VisibleFloatExclusion {
                        para_index: para_idx,
                        top: placement.table_top,
                        bottom: placement.occupied_bottom,
                    });
                }
            }
        };

        // [Task #1046 Stage 2 진단] walk 결과 — fragment 경계/소비 높이. 동작 불변.
        if std::env::var("RHWP_TABLE_DRIFT").is_ok() {
            eprintln!(
                "TABLE_SPLIT_RESULT: pi={} sec={} cursor_row={} end_row={} consumed={:.1} partial_h={:.1} split_end_limit={:.1} avail_for_rows={:.1} fits={}",
                para_idx, st.section_index, cursor_row, end_row, consumed, partial_height,
                split_end_limit, avail_for_rows, consumed <= avail_for_rows + 0.1,
            );
        }

        if end_row >= row_count && split_end_limit == 0.0 {
            let skip_terminal_empty_sliver = is_continuation
                // 재조판한 선언 꼬리는 마지막 내용 유닛 뒤에도 실제 물리 공간을 소유한다.
                // 이 프레임을 빈 잔여 조각으로 판단해 제거하지 않는다.
                && !(start_row_height_override.is_some()
                    && layout_engine.row_uses_reflow_physical_frame(table, cursor_row))
                && !start_cut.is_empty()
                && !start_cut_is_block
                && mt.allows_row_break_split()
                && caption_overhead <= 0.5
                && partial_height < MIN_TOP_KEEP_PX
                && (cursor_row..end_row).all(|r| {
                    let su: &[usize] = if r == cursor_row { &start_cut } else { &[] };
                    !layout_engine.row_cut_range_has_visible_content(
                        row_geometry_table,
                        r,
                        su,
                        &[],
                        styles,
                    )
                });
            if skip_terminal_empty_sliver {
                continuation.finish(row_count, false);
                return TableContinuationIteration::Complete;
            }

            // 나머지 전부가 현재 페이지에 들어감
            let bottom_caption_extra = if !caption_is_top {
                caption_overhead
            } else {
                0.0
            };
            // [#7095] 끝 조각 상자 = max(내용, 저장 칸 높이 − 앞 조각 상자 합).
            // 7062 10쪽: 저장 9346.35 − 앞 조각 8472.3 = 874.0 (정본 874.04), 내용 850.2.
            // 앞 조각이 이미 저장 높이를 넘은 표(148738070, 1382000 `pi=90`)는 음수라 불변이다.
            // 근거는 `valign=Center` 칸뿐이다(7062, 1382000 `pi=93/95/99`). `valign=Top` 칸은
            // 정본 상자 합이 저장 높이와 맞지 않는다(148738070: 합 4177px ↔ 저장 2529px).
            let center_cell = table.cells.first().is_some_and(|cell| {
                matches!(
                    cell.vertical_align,
                    crate::model::table::VerticalAlign::Center
                )
            });
            if single_cell_fragment_shape
                && center_cell
                && is_continuation
                && continuation.single_cell_box_sum_px > 0.0
                && end_row_height_override.is_none()
            {
                let stored_cell_px = table.cells.first().map_or(0.0, |cell| {
                    hwpunit_to_px(cell.height.min(i32::MAX as u32) as i32, self.dpi)
                });
                let remainder = stored_cell_px - continuation.single_cell_box_sum_px;
                // 상한: 다음 문단이 한/글 저장 자리(첫 줄 vpos)보다 내려가지 않게 한다. 저장 칸
                // 높이가 상자 합이 아닌 표(rowbreak-problem-pages `pi=13`: 다음 문단이 이미
                // 저장 자리 0.9px 안)는 늘리지 않는다. 다음 문단이 없거나 다음 쪽에서 되감긴
                // 표는 한/글 자리를 모르므로 늘리지 않는다.
                let next_para_room =
                    input
                        .prepared
                        .next_para_stored_top
                        .and_then(|(vpos, spacing_before)| {
                            let stored_px = hwpunit_to_px(
                                vpos.saturating_sub(st.vpos_page_base.unwrap_or(0)),
                                self.dpi,
                            );
                            let projected_px = st.current_height
                                + host_before_overhead
                                + vert_offset_overhead
                                + partial_height
                                + terminal_outer_bottom_overhead
                                + host_spacing_after_only
                                + terminal_host_spacing
                                + spacing_before;
                            (stored_px > st.current_height).then_some(stored_px - projected_px)
                        });
                let extension =
                    next_para_room.map_or(0.0, |room| (remainder - partial_height).min(room));
                if extension > 0.5 {
                    end_row_height_override = Some(partial_height + extension);
                    partial_height += extension;
                }
            }
            if cursor_row == 0
                && !is_continuation
                && start_cut.is_empty()
                // 마지막 행의 물리 높이를 바꾼 결과는 조각 배치에서 소비한다.
                // 통째 표로 되돌리면 원래 행 높이가 복원되어 예약 하단을 넘는다.
                && end_row_height_override.is_none()
            {
                st.append_item(PageItem::Table {
                    para_index: para_idx,
                    control_index: ctrl_idx,
                });
                st.advance_flow_by(partial_height + host_spacing_total);
            } else {
                self.record_stored_fragment_rows(
                    st,
                    continuation,
                    input,
                    super::physical_rows::StoredFragmentEnd {
                        row: end_row,
                        complete_cut: split_end_cut.is_empty() && split_end_limit <= 0.0,
                        block_start: split_block_start,
                        last_height: end_row_height_override,
                        restart_frame_closes_body: false,
                    },
                );
                st.append_item(PageItem::PartialTable {
                    para_index: para_idx,
                    control_index: ctrl_idx,
                    start_row: cursor_row,
                    end_row,
                    is_continuation,
                    start_cut: continuation.start_cut.clone(),
                    end_cut: Vec::new(),
                    // 기존 블록 조각 게이트를 유지한다. 시작 컷의 공간은 별도 필드가 소유한다.
                    is_block_split: start_cut_is_block,
                    start_cut_is_block,
                    row_cursor_is_nested,
                    end_row_height_override,
                    start_row_height_override,
                    straddle_row_relief: straddle_row_relief.clone(),
                    straddle_page_bands: straddle_page_bands.clone(),
                    fragment_row_heights: input.fragment_row_heights.map(<[f64]>::to_vec),
                    end_row_owns_stored_frame: false,
                });
                // 마지막 fragment: spacing_after만 포함 (Paginator engine.rs:1051 동일)
                // host line advance/positive offset은 원 anchor 조각의 계약이며,
                // continuation 끝에서 다시 더하면 다음 본문을 이중으로 민다(#2439).
                let occupied = host_before_overhead
                    + vert_offset_overhead
                    + partial_height
                    + bottom_caption_extra
                    + terminal_outer_bottom_overhead;
                let terminal_spacing = is_continuation
                    .then(|| st.table_flow_successor(para_idx, input.source.paragraphs_all))
                    .flatten()
                    .and_then(|(_, next)| {
                        layout_engine.native_text_frame_terminal_successor_gap(
                            input.source.paragraph,
                            next,
                            table,
                            &st.inline_flow_column(),
                            st.inline_flow_column().y + st.current_height + occupied,
                            styles,
                        )
                    })
                    .unwrap_or(host_spacing_after_only + terminal_host_spacing);
                if std::env::var_os("RHWP_DIAG_SCAN").is_some() {
                    eprintln!("DIAG_SCAN FLOW_END pi={para_idx} row={cursor_row} cont={is_continuation} current={:.3} occupied={occupied:.3} terminal_spacing={terminal_spacing:.3} outer={} terminal_flow={:.3} reserved={terminal_outer_bottom_overhead:.3} host_tags={:?} next_line={:?}", st.current_height, table.outer_margin_bottom, input.prepared.terminal_outer_bottom_flow, input.source.paragraph.line_segs.iter().map(|line| line.tag).collect::<Vec<_>>(), input.source.paragraphs_all.get(para_idx + 1).and_then(|para| para.line_segs.first()).map(|line| (line.vertical_pos, line.tag)));
                }
                st.advance_flow_by(occupied + terminal_spacing);
                self.consume_stored_terminal_empty_line_gap(st, &input, is_continuation);
            }
            commit_fragment(
                st,
                caption_extra + partial_height + bottom_caption_extra,
                true,
            );
            if queue_table_footnotes {
                self.register_queued_table_footnotes(
                    st,
                    continuation,
                    table_footnotes,
                    table,
                    input.source.paragraphs_all,
                    styles,
                    layout_engine,
                    &[],
                    para_idx,
                    ctrl_idx,
                    cursor_row,
                    end_row,
                    start_cut,
                    true,
                    relax_terminal_table_footnote_fit && is_continuation,
                    false,
                );
                // terminal fragment에 들어가지 못한 URL 각주는 새 page의 footer
                // lane에 먼저 예약한다. 다음 본문은 그 reservation을 보고 같은
                // page에 fit하거나 필요할 때만 다음 page로 분할된다.
                while continuation.pending_table_footnote_fragment.is_some()
                    || continuation.next_table_footnote < table_footnotes.len()
                {
                    let before = (
                        continuation.next_table_footnote,
                        continuation.pending_table_footnote_fragment.is_some(),
                    );
                    st.force_new_page();
                    self.register_queued_table_footnotes(
                        st,
                        continuation,
                        table_footnotes,
                        table,
                        input.source.paragraphs_all,
                        styles,
                        layout_engine,
                        &[],
                        para_idx,
                        ctrl_idx,
                        cursor_row,
                        end_row,
                        start_cut,
                        true,
                        relax_terminal_table_footnote_fit && is_continuation,
                        true,
                    );
                    st.request_vpos_reset_after_queued_footnote();
                    let after = (
                        continuation.next_table_footnote,
                        continuation.pending_table_footnote_fragment.is_some(),
                    );
                    debug_assert!(
                        after != before,
                        "fresh page must accept one queued table footnote or pending tail"
                    );
                    if after == before {
                        break;
                    }
                }
            }
            continuation.finish(row_count, true);
            return TableContinuationIteration::Complete;
        }

        // 최종 행의 컷이 모든 가시 유닛을 소비했다면 다음 조각은 없다.
        // 컷을 지우면 원래 행 높이가 복원되므로 paint 컷은 그대로 보존하고,
        // 빈 후속 페이지를 할당하기 전에 continuation만 종료한다.
        let terminal_cut_consumed = input.closes_terminal_content_cut(
            end_row,
            split_end_limit,
            &split_end_cut,
            split_block_start,
            end_row_height_override,
        );

        // 스캔이 마지막 행 상자에 이미 예약한 빈 띠도 이어받기 장부에 싣는다.
        // 아래에서 새로 확장하는 띠와 구분해 같은 물리 공간을 두 번 더하지 않는다.
        let reserved_straddle_end_band = if split_end_limit == 0.0
            && split_end_cut.is_empty()
            && split_block_start.is_none()
            && !start_cut_is_block
        {
            layout_engine.reserved_straddle_end_band(
                row_geometry_table,
                &mt.row_heights,
                cursor_row,
                end_row,
                start_cut,
                is_continuation,
                start_row_height_override,
                end_row_height_override,
                straddle_row_relief,
                straddle_page_bands,
                styles,
            )
        } else {
            0.0
        };

        // 완료 행 경계를 가로지르는 원본 병합 셀은 끝 물리 쪽의 띠도
        // 소유한다. 수용한 조각의 원점으로 예약하고, 실제 그리기에 쓰는
        // 동일하게 확장된 행 상자에서 그 셀의 유닛을 선택한다.
        let accepted_straddle_end_band = if split_end_limit == 0.0
            && split_block_start.is_none()
            && end_row > cursor_row
            && table_footnotes.is_empty()
            && !st.profile.session_edited()
            && !self
                .render_normalization
                .table_text_reflowed(row_geometry_table)
            && mt.allows_row_break_split()
            && row_geometry_table.cells.iter().any(|cell| {
                usize::from(cell.row) < end_row
                    && end_row < usize::from(cell.row) + usize::from(cell.row_span)
            }) {
            let outer_bottom = (hwpunit_to_px(i32::from(table.outer_margin_bottom), self.dpi)
                - fragment_outer_bottom_overhead)
                .max(0.0);
            let band = (avail_for_rows + header_overhead - partial_height - outer_bottom).max(0.0);
            if band > 0.0 {
                let last_row = end_row - 1;
                let last_height = end_row_height_override.unwrap_or(cut_row_h[last_row]);
                end_row_height_override = Some(last_height + band);
                partial_height += band;
            }
            band
        } else {
            0.0
        };

        let accepted_straddle_cells = if !start_cut_is_block && split_block_start.is_none() {
            layout_engine.accepted_straddle_fragment_units(
                row_geometry_table,
                &mt.row_heights,
                cursor_row,
                end_row,
                start_cut,
                &split_end_cut,
                is_continuation,
                start_row_height_override,
                end_row_height_override,
                straddle_row_relief,
                straddle_page_bands,
                styles,
            )
        } else {
            straddle_page_bands.clone()
        };
        // 중간 또는 내용이 완전히 소비된 최종 컷 fragment 배치
        let complete_physical_end_cut = split_block_start.is_none()
            && split_end_limit > 0.0
            && end_row_height_override.is_some()
            && !split_end_cut.is_empty()
            && end_row > cursor_row
            && layout_engine.row_cut_consumes_all_ordinary_units(
                row_geometry_table,
                end_row - 1,
                &split_end_cut,
                styles,
            );
        let complete_source_row_frame = split_block_start.is_none()
            && split_end_cut.is_empty()
            && split_end_limit <= 0.0
            && budget
                .source_complete_frame_last_row
                .is_some_and(|(row, _)| row + 1 == end_row);
        let source_restart_frame_closes_body = is_continuation
            && split_block_start.is_none()
            && end_row_height_override.is_some()
            && layout_engine.original_body_closed_table_frame(
                input.source.paragraph,
                table,
                st.layout.body_area.height,
            )
            && end_row.checked_sub(1).is_some_and(|row| {
                layout_engine.row_cut_closes_original_restart_frame(
                    table,
                    row,
                    &split_end_cut,
                    input.fragment_row_heights.unwrap_or(&mt.row_heights),
                    styles,
                )
            });
        self.record_stored_fragment_rows(
            st,
            continuation,
            input,
            super::physical_rows::StoredFragmentEnd {
                row: end_row,
                // 내용을 모두 소비해도 물리 행 전체가 끝났다는 뜻은 아니다.
                // 병합 셀의 소유 유닛은 같은 행 안에서 계속 이어질 수 있다.
                complete_cut: (split_end_cut.is_empty() && split_end_limit <= 0.0)
                    || (complete_physical_end_cut && source_restart_frame_closes_body),
                block_start: split_block_start,
                last_height: end_row_height_override,
                restart_frame_closes_body: source_restart_frame_closes_body,
            },
        );
        st.append_item(PageItem::PartialTable {
            para_index: para_idx,
            control_index: ctrl_idx,
            start_row: cursor_row,
            end_row,
            is_continuation,
            start_cut: continuation.start_cut.clone(),
            end_cut: split_end_cut.clone(),
            // 시작·끝 컷의 인덱스 공간은 독립이다. 시작 블록의 소유는
            // start_cut_is_block에, 이번 끝 블록의 소유만 이 필드에 싣는다.
            is_block_split: split_block_start.is_some(),
            start_cut_is_block,
            row_cursor_is_nested,
            end_row_height_override,
            start_row_height_override,
            straddle_row_relief: straddle_row_relief.clone(),
            straddle_page_bands: accepted_straddle_cells.clone(),
            fragment_row_heights: input.fragment_row_heights.map(<[f64]>::to_vec),
            end_row_owns_stored_frame: is_continuation && stored_complete_extent.is_some(),
        });
        // 저장 host 원점이 없는 조각은 흐름 좌표로 같은 상자를 잰다 — 위는 흐름 커서 +
        // host·세로 오프셋, 아래는 비끝 조각 상자 바닥(7062 2~9쪽 996.49 ↔ 정본 996.43).
        // 첫 조각의 위는 흐름 커서가 아니라 host 저장 vpos 다 — 렌더러가 그 자리에 칠하고,
        // 흐름은 저장 사다리보다 늦을 수 있다(7062: 흐름 484.3 ↔ 저장 488.8, 156645214:
        // 116.3 ↔ 121.0). 흐름으로 재면 첫 상자가 그만큼 커져 끝 상자가 모자란다.
        let single_cell_box_height = if declared_opening_frame {
            Some(partial_height)
        } else {
            single_cell_box_height
        }
        .or_else(|| {
            single_cell_fragment_shape.then(|| {
                let flow_top = if let Some(placement) = fragment_placement {
                    placement.table_top - host_before_overhead - vert_offset_overhead
                } else if is_continuation {
                    st.current_height
                } else {
                    input
                        .source
                        .paragraph
                        .line_segs
                        .iter()
                        .find(|seg| !is_synthetic_line_seg(seg))
                        .map(|seg| {
                            // [#7418] 저장 앵커는 문단 상단이다. host 글을 선방출했으면
                            // `vert_offset_overhead` 는 host 뒤 흐름 기준이라, 같은 기준으로
                            // 맞추려면 앵커에도 host 높이를 더한다(156403546 3쪽: 섞이면 첫
                            // 상자가 host 만큼 커져 끝 조각의 1×1 칸 연장 16.8px 가 사라졌다).
                            hwpunit_to_px(
                                seg.vertical_pos
                                    .saturating_sub(st.vpos_page_base.unwrap_or(0)),
                                self.dpi,
                            ) + st
                                .pre_emitted_host_heights
                                .get(&input.source.para_index)
                                .copied()
                                .unwrap_or(0.0)
                        })
                        .filter(|&anchor| anchor >= st.current_height)
                        .unwrap_or(st.current_height)
                };
                (crate::renderer::float_placement::single_cell_page_fragment_bottom(
                    table,
                    st.available_height(),
                    self.dpi,
                ) - (flow_top + host_before_overhead + vert_offset_overhead))
                    .max(0.0)
            })
        });
        if let Some(box_height) = single_cell_box_height {
            continuation.single_cell_box_sum_px += box_height;
        }
        let accepted_stored_row_frame = source_opening_row_frame
            .map(|(_, opening, _)| (0, 0.0, *opening))
            .or_else(|| {
                stored_row_frame.map(|(row, used, height, _)| {
                    (row, used, end_row_height_override.unwrap_or(height))
                })
            });
        if let Some((row, used, height)) = accepted_stored_row_frame {
            continuation.stored_row_box_sum = Some((row, used + height));
        }
        // [#2238] 중간 fragment 가시높이 부기 — used_height(flush 시 current_height)
        // 표시용. advance 직후 current_height 가 리셋되므로 흐름/기하 불변.
        st.advance_flow_by(
            host_before_overhead
                + vert_offset_overhead
                + partial_height
                + fragment_outer_bottom_overhead,
        );
        if terminal_cut_consumed {
            let terminal_spacing = is_continuation
                .then(|| st.table_flow_successor(para_idx, input.source.paragraphs_all))
                .flatten()
                .and_then(|(_, next)| {
                    layout_engine.native_text_frame_terminal_successor_gap(
                        input.source.paragraph,
                        next,
                        table,
                        &st.inline_flow_column(),
                        st.inline_flow_column().y + st.current_height,
                        styles,
                    )
                })
                .unwrap_or(host_spacing_after_only + terminal_host_spacing);
            st.advance_flow_by(terminal_spacing);
            self.consume_stored_terminal_empty_line_gap(st, &input, is_continuation);
            commit_fragment(st, caption_extra + partial_height, true);
            continuation.finish(row_count, true);
            return TableContinuationIteration::Complete;
        }
        let source_frame_closes_body = !is_continuation
            && stored_complete_extent.is_some()
            && table.caption.is_none()
            && st.current_zone_y_offset == 0.0
            && fragment_placement.is_some_and(|placement| {
                budget.source_frame_bottom.is_some_and(|source_bottom| {
                    let bottom = placement.table_top + partial_height;
                    (bottom - source_bottom).abs()
                        <= hwpunit_to_px(2, self.dpi)
                            + f64::EPSILON * source_bottom.abs().max(1.0) * 4.0
                })
            });
        commit_fragment(st, caption_extra + partial_height, false);
        if !is_continuation && stored_complete_extent.is_some() {
            // 내용 컷과 독립적으로 인증한 첫 조각 상자가 실제 지면을 차지했다.
            // 다음 조각의 여백은 같은 커서 사실을 예산·배치가 함께 소비한다.
            continuation.stored_fragment_outer_frame = true;
            continuation.stored_fragment_closes_body = source_frame_closes_body;
        }
        // 큰 RowBreak 표가 기존 각주를 이미 가진 page에서 시작할 때에는 첫 fragment의
        // cell-footnote를 같은 lane에 섞지 않는다. 그 page의 기존 각주(표 25의
        // 105·106)를 보존하고, 표가 이어지는 fresh page에서 cell-footnote를 순서대로
        // 배치해야 원본 HWP/PDF의 107–111 / 112– 분할을 재현한다.
        let defer_large_first_fragment_notes = st.profile.hwp5_stored_pagination_layout()
            && queue_table_footnotes
            && !is_continuation
            && cursor_row == 0
            && table_footnotes.len() >= 8;
        if queue_table_footnotes && !defer_large_first_fragment_notes {
            self.register_queued_table_footnotes(
                st,
                continuation,
                table_footnotes,
                table,
                input.source.paragraphs_all,
                styles,
                layout_engine,
                &split_end_cut,
                para_idx,
                ctrl_idx,
                cursor_row,
                end_row,
                start_cut,
                false,
                false,
                false,
            );
        }
        st.advance_column_or_new_page();

        // 커서 전진 — [Task #993] 컷은 절대 유닛 인덱스이므로 누적 없이 대입.
        let empty_opening_next_height = input
            .prepared
            .empty_opening_row_frame
            .filter(|_| !is_continuation && cursor_row == 0 && split_end_cut == [0])
            .map(|frame| frame.continuation_height);
        let next_source_cut = if split_end_limit > 0.0 {
            split_end_cut.clone()
        } else {
            Vec::new()
        };
        let next_cut = if split_end_limit > 0.0 {
            // 높이·프레임 계산은 물리 시작점을 쓴다. 소스 컷 자체를 바꾸면 앞 조각의
            // end_cut과 다음 start_cut 사이에 소비 주인이 없는 유닛이 생긴다.
            match (split_block_start, end_row.checked_sub(1)) {
                (None, Some(row)) => layout_engine.skip_leading_free_spacers_in_row_cut(
                    row_geometry_table,
                    row,
                    &split_end_cut,
                    styles,
                ),
                _ => split_end_cut,
            }
        } else {
            Vec::new()
        };
        // 빈 컷의 블록 경계는 모든 앞 행의 내용을 끝낸 상태다. 병합 셀의
        // 저장 물리 높이에서 실제 그린 밴드를 빼 다음 완전 행 상자로 넘긴다.
        // 이 잔여는 병합 칸의 최소 점유다. 다음 행의 자체 내용·선언 높이도
        // 아직 소비하지 않았으므로 같은 행 요구와 함께 예약하고 그대로 그린다.
        let complete_block_next_height = split_block_start
            .filter(|_| split_end_limit == 0.0 && next_cut.is_empty())
            .and_then(|bs| {
                let last_height = end_row_height_override?;
                let source = table.cells.iter().find(|cell| {
                    cell.row as usize == bs
                        && cell.row as usize + cell.row_span as usize == end_row + 1
                })?;
                let used = mt.row_heights[bs..end_row.saturating_sub(1)]
                    .iter()
                    .sum::<f64>()
                    + mt.cell_spacing * end_row.saturating_sub(bs + 1) as f64
                    + last_height;
                let span_remaining = hwpunit_to_px(source.height as i32, self.dpi) - used;
                let whole_row = layout_engine.row_cut_content_height(
                    row_geometry_table,
                    end_row,
                    &[],
                    &[],
                    styles,
                );
                Some(span_remaining.max(whole_row))
            });
        let source_first_frame_remaining = (!is_continuation
            && cursor_row == 0
            && start_cut.is_empty()
            && split_block_start.is_none()
            && split_end_limit > 0.0)
            .then(|| {
                layout_engine.stored_first_frame_row_remainder(
                    table,
                    end_row.checked_sub(1)?,
                    &next_cut,
                    &mt.row_heights,
                    styles,
                )
            })
            .flatten();
        // 원본 쪽 프레임의 빈 띠는 해당 쪽의 물리 공간이다. 전체 행을 압축한
        // 높이가 아니므로 `full - limit`으로 다음 쪽의 행 최소 높이를 만들지 않는다.
        // 시작 블록 컷과 현재 일반 행의 종료 컷은 각각 다른 소유자를 가질 수 있다.
        let closes_original_body_frame = is_continuation
            && split_block_start.is_none()
            && layout_engine.original_body_closed_table_frame(
                input.source.paragraph,
                table,
                st.layout.body_area.height,
            )
            && end_row.checked_sub(1).is_some_and(|row| {
                layout_engine.row_cut_closes_shared_original_frame(table, row, &next_cut, styles)
                    || (complete_physical_end_cut
                        && layout_engine.row_cut_closes_original_restart_frame(
                            table,
                            row,
                            &next_cut,
                            input.fragment_row_heights.unwrap_or(&mt.row_heights),
                            styles,
                        ))
            });
        let next_start_row_height_override = source_opening_row_frame
            .as_ref().map(|(_, _, remaining)| *remaining)
            .or(empty_opening_next_height)
            .or_else(|| {
                let frames = input.prepared.source_single_cell_row_frames?;
                (!is_continuation
                    && next_cut.as_slice() == [frames.opening_end]
                    && end_row_height_override == Some(frames.opening_height))
                    .then_some(frames.closing_height)
            })
            .or(complete_block_next_height)
            .or(source_first_frame_remaining)
            .or_else(|| {
                let (row, used, height) = accepted_stored_row_frame?;
                let full = layout_engine.stored_full_width_row_declared_height(
                    table, row, styles, st.layout.body_area.height,
                ).or_else(|| (row == 0).then_some(source_first_row_frame.as_ref())
                    .flatten().map(|(_, opening, remaining)| opening + remaining))?;
                let remaining = (full - used - height).max(0.0);
                (remaining > 0.5).then_some(remaining)
            })
            .or_else(|| {
                if !source_restart_frame_closes_body || complete_physical_end_cut {
                    return None;
                }
                let row = end_row.checked_sub(1)?;
                // 첫0→0 재시작의 미소비 일반 행은 원본 최소 상자의 잔여도
                // 소유한다. 남은 유닛이 더 크면 그 같은 요구 높이를 하한으로
                // 예약한다. 완료 행의 빈 꼬리는 이 경로로 전달하지 않는다.
                let full = layout_engine.row_cut_content_height(table, row, &[], &[], styles);
                let remaining = full - end_row_height_override?;
                let content = layout_engine.row_cut_content_height(
                    table, row, &next_cut, &[], styles,
                );
                Some(remaining.max(content))
            })
            .or_else(|| {
                let row = end_row.checked_sub(1)?;
                if split_block_start.is_some()
                    || split_end_limit <= 0.0
                    || (row == cursor_row && !start_cut.is_empty())
                {
                    return None;
                }
                layout_engine.stored_shared_frame_row_remainder(
                    table, row, &next_cut, &mt.row_heights, end_row_height_override, styles,
                )
            })
            .or_else(|| {
                let row = end_row.checked_sub(1)?;
                if split_block_start.is_some() || split_end_limit <= 0.0 { return None; }
                let carried = (row == cursor_row)
                    .then_some(start_row_height_override).flatten();
                // 물리 잔여를 다시 분할했으면 스캔과 paint가 함께 쓴 현재 컷의
                // 패딩 포함 점유를 차감한다. 전체 행의 첫 조각으로 되돌리지 않는다.
                let opening = end_row_height_override.or_else(|| carried.map(|_| {
                    layout_engine.row_cut_content_height(
                        table, row, start_cut, &next_cut, styles,
                    )
                }))?;
                let remaining = layout_engine.stored_ordinary_row_remainder(
                    table, row, &next_cut, opening, carried, styles,
                )?;
                (!layout_engine.stored_complete_row_tail_is_line_advance(
                    table, row, &next_cut, remaining, styles,
                )).then_some(remaining)
            })
            .or(saved_closing_frame.filter(|_| first_fragment_blank_band))
            .or_else(|| {
                let row = end_row.checked_sub(1)?;
                if split_end_limit <= 0.0 {
                    return None;
                }
                let opening = layout_engine.parallel_picture_row_opening_height(
                    table, row, &[], &next_cut, styles,
                )?;
                if end_row_height_override != Some(opening) {
                    return None;
                }
                // 첫 빈 조각은 그림 유닛이 아니라 물리 공간을 소비했다.
                // 겹치는 완전한 행의 내용 상자에서 첫 조각을 차감하지 않고 남은 소유 유닛의 전체 높이를 이어받는다.
                Some(layout_engine.row_cut_content_height(table, row, &next_cut, &[], styles))
            })
            .or_else(|| {
                let row = end_row.checked_sub(1)?;
                if split_end_limit <= 0.0 || !layout_engine.row_uses_reflow_physical_frame(table, row) {
                    return None;
                }
                let carried = (row == cursor_row).then_some(start_row_height_override).flatten();
                let full = if let Some(height) = carried {
                    height
                } else {
                    // 최소 높이가 전체 내용의 점유 높이를 넘을 때만 물리 꼬리를 생성한다.
                    // 내용이 요구하는 높이는 남은 유닛에 이미 반영되어 있으므로 중복해서 이어받지 않는다.
                    layout_engine.reflow_row_physical_minimum(table, row, styles)?
                };
                let start = if row == cursor_row { start_cut.as_slice() } else { &[] };
                let used = end_row_height_override.unwrap_or_else(|| {
                    layout_engine.row_cut_content_height(table, row, start, &next_cut, styles)
                });
                let tail = (full - used).max(0.0);
                (tail > 0.5).then_some(tail)
            })
            .or_else(|| {
                // 원시 행 잔여는 병합 공간을 보존한 문단 내부 저장 컷만 소유한다.
                // 본문을 닫는 noAdjust 원본은 문단 간 저장 쪽 경계도 같은
                // 첫 프레임에서 뺀 물리 잔여를 소유한다. 일반 내용 컷은 제외한다.
                let base_remaining = (|| {
                if !first_fragment_blank_band
                    || !layout_engine.row_cut_remaining_is_single_stored_frame(
                        table, end_row - 1, &next_cut, split_block_start, styles,
                    )
                    || !(saved_block_opening_frame.is_some()
                        || (body_filling_source_frame
                            && table.raw_table_record_attr & 0x08 != 0
                            && layout_engine.row_cut_ends_at_plain_text_saved_reset(
                                table, end_row - 1, start_cut, &next_cut, styles,
                            ))
                        || layout_engine.row_cut_starts_intra_paragraph_stored_frame(
                            table, end_row - 1, &next_cut, styles,
                        ))
                { return None; }
                let first = end_row_height_override?;
                let raw = *table.get_raw_row_heights().get(end_row.checked_sub(1)?)?;
                let remaining = hwpunit_to_px(raw as i32, self.dpi) - first;
                // 줄간격 트림이 만든 빈 밴드를 넘길 때, 넘겨받는 행 상자는 이 컷에서 갈린 칸의 남은 내용을 담아야 한다. 아직 시작하지
                // 않은 칸(컷 0)은 저장 행 높이가 소유한다. 저장 행 − 첫 프레임이 갈린 칸의 남은
                // 내용보다 작으면 그 차이는 빈 밴드가 아니다(issue1937 24쪽 행 2: 54.2 < 남은
                // 내용 111.1, 정본 111.1).
                let split_only_cut: Vec<usize> = next_cut
                    .iter()
                    .map(|&cut| if cut == 0 { usize::MAX } else { cut })
                    .collect();
                let split_content = layout_engine.row_cut_content_height(
                    table,
                    end_row - 1,
                    &split_only_cut,
                    &[],
                    styles,
                );
                // 원본 행의 최소 공간을 거의 소진해도 가시 줄은 다음 프레임에
                // 속할 수 있다. 여기서 완전한 이어받기 상자를 만들어 스캐너와
                // 배치가 같은 값을 소비하게 한다. 빈 최소 공간만 예약한 뒤
                // 글자 상자를 늘리지 않는다. 블록 컷은 별도의 여러 행 프레임
                // 계산을 유지한다.
                let height = if split_block_start.is_none() {
                    let content = layout_engine.row_cut_content_height(
                        table, end_row - 1, &next_cut, &[], styles,
                    );
                    remaining.max(content)
                } else {
                    remaining
                };
                (remaining > 0.0
                    && (!blank_band_within_reset_trim || remaining + 0.5 >= split_content))
                    .then_some(height)
                })();
                base_remaining.or_else(|| {
                // 저장된 첫 프레임은 다음 프레임의 유닛을 소비하지 않고 빈 공간을 소유한다.
                // 원본 행의 최소 높이는 이어받기 커서를 통해 스캔과 실제 그리기에 공유한다.
                if !first_fragment_blank_band || !st.profile.hwp5_stored_pagination_layout() {
                    return None;
                }
                let row = end_row.checked_sub(1)?;
                // 남은 내용에 저장 쪽 경계(셀 사다리 되감김)가 더 있으면 그 행의 선언
                // 최소는 한 프레임의 빈 공간이 아니다 — 한/글은 다음 되감김에서 다시
                // 가른다. 이어 받는 조각이 넘겨받은 높이를 한 덩어리로 소비하면 그 경계를
                // 잃는다(편람 부록 103×2 행 5: 선언 1210.5 − 첫 프레임 223.9 = 986.6 을
                // 한 쪽에 실어 본문을 495px 넘겼다; 저장 되감김은 줄 10·33).
                if split_block_start.is_none()
                    && !layout_engine.row_cut_remaining_is_single_stored_frame(
                        table,
                        row,
                        &next_cut,
                        split_block_start,
                        styles,
                    )
                {
                    return None;
                }
                let cells: Vec<_> = table
                    .cells
                    .iter()
                    .filter(|cell| {
                        cell.row as usize <= row
                            && row < cell.row as usize + cell.row_span as usize
                    })
                    .collect();
                if cells.is_empty()
                    || cells.iter().any(|cell| {
                        cell.row as usize != row
                            || cell.row_span != 1
                            || cell.height >= 0x8000_0000
                    })
                {
                    return None;
                }
                let minimum = cells
                    .iter()
                    .map(|cell| hwpunit_to_px(cell.height as i32, self.dpi))
                    .fold(0.0, f64::max);
                let remaining = minimum - end_row_height_override?;
                let content =
                    layout_engine.row_cut_content_height(table, row, &next_cut, &[], styles);
                (remaining > content + 0.5).then_some(remaining)
                })
            })
            .or_else(|| end_row_height_override
            // 완전히 소비한 것으로 확인된 원본 행은 다음 행으로 전진한다.
            // 수용한 앞부분에 병합 셀 소유자의 공간이 이미 반영되어 있으므로 이전 행의 빈 꼬리로 새 행의 상자를 대체하지 않는다.
            .filter(|_| !layout_engine.stored_heading_before_inline_frame_cut(
                table, end_row.saturating_sub(1),
                if end_row - 1 == cursor_row { start_cut } else { &[] }, &next_cut, styles,
            ) && !closes_original_body_frame && !complete_source_row_frame && !first_fragment_blank_band && !source_frame_trailing_trim_applied && stored_row_frame.is_none()
                && !layout_engine.row_uses_reflow_physical_frame(table, end_row.saturating_sub(1)))
            .and_then(|limit| {
            let row = end_row.checked_sub(1)?;
            // 같은 일반 행을 다시 나누는 경우 앞 조각까지 차감한 물리 상자에서
            // 이번에 수용한 상자를 뺀다. 블록 컷의 별도 소유 공간은 유지한다.
            let carried = (row == cursor_row && !start_cut_is_block)
                .then_some(start_row_height_override)
                .flatten();
            let full = carried.or_else(|| cut_row_h.get(row).copied())?;
            let tail = (full - limit).max(0.0);
            if complete_physical_end_cut
                && layout_engine.stored_complete_row_tail_is_line_advance(
                    table, end_row - 1, &next_cut, tail, styles,
                ) {
                return None;
            }
            // [#5714] 압축된 끝행의 빈 tail 밴드는 **물리적으로 이어지는
            // 것이 있을 때만** 다음 조각으로 넘어간다: intra-row 컷이면 그
            // 행 자신이 이어지고, 행 경계 끝이면 경계를 가로지르는 rowspan
            // 셀의 선언 공간이 이어진다(76076 p36 의 24.1px 밴드 — 한컴 PDF
            // 실측, r8 rs=7 셀이 경계를 걸침). 둘 다 아니면 완결된 행의
            // 선언 잔여는 쪽 경계에서 죽는다 — rowspan 없는 19×2 표에서
            // 이 tail(11.0px)이 다음 조각 첫 행(새 행, 3줄 59.3px)에
            // 씌워져 글자가 아래 행과 포개졌다(한컴 PDF 는 밴드 없이 새
            // 행을 전체 높이로 시작).
            let tail_band_continues = split_end_limit > 0.0
                || table.cells.iter().any(|cell| {
                    (cell.row as usize) < end_row
                        && cell.row as usize + (cell.row_span as usize).max(1) > end_row
                });
            if std::env::var("RHWP_DIAG_5714").is_ok() {
                eprintln!(
                    "DIAG_5714 FRAG pi={} cursor_row={} end_row={} limit={:.1} full={:.1} tail={:.1} split_end_limit={:.1} continues={}",
                    para_idx, cursor_row, end_row, limit, full, tail,
                    split_end_limit, tail_band_continues
                );
            }
            // 이 잔여는 한 조각의 물리 하한이며 남은 전체 내용의 높이가 아니다.
            // 내용은 여러 조각에 걸칠 수 있으므로 그 전체 높이로 현재 조각을
            // 늘리지 않는다. 다음 스캔이 이 컷에서 실제 수용할 유닛을 선택한다.
            (tail > 0.5 && tail_band_continues).then_some(tail)
        }));
        // [#7531] 행 경계에서 끝난 조각의 쪽 끝 빈 띠는 그 경계를 넘는 걸침 칸이 차지한다.
        // 행 안에서 끊긴 조각도 남은 물리 공간은 같은 띠다(끝 행이 쪽을 채우면 0).
        let page_end_band = if split_block_start.is_some() {
            0.0
        } else {
            // 한/글은 쪽 끝 행 상자를 본문 바닥에서 표 바깥 아래 여백을 뺀 자리까지 늘린다.
            let outer_bottom = (hwpunit_to_px(i32::from(table.outer_margin_bottom), self.dpi)
                - fragment_outer_bottom_overhead)
                .max(0.0);
            source_frame_band.max(reserved_straddle_end_band)
                + accepted_straddle_end_band
                + (avail_for_rows + header_overhead - partial_height - outer_bottom).max(0.0)
        };
        continuation.straddle_page_bands =
            crate::renderer::layout::LayoutEngine::straddle_page_bands_after_fragment(
                row_geometry_table,
                &accepted_straddle_cells,
                if split_end_limit > 0.0 {
                    end_row.saturating_sub(1)
                } else {
                    end_row
                },
                page_end_band,
            );
        continuation.advance(
            end_row,
            split_block_start,
            next_source_cut,
            split_end_limit > 0.0,
        );
        continuation.start_row_height_override = next_start_row_height_override;
        if let Some(((_, row, cut, height), _)) = stored_rowspan_frame {
            continuation.row = row;
            continuation.start_cut = cut;
            continuation.start_cut_is_block = false;
            continuation.start_row_height_override = Some(height);
        }

        TableContinuationIteration::Emitted
    }

    /// 다음 실제 흐름 문단이 증언한 마지막 표 여백만 수용한다.
    /// deferred float 앞에서 이미 채운 문단은 이 조각의 후속 문단이 아니다.
    fn consume_stored_terminal_empty_line_gap(
        &self,
        st: &mut TypesetState,
        input: &FragmentInput<'_>,
        is_continuation: bool,
    ) {
        let host = input.source.paragraph;
        if std::env::var_os("RHWP_DIAG_SCAN").is_some() {
            eprintln!(
                "DIAG_SCAN TERMINAL_SOURCE_CANDIDATE pi={} ci={} continuation={} columns={} host_dirty={} cell_dirty={} last_control={} current={:.3}",
                input.source.para_index,
                input.source.control_index,
                is_continuation,
                st.col_count,
                host.stored_text_partition_is_dirty(),
                host.cell_format_vpos_dirty,
                input.source.control_index + 1 == host.controls.len(),
                st.current_height,
            );
        }
        if !is_continuation
            || st.col_count != 1
            || host.stored_text_partition_is_dirty()
            || host.cell_format_vpos_dirty
            || input.source.control_index + 1 != host.controls.len()
            || host
                .text
                .chars()
                .any(|ch| ch > '\u{001F}' && ch != '\u{FFFC}' && !ch.is_whitespace())
        {
            return;
        }
        let Some((index, next)) =
            st.table_flow_successor(input.source.para_index, input.source.paragraphs_all)
        else {
            return;
        };
        if st.prefilled_line_prefixes.contains_key(&index) {
            return;
        }
        if std::env::var_os("RHWP_DIAG_SCAN").is_some() {
            eprintln!(
                "DIAG_SCAN TERMINAL_SOURCE_OWNER pi={} next={} text_len={} controls={} lines={} source_lines={:?} first_line={:?} current={:.3} margin={} hwp5={} hwpx={}",
                input.source.para_index,
                index,
                next.text.len(),
                next.controls.len(),
                next.line_segs.len(),
                next.source_line_seg_vertical_pos.as_ref().map(Vec::len),
                next.line_segs.first().map(|line| (line.vertical_pos, line.line_height, line.tag)),
                st.current_height,
                input.source.table.outer_margin_bottom,
                st.profile.hwp5_stored_pagination_layout(),
                st.profile.hwpx_stored_layout(),
            );
        }
        let Some(gap) = crate::renderer::float_placement::stored_terminal_empty_line_gap(
            next,
            input.source.table,
            input.source.styles,
            st.profile,
            st.current_height,
            self.dpi,
        ) else {
            if st.deferred_table_successor_start.is_some()
                && input.source.table.outer_margin_bottom > 0
                && !st.profile.session_edited()
            {
                st.defer_table_terminal_gap(input.source.para_index, input.source.control_index);
            }
            return;
        };
        if std::env::var_os("RHWP_DIAG_SCAN").is_some() {
            eprintln!(
                "DIAG_SCAN TERMINAL_SOURCE_GAP pi={} next={} current={:.3} gap={:.3} resolved={:.3}",
                input.source.para_index, index, st.current_height, gap, st.current_height + gap,
            );
        }
        st.advance_flow_by(gap);
        st.record_stored_paragraph_flow_end(input.source.para_index, st.current_height);
    }
}
