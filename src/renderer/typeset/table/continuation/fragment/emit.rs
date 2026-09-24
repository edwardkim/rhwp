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
        let cut_row_h = &input.prepared.cut_row_heights;
        let caption_is_top = input.prepared.caption_is_top;
        let caption_overhead = input.prepared.caption_overhead;
        let queue_table_footnotes = input.prepared.queue_table_footnotes;
        let table_footnotes = &input.prepared.table_footnotes;
        let host_spacing_total = input.prepared.host_spacing_total;
        let host_spacing_after_only = input.prepared.host_spacing_after_only;
        let terminal_nested_child_host_line_spacing =
            input.prepared.terminal_nested_child_host_line_spacing;
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
            terminal_outer_bottom_overhead,
            fragment_outer_bottom_overhead,
            vert_offset_overhead,
            page_avail,
            fragment_placement,
            header_overhead,
            avail_for_rows,
            single_cell_fragment_shape,
            single_cell_box_height,
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
        // [Task #1022] walk 가 consumed 에 분할 행 기여까지 누적하므로
        // partial_height = consumed + header_overhead 로 단일화.
        let mut partial_height: f64 = consumed + header_overhead;
        let commit_fragment = |st: &mut TypesetState, owner_height: f64, terminal: bool| {
            if let Some(mut placement) = fragment_placement {
                placement.occupied_bottom = placement.table_top
                    + owner_height
                    + hwpunit_to_px(table.outer_margin_bottom as i32, self.dpi);
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

        // 마지막 파트에 Bottom 캡션 공간 확보
        if end_row >= row_count
            && split_end_limit == 0.0
            && !caption_is_top
            && caption_overhead > 0.0
        {
            let total_with_caption = partial_height + caption_overhead;
            let avail = if is_continuation {
                (page_avail - header_overhead).max(0.0)
            } else {
                page_avail
            };
            if total_with_caption > avail {
                end_row = end_row.saturating_sub(1);
                if end_row <= cursor_row {
                    end_row = cursor_row + 1;
                }
            }
        }

        if end_row >= row_count && split_end_limit == 0.0 {
            let skip_terminal_empty_sliver = is_continuation
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
                                + terminal_nested_child_host_line_spacing
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
            if cursor_row == 0 && !is_continuation && start_cut.is_empty() {
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
                        + terminal_nested_child_host_line_spacing,
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
            // 기존 블록 조각 게이트는 시작/끝 블록을 포함한다. 이를 끝 컷 전용으로
            // 바꾸면 block→row 조각의 예약/배치 계약도 함께 바꿔야 한다.
            // 시작 컷 해석에는 이 게이트 대신 start_cut_is_block을 사용한다.
            is_block_split: split_block_start.is_some() || start_cut_is_block,
            start_cut_is_block,
            row_cursor_is_nested,
            end_row_height_override,
            start_row_height_override,
        });
        // 저장 host 원점이 없는 조각은 흐름 좌표로 같은 상자를 잰다 — 위는 흐름 커서 +
        // host·세로 오프셋, 아래는 비끝 조각 상자 바닥(7062 2~9쪽 996.49 ↔ 정본 996.43).
        // 첫 조각의 위는 흐름 커서가 아니라 host 저장 vpos 다 — 렌더러가 그 자리에 칠하고,
        // 흐름은 저장 사다리보다 늦을 수 있다(7062: 흐름 484.3 ↔ 저장 488.8, 156645214:
        // 116.3 ↔ 121.0). 흐름으로 재면 첫 상자가 그만큼 커져 끝 상자가 모자란다.
        let single_cell_box_height = single_cell_box_height.or_else(|| {
            single_cell_fragment_shape.then(|| {
                let flow_top = if is_continuation {
                    st.current_height
                } else {
                    input
                        .source
                        .paragraph
                        .line_segs
                        .iter()
                        .find(|seg| !is_synthetic_line_seg(seg))
                        .map(|seg| {
                            hwpunit_to_px(
                                seg.vertical_pos
                                    .saturating_sub(st.vpos_page_base.unwrap_or(0)),
                                self.dpi,
                            )
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
        // [#2238] 중간 fragment 가시높이 부기 — used_height(flush 시 current_height)
        // 표시용. advance 직후 current_height 가 리셋되므로 흐름/기하 불변.
        st.advance_flow_by(
            host_before_overhead
                + vert_offset_overhead
                + partial_height
                + fragment_outer_bottom_overhead,
        );
        if terminal_cut_consumed {
            st.advance_flow_by(host_spacing_after_only + terminal_nested_child_host_line_spacing);
            commit_fragment(st, caption_extra + partial_height, true);
            continuation.finish(row_count, true);
            return TableContinuationIteration::Complete;
        }
        commit_fragment(st, caption_extra + partial_height, false);
        // 큰 RowBreak 표가 기존 각주를 이미 가진 page에서 시작할 때에는 첫 fragment의
        // cell-footnote를 같은 lane에 섞지 않는다. 그 page의 기존 각주(표 25의
        // 105·106)를 보존하고, 표가 이어지는 fresh page에서 cell-footnote를 순서대로
        // 배치해야 원본 HWP/PDF의 107–111 / 112– 분할을 재현한다.
        let defer_large_first_fragment_notes = queue_table_footnotes
            && !is_continuation
            && cursor_row == 0
            && table_footnotes.len() >= 8;
        if queue_table_footnotes && !defer_large_first_fragment_notes {
            self.register_queued_table_footnotes(
                st,
                continuation,
                table_footnotes,
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
        let next_cut = if split_end_limit > 0.0 {
            split_end_cut
        } else {
            Vec::new()
        };
        let next_start_row_height_override = end_row_height_override.and_then(|limit| {
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
        });
        continuation.advance(end_row, split_block_start, next_cut, split_end_limit > 0.0);
        continuation.start_row_height_override = next_start_row_height_override;
        TableContinuationIteration::Emitted
    }
}
