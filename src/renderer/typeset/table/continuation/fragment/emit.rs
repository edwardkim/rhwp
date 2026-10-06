//! Commit an accepted fragment, queued notes and cursor advance in their original order.

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
            .as_ref()
            .unwrap_or(&input.prepared.cut_row_heights);
        let straddle_row_relief = &input.start.straddle_row_relief;
        let caption_is_top = input.prepared.caption_is_top;
        let caption_overhead = input.prepared.caption_overhead;
        let queue_table_footnotes = input.prepared.queue_table_footnotes;
        let table_footnotes = &input.prepared.table_footnotes;
        let host_spacing_total = input.prepared.host_spacing_total;
        let host_spacing_after_only = input.prepared.host_spacing_after_only;
        let terminal_host_spacing = input.prepared.terminal_host_spacing;
        let relax_terminal_table_footnote_fit = input.prepared.relax_terminal_table_footnote_fit;
        let cursor_row = input.start.cursor_row;
        let is_continuation = input.start.is_continuation;
        let start_cut_is_block = input.start.start_cut_is_block;
        let start_row_height_override = input.start.start_row_height_override;
        let start_cut = &input.start.start_cut;
        let fragment_starts_intra_row = input.start.fragment_starts_intra_row;
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
        let BlockTableRowScan {
            consumed,
            mut end_row,
            split_block_start,
            split_end_cut,
            split_end_limit,
            mut end_row_height_override,
        } = scan;
        // A cut consumes content; the declared cell minimum also owns blank
        // physical space. Carry actual accepted boxes rather than estimating
        // them from preceding content cuts.
        let stored_row_frame = end_row.checked_sub(1).and_then(|row| {
            if split_end_limit <= 0.0
                || split_end_cut.is_empty()
                || split_block_start.is_some()
                || end_row_height_override.is_some()
            {
                return None;
            }
            // Frame ownership uses the same unreserved source body height as
            // paint. Footnotes and zones constrain acceptance below, not the
            // existence of the source-owned minimum.
            let declared = layout_engine.stored_full_width_row_declared_height(
                table,
                row,
                styles,
                st.layout.body_area.height,
            )?;
            let used = continuation
                .stored_row_box_sum
                .filter(|(owner, _)| *owner == row)
                .map_or(0.0, |(_, height)| height);
            let before_row = (consumed - split_end_limit).max(0.0);
            let available = (avail_for_rows - before_row).max(0.0);
            let height = (declared - used)
                .max(0.0)
                .min(available)
                .max(split_end_limit);
            Some((row, used, height))
        });
        // 행 컷이 소비한 내용과 header의 요구 높이. 저장 상자의 빈 밴드는
        // 아래에서 별도로 물리 점유에 포함하며 컷 유닛을 더 소비하지 않는다.
        let mut partial_height: f64 = consumed + header_overhead;
        if let Some((_, _, height)) = stored_row_frame {
            if height > split_end_limit + 0.5 {
                end_row_height_override = Some(height);
                partial_height += height - split_end_limit;
            }
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
            && st.profile.hwpx_stored_layout()
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
        // 실제 컷이 새 원본 쪽 프레임에서 시작한 경우에만 선언 차이를 쓴다.
        let single_cell_closing_frame = (is_continuation
            && cursor_row == 0
            && end_row == 1
            && split_end_cut.is_empty()
            && start_cut.len() == 1
            && start_cut[0] > 0
            && !st.profile.session_edited()
            && st.profile.hwpx_stored_layout()
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
                    crate::renderer::float_placement::stored_single_cell_closing_frame_height(
                        input.source.paragraph,
                        next,
                        table,
                        self.dpi,
                    )
                })
        })
        .flatten();
        if let Some(height) = single_cell_closing_frame
            .filter(|height| *height >= partial_height && *height <= avail_for_rows)
        {
            end_row_height_override = Some(height);
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
        if !is_continuation
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
                    if frame_height > partial_height + 0.5
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
                st.profile.hwpx_stored_layout(),
                st.profile.session_edited(),
            );
        let first_fragment_blank_band = !is_continuation
            && (opening_frame_has_source_boundary
                || saved_block_opening_frame.is_some()
                || body_filling_source_frame)
            && (split_block_start.is_none() || saved_block_opening_frame.is_some())
            && end_row_height_override.is_none()
            && std::ptr::eq(table, row_geometry_table)
            && (crate::renderer::float_placement::object_only_saved_table_anchor(
                input.source.paragraph,
                table,
            ) || body_filling_source_frame
                || saved_closing_frame.is_some()
                || saved_block_opening_frame.is_some()
                || (input.source.paragraph.text.is_empty()
                    && matches!(
                        input.source.paragraph.controls.as_slice(),
                        [crate::renderer::typeset::Control::Table(_)]
                    )
                    && layout_engine.row_cut_starts_intra_paragraph_stored_frame(
                        table,
                        end_row - 1,
                        &split_end_cut,
                        styles,
                    )))
            && saved_opening_frame.is_some_and(|frame_height| {
                (frame_height > partial_height + 0.5
                    || (body_filling_source_frame && frame_height >= partial_height)
                    || saved_block_opening_frame.is_some())
                    && frame_height <= avail_for_rows + header_overhead
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
        // 종료 조각의 실제 프레임과 뒤 저장 줄이 아래 여백을 닫으면 동일한
        // 배치 계획으로 예약과 paint 흐름을 함께 전진시킨다.
        if is_continuation
            && end_row >= row_count
            && split_end_limit == 0.0
            && fragment_placement.is_none()
            && terminal_outer_bottom_overhead == 0.0
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
                terminal_outer_bottom_overhead = margin;
                fragment_placement =
                    Some(crate::renderer::float_placement::ParagraphFloatPlacement {
                        flow: crate::renderer::float_placement::ParagraphFloatFlow::NextLine,
                        anchor_y: st.current_height,
                        stored_host_origin: None,
                        stored_successor_line_origin: None,
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
        let declared_opening_frame = single_cell_fragment_shape
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
            );
        if declared_opening_frame {
            partial_height = hwpunit_to_px(table.common.height as i32, self.dpi);
            end_row_height_override = Some(partial_height);
        }
        // 종료 상자의 마지막 유닛 뒤 빈 공간은 누적 상자와 잔여의 합이 cellSz를 닫고
        // 다음 원본 줄이 전체 바깥 상자를 닫을 때만 수용한다. 같은 계획이 후속 간격을 소유한다.
        let mut terminal_saved_successor_closure = false;
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
                terminal_saved_successor_closure = true;
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
        let commit_fragment = |st: &mut TypesetState, owner_height: f64, terminal: bool| {
            if let Some(mut placement) = fragment_placement {
                // 실제 조각의 컷/쪽 소유로 바뀌었으므로 전체 프레임의 후속 원점은 재사용하지 않는다.
                if !terminal || !terminal_saved_successor_closure {
                    placement.stored_successor_line_origin = None;
                }
                placement.occupied_bottom = placement.table_top
                    + owner_height
                    + if terminal {
                        terminal_outer_bottom_overhead
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
                            host_spacing_after_only
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
                // A reflowed declared tail owns real physical space even after its
                // final content unit. Do not erase that frame as an empty sliver.
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
                });
                // 마지막 fragment: spacing_after만 포함 (Paginator engine.rs:1051 동일)
                // host line advance/positive offset은 원 anchor 조각의 계약이며,
                // continuation 끝에서 다시 더하면 다음 본문을 이중으로 민다(#2439).
                st.advance_flow_by(
                    host_before_overhead
                        + vert_offset_overhead
                        + partial_height
                        + bottom_caption_extra
                        + terminal_outer_bottom_overhead
                        + host_spacing_after_only
                        + terminal_host_spacing,
                );
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
                    fragment_starts_intra_row,
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
                        fragment_starts_intra_row,
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
        let terminal_cut_consumed = end_row >= row_count
            && split_end_limit > 0.0
            && !split_end_cut.is_empty()
            && split_block_start.is_none()
            && !start_cut_is_block
            && !row_cursor_is_nested
            && end_row_height_override.is_none()
            && mt.allows_row_break_split()
            && caption_overhead <= 0.0
            && !queue_table_footnotes
            && can_intra_split
            && layout_engine
                .advance_row_cut(
                    row_geometry_table,
                    row_count - 1,
                    &split_end_cut,
                    f64::MAX,
                    styles,
                )
                .consumed_height
                <= 0.0
            && layout_engine
                .straddle_continuation_demand(
                    row_geometry_table,
                    row_count - 1,
                    row_count - 1,
                    &split_end_cut,
                    None,
                    &mt.row_heights,
                    styles,
                    (row_count, true),
                )
                .is_none_or(|remaining| remaining <= 0.0);

        // 중간 또는 내용이 완전히 소비된 최종 컷 fragment 배치
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
        if let Some((row, used, height)) = stored_row_frame {
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
            st.advance_flow_by(host_spacing_after_only + terminal_host_spacing);
            commit_fragment(st, caption_extra + partial_height, true);
            continuation.finish(row_count, true);
            return TableContinuationIteration::Complete;
        }
        commit_fragment(st, caption_extra + partial_height, false);
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
                fragment_starts_intra_row || !split_end_cut.is_empty(),
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
        let next_cut = if split_end_limit > 0.0 {
            split_end_cut
        } else {
            Vec::new()
        };
        // 빈 컷의 블록 경계는 모든 앞 행의 내용을 끝낸 상태다. 병합 셀의
        // 저장 물리 높이에서 실제 그린 밴드를 빼 다음 완전 행 상자로 넘긴다.
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
                Some(hwpunit_to_px(source.height as i32, self.dpi) - used)
            });
        let next_start_row_height_override = empty_opening_next_height
            .or(complete_block_next_height)
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
                // The blank opening consumed physical space, not picture units.
                // Carry the remaining owners' full height rather than subtracting
                // the opening from the overlapping complete-row content box.
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
                    // A minimum creates a physical tail only when it exceeds the
                    // complete content occupancy. Content-driven height is already
                    // represented by the remaining units and must not be carried twice.
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
                (remaining > 0.0).then_some(remaining)
                })();
                base_remaining.or_else(|| {
                // A stored opening frame owns blank space without consuming
                // the next frame's units. Its source row minimum is shared
                // with scan and paint through the continuation cursor.
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
            .filter(|_| !first_fragment_blank_band && !source_frame_trailing_trim_applied && stored_row_frame.is_none()
                && !layout_engine.row_uses_reflow_physical_frame(table, end_row.saturating_sub(1)))
            .and_then(|limit| {
            let full = cut_row_h.get(end_row.saturating_sub(1)).copied()?;
            let tail = (full - limit).max(0.0);
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
            (tail > 0.5 && tail_band_continues).then_some(tail)
        }));
        // [#7531] 행 경계에서 끝난 조각의 쪽 끝 빈 띠는 그 경계를 넘는 걸침 칸이 차지한다.
        // 행 안에서 끊긴 조각도 남은 물리 공간은 같은 띠다(끝 행이 쪽을 채우면 0).
        let page_end_band = if split_block_start.is_some() {
            0.0
        } else {
            // 한/글은 쪽 끝 행 상자를 본문 바닥에서 표 바깥 아래 여백을 뺀 자리까지 늘린다.
            let outer_bottom = hwpunit_to_px(i32::from(table.outer_margin_bottom), self.dpi);
            (avail_for_rows + header_overhead - partial_height - outer_bottom).max(0.0)
        };
        continuation.straddle_page_bands =
            crate::renderer::layout::LayoutEngine::straddle_page_bands_after_fragment(
                row_geometry_table,
                &continuation.straddle_page_bands,
                if split_end_limit > 0.0 {
                    end_row.saturating_sub(1)
                } else {
                    end_row
                },
                page_end_band,
            );
        continuation.advance(end_row, split_block_start, next_cut, split_end_limit > 0.0);
        continuation.start_row_height_override = next_start_row_height_override;
        if let Some(((_, row, cut, height), _)) = stored_rowspan_frame {
            continuation.row = row;
            continuation.start_cut = cut;
            continuation.start_cut_is_block = false;
            continuation.start_row_height_override = Some(height);
        }

        TableContinuationIteration::Emitted
    }
}
