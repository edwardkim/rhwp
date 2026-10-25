//! 수용한 조각의 배치, 대기 중인 각주 처리, 커서 전진을 원래 순서대로 확정한다.

use crate::renderer::typeset::{
    hwpunit_to_px, is_synthetic_line_seg, table, BlockTableRowScan, PageItem,
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
        let table_footnotes_follow_terminal = table.cells.iter().any(|cell| cell.row_span > 1);
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
        let nested_opening_extent = (!is_continuation
            && cursor_row == 0
            && start_cut.is_empty()
            && scan.split_block_start.is_none()
            && std::ptr::eq(table, row_geometry_table))
        .then(|| {
            let frame = layout_engine.stored_nested_parent_opening_frame(
                row_geometry_table,
                scan.end_row.checked_sub(1)?,
                &scan.split_end_cut,
                &mt.row_heights,
                styles,
            )?;
            let (height, last) = stored_complete_extent?;
            let tolerance = hwpunit_to_px(2, self.dpi);
            ((frame.height - height).abs() <= tolerance
                && (frame.last_row_height - last).abs() <= tolerance)
                .then_some(frame)
        })
        .flatten();
        let block_acceptance =
            self.accept_stored_block_frame(st, continuation, input, budget, &scan);
        let mut fragment_block_frames = mt.fragment_block_frames.clone();
        let fragment_rows = if let Some(accepted) = &block_acceptance {
            let range = accepted.range;
            if !fragment_block_frames.contains(&range) {
                fragment_block_frames.push(range);
            }
            Some(accepted.rows.clone())
        } else {
            input.fragment_row_heights.map(<[f64]>::to_vec)
        };
        let BlockTableRowScan {
            source_frame_band,
            plain_cut_owns_blank_band,
            mut consumed,
            accepted_end_row_box,
            accepted_start_row_box,
            accepted_partial_row_box,
            mut end_row,
            split_block_start,
            split_end_cut,
            split_end_limit,
            split_end_reserved,
            split_block_physical_height: _,
            mut end_row_height_override,
        } = scan;
        // 내용을 모두 담은 끝 행의 상자는 그리기와 같은 비끝 조각 닫는 선에서 끝난다.
        // 저장 줄 행의 내용 컷 예산은 본문 아래까지 쓰지만, 닫는 선 아래 몫은 이 행도,
        // 행을 덮는 병합 칸도 소비하지 않는다. 남은 선언과 병합 칸 원장은 닫는 선까지의
        // 상자에서 잰다(1342000 58쪽 행 9: 103.99 → 98.88 = 한/글 2020, 59쪽 띠 25.9 ·
        // 61쪽 병합 칸 잔여 51.6).
        if let (Some(height), Some(frame_rows)) =
            (end_row_height_override, budget.reflow_frame_avail_for_rows)
        {
            let excess = consumed - frame_rows;
            let row = end_row.saturating_sub(1);
            let start: &[usize] = if row == cursor_row { start_cut } else { &[] };
            if excess > 0.0
                && split_end_limit > 0.0
                && split_block_start.is_none()
                && end_row > cursor_row
                && layout_engine.row_cut_consumes_all_ordinary_units(
                    row_geometry_table,
                    row,
                    &split_end_cut,
                    styles,
                )
                && layout_engine.row_cut_content_height(table, row, start, &split_end_cut, styles)
                    <= height - excess + 0.5
            {
                end_row_height_override = Some(height - excess);
                consumed = frame_rows;
            }
        }
        let completed_empty_row_frame = split_block_start.is_none()
            && split_end_cut.is_empty()
            && split_end_limit == 0.0
            && end_row_height_override.is_some()
            && accepted_end_row_box.is_some_and(|(row, height)| {
                row + 1 == end_row
                    && layout_engine
                        .stored_empty_row_frame_height(row_geometry_table, row, styles)
                        .is_some_and(|minimum| height >= minimum)
            });
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
        // 컷은 내용을 소비하며, 선언된 셀 최소 높이는 빈 물리 공간도 차지한다.
        // 앞선 내용 컷으로 상자 높이를 추정하지 않고,
        // 실제로 수용한 상자를 이어받는다.
        let stored_row_frame = end_row.checked_sub(1).and_then(|row| {
            if split_end_limit <= 0.0
                || split_end_cut.is_empty()
                || split_block_start.is_some()
                || source_opening_row_frame.is_some()
            {
                return None;
            }
            // 프레임 소유를 판단할 때는 그리기와 동일하게 예약 공간을 빼기 전의 원본 본문 높이를 쓴다.
            // 각주와 영역은 아래에서 수용 가능 범위를 제한하며,
            // 원본이 소유한 최소 공간의 존재 여부를 바꾸지는 않는다.
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
            // 저장 행의 추가 빈 띠도 예산이 생산한 같은 물리 하단을 소비한다.
            // 내용 수용 경계로 다시 늘리면 paint가 접은 몫까지 이미 사용한
            // 행 상자로 기록해 다음 조각의 잔여를 줄인다.
            let blank_available = budget
                .reflow_blank_avail_for_rows
                .map_or(avail_for_rows, |height| avail_for_rows.min(height));
            let available = (blank_available - before_row).max(0.0);
            // 스캔의 높이 재정의는 수용한 내용 컷만 나타낼 수 있다.
            // 이 값은 하한이며, 저장 행의 빈 물리 밴드가 이미 예약됐다는
            // 증거는 아니다. 가용 조각에 해당 밴드를 배정하고,
            // 실제 상자를 누적 기록에 반영한다.
            // 뒤쪽 내용 컷은 앞쪽 페이지에서 이미 수용한 물리 상자가 아니다.
            // 특히 저장된 쪽 경계에서 좌표가 되감기면 높은 원본 프레임 안에
            // 가운데 정렬된 그림이 남을 수 있다. 앞으로 배치할 내용의 높이가 아니라,
            // 실제로 배치한 상자만 차감한다.
            let source_space = (declared - used).max(0.0).min(available);
            let height = accepted_scan_height.max(current_content).max(source_space);
            Some((row, used, height, accepted_scan_height))
        });
        // 행 컷이 소비한 내용과 머리행의 요구 높이. 저장 상자의 빈 밴드는
        // 아래에서 별도로 물리 점유에 포함하며 컷 유닛을 더 소비하지 않는다.
        let mut partial_height: f64 =
            consumed + header_overhead + hwpunit_to_px(table.outer_cell_spacing_hu(), self.dpi);
        if let Some(accepted) = &block_acceptance {
            partial_height += accepted.extra;
        }
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
        // #7095의 동일한 쪽 하단 상자를 저장 병합 셀 경계에도 적용한다.
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
        // 뒤 조각의 유닛은 그대로 남기며, 이 밴드를 내용 잔여에서 차감하지 않는다.
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
        // 마지막 한 행의 이어받기일 때 동일 높이를 예약과 그리기에 전달한다.
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
        // 배치 계획으로 예약과 그리기 흐름을 함께 전진시킨다.
        if is_continuation
            && end_row >= row_count
            && split_end_limit == 0.0
            && fragment_placement.is_none()
            // [#5585] 끝 조각 뒤 흐름을 중첩 자식의 저장 빈 엔터 줄(`terminal_host_spacing`)가
            // 소유하면 그 간격이 이미 표 아래를 닫는다. 바깥 아래 여백을 따로 열면 둘을
            // 겹쳐 더한다 — 한글 2024 PDF 86712 26쪽: 조각 아래 괘선 145.76px 뒤 다음 표
            // 193.87px 은 엔터 줄 간격만 더한 위치다(여백까지 더하면 +1.9px).
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
        // 예약과 그리기가 같은 선언 물리 프레임을 소비한다.
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
            && table_footnotes.is_empty()
            && input
                .source
                .paragraphs_all
                .get(para_idx + 1)
                .is_some_and(|next| {
                    layout_engine
                        .stored_single_cell_two_frame_opening(
                            input.source.paragraph,
                            next,
                            table,
                            &split_end_cut,
                            styles,
                        )
                        .is_some_and(|height| height >= partial_height && height <= avail_for_rows)
                });
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
        // 종료 프레임의 원본 정보를 조회하면서 스캔 이후 여백이 예약됐을 수 있다.
        // 최종 수용한 예약량을 기준으로 아직 소유되지 않은 흐름 잔여를 구한다.
        // 끝 조각의 물리 상자와 아래 바깥여백은 같은 영역을 소유한다.
        // 엄격 예산·캡션 등이 이미 예약한 몫을 다시 더하지 않는다.
        let parent_owned = (!row_cursor_is_nested)
            .then(|| {
                layout_engine.reflow_unrestricted_fragment_frame(
                    table,
                    start_cut,
                    &split_end_cut,
                    styles,
                )
            })
            .flatten()
            .is_none_or(|frame| frame.parent_owned);
        if !parent_owned {
            terminal_outer_bottom_overhead = 0.0;
        }
        let terminal_flow_outer_bottom = if parent_owned {
            (input.prepared.terminal_outer_bottom_flow - terminal_outer_bottom_overhead).max(0.0)
        } else {
            0.0
        };
        let host_spacing_after_only =
            input.prepared.host_spacing_after_only + terminal_flow_outer_bottom;
        // 행을 나누려고 이월했더라도 결국 통째 표로 확정하면 외곽 프레임을
        // 그대로 소비한다. 자식 행 합으로 예약한 뒤 더 큰 whole 상자를 그리지 않는다.
        if cursor_row == 0
            && !is_continuation
            && start_cut.is_empty()
            && end_row >= row_count
            && split_end_limit == 0.0
            && end_row_height_override.is_none()
        {
            let whole_height = crate::renderer::height_measurer::whole_table_frame_height(
                table,
                partial_height,
                self.dpi,
                st.profile.hwp5_stored_pagination_layout(),
                styles,
            );
            if whole_height > partial_height
                && whole_height > avail_for_rows + 0.1
                && !st.current_items.is_empty()
            {
                st.advance_column_or_new_page();
                return TableContinuationIteration::Skipped;
            }
            partial_height = whole_height;
        }

        // `terminal_successor_gap`: 끝 조각 뒤 문단이 증언한 공통 간격이 있으면
        // paint와 같은 흐름 끝을 사용하고 호스트 아래 간격으로 다시 덮어쓰지 않는다.
        let commit_fragment = |st: &mut TypesetState,
                               owner_height: f64,
                               terminal: bool,
                               terminal_successor_gap: Option<f64>| {
            if let Some(mut placement) = fragment_placement {
                // 끝 조각이 아닌 컷은 후속 문단을 소유할 수 없다. 끝 조각의 원본 원점은
                // 유지하되, 간격을 소비하는 경로에서는 최종 수용한 물리 하단과 일치해야 한다.
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
                let successor_flow_end = terminal_successor_gap.filter(|_| terminal).map(|gap| {
                    placement.table_top + owner_height + terminal_outer_bottom_overhead + gap
                });
                // 다음 줄 계획(NextLine)은 paint 가 문단 끝을 `상자 아래 + 호스트 아래 간격`으로 다시
                // 정한다. 뒤 문단 저장 줄 윗변이 상자 아래와 같다고 증언하면 공통 인증
                // (`stored_successor_line_origin`)으로 남겨, paint 도 호스트 아래 간격을 더하지 않고
                // 그 문단 위 간격을 한 번만 쓰게 한다(1613000 125쪽: 빈 호스트 표 끝 조각·캡션 아래
                // 156.08 = 다음 빈 줄 vpos 11706). typeset 흐름 끝은 위 `successor_flow_end` 그대로다.
                if matches!(
                    placement.flow,
                    crate::renderer::float_placement::ParagraphFloatFlow::NextLine
                ) {
                    if let Some(end) = successor_flow_end {
                        let successor_before = st
                            .table_flow_successor(para_idx, input.source.paragraphs_all)
                            .and_then(|(_, next)| {
                                styles.para_styles.get(next.para_shape_id as usize)
                            })
                            .map_or(0.0, |style| style.spacing_before);
                        // 뒤 문단의 저장 줄 윗변(= 흐름 끝 + 그 문단 위 간격)이 상자 아래 이하에
                        // 있으면 그 줄이 문단을 닫는다. 위 간격이 바깥 아래 여백과 겹쳐도 같다
                        // (1613000 59쪽: 상자 아래 226.93, 뒤 빈 줄 vpos 17444 = 232.59).
                        // 위 간격이 없는 뒤 문단도 같다 — 저장 줄이 상자 아래에서 시작한다고 증언했다
                        // (1790387 57쪽: 글자처럼 표 문단 pi238 vpos 15555 = 상자 아래 207.4).
                        let successor_line = end + successor_before;
                        if successor_line >= placement.occupied_bottom - 0.5 {
                            placement.stored_successor_line_origin =
                                Some(successor_line.max(placement.occupied_bottom));
                        }
                    }
                }
                st.record_paragraph_float_placement((para_idx, ctrl_idx), placement);
                st.align_flow_to(successor_flow_end.unwrap_or(
                    placement.occupied_bottom
                        + if terminal {
                            input.prepared.host_spacing_after_only
                        } else {
                            0.0
                        },
                ));
                if terminal {
                    // 뒤 문단의 저장 줄이 이미 표 아래에 있다고 증언하면(공통 간격) 그 문단 시작
                    // (위 간격 포함)을 표 영역이 다시 밀지 않는다 — 배제 띠는 흐름 끝까지만.
                    st.add_visible_float_exclusion(VisibleFloatExclusion {
                        para_index: para_idx,
                        top: placement.table_top,
                        bottom: successor_flow_end.map_or(placement.occupied_bottom, |end| {
                            end.min(placement.occupied_bottom)
                        }),
                    });
                }
            }
            // 글줄을 따로 소유하지 않는 호스트의 마지막 개체가 끝나면,
            // 수용한 조각·캡션·바깥여백의 최종 흐름 끝도 배치에 전달한다.
            // paint가 표 상자만으로 다시 계산하면 아래 바깥여백이 빠진다.
            if terminal
                && !crate::renderer::typeset::para_has_visible_text(input.source.paragraph)
                && ctrl_idx + 1 == input.source.paragraph.controls.len()
            {
                st.record_stored_paragraph_flow_end(para_idx, st.current_height);
            }
        };

        // [작업 #1046 2단계 진단] 순회 결과 — 조각 경계/소비 높이. 동작 불변.
        if std::env::var("RHWP_TABLE_DRIFT").is_ok() {
            eprintln!(
                "TABLE_SPLIT_RESULT: pi={} sec={} cursor_row={} end_row={} consumed={:.1} partial_h={:.1} split_end_limit={:.1} avail_for_rows={:.1} fits={}",
                para_idx, st.section_index, cursor_row, end_row, consumed, partial_height,
                split_end_limit, avail_for_rows, consumed <= avail_for_rows + 0.1,
            );
        }

        if end_row >= row_count && split_end_limit == 0.0 {
            let skip_terminal_empty_sliver = is_continuation
                // 재조판한 선언 잔여는 마지막 내용 유닛 뒤에도 실제 물리 공간을 차지한다.
                // 이 프레임을 작은 빈 조각으로 간주해 지우지 않는다.
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
            let mut terminal_successor_gap = None;

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
            // [#7677] 끝 조각 뒤 흐름이 조각 하단에서 더 내려간 몫(바깥 아래 여백 등) — host
            // 글줄을 예약할 때 배치가 같은 자리에서 시작하도록 넘긴다.
            let mut terminal_host_push = 0.0f64;
            let mut flow_after_terminal_advance = f64::NAN;
            if cursor_row == 0
                && !is_continuation
                && start_cut.is_empty()
                // 마지막 행의 물리 높이를 바꾼 결과는 조각 배치에서 소비한다.
                // 통째 표로 되돌리면 원래 행 높이가 복원되어 예약 하단을 넘는다.
                && end_row_height_override.is_none()
            {
                // 한 쪽에 통째로 든 첫 조각도 끝 조각이다. 문단 기준 자리차지 표가 호스트 글줄을
                // 지나 놓이면 뒤 문단 줄은 표 상자(위 오프셋·바깥 여백 포함) 아래 자기 저장 줄에
                // 놓인다 — 끝 조각과 같은 공통 helper 로 흐름 끝을 정한다(1613000 191쪽 pi=9:
                // 상자 아래 631.9 = 다음 줄 vpos 47394, 한/글 2020 정본).
                let whole_box_bottom = host_before_overhead
                    + vert_offset_overhead
                    + partial_height
                    + hwpunit_to_px(table.outer_margin_bottom as i32, self.dpi);
                let whole_successor_flow_end = st
                    .table_flow_successor(para_idx, input.source.paragraphs_all)
                    .and_then(|(_, next)| {
                        layout_engine.text_host_terminal_successor_gap(
                            input.source.paragraph,
                            next,
                            table,
                            &st.inline_flow_column(),
                            st.inline_flow_column().y + st.current_height + whole_box_bottom,
                            styles,
                        )
                    })
                    .map(|gap| st.current_height + whole_box_bottom + gap);
                st.append_item(PageItem::Table {
                    para_index: para_idx,
                    control_index: ctrl_idx,
                });
                // [#8149] 통째 첫 조각도 끝 조각과 같은 점유(아래 캡션·바깥 아래 여백·뒤 간격)로
                // 흐름을 닫는다. host 간격 합(`host_spacing_total`)의 뒤 몫이 그 점유보다 작을
                // 때만 모자란 만큼 더한다 — 한글문서파일형식 33쪽 표 199 는 뒤 몫 0 이라 캡션
                // 24.0 + 바깥 아래 여백 26.5 를 빠뜨려 제목 200 이 캡션과 겹쳤다. host 앞 간격과
                // host 줄간격은 host 간격 합의 것을 그대로 쓴다: 단 맨 위 표의 바깥 위 여백은
                // paint 위치만 바꾸고 흐름은 바꾸지 않는다(issue_4068).
                let host_spacing_after_in_total =
                    (host_spacing_total - input.prepared.host_spacing_before).max(0.0);
                let terminal_occupancy = bottom_caption_extra
                    + terminal_outer_bottom_overhead
                    + host_spacing_after_only
                    + terminal_host_spacing;
                match whole_successor_flow_end {
                    Some(end) => st.align_flow_to(end),
                    None => st.advance_flow_by(
                        partial_height
                            + host_spacing_total
                            + (terminal_occupancy - host_spacing_after_in_total).max(0.0),
                    ),
                }
            } else {
                self.record_stored_fragment_rows(
                    st,
                    continuation,
                    input,
                    super::physical_rows::StoredFragmentEnd {
                        row: end_row,
                        complete_cut: split_end_cut.is_empty() && split_end_limit <= 0.0,
                        block_start: split_block_start,
                        cut: &split_end_cut,
                        first_height: accepted_start_row_box,
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
                    fragment_row_heights: fragment_rows.clone(),
                    fragment_block_frames: fragment_block_frames.clone(),
                    end_row_owns_stored_frame: completed_empty_row_frame,
                });
                // 마지막 조각: spacing_after만 포함 (페이지 분할기 engine.rs:1051과 동일)
                // 호스트 줄 전진과 양의 오프셋은 원래 앵커 조각의 계약이며,
                // 이어받기 끝에서 다시 더하면 다음 본문을 이중으로 민다(#2439).
                let occupied = host_before_overhead
                    + vert_offset_overhead
                    + partial_height
                    + bottom_caption_extra
                    + terminal_outer_bottom_overhead;
                terminal_successor_gap = is_continuation
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
                    });
                let terminal_spacing = terminal_successor_gap
                    .unwrap_or(host_spacing_after_only + terminal_host_spacing);
                if std::env::var_os("RHWP_DIAG_SCAN").is_some() {
                    eprintln!("DIAG_SCAN FLOW_END pi={para_idx} row={cursor_row} cont={is_continuation} current={:.3} occupied={occupied:.3} terminal_spacing={terminal_spacing:.3} outer={} terminal_flow={:.3} reserved={terminal_outer_bottom_overhead:.3} host_tags={:?} next_line={:?}", st.current_height, table.outer_margin_bottom, input.prepared.terminal_outer_bottom_flow, input.source.paragraph.line_segs.iter().map(|line| line.tag).collect::<Vec<_>>(), input.source.paragraphs_all.get(para_idx + 1).and_then(|para| para.line_segs.first()).map(|line| (line.vertical_pos, line.tag)));
                }
                st.advance_flow_by(occupied + terminal_spacing);
                terminal_host_push = terminal_spacing;
                flow_after_terminal_advance = st.current_height;
                self.consume_stored_terminal_empty_line_gap(st, &input, is_continuation);
            }
            commit_fragment(
                st,
                caption_extra + partial_height + bottom_caption_extra,
                true,
                terminal_successor_gap,
            );
            // [#7677] 쪽을 넘겨 온 표의 마지막 조각 뒤 host 글줄은 여기서 흐름에 예약한다 —
            // 흐름은 방금 조각 하단 + 바깥 아래 여백에 섰다(한/글이 그 줄을 두는 자리).
            if is_continuation && input.prepared.computed_host_tail.is_none() {
                if flow_after_terminal_advance.is_finite() {
                    terminal_host_push +=
                        (st.current_height - flow_after_terminal_advance).max(0.0);
                }
                self.post_emit_visible_rowbreak_host_text(
                    st,
                    para_idx,
                    ctrl_idx,
                    input.source.paragraph,
                    table,
                    input.source.composed_all,
                    styles,
                    terminal_host_push,
                );
            }
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
                    table_footnotes_follow_terminal,
                    relax_terminal_table_footnote_fit && is_continuation,
                    false,
                );
                // 끝 조각에 들어가지 못한 URL 각주는 새 쪽의 하단 각주
                // 영역에 먼저 예약한다. 다음 본문은 그 예약량을 보고 같은
                // 쪽에 들어가거나 필요할 때만 다음 쪽으로 분할된다.
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
                        table_footnotes_follow_terminal,
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
        // 컷을 지우면 원래 행 높이가 복원되므로 그리기 컷은 그대로 보존하고,
        // 빈 후속 페이지를 할당하기 전에 이어받기만 종료한다.
        let terminal_cut_consumed = input.closes_terminal_content_cut(
            end_row,
            split_end_limit,
            &split_end_cut,
            split_block_start,
            end_row_height_override,
            consumed,
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

        // 독립 저장 프레임이 먼저 물리 상자를 확정한다. 내용 예약만으로
        // override의 존재를 미리 만들면 그 프레임의 선택 조건이 달라진다.
        // 프레임 없는 일반 컷만 스캔의 수용 증분을 paint와 원장에 넘긴다.
        if split_block_start.is_none()
            && split_end_limit > 0.0
            && (self.stored_header_physical_rows(st, input.source)
                || layout_engine.has_stored_continuous_spans(row_geometry_table))
        {
            end_row_height_override = end_row_height_override.or_else(|| {
                accepted_partial_row_box
                    .filter(|(row, _)| row + 1 == end_row)
                    .map(|(_, height)| height)
            });
        }

        // 쪽 끝 빈 띠는 실제 물리 경계와 수용한 조각 상자의 차이다.
        // 원본 프레임이 이미 예약한 띠는 그리기 높이에 다시 더하지 않는다.
        let ledger_page_end_band = if split_block_start.is_some() {
            0.0
        } else if let Some(frame_rows) = budget.reflow_blank_avail_for_rows {
            // 물리 경계에는 아래 바깥여백과 실제 각주 공간이 이미 반영돼 있다.
            (frame_rows + header_overhead - partial_height).max(0.0)
        } else {
            let outer_bottom = (hwpunit_to_px(i32::from(table.outer_margin_bottom), self.dpi)
                - fragment_outer_bottom_overhead)
                .max(0.0);
            (avail_for_rows + header_overhead - partial_height - outer_bottom).max(0.0)
        };
        // 그리는 띠는 비끝 조각 닫는 선(본문 아래 − 바깥 아래 여백 − 100HU)까지다(한/글 2020: 1613000
        // 구역 1·3 표 아래 731.79·712.91px = 본문 아래 − 3.77 − 1.43). 닫는 선 아래 몫은 걸침 칸이
        // 다음 쪽으로 넘기지 않는다 — 이월 원장은 줄이기 전 띠(`ledger_page_end_band`)를 소비한다
        // (21761835 3쪽: 그 몫을 넘기면 첫 행들이 한/글보다 커진다).
        // 내용 컷으로 끝난 행(`split_end_limit > 0`)은 다음 조각이 같은 행을 잇는 상자라 이 선을 쓰지
        // 않는다(1342000 202·203쪽: 줄이면 다음 쪽 시작 줄이 한 줄 당겨진다). 통째로 받은 끝 행만 닫는다.
        let unreserved_page_end_band = if budget.reflow_blank_avail_for_rows.is_some() {
            // 공통 물리 하단 생산자가 이미 바깥 여백과 inset을 차감했다.
            // 같은 하단을 예약과 paint가 소비하며 여기서 다시 접지 않는다.
            ledger_page_end_band
        } else {
            (ledger_page_end_band
                - hwpunit_to_px(
                    crate::renderer::float_placement::rowbreak_fragment_bottom_inset_hu(table),
                    self.dpi,
                ))
            .max(0.0)
        };
        let mut ledger_end_row_trim = 0.0;
        // 내용 컷을 넘는 셀과 뒤 행까지 이어지는 물리 걸침은 다르다.
        // 같은 공통 경계에 빈 컷을 질의하면 내용 컷 없이도 뒤 행이 남는지 알 수 있다.
        // 뒤 행이 없는 내부 컷의 허용만으로 빈 종료 띠까지 소유했다고 판정하지 않는다.
        let span_has_following_rows = row_geometry_table.cells.iter().any(|cell| {
            cell.row_span > 1
                && crate::renderer::layout::cell_crosses_fragment_end(cell, end_row, &[])
        });
        let plain_paragraph_cut = split_end_limit > 0.0
            && end_row > cursor_row
            && layout_engine.row_cut_respects_plain_paragraph_keeps(
                row_geometry_table,
                end_row - 1,
                if end_row - 1 == cursor_row {
                    start_cut
                } else {
                    &[]
                },
                &split_end_cut,
                styles,
            )
            && (span_has_following_rows
                || layout_engine.row_cut_keeps_close_a_plain_paragraph(
                    row_geometry_table,
                    end_row - 1,
                    if end_row - 1 == cursor_row {
                        start_cut
                    } else {
                        &[]
                    },
                    &split_end_cut,
                    styles,
                ));
        // 문단 유지 계약 밖의 내용 컷(저장 쪽 재시작이나 예산 컷)도, 끝 행을 지나 이어지는 걸침 칸이
        // 있으면 같은 경로로 든다. 다만 띠는 나뉜 행이나 걸침 칸의 선언에 남는 몫까지만 준다
        // (아래 `crossing_span_page_end_band`). 21761835 2쪽 행 20(빈 문단 뒤 vpos 0 재시작),
        // 1342000 200쪽 행 75(예산 컷) 모두 한/글이 쪽 끝까지 채운다.
        let other_content_cut = split_end_limit > 0.0 && !plain_paragraph_cut;
        // 스캔이 같은 내용 컷 밖의 빈 띠를 이미 예약했어도 그 상자의 소유는 남는다.
        // 추가 높이가 0이라는 이유로 소유를 지우면 paint가 끝 행을 다시 접고,
        // 다음 조각도 실제 수용한 상자를 잃는다. 자연 내용 높이만으로 추정하지 않는다.
        let reserved_plain_end_band = plain_paragraph_cut
            && (source_frame_band > 0.0 || plain_cut_owns_blank_band)
            && accepted_partial_row_box.is_some_and(|(row, _)| row + 1 == end_row);
        let mut span_band_owns_end_row = false;
        let mut span_band_row_box = None;
        // 선언 최소 공간은 저장 글줄 캐시가 아니다. 현재 내용 컷과 앞에서
        // 실제 수용한 상자를 차감하므로 다른 셀의 줄 변경으로 폐기하지 않는다.
        if (split_end_limit == 0.0 || plain_paragraph_cut || other_content_cut)
            && split_block_start.is_none()
            && end_row > cursor_row
            && table_footnotes.is_empty()
            && !st.profile.session_edited()
            && mt.allows_row_break_split()
            && (row_geometry_table.cells.iter().any(|cell| {
                cell.row_span > 1
                    && crate::renderer::layout::cell_crosses_fragment_end(
                        cell,
                        end_row,
                        &split_end_cut,
                    )
            })
                // [#8105] 걸침 칸이 없어도 내용 컷으로 나뉜 행의 선언 높이가 이 조각 상자보다 크면 그 행
                // 자신이 쪽 끝 띠를 소유한다 — 한/글은 첫 조각의 아래 괘선을 닫는 선에 긋는다(1342000 94쪽
                // 행 25 제주 칸: 선언 123.3px, 컷 내용 55.7px, 한/글 괘선 712.7 = 닫는 선; task1749 4쪽
                // 행 2: 한/글 괘선 1024.0). 저장 행 프레임이 상자를 정한 행은 그 프레임을 따른다.
                || (split_end_limit > 0.0
                    && stored_row_frame.is_none()
                    // 앞 쪽에서 이미 그린 행을 잇는 가운데 조각은 앞 상자들을 장부로 받지 않아 남는
                    // 몫을 모른다 — 행이 이 조각에서 시작할 때만(편람 214쪽 3쪽짜리 행).
                    && !(end_row - 1 == cursor_row && !start_cut.is_empty())
                    && super::physical_rows::declared_row_height(
                        row_geometry_table,
                        end_row - 1,
                        self.dpi,
                    ) > split_end_reserved.max(split_end_limit) + 0.5))
            && (ledger_page_end_band > 0.0 || reserved_plain_end_band)
        {
            let last_row = end_row - 1;
            let last_height = end_row_height_override
                .or_else(|| {
                    accepted_end_row_box
                        .filter(|(row, _)| *row == last_row)
                        .map(|(_, height)| height)
                })
                .unwrap_or_else(|| {
                    if plain_paragraph_cut || other_content_cut {
                        // 컷 뒤의 내용을 담은 전체 행 높이는 앞 조각의 물리 상자가 아니다.
                        layout_engine.row_cut_content_height(
                            row_geometry_table,
                            last_row,
                            if last_row == cursor_row {
                                start_cut
                            } else {
                                &[]
                            },
                            &split_end_cut,
                            styles,
                        )
                    } else {
                        cut_row_h[last_row]
                    }
                });
            // 그 밖의 내용 컷으로 이 경로에 든 조각은 남는 몫까지만 띠를 준다. 몫이 없으면 한/글도
            // 띠를 비워 둔다(70833 2쪽: 표가 973px 에서 끝난다).
            if split_end_limit == 0.0 || plain_paragraph_cut {
                // 저장 걸침 프레임이 있는 표는 끝 행도 걸침 칸의 몫까지만 띠를 받는다(아래와 같은 규칙,
                // 1613000 144쪽 행 139: 한/글 38.40px = 행 높이, 띠 없음). 그 밖의 표는 종전처럼 남은 띠 전부.
                let span_rule = (self.stored_header_physical_rows(st, input.source)
                    || layout_engine.has_stored_continuous_spans(row_geometry_table))
                    && super::physical_rows::row_cells_hold_only_text_lines(
                        row_geometry_table,
                        last_row,
                    );
                let (band, ledger_band) = if span_rule {
                    let shared = |room: f64| {
                        self.crossing_span_page_end_band(
                            st,
                            continuation,
                            input,
                            (last_row, &[], &split_end_cut),
                            last_height,
                            room,
                            cut_row_h,
                        )
                        .unwrap_or(0.0)
                    };
                    (
                        shared(unreserved_page_end_band),
                        shared(ledger_page_end_band),
                    )
                } else {
                    (unreserved_page_end_band, ledger_page_end_band)
                };
                end_row_height_override = Some(last_height + ledger_band);
                partial_height += ledger_band;
                ledger_end_row_trim = (ledger_band - band).max(0.0);
                // 걸침 몫으로 정한 끝 행 상자는 원본 프레임이다 — 배치가 끝 줄간격으로 다시 접지 않는다.
                span_band_owns_end_row = span_rule && band > 0.5;
                // 문단 컷의 수용 원장도 같은 걸침 몫과 물리 행 상자를 소비한다.
                if plain_paragraph_cut {
                    span_band_owns_end_row = true;
                    span_band_row_box = Some((last_row, last_height + ledger_band));
                }
            } else {
                // 쪽 끝까지의 방은 예약(`consumed`)이 이 행에 실제로 준 높이 위에 남은 띠를 더한
                // 값이고, 상자는 끝 컷 내용(`last_height`) 이상·걸침 칸의 남는 몫 이하다. 컷 경로에
                // 따라 그 예약은 내용 전진량이 아니라 패딩 포함 컷 상자다(21761835 2쪽 행 20:
                // 전진량 47.25px · 예약 50.99px — 전진량으로 재면 닫는 선보다 패딩만큼 짧다).
                let reserved = if split_end_reserved > 0.0 {
                    split_end_reserved
                } else {
                    split_end_limit
                };
                let room = reserved + unreserved_page_end_band;
                let ledger_room = reserved + ledger_page_end_band;
                let ledger_band = self.crossing_span_page_end_band(
                    st,
                    continuation,
                    input,
                    (last_row, &[], &split_end_cut),
                    last_height,
                    (ledger_room - last_height).max(0.0),
                    cut_row_h,
                );
                if let Some(band) = self.crossing_span_page_end_band(
                    st,
                    continuation,
                    input,
                    (
                        last_row,
                        if last_row == cursor_row {
                            start_cut
                        } else {
                            &[]
                        },
                        &split_end_cut,
                    ),
                    last_height,
                    (room - last_height).max(0.0),
                    cut_row_h,
                ) {
                    let row_box = last_height + band;
                    let ledger_box = last_height + ledger_band.unwrap_or(band).max(band);
                    end_row_height_override = Some(ledger_box);
                    partial_height += ledger_box - reserved;
                    if std::env::var("RHWP_DIAG_SCAN").is_ok() {
                        eprintln!(
                            "DIAG_SPAN_END_BAND row={} content={:.6} physical_before={:.6} band={:.6} physical_after={:.6} fragment_after={:.6}",
                            last_row, split_end_limit, last_height, band, row_box, partial_height,
                        );
                    }
                    span_band_owns_end_row = true;
                    ledger_end_row_trim = ledger_box - (last_height + band);
                    span_band_row_box = Some((last_row, ledger_box));
                }
            }
        }

        // 위 정렬 원본 셀의 선언 높이는 최소 점유다. 앞 조각이 수용한 물리 상자를
        // 먼저 빼고, 실제 저장 프레임 컷의 내용과 남은 최소 점유 중 큰 값을 쓴다.
        // 내용 자체를 줄이지 않으며, 같은 상자를 예약·paint·다음 조각 원장에 전달한다.
        let source_single_cell_band = (single_cell_fragment_shape
            && is_continuation
            && split_block_start.is_none()
            && split_end_limit > 0.0
            && start_cut.len() == 1
            && split_end_cut.len() == 1
            && source_frame_band == 0.0
            && !span_band_owns_end_row
            && !completed_empty_row_frame
            && !st.profile.session_edited()
            && st.profile.hwp5_stored_pagination_layout()
            && !self.render_normalization.table_text_reflowed(table)
            && table_footnotes.is_empty())
        .then(|| {
            let cell = table.cells.first()?;
            if !matches!(cell.vertical_align, crate::model::table::VerticalAlign::Top)
                || cell.height > i32::MAX as u32
                || hwpunit_to_px(cell.height as i32, self.dpi) <= st.layout.body_area.height
                || !layout_engine.cell_unit_opens_stored_page_frame(
                    cell,
                    table,
                    styles,
                    start_cut[0],
                )
                || !layout_engine.cell_unit_opens_stored_page_frame(
                    cell,
                    table,
                    styles,
                    split_end_cut[0],
                )
            {
                return None;
            }
            let physical_top = fragment_placement.map_or(fragment_top, |p| p.table_top);
            let capacity = single_cell_box_height.unwrap_or_else(|| {
                (crate::renderer::float_placement::single_cell_page_fragment_bottom(
                    table,
                    st.available_height(),
                    self.dpi,
                ) - physical_top)
                    .max(0.0)
            });
            let remaining = (hwpunit_to_px(cell.height as i32, self.dpi)
                - continuation.single_cell_box_sum_px)
                .max(0.0);
            Some(partial_height.max(remaining.min(capacity)))
        })
        .flatten();
        if let Some(height) = source_single_cell_band {
            partial_height = height;
            end_row_height_override = Some(height);
        }

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
            let trailing_start =
                (start_cut_is_block && !start_cut.is_empty() && split_block_start.is_none())
                    .then(|| mt.row_block_for(cursor_row).1)
                    .filter(|&start| start > cursor_row && start < end_row);
            layout_engine.accepted_straddle_after_block(
                row_geometry_table,
                &mt.row_heights,
                trailing_start,
                end_row,
                &split_end_cut,
                is_continuation,
                end_row_height_override,
                straddle_row_relief,
                straddle_page_bands,
                styles,
            )
        };

        // 중간 또는 내용이 완전히 소비된 최종 컷 조각 배치
        let complete_physical_end_cut = split_block_start.is_none()
            && split_end_limit > 0.0
            && end_row_height_override.is_some()
            && end_row > cursor_row
            && ((!split_end_cut.is_empty()
                && layout_engine.row_cut_consumes_all_ordinary_units(
                    row_geometry_table,
                    end_row - 1,
                    &split_end_cut,
                    styles,
                ))
                || (split_end_cut.is_empty()
                    && layout_engine.row_band_completes_ending_reflow_spans(
                        row_geometry_table,
                        end_row - 1,
                        &accepted_straddle_cells,
                        styles,
                    )));
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
                cut: &split_end_cut,
                first_height: accepted_start_row_box,
                // 걸침 칸이 다음 조각에서 세는 앞 행 상자는 그린 상자다 — 내용 컷 행에서 닫는 선으로 줄인 몫은
                // 병합 칸의 뒤 조각이 잇는다(fragment_ghost_vrules 14·15쪽 (34,1): 앞+뒤 = 선언 326.96px, 한/글
                // 2020 15쪽 첫 행 237.3px). 같은 행 자신의 다음 부분(`next_start_row_height_override`)은 원장 값을
                // 쓴다(1342000 336·337쪽: 넘기면 컷이 한 단위 당겨진다). 통째로 받은 끝 행은 원장 그대로.
                last_height: if split_end_limit > 0.0 {
                    end_row_height_override.map(|height| height - ledger_end_row_trim)
                } else {
                    end_row_height_override
                },
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
            // 그리는 끝 행 상자는 닫는 선으로 줄인 값이다(원장 값 − 줄인 몫).
            end_row_height_override: end_row_height_override
                .map(|height| height - ledger_end_row_trim),
            start_row_height_override,
            straddle_row_relief: straddle_row_relief.clone(),
            straddle_page_bands: accepted_straddle_cells.clone(),
            fragment_row_heights: fragment_rows.clone(),
            fragment_block_frames: fragment_block_frames.clone(),
            // 걸침 칸·나뉜 행의 몫으로 쪽 끝 띠까지 수용한 끝 행 상자도 원본 프레임이다.
            // 배치는 이 상자를 끝 줄간격 접기로 다시 줄이지 않는다.
            end_row_owns_stored_frame: source_frame_band > 0.0
                || first_fragment_blank_band
                || original_single_cell_opening_frame
                || source_single_cell_band.is_some()
                || (is_continuation && stored_complete_extent.is_some())
                || span_band_owns_end_row
                || completed_empty_row_frame,
        });
        // 저장 호스트 원점이 없는 조각은 흐름 좌표로 같은 상자를 잰다 — 위는 흐름 커서 +
        // 호스트·세로 오프셋, 아래는 비끝 조각 상자 바닥(7062 2~9쪽 996.49 ↔ 정본 996.43).
        // 첫 조각의 위는 흐름 커서가 아니라 호스트의 저장 vpos 다 — 렌더러가 그 자리에 칠하고,
        // 흐름은 저장 사다리보다 늦을 수 있다(7062: 흐름 484.3 ↔ 저장 488.8, 156645214:
        // 116.3 ↔ 121.0). 흐름으로 재면 첫 상자가 그만큼 커져 끝 상자가 모자란다.
        let single_cell_box_height = if let Some(height) = source_single_cell_band {
            Some(height)
        } else if declared_opening_frame {
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
                            // [#7418] 저장 앵커는 문단 상단이다. 호스트 글을 선방출했으면
                            // `vert_offset_overhead` 는 호스트 뒤 흐름 기준이라, 같은 기준으로
                            // 맞추려면 앵커에도 호스트 높이를 더한다(156403546 3쪽: 섞이면 첫
                            // 상자가 호스트만큼 커져 끝 조각의 1×1 칸 연장 16.8px 가 사라졌다).
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
        // [#2238] 중간 조각의 가시 높이 기록 — used_height(페이지 확정 시 current_height)
        // 표시용. 전진 직후 current_height 가 리셋되므로 흐름/기하 불변.
        st.advance_flow_by(
            host_before_overhead
                + vert_offset_overhead
                + partial_height
                + fragment_outer_bottom_overhead,
        );
        if terminal_cut_consumed {
            let terminal_successor_gap = is_continuation
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
                });
            st.advance_flow_by(
                terminal_successor_gap.unwrap_or(host_spacing_after_only + terminal_host_spacing),
            );
            self.consume_stored_terminal_empty_line_gap(st, &input, is_continuation);
            commit_fragment(
                st,
                caption_extra + partial_height,
                true,
                terminal_successor_gap,
            );
            continuation.finish(row_count, true);
            return TableContinuationIteration::Complete;
        }
        // 인증한 첫 상자의 실제 하단이 소유한 inset을 뒤 조각까지 전달한다.
        // 기본 본문 끝과 Native 저장 프레임의 기존 inset을 각각 확인하며,
        // 단순히 원본 표라는 이유로 뒤 내용 예산을 줄이지 않는다.
        let source_frame_body_bottom_inset = (!is_continuation
            && stored_complete_extent.is_some()
            && table.caption.is_none()
            && st.current_zone_y_offset == 0.0)
            .then_some(fragment_placement)
            .flatten()
            .zip(budget.source_frame_bottom)
            .and_then(|(placement, source_bottom)| {
                let bottom = placement.table_top + partial_height;
                let inset = source_bottom - bottom;
                let tolerance =
                    hwpunit_to_px(2, self.dpi) + f64::EPSILON * source_bottom.abs().max(1.0) * 4.0;
                if inset.abs() <= tolerance {
                    Some(0.0)
                } else {
                    let native_inset = hwpunit_to_px(
                        crate::renderer::float_placement::rowbreak_fragment_bottom_inset_hu(table),
                        self.dpi,
                    );
                    (st.profile.hwp5_stored_pagination_layout()
                        && table.row_count > 1
                        && (inset - native_inset).abs() <= tolerance)
                        .then_some(inset)
                }
            });
        commit_fragment(st, caption_extra + partial_height, false, None);
        // 부모 프레임만 현재 쪽에 남은 경우, 독립 자식의 이월은 뒤 본문까지
        // 소유하지 않는다. 수용한 소스 컷의 사실로 기존 후속 선배치를 공유한다.
        if !row_cursor_is_nested
            && ctrl_idx + 1 == input.source.paragraph.controls.len()
            && layout_engine
                .reflow_unrestricted_fragment_frame(table, start_cut, &split_end_cut, styles)
                .is_some_and(|frame| frame.parent_owned && !frame.external_content_owned)
        {
            let host_end = st.current_height;
            if self.prefill_computed_float_successors(
                st,
                para_idx,
                input.source.paragraph,
                input.source.paragraphs_all,
                input.source.composed_all,
                styles,
            ) {
                // 수용한 부모 프레임의 끝을 paint에도 전달한다. 빈 호스트의
                // 글줄 높이만 재사용하면 뒤 본문이 부모 칸 안으로 되돌아간다.
                st.record_stored_paragraph_flow_end(para_idx, host_end);
            }
        }
        if !is_continuation && stored_complete_extent.is_some() {
            // 내용 컷과 독립적으로 인증한 첫 조각 상자가 실제 지면을 차지했다.
            // 다음 조각의 여백은 같은 커서 사실을 예산·배치가 함께 소비한다.
            continuation.stored_fragment_outer_frame = true;
            continuation.stored_fragment_body_bottom_inset = source_frame_body_bottom_inset;
        }
        // 큰 RowBreak 표가 기존 각주를 이미 가진 쪽에서 시작할 때에는 첫 조각의
        // 셀 각주를 같은 영역에 섞지 않는다. 그 쪽의 기존 각주(표 25의
        // 105·106)를 보존하고, 표가 이어지는 새 쪽에서 셀 각주를 순서대로
        // 배치해야 원본 HWP/PDF의 107–111 / 112– 분할을 재현한다.
        let defer_large_first_fragment_notes = st.profile.hwp5_stored_pagination_layout()
            && queue_table_footnotes
            && !is_continuation
            && cursor_row == 0
            && table_footnotes.len() >= 8;
        if queue_table_footnotes
            && !defer_large_first_fragment_notes
            && !table_footnotes_follow_terminal
        {
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
                table_footnotes_follow_terminal,
                false,
                false,
            );
        }
        st.advance_column_or_new_page();

        // 커서 전진 — [작업 #993] 컷은 절대 유닛 인덱스이므로 누적 없이 대입.
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
        // 여러 행에 걸친 셀은 일반 행의 내용 유닛을 모두 소비했더라도
        // 이 조각 경계를 넘어 물리 공간을 소유한다.
        // 일반 칸의 독립 최소와 걸침 칸의 누적 공간은 각 장부가 소유한다.
        // 내용을 소진했다는 이유로 어느 쪽의 잔여 밴드도 없애지 않는다.
        let crossing_span_owns_tail = table.cells.iter().any(|cell| {
            (cell.row as usize) < end_row
                && cell.row as usize + (cell.row_span as usize).max(1) > end_row
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
                layout_engine.stored_inline_picture_closing_row_height(
                    table,
                    end_row.checked_sub(1)?,
                    &next_cut,
                    styles,
                )
            })
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
                    table, row, &next_cut,
                    input.fragment_row_heights.unwrap_or(&mt.row_heights),
                    end_row_height_override, styles,
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
                // 잔여 상자를 처음 발행하는 시점이 첫 조각일 필요는 없다.
                // 이미 기록한 같은 행의 모든 수용 상자를 원본 최소에서 빼고,
                // 잔여 상자를 이어받았으면 이번 조각만 그 잔여에서 차감한다.
                let total_accepted = continuation.pending_row_box
                    .filter(|(owner, _)| *owner == row)
                    .map_or(opening, |(_, height)| height);
                let accepted = if carried.is_some() { opening } else { total_accepted };
                let remaining = layout_engine.stored_ordinary_row_remainder(
                    table, row, &next_cut, accepted, carried, styles,
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
                // 빈 시작 조각은 그림 유닛을 소비하지 않고 물리 공간만 차지했다.
                // 겹치는 완전 행의 내용 상자에서 시작 조각을 빼지 않고,
                // 남은 소유 유닛의 전체 높이를 이어받는다.
                Some(layout_engine.row_cut_content_height(table, row, &next_cut, &[], styles))
            })
            .or_else(|| {
                let row = end_row.checked_sub(1)?;
                let minimum = layout_engine.reflow_row_physical_minimum(table, row, mt.row_heights[row], Some(&next_cut), styles);
                // 블록 컷은 여러 행의 셀 슬롯이다. 마지막 일반 행의 최소와
                // 빼서 시작 행 잔여로 넘기면 이미 소비한 행의 빈 띠를 재생성한다.
                if split_block_start.is_some()
                    || split_end_limit <= 0.0
                    || (!layout_engine.row_uses_reflow_physical_frame(table, row)
                        && minimum.is_none())
                {
                    return None;
                }
                let carried = (row == cursor_row).then_some(start_row_height_override).flatten();
                let full = if let Some(height) = carried {
                    height
                } else {
                    // 최소 높이가 전체 내용의 점유 높이보다 클 때만 물리 잔여가 생긴다.
                    // 내용에서 비롯된 높이는 이미 남은 유닛으로 표현되므로,
                    // 이중으로 이어받지 않는다.
                    minimum?
                };
                let start = if row == cursor_row { start_cut.as_slice() } else { &[] };
                let used = end_row_height_override.unwrap_or_else(|| {
                    layout_engine.row_cut_content_height(table, row, start, &next_cut, styles)
                });
                let tail = (full - used).max(0.0);
                // 독립적인 일반 칸의 큰 잔여는 병합 칸의 원장과 별도로 보존한다.
                // 내용까지 끝난 한 줄 이하의 빈 꼬리는 쪽 경계에서 흡수한다.
                if complete_physical_end_cut
                    && layout_engine.row_blank_tail_is_line_sliver(table, row, tail, styles)
                {
                    return None;
                }
                #[cfg(not(target_arch = "wasm32"))]
                if std::env::var_os("RHWP_DIAG_ROW_TAIL").is_some() {
                    eprintln!(
                        "DIAG_ROW_TAIL reflow pi={para_idx} row={row} complete={complete_physical_end_cut} full={full:.6} used={used:.6} tail={tail:.6} carried={carried:?} cut={next_cut:?}"
                    );
                }
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
                // 저장된 시작 프레임은 다음 프레임의 유닛을 소비하지 않고 빈 공간을 소유한다.
                // 원본 행의 최소 높이는 이어받기 커서를 통해
                // 스캔과 그리기가 공유한다.
                // 원본 HWPX에도 동일한 행 최소 높이가 저장된다. 한/글은
                // 분할 셀 높이를 각 조각 상자 높이의 합으로 기록한다
                // (36308670 1·2쪽: 2024 PDF에서 35618HU = 394.6 + 80.3px).
                if !first_fragment_blank_band
                    || !(st.profile.hwp5_stored_pagination_layout()
                        || st.profile.hwpx_stored_layout())
                {
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
                // 이어받은 잔여는 다음 단에서 하나의 시작 행 프레임으로 소비한다
                // (스캔은 저장된 이어받기 밴드를 다시 자르지 않는다).
                // 잔여 높이가 해당 단보다 크면 하나의 프레임이 아니므로,
                // 내용 컷이 계속 이어받기를 소유한다(113424 3쪽:
                // 선언 잔여 1271.7px > 본문 956.2px이며, 한컴 2024 PDF는
                // 내용에 따라 행을 3·4쪽으로 이어 배치한다).
                let repeated_header: f64 = if table.repeat_header {
                    table
                        .leading_header_rows()
                        .iter()
                        .map(|&r| mt.row_heights.get(r).copied().unwrap_or(0.0) + mt.cell_spacing)
                        .sum()
                } else {
                    0.0
                };
                (remaining > content + 0.5
                    && remaining + repeated_header <= st.available_height() + 0.5)
                    .then_some(remaining)
                })
            })
            .or_else(|| end_row_height_override
            // 완전히 소비한 것으로 확인된 원본 행은 다음 행으로 전진한다.
            // 수용한 앞부분에 병합 셀 소유자의 공간이 이미 반영되어 있으므로 이전 행의 빈 꼬리로 새 행의 상자를 대체하지 않는다.
            // 자식의 저장 컷이 닫는 첫 물리 띠는 전체 논리 행의 소비량이 아니다.
            // 이 띠를 원본 최소에서 빼 남은 내용보다 작은 상자를 이월하지 않는다.
            .filter(|_| nested_opening_extent.is_none() && !layout_engine.stored_heading_before_inline_frame_cut(
                table, end_row.saturating_sub(1),
                if end_row - 1 == cursor_row { start_cut } else { &[] }, &next_cut, styles,
            ) && !closes_original_body_frame && !complete_source_row_frame && !first_fragment_blank_band && !source_frame_trailing_trim_applied && stored_row_frame.is_none()
                && layout_engine.reflow_row_physical_minimum(table, end_row.saturating_sub(1), mt.row_heights[end_row.saturating_sub(1)], Some(&next_cut), styles).is_none()
                && (!layout_engine.row_uses_reflow_physical_frame(table, end_row.saturating_sub(1))
                    || crossing_span_owns_tail))
            .and_then(|limit| {
            let row = end_row.checked_sub(1)?;
            // 같은 일반 행을 다시 나누는 경우 앞 조각까지 차감한 물리 상자에서
            // 이번에 수용한 상자를 뺀다. 블록 컷의 별도 소유 공간은 유지한다.
            let carried = (row == cursor_row && !start_cut_is_block)
                .then_some(start_row_height_override)
                .flatten();
            // 내용 측정 높이는 앞 프레임의 줄간격도 담을 수 있다. 이미 닫힌
            // 앞 상자를 그 합에서 빼서 미소비 물리 공간으로 넘기지 않는다.
            // 수용한 잔여가 없으면 원본 행의 선언 최소 상자만 빈 띠를 소유한다.
            // 앞 조각의 실제 상자로 다시 푼 병합 종료 행은 그 공통 계획의
            // 최소 높이를 소비한다. 원시 선언으로 돌아가면 새 잔여를 잃는다.
            let resolved = (split_block_start.is_none())
                .then_some(input.fragment_row_heights)
                .flatten()
                // 원본 일반 행의 물리 최소는 아래의 독립 셀 선언에서 조회한다.
                // 측정 내용 합이나 이미 닫힌 프레임의 간격을 다시 넘기지 않는다.
                .filter(|_| !layout_engine.stored_ordinary_row_uses_declared_minimum(table, row))
                .and_then(|rows| rows.get(row).copied());
            // [#8105] 행 안 내용 컷으로 나뉜 행의 남는 몫은 그 행 자신의 칸 선언 높이에서 센다. 측정·저장
            // 행 높이는 내용으로 자란 행 전체(줄 사이 간격 포함)라 그것에서 이번 상자를 빼면 남은 내용에
            // 컷 줄의 줄간격까지 얹은 빈 띠가 된다(한/글 2020: 1613000 141·142쪽 행 88 — 칸 선언 1282HU
            // = 17.1px, 측정 행 38.4px; 한/글은 두 조각 모두 내용 17.1px). 남은 내용은 다음 스캔이 잰다.
            // 이 행에서 끝나는 병합 칸의 선언이 지나는 행들의 칸 선언 합보다 크면 남는 몫이 측정 행
            // 높이에 실린다 — 그 행은 공통 계획 높이를 그대로 쓴다(1342000 127·128쪽 행 78–80 칸
            // 263.3px). 선언이 행 합과 같으면 행 자신의 선언으로 잰다(1613000 143·144쪽 행 121–122 칸
            // 2564HU = 1282 × 2).
            let row_declared_hu = |r: usize| {
                table
                    .cells
                    .iter()
                    .filter(|cell| usize::from(cell.row) == r && cell.row_span == 1)
                    .filter(|cell| cell.height <= i32::MAX as u32)
                    .map(|cell| u64::from(cell.height))
                    .max()
                    .unwrap_or(0)
            };
            let span_ends_on_row = table.cells.iter().any(|cell| {
                cell.row_span > 1
                    && usize::from(cell.row) + usize::from(cell.row_span) == row + 1
                    && u64::from(cell.height)
                        > (usize::from(cell.row)..=row).map(row_declared_hu).sum::<u64>()
            });
            // 개체를 담은 칸의 행은 공통 계획 높이를 그대로 쓴다 — 개체 높이는 칸 선언 밖에서 자라고
            // 그 남은 몫은 내용 측정이 따로 잰다(86712 27·28쪽 행 27: 칸 선언 5.9px, 개체 글상자 칸).
            let row_cells_are_plain_text = table
                .cells
                .iter()
                .filter(|cell| usize::from(cell.row) == row && cell.row_span == 1)
                .all(|cell| cell.paragraphs.iter().all(|para| para.controls.is_empty()));
            let row_cell_declared = (split_end_limit > 0.0
                && !span_ends_on_row
                && row_cells_are_plain_text)
                .then(|| {
                    table
                        .cells
                        .iter()
                        .filter(|cell| usize::from(cell.row) == row && cell.row_span == 1)
                        .filter(|cell| cell.height <= i32::MAX as u32)
                        .map(|cell| hwpunit_to_px(cell.height as i32, self.dpi))
                        .reduce(f64::max)
                })
                .flatten();
            // 앞 조각이 이미 그린 행을 다시 잇는 조각은 앞 상자들을 장부로 받지 않았으면(`carried` 없음)
            // 행 전체 높이에서 이번 상자만 빼서는 안 된다 — 앞 쪽들이 그린 몫까지 넘긴다(86712 32–34쪽
            // 행 1: 측정 2262.5px − 이 쪽 929.6px = 1332.8px 를 34쪽에 실어 34쪽이 아래로 밀렸다).
            if carried.is_none()
                && row_cell_declared.is_none()
                && row == cursor_row
                && !start_cut.is_empty()
            {
                return None;
            }
            let full = carried.or(row_cell_declared).or(resolved).or_else(|| {
                let declared = *table.get_raw_row_heights().get(row)?;
                (declared <= i32::MAX as u32)
                    .then(|| hwpunit_to_px(declared as i32, self.dpi))
            })?;
            let tail = (full - limit).max(0.0);
            if complete_physical_end_cut
                && layout_engine.stored_complete_row_tail_is_line_advance(
                    table, end_row - 1, &next_cut, tail, styles,
                ) {
                return None;
            }
            // 내용이 끝난 행의 작은 빈 꼬리는 쪽 경계에서 흡수한다. 행을 덮는 병합 칸은
            // 그 몫을 자기 마지막 행에서 원장으로 이어받는다(#7642). 상한을 넘는 꼬리는
            // 병합 칸 유무와 무관하게 다음 쪽 첫머리 빈 띠다(한/글 2024 합성 사다리 #8105,
            // 1342000 59쪽 25.9px).
            if complete_physical_end_cut && layout_engine.complete_row_blank_tail_is_absorbed(tail) {
                return None;
            }
            // [#5714] 압축된 끝행의 빈 잔여 띠는 **물리적으로 이어지는
            // 것이 있을 때만** 다음 조각으로 넘어간다: 행 내부 컷이면 그
            // 행 자신이 이어지고, 행 경계 끝이면 경계를 가로지르는 병합
            // 셀의 선언 공간이 이어진다(76076 36쪽의 24.1px 밴드 — 한컴 PDF
            // 실측, r8 rs=7 셀이 경계를 걸침). 둘 다 아니면 완결된 행의
            // 선언 잔여는 쪽 경계에서 죽는다 — 병합 셀 없는 19×2 표에서
            // 이 잔여(11.0px)가 다음 조각 첫 행(새 행, 3줄 59.3px)에
            // 씌워져 글자가 아래 행과 포개졌다(한컴 PDF 는 밴드 없이 새
            // 행을 전체 높이로 시작).
            let tail_band_continues = split_end_limit > 0.0 || crossing_span_owns_tail;
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
        // 완료 일반 칸의 잔여와 같은 끝 행에 걸친 병합 칸의 잔여는 별도 소유다.
        let next_start_row_height_override = match (
            next_start_row_height_override,
            layout_engine.ending_span_minimum_remainder(
                row_geometry_table,
                end_row,
                &accepted_straddle_cells,
                complete_physical_end_cut,
                styles,
            ),
        ) {
            (Some(ordinary), Some(spanning)) => Some(ordinary.max(spanning)),
            (ordinary, spanning) => ordinary.or(spanning),
        };
        #[cfg(not(target_arch = "wasm32"))]
        if std::env::var_os("RHWP_DIAG_ROW_TAIL").is_some() {
            eprintln!(
                "DIAG_ROW_TAIL selected pi={para_idx} cursor={cursor_row} end={end_row} complete={complete_physical_end_cut} crossing_span={crossing_span_owns_tail} reflow={} override={next_start_row_height_override:?} cut={next_cut:?}",
                layout_engine.row_uses_reflow_physical_frame(table, end_row.saturating_sub(1))
            );
        }
        // 이미 예약한 띠와 이번 조각의 미예약 띠를 각각 한 번만 이어받는다.
        // 미예약 띠는 실제 행 상자를 늘리기 전에 계산해 예약과 paint가 공유한다.
        let page_end_band = if split_block_start.is_some() {
            0.0
        } else {
            source_frame_band.max(reserved_straddle_end_band) + ledger_page_end_band
        };
        // 완료 일반 행을 다시 열지 않는다. 계속되는 병합 칸의 실제 공간은 공통 행 계획이 예약한다.
        let has_intra_row_tail = split_end_limit > 0.0
            && !(complete_physical_end_cut && next_start_row_height_override.is_none());
        continuation.straddle_page_bands =
            crate::renderer::layout::LayoutEngine::straddle_page_bands_after_fragment(
                row_geometry_table,
                &accepted_straddle_cells,
                if has_intra_row_tail {
                    end_row.saturating_sub(1)
                } else {
                    end_row
                },
                page_end_band,
                plain_cut_owns_blank_band,
            );
        continuation.stored_block_frame = block_acceptance.and_then(|accepted| accepted.frame);
        continuation.advance(
            end_row,
            split_block_start,
            next_source_cut,
            has_intra_row_tail,
        );
        continuation.start_row_height_override = next_start_row_height_override;
        continuation.stored_reset_band_row_box = span_band_row_box;
        if let Some(((_, row, cut, height), _)) = stored_rowspan_frame {
            continuation.row = row;
            continuation.start_cut = cut;
            continuation.start_cut_is_block = false;
            continuation.start_row_height_override = Some(height);
        }

        TableContinuationIteration::Emitted
    }

    /// 다음 실제 흐름 문단이 증언한 마지막 표 여백만 수용한다.
    /// 지연 배치하는 떠 있는 개체 앞에서 이미 채운 문단은 이 조각의 후속 문단이 아니다.
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
