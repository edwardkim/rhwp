//! Split geometry and reservation preparation. Runs only after whole-placement fails.

use crate::renderer::typeset::{
    controls, hwpunit_to_px, hwpx_stored_tac_table_starts_at_page_top, is_para_topbottom_float,
    is_synthetic_line_seg, is_two_row_picture_caption_rowbreak_table,
    native_hwp5_rowbreak_host_precedes_first_fragment, native_terminal_child_host_line_spacing,
    none_table_is_atomic_here, notes, para_has_visible_text, paragraph,
    partial_rowbreak_fragment_spacing_px, row_geometry_table,
    rowbreak_table_has_internal_saved_vpos_reset, stored_square_picture_has_adjacent_text, table,
    BlockTableContinuationContext, BlockTableContinuationPreparedState,
    BlockTableContinuationSource, CaptionDirection, Control, PageItem, TypesetEngine, TypesetState,
    MIN_TOP_KEEP_PX, SINGLE_ROW_DECLARED_TRUST_MAX_RATIO,
};

use super::{BlockTableInput, SplitTableEntry};

impl TypesetEngine {
    pub(super) fn prepare_block_table_continuation<'a>(
        &self,
        st: &mut TypesetState,
        input: BlockTableInput<'a>,
        entry: SplitTableEntry<'a>,
    ) -> (
        BlockTableContinuationContext,
        BlockTableContinuationSource<'a>,
    ) {
        let BlockTableInput {
            para_idx,
            ctrl_idx,
            para,
            table,
            ft,
            fmt,
            styles,
            budget_para_start_height,
            paragraphs_all,
            composed_all,
            ..
        } = input;
        let SplitTableEntry {
            mut total_footnote,
            next_starts_new_page,
            next_rewinds_after_table,
            host_spacing_total,
            mut table_total,
            native_ordinary_rowbreak_rewind_uses_actual_footnote_boundary,
            mut fn_margin,
            mut available,
            declared_object_total,
            native_hwp5_internal_reset_rewind_needs_anchor_resync,
            placement_para_start_height,
            source_anchor_splits_here,
            native_hwp5_rewinding_rowbreak_uses_painted_row_footprint,
            unconstrained_host_placement,
            constrain_host_placement,
            mt,
            row_geometry_table,
            declared_table_height,
            ..
        } = entry;
        let row_count = mt.row_heights.len();
        let cs = mt.cell_spacing;
        let can_intra_split = !mt.cells.is_empty();
        let base_available = st.base_available_height();
        // Partial table borders are rendered against the visible body area. The paginator-level
        // bottom tolerance is useful for text fit heuristics, but if row cuts spend it here the
        // table fragment can be painted into the footer/body edge and get clipped.
        let mut table_available = (available - st.layout.pagination_tolerance_px).max(0.0);

        // [Task #993] advance_row_cut 호출용 LayoutEngine — 컷 측정은 dpi 와
        // 셀 패딩/중첩 표 높이 계산에만 의존하므로 ad hoc 인스턴스로 충분하다.
        let layout_engine = crate::renderer::layout::LayoutEngine::new(self.dpi);
        layout_engine.set_layout_profile(st.profile);
        // Row-cut measurement is a renderer consumer, not an independent
        // session. It must see the same table provenance as this typesetter.
        layout_engine
            .set_render_normalization_overlay(std::sync::Arc::clone(&self.render_normalization));
        // [Task #993] rowspan(row_span>1) 셀이 걸친 행 — 컷 모델(advance_row_cut)은
        // row_span==1 셀만 다루므로 rowspan 셀 높이를 측정하지 못한다. 구현계획서
        // §4대로 rowspan 행은 MeasuredTable 행 높이를 권위로 쓴다(렌더러도 동일).
        let rowspan_touched: Vec<bool> = (0..row_count)
            .map(|r| {
                row_geometry_table.cells.iter().any(|c| {
                    c.row_span > 1
                        && (c.row as usize) <= r
                        && r < c.row as usize + c.row_span as usize
                })
            })
            .collect();
        // [Task #993/#1022] 행별 전체 높이(fresh, 빈 컷). HeightMeasurer 와 정합된
        // row_cut_content_height(셀별 max(cell.height, content+pad_cell) 의 행 max)
        // 로 측정해 렌더러와 단일 측정 공간을 공유한다.
        //
        // rowspan 행은 기본적으로 저장 행 높이(MeasuredTable)를 기준으로 삼는다. 다만
        // 같은 행의 row_span==1 셀 내용이 저장 행 높이를 초과하면, 저장 높이가 실제
        // 셀 내용보다 작게 기록된 경우이므로 컷 높이를 쓴다.
        // 이 판정은 파일명/페이지가 아니라 표 셀 내용 높이와 저장 행 높이의 차이에 근거한다.
        let row_has_stored_square_picture_flow = |row: usize| {
            if !st.profile.hwp5_stored_pagination_layout()
                || !matches!(
                    row_geometry_table.page_break,
                    crate::model::table::TablePageBreak::RowBreak
                )
            {
                return false;
            }
            row_geometry_table.cells.iter().any(|cell| {
                cell.row as usize == row
                    && cell.row_span == 1
                    && cell.paragraphs.iter().enumerate().any(|(para_idx, para)| {
                        para.controls.iter().enumerate().any(|(control_idx, _)| {
                            stored_square_picture_has_adjacent_text(cell, para_idx, control_idx)
                        })
                    })
            })
        };
        let cut_row_h: Vec<f64> = (0..row_count)
            .map(|r| {
                let has_single_row_cells = row_geometry_table
                    .cells
                    .iter()
                    .any(|c| c.row as usize == r && c.row_span == 1);
                let row_cut_h = if row_has_stored_square_picture_flow(r) {
                    // native HWP5 RowBreak stores the adjacent text LINE_SEG ladder as
                    // the authoritative row height. Recomputing this whole row from
                    // cell_units would add the same Square flow band again.
                    mt.row_heights[r]
                } else if has_single_row_cells {
                    layout_engine.row_cut_content_height(
                        row_geometry_table,
                        r,
                        &[],
                        &[],
                        styles,
                        false,
                    )
                } else {
                    0.0
                };
                let row_content_exceeds_stored =
                    has_single_row_cells && row_cut_h > mt.row_heights[r];
                let allow_rowspan_content_height = row_content_exceeds_stored;
                if rowspan_touched[r] && (!has_single_row_cells || !allow_rowspan_content_height) {
                    mt.row_heights[r]
                } else if has_single_row_cells {
                    row_cut_h
                } else {
                    mt.row_heights[r]
                }
            })
            .collect();
        // 온전한 행을 받는 경로는 실제 paint footprint를 예약한다. 셀 선언
        // 높이에 비해 패딩이 과대하면 내용 컷은 축소된 패딩을 쓰지만, 전체 행은
        // MeasuredTable의 콘텐츠+패딩 높이를 그린다(#7234). 부분 컷 좌표는 유지한다.
        // 중첩 표 등 컷 높이를 실제 배치에도 쓰는 행은 같은 owner 판정으로 제외한다.
        let painted_row_heights = layout_engine.resolve_row_heights_trusting_declared(
            row_geometry_table,
            row_geometry_table.col_count as usize,
            row_count,
            Some(mt),
            styles,
            row_geometry_table.common.treat_as_char,
            false,
        );
        // 선언 높이가 첫 저장 조각만 나타내는 표는 전체 MeasuredTable 높이를
        // 행별 예약으로 되돌려 넣을 수 없다. 전체 행의 paint 합과 선언 object
        // 상자가 일치할 때만 같은 온전한 표의 패딩 차이라고 입증된다.
        let declared_whole_table_matches_paint = row_geometry_table.common.height > 0
            && (painted_row_heights
                .iter()
                .zip(&cut_row_h)
                .enumerate()
                .map(|(row, (painted, cut))| {
                    // 중첩 행은 아래 paint에서도 내용 컷 높이를 소비한다. 서로 다른
                    // 소유자의 높이를 모두 MeasuredTable로 합치면 짧은 중첩 꼬리가
                    // 있는 표의 앞 행까지 과예약한다(76076 p81).
                    if layout_engine
                        .whole_fragment_row_uses_measured_height(row_geometry_table, row)
                    {
                        *painted
                    } else {
                        *cut
                    }
                })
                .sum::<f64>()
                + cs * row_count.saturating_sub(1) as f64
                - hwpunit_to_px(row_geometry_table.common.height as i32, self.dpi))
            .abs()
                <= 0.1;
        let whole_row_fit_h: Vec<f64> = cut_row_h
            .iter()
            .zip(&painted_row_heights)
            .enumerate()
            .map(|(row, (cut, painted))| {
                let padding_explains_drift = layout_engine
                    .whole_row_height_diff_is_padding_reduction(
                        row_geometry_table,
                        row,
                        *cut,
                        *painted,
                    );
                if native_hwp5_rewinding_rowbreak_uses_painted_row_footprint {
                    // 기존 저장 rewind 경로는 MeasuredTable의 물리 행 높이를 소비한다.
                    // 패딩 축소 복구와 무관한 행을 새 resolve 결과로 바꾸지 않는다.
                    cut.max(mt.row_heights[row])
                } else if declared_whole_table_matches_paint
                    && padding_explains_drift
                    && layout_engine
                        .whole_fragment_row_uses_measured_height(row_geometry_table, row)
                {
                    cut.max(*painted)
                } else {
                    *cut
                }
            })
            .collect();
        // p106은 paint footprint 기준 row 0–3이 body bottom보다 3.9px 앞에서
        // 끝나지만, 한컴은 다음 row를 continuation으로 소유한다. 이 4px은 native
        // HWP5 stored-rewind first fragment의 footer-local slack이며 전역 safety
        // margin이 아니다. partial row와 continuation에는 적용하지 않는다.
        const NATIVE_REWIND_FIRST_FRAGMENT_PAINT_FOOTER_GUARD_PX: f64 = 4.0;
        let first_fragment_painted_row_footer_guard =
            if native_hwp5_rewinding_rowbreak_uses_painted_row_footprint
                // A visible host line that occupies the table's own positive
                // offset lane moves the painted fragment down by only the
                // offset remainder.  The exact existing-footnote boundary is
                // already authoritative for this path; subtracting the p106
                // empty-host safety guard again drops table 27's final fitting
                // row by ~1px after its caption is restored.
                && !native_hwp5_rowbreak_host_precedes_first_fragment(para, table)
                && whole_row_fit_h
                    .iter()
                    .zip(&cut_row_h)
                    .any(|(painted, cut)| painted > &(cut + 0.5))
            {
                NATIVE_REWIND_FIRST_FRAGMENT_PAINT_FOOTER_GUARD_PX
            } else {
                0.0
            };

        // [Task #1046 Stage 1] 분할 표 cut 행높이 vs 렌더러 MeasuredTable 행높이 비교.
        if std::env::var("RHWP_TABLE_DRIFT").is_ok() {
            let cut_sum: f64 = cut_row_h.iter().sum();
            let mt_sum: f64 = mt.row_heights.iter().sum();
            eprintln!(
                "TABLE_CUT_DRIFT: pi={} sec={} cut_sum={:.1} mt_sum={:.1} diff={:+.1} cut_rows={:?} mt_rows={:?}",
                para_idx, st.section_index, cut_sum, mt_sum, mt_sum - cut_sum,
                cut_row_h.iter().map(|h| (h * 10.0).round() / 10.0).collect::<Vec<_>>(),
                mt.row_heights.iter().map(|h| (h * 10.0).round() / 10.0).collect::<Vec<_>>(),
            );
        }

        // [#3738 Stage 9/17] RowBreak 표의 셀 각주를 첫 행 전부터 전부 예약하면,
        // 표가 여러 physical page로 나뉘는 경우에도 첫 fragment가 통째로 밀린다.
        // 실제로 표 25(pi=885)는 18개 URL 각주 667px을 먼저 빼서 p78의 표 시작을
        // 막고, 마지막 fragment에 전부 등록해 p80 표 위로 겹쳤다. native HWP5의
        // rowspan 없는 RowBreak 표는 fragment를 먼저 확정한 뒤 그 page에 들어가는
        // 각주만 순서대로 queue한다. HWPX와 rowspan/intra-row ownership은 기존 경로를
        // 유지한다.
        let no_table_note_available = {
            let projected = st.projected_footnote_height(0.0, 0);
            let margin = if projected > 0.0 {
                st.footnote_safety_margin
            } else {
                0.0
            };
            (st.base_available_height()
                - projected
                - margin
                - st.current_zone_y_offset
                - st.layout.pagination_tolerance_px)
                .max(0.0)
        };
        // [#3820 Stage 11] p174의 표 46처럼 1×1 빈-host RowBreak 표도 실제 셀은
        // 여러 physical fragment로 나뉠 수 있다. 이 형상에 표 전체 각주를 먼저
        // 예약하면, 첫 marker(223)가 현쪽에 있더라도 그림과 뒤 문단을 위한 본문
        // 공간까지 빼앗아 p174→p176으로 한 쪽씩 늦어진다. 한글은 첫 fragment에는
        // marker만 두고, 다음 fragment의 footer lane에 223–231을 둔다.
        //
        // 일반 단일 행 표까지 queue로 넓히면 작은 고정-height 표의 각주 ownership이
        // 바뀐다. native HWP5·비-TAC·TopAndBottom·빈 host·RowBreak·1×1,
        // 8개 이상 각주, 그리고 measured 높이가 declared 높이의 1.5배를 넘는
        // 실제 대형 cell이라는 저장/측정 계약을 모두 요구한다. 최소 한 줄만
        // 시작할 수 있는 본문 여백도 확인해, 빈 조각을 만드는 경우는 제외한다.
        let native_hwp5_oversized_single_row_fragment_queues_footnotes =
            !table.common.treat_as_char
                && st.profile.hwp5_stored_pagination_layout()
                && is_para_topbottom_float(&table.common)
                && matches!(
                    table.page_break,
                    crate::model::table::TablePageBreak::RowBreak
                )
                && !para_has_visible_text(para)
                && row_count == 1
                && table.cells.len() == 1
                && ft.table_footnote_count >= 8
                && table_total > declared_object_total * SINGLE_ROW_DECLARED_TRUST_MAX_RATIO
                && st.current_height > 0.5
                && no_table_note_available >= st.current_height + MIN_TOP_KEEP_PX + 0.5;
        // 한컴이 cell-footnote의 첫 두 stored line을 모두 `vpos=0`으로 저장한
        // 경우에는 표가 작고 terminal이어도 각주 자체가 physical page 둘을
        // 소유한다(p176 note 234). 전체 각주를 먼저 예약하는 일반 경로에서는
        // 이 명시적 경계를 보존할 수 없으므로 fragment queue가 맡는다.
        let native_hwp5_stored_page_footnote_split = ft.table_footnotes.iter().any(|note| {
            note.fragment_split
                .is_some_and(|split| split.force_next_page)
        });
        let queue_table_footnotes = !table.common.treat_as_char
            && st.profile.hwp5_stored_pagination_layout()
            && matches!(
                table.page_break,
                crate::model::table::TablePageBreak::RowBreak
            )
            && !ft.table_footnotes.is_empty()
            && table.cells.iter().all(|cell| cell.row_span == 1)
            && ((row_count > 1
                // 기존 page의 일반 각주는 유지한 채, 표 첫 행만은 실제로 시작할 수 있어야 한다.
                && no_table_note_available >= st.current_height + cut_row_h[0] + 0.5)
                || native_hwp5_oversized_single_row_fragment_queues_footnotes
                || native_hwp5_stored_page_footnote_split);
        if queue_table_footnotes {
            if std::env::var("RHWP_TABLE_DRIFT").is_ok() {
                eprintln!(
                    "TABLE_FOOTNOTE_QUEUE pi={} rows={} notes={} no_table_available={:.1} cur_h={:.1}",
                    para_idx,
                    row_count,
                    ft.table_footnote_count,
                    no_table_note_available,
                    st.current_height,
                );
            }
            total_footnote = st.projected_footnote_height(0.0, 0);
            fn_margin = if total_footnote > 0.0 {
                if next_starts_new_page {
                    0.0
                } else {
                    st.footnote_safety_margin
                }
            } else {
                0.0
            };
            available = (st.base_available_height()
                - total_footnote
                - fn_margin
                - st.current_zone_y_offset)
                .max(0.0);
            // row cut과 문서 저장 높이의 sub-pixel 변환 차이로, 기준 PDF에서
            // 정확히 바닥에 닿는 마지막 행을 불필요하게 다음 fragment로 보내지
            // 않도록 이 작은 queue 경로에만 round-off 여유를 준다. 위의 대형
            // 단일-cell 계약은 그림 66 뒤 마지막 1줄을 2.4px 차이로 p175에
            // 남기는 실제 HWP5 저장 반올림도 함께 보정한다. 이 값은 해당
            // footnote-queue 형상에만 적용되며 일반 표의 body 경계를 넓히지 않는다.
            let queue_footer_roundoff_slack =
                if native_hwp5_oversized_single_row_fragment_queues_footnotes {
                    4.0
                } else {
                    1.0
                };
            table_available = (available - st.layout.pagination_tolerance_px
                + queue_footer_roundoff_slack)
                .min(st.base_available_height() + queue_footer_roundoff_slack)
                .max(0.0);
            st.mark_fragment_footnotes_queued((para_idx, ctrl_idx));
        }

        // 첫 행이 남은 공간보다 크면 다음 페이지로 (인트라-로우 분할 가능성 확인).
        let host_frame = (
            st.pages.len(),
            st.current_column,
            st.current_zone_y_offset.to_bits(),
        );
        // The first fragment's border is paragraph-relative, whereas a whole
        // object's placement includes its outer-margin box. Convert before
        // exclusions, and share this result with both the row budget and paint.
        let fragment_host_placement = unconstrained_host_placement
            .filter(|_| placement_para_start_height + fmt.height_for_fit <= available)
            .map(|placement| {
                let applied_before = if placement_para_start_height > 0.0 {
                    fmt.spacing_before
                } else {
                    0.0
                };
                let host_line_height = fmt.computed_host_lines.as_ref().map_or_else(
                    || {
                        para.line_segs
                            .last()
                            .map_or(0.0, |line| hwpunit_to_px(line.line_height, self.dpi))
                    },
                    |lines| lines.last().map_or(0.0, |line| line.height),
                );
                constrain_host_placement.constrain(
                    placement.for_first_fragment(table, applied_before, host_line_height, self.dpi),
                    st,
                )
            });
        if fragment_host_placement.is_some() && !st.pre_emitted_host_paras.contains(&para_idx) {
            // 첫 조각과 이월 모두 같은 계산 줄을 소비한다. 저장 줄로 재측정하지 않는다.
            let already_emitted = st.current_items.iter().any(|item| {
                matches!(item,
                PageItem::FullParagraph { para_index }
                | PageItem::PartialParagraph { para_index, start_line: 0, .. }
                if *para_index == para_idx)
            });
            if !already_emitted {
                st.append_item(PageItem::PartialParagraph {
                    para_index: para_idx,
                    start_line: 0,
                    end_line: fmt.line_heights.len(),
                });
                let host_h = fmt.line_advances_sum(0..fmt.line_heights.len());
                st.align_flow_to(st.current_height.max(placement_para_start_height + host_h));
                st.record_pre_emitted_host_height(para_idx, host_h);
            }
            st.mark_pre_emitted_host(para_idx);
        }
        // Task #398: rowspan>1 셀이 행 0의 시작점이면 블록 전체 높이로 판정.
        // [Task #1046 Stage 2] 첫(비연속) fragment 의 렌더러 y_start 점프 — host_spacing.before
        // 와 문단 기준 양수 vertical_offset — 를 잔여공간에서 차감한다.
        // 종전엔 미차감해 잔여를 과대평가 → 첫 행이 실제 안 들어가는데도 가드를 통과시켜
        // 일반 행 강제 배치 경로가 통째로 밀어넣어 본문 초과(예: pi=242 vert_off 38px,
        // 잔여 65.4px 로 보였으나 실가용 23.4px < 행0 34.9px). 루프 내 page_avail
        // (host_before_overhead/vert_offset_overhead) 와 동일 overhead 를 가드에도 적용.
        let first_frag_overhead = {
            let (host_before, fragment_outer_bottom) = partial_rowbreak_fragment_spacing_px(
                table,
                ft.host_spacing.before,
                false,
                ft.strict_following_plain_text_fit,
                crate::renderer::float_placement::native_empty_host_cellbreak_fragment_repeats_outer_margin(
                    self.profile.get().hwp5_stored_pagination_layout(),
                    para,
                    table,
                ),
                self.dpi,
            );
            let vert_off = {
                use crate::model::shape::VertRelTo as VR;
                let is_para_relative_table =
                    !table.common.treat_as_char && matches!(table.common.vert_rel_to, VR::Para);
                let v = table.common.vertical_offset as i32;
                if is_para_relative_table && v > 0 {
                    hwpunit_to_px(v, self.dpi)
                } else {
                    0.0
                }
            };
            host_before + vert_off + fragment_outer_bottom
        };
        let remaining_on_page = fragment_host_placement.map_or_else(
            || (table_available - st.current_height - first_frag_overhead).max(0.0),
            |p| {
                (table_available
                    - p.table_top
                    - hwpunit_to_px(table.outer_margin_bottom as i32, self.dpi)
                    - table
                        .caption
                        .as_ref()
                        .filter(|cap| {
                            matches!(cap.direction, CaptionDirection::Top)
                                && ft.caption_height > 0.0
                        })
                        .map_or(0.0, |cap| {
                            ft.caption_height + hwpunit_to_px(cap.spacing as i32, self.dpi)
                        }))
                .max(0.0)
            },
        );
        let (first_block_start, first_block_end, first_block_h) = if row_count > 0 {
            mt.row_block_for(0)
        } else {
            (0, 0, 0.0)
        };
        let first_block_size = first_block_end.saturating_sub(first_block_start);
        let first_block_is_single_row = first_block_size == 1;
        let first_block_has_protectable_rowspan = first_block_size >= 2
            && first_block_size <= crate::renderer::height_measurer::BLOCK_UNIT_MAX_ROWS
            && (first_block_start..first_block_end)
                .any(|r| rowspan_touched.get(r).copied().unwrap_or(false));
        let first_rowbreak_block_has_hard_break =
            if mt.allows_row_break_split() && first_block_has_protectable_rowspan {
                layout_engine.row_block_has_internal_hard_break(
                    table,
                    first_block_start,
                    first_block_end,
                    styles,
                )
            } else {
                false
            };
        // [Task #1145] RowBreak 표도 작은 rowspan 제목/라벨 블록은 내부 hard-break가
        // 없으면 중간 행에서 자르지 않는다. 일반 RowBreak 행 경계 분할은 유지한다.
        let first_block_protected = first_block_has_protectable_rowspan
            && (!mt.allows_row_break_split() || !first_rowbreak_block_has_hard_break);
        // Task #398 v2: 보호 블록(2~3 rows)만 블록 전체 높이로 판정. 큰 rowspan(>3)은 행 단위 분할.
        // [#2097] 이월 게이트는 스캔과 같은 측정 공간을 본다: 스캔은 cut_row_h
        // (콘텐츠)로 배치를 판정하는데 게이트가 선언(mt.row_heights)만 보면,
        // 내용이 선언을 크게 웃도는 첫 행(82802 pi67: 선언 40.3px vs 컷
        // 465.3px, 셀 내 중첩 확장)에서 게이트가 침묵해 unsplittable 행이
        // 잔여 84.9px 에 강제 통째 배치 — 쪽 밖 428px 오버플로. max() 로
        // 콘텐츠 초과분을 게이트에 반영한다 (#874 선언>내용 형상은 불변).
        // [#7288] «쪽 경계에서» 가 원자 단위를 정한다. 값 0 «나누지 않음» 은 표
        // **전체**가 단위이므로, 첫 행만 보고 남은 공간에 붙잡으면 안 된다. 행 높이와
        // 같은 측정 공간(선언과 컷의 max)을 써서 이월 게이트가 스캔과 어긋나지 않게 한다.
        let all_rows_h = {
            let mut sum = 0.0;
            for r in 0..row_count {
                if r > 0 {
                    sum += cs;
                }
                sum += mt
                    .row_heights
                    .get(r)
                    .copied()
                    .unwrap_or(0.0)
                    .max(cut_row_h.get(r).copied().unwrap_or(0.0));
            }
            sum
        };
        // 저장본이 이 표 **안**에 쪽 프레임 리셋을 기록했다면, 그 문서는 실제로 이
        // 표를 나눠 저장한 것이다. 기록된 분할은 «쪽 경계에서» 의 재조판 규칙보다
        // 구체적인 증거이므로 원자 단위 규칙을 적용하지 않는다 (#6132 저장 vpos 계약).
        let stored_declares_table_split = rowbreak_table_has_internal_saved_vpos_reset(table);
        // [#7288] «나누지 않음» 의 원자 규칙은 **흐름이 표 밴드를 소비하는 배치**에만 건다.
        // 양수 세로 오프셋의 가시-host 자리차지 float 은 흐름을 전진시키지 않고 배제 영역에
        // 맡기는 별도 갈래이고(signed_vertical_offset > 0), 뒤 본문의 y 는 저장 사다리에서
        // 온다. 두 좌표계가 화해되지 않은 상태(#7198 축)에서 표만 옮기면 글 위에 표가
        // 그려진다(1490000 149쪽 실측: 겹침 134→156). 그 축을 먼저 풀기 전까지 미적용으로
        // 둔다 — 오프셋 0 float 과 treat_as_char 표는 흐름이 밴드를 소비하므로 해당 없다.
        let atomic_rule_applies = none_table_is_atomic_here(table) && !stored_declares_table_split;
        let split_unit_h = if atomic_rule_applies {
            all_rows_h
        } else if first_block_protected {
            first_block_h
        } else {
            mt.row_heights
                .first()
                .copied()
                .unwrap_or(0.0)
                .max(cut_row_h.first().copied().unwrap_or(0.0))
        };
        // native HWP5의 이 좁은 그림+caption 형상은 일반 RowBreak table의
        // conservative clean-defer budget보다, 실제 기존 FootnoteArea 경계가
        // 우선한다. `table_total`은 첫 fragment의 host/row/caption fit overhead를
        // 이미 포함하므로, 이 경계 안에 들어오는지 한 번 계산해 뒤의 두 defer
        // gate에서 같은 계약으로 사용한다.
        // [#3674 진단] 분할 경로 도달 브래킷 — 동작 불변.
        if std::env::var("RHWP_DIAG_SPLITSCAN").is_ok() {
            eprintln!(
                "DIAG_PRESPLIT pi={} sec={} remaining={:.1} unit_h={:.1} blk=({},{},{:.1}) items={}",
                para_idx, st.section_index,
                remaining_on_page,
                split_unit_h,
                first_block_start,
                first_block_end,
                first_block_h,
                st.current_items.len(),
            );
        }
        let native_picture_caption_fits_actual_footnote_boundary =
            st.profile.hwp5_stored_pagination_layout()
                && !table.common.treat_as_char
                && is_para_topbottom_float(&table.common)
                && matches!(
                    table.page_break,
                    crate::model::table::TablePageBreak::RowBreak
                )
                && ft.table_footnotes.is_empty()
                && st.current_footnote_height > 0.0
                && is_two_row_picture_caption_rowbreak_table(table)
                && st.current_height + table_total
                    <= st.base_available_height()
                        - st.current_footnote_height
                        - st.current_zone_y_offset
                        - st.current_bottom_fixed_exclusion
                        + 0.5;
        let stored_page_top_tac_table = st.profile.hwpx_stored_layout()
            && hwpx_stored_tac_table_starts_at_page_top(
                para,
                table,
                st.current_items.is_empty(),
                st.current_height,
                st.base_available_height(),
                self.dpi,
            );
        if stored_page_top_tac_table
            || (remaining_on_page < split_unit_h && !st.current_items.is_empty())
        {
            // [#7288] 원자 단위가 새 쪽에 통째로 들어가면 여기서 자르지 않고 이월한다.
            // 값 2 «나눔» 만 그 자리에서 행 내부를 자르고, 값 0 «나누지 않음»·값 1
            // «셀 단위로 나눔» 은 단위가 한 쪽에도 안 들어갈 때만 불가피하게 자른다.
            // 한/글 정본: 편람 PDF 158→159쪽은 큰 행을 통째로 넘기며 앞쪽 바닥에 빈
            // 공간을 남긴다(값 1), 315→316쪽은 문장 한가운데를 자른다(값 2).
            let unit_fits_fresh_page =
                split_unit_h <= (base_available - first_frag_overhead).max(0.0);
            let intra_row_cut_here = !atomic_rule_applies || !unit_fits_fresh_page;
            let first_row_splittable = (first_block_is_single_row || !first_block_protected)
                && can_intra_split
                && intra_row_cut_here
                && mt.is_row_splittable(0);
            // [Task #874 #6] 한컴 PDF (aift.hwp p19~20 표 pi=236 "기능 간 이벤트 연계
            // 구성도 이미지") 정합: 1×1 표 의 셀이 content 보다 훨씬 큰 cell.height
            // 를 가질 때 (line_count == 1 → is_row_splittable=false 라 의도 분할 불가)
            // 한컴은 page 경계에서 셀 빈 영역을 자르고 다음 페이지로 연속 렌더한다.
            // can_intra_split 이고 첫 행이 가용 공간보다 큰 force-split 케이스로 분기.
            // 빈 host의 단일 RowBreak 그림 표는 셀 안 그림의 실제 높이가 표 선언 높이보다
            // 커도, 다음 문단의 저장 vpos가 되감기면 새 물리 페이지에서 시작한다. 이를
            // 일반 1×1 force-split으로 처리하면 남은 공간에 표 상자만 두고 그림을 위로
            // 끌어올린다(정책연구용역 보고서 그림 23: PDF p24 → rhwp p23). 한 페이지에
            // 통째로 들어가는 native HWP 및 original HWPX 저장 그림 표에만 적용해,
            // 실제로 페이지보다 큰 1×1 표의 셀 내부 분할 계약은 보존한다.
            let rewound_empty_figure_float_should_defer =
                (st.profile.hwp5_stored_pagination_layout() || st.profile.hwpx_stored_layout())
                    && !table.common.treat_as_char
                    && is_para_topbottom_float(&table.common)
                    && matches!(
                        table.page_break,
                        crate::model::table::TablePageBreak::RowBreak
                    )
                    && !para_has_visible_text(para)
                    && table.row_count == 1
                    && table.col_count == 1
                    && table.cells.len() == 1
                    && table.cells.iter().any(|cell| {
                        cell.paragraphs.iter().any(|cell_para| {
                            cell_para
                                .controls
                                .iter()
                                .any(|control| matches!(control, Control::Picture(_)))
                        })
                    })
                    && para
                        .line_segs
                        .iter()
                        .find(|seg| !is_synthetic_line_seg(seg))
                        .zip(paragraphs_all.get(para_idx + 1).and_then(|next| {
                            next.line_segs
                                .iter()
                                .find(|seg| !is_synthetic_line_seg(seg))
                        }))
                        .is_some_and(|(current, next)| next.vertical_pos < current.vertical_pos)
                    && table_total <= (base_available - first_frag_overhead).max(0.0);
            let first_row_force_splittable = !first_block_protected
                && can_intra_split
                // [#7288] force-split 은 «나눔» 의 갈래다. 값 0·1 표를 한 줄 + padding
                // 예산으로 현재 쪽에 붙잡으면 정책이 정한 원자 단위가 깨진다.
                && intra_row_cut_here
                && remaining_on_page > 0.0
                && !rewound_empty_figure_float_should_defer;
            let min_content = if first_row_splittable {
                mt.min_first_line_height_for_row(0, 0.0) + mt.max_padding_for_row(0)
            } else if first_row_force_splittable {
                // force-split 케이스: 콘텐츠 한 줄 + padding 정도면 분할 가능
                let pad = mt.max_padding_for_row(0);
                let line_h = mt.row_heights.first().copied().unwrap_or(0.0).min(20.0);
                pad + line_h
            } else {
                f64::MAX
            };
            // [Task #1046 Stage 3] 다행(多行) 표의 비분할 첫 행/블록이 잔여공간엔 안
            // 들어가지만 fresh 페이지엔 통째 들어가면 다음 페이지로 이월한다. 첫 행은
            // 행 내부 분할이 안 되고(=is_row_splittable=false) 표에 후속 행 경계가 있어
            // 깨끗한 이월이 가능하므로(요구사항 표 계열, 한컴 PDF상 통째 배치) force-split
            // 추정으로 현재 페이지에 붙잡지 않는다(pi=290 8.7px). genuine page-larger 와
            // 1×1 단일 셀(row_count==1, 행 경계 없어 셀 내부 컷 필요, #874)은 제외 —
            // fits_fresh_page/row_count 조건으로 기존 force-split(렌더러 경계 컷) 유지.
            let fits_fresh_page = unit_fits_fresh_page;
            let multirow_clean_defer = !first_row_splittable
                && row_count > 1
                && first_block_end < row_count
                && fits_fresh_page;
            // A native HWP5 host whose stored anchor overlaps the current flow,
            // whose declared bottom crosses this body frame, and whose following
            // paragraph rewinds owns a physical first fragment here.  The generic
            // clean-defer rule must not move that source-owned prefix wholesale to
            // the next page merely because its first row is atomic.  The force-split
            // floor above proves that at least the visible first-row content fits;
            // the row scanner then admits only that first row and resumes at the
            // recorded next-page frame.
            let source_owned_atomic_first_fragment = source_anchor_splits_here
                && next_rewinds_after_table
                && first_row_force_splittable
                && remaining_on_page >= min_content;
            // native HWP5 2행 그림+caption 표는 첫 그림 행만으로 계산한 일반
            // clean-defer budget이 기존 각주의 40px safety buffer 때문에 소폭 부족해도,
            // 표 전체가 실제 FootnoteArea 앞에 끝날 수 있다. 이 경우까지 다음 page로
            // 미루면 그림 49처럼 빈 page와 이후 page drift가 생긴다. 표 안 각주·span·
            // 일반 다행 표에는 적용하지 않고, safety buffer를 빼기 전 physical boundary를
            // 실제 table total로 다시 확인한다.
            let native_picture_caption_fits_actual_footnote =
                multirow_clean_defer && native_picture_caption_fits_actual_footnote_boundary;
            let multirow_clean_defer = multirow_clean_defer
                && !native_picture_caption_fits_actual_footnote
                && !source_owned_atomic_first_fragment;
            // [#2097 진단] 첫 행 이월 결정 입력 — 동작 불변.
            if std::env::var("RHWP_DIAG_SCAN").is_ok() {
                eprintln!(
                    "DIAG_SCAN FIRSTROW_DEFER? pi={} remaining={:.1} unit_h={:.1} min_content={:.1} splittable={} force={} clean_defer={}",
                    para_idx,
                    remaining_on_page,
                    split_unit_h,
                    min_content,
                    first_row_splittable,
                    first_row_force_splittable,
                    multirow_clean_defer
                );
            }
            if stored_page_top_tac_table
                || (!first_row_splittable && !first_row_force_splittable)
                || remaining_on_page < min_content
                || multirow_clean_defer
            {
                // [Task #1753] visible-host 자리차지 RowBreak 표가 다음 쪽으로 이월되기
                // 직전, 후속 문단을 현재 쪽 잔여 공간에 선행 채움(한글 fill-before-
                // deferred-float 정합 — 2814765 pi52/53 은 9쪽 하단, 표는 10쪽부터).
                self.prefill_before_deferred_table(
                    st,
                    para_idx,
                    para,
                    table,
                    paragraphs_all,
                    composed_all,
                    styles,
                );
                st.advance_column_or_new_page();
            }
        }

        // 캡션 처리
        let caption_is_top = para
            .controls
            .get(ctrl_idx)
            .and_then(|c| {
                if let Control::Table(t) = c {
                    t.caption
                        .as_ref()
                        .map(|cap| matches!(cap.direction, CaptionDirection::Top))
                } else {
                    None
                }
            })
            .unwrap_or(false);

        let host_line_spacing_for_caption = para
            .line_segs
            .first()
            .map(|seg| hwpunit_to_px(seg.line_spacing, self.dpi))
            .unwrap_or(0.0);
        let caption_base_overhead = {
            let ch = ft.caption_height;
            if ch > 0.0 {
                let cs_val = para
                    .controls
                    .get(ctrl_idx)
                    .and_then(|c| {
                        if let Control::Table(t) = c {
                            t.caption
                                .as_ref()
                                .map(|cap| hwpunit_to_px(cap.spacing as i32, self.dpi))
                        } else {
                            None
                        }
                    })
                    .unwrap_or(0.0);
                ch + cs_val
            } else {
                0.0
            }
        };
        let caption_overhead = if caption_base_overhead > 0.0 && !caption_is_top {
            caption_base_overhead + host_line_spacing_for_caption
        } else {
            caption_base_overhead
        };

        // [Task #1831] 한글 float 정합 — **같은 문단에 앵커된 선행 float(표)
        // 아래로 밀려 내려온** 다행(多行) 자리차지 표가 현재 단 잔여 공간에
        // 통째로 들어가지 않으면 첫 조각을 단 중간에 만들지 않고 표 전체를
        // 다음 단/페이지 상단으로 민다. 실측(2448877 별표4 + 변형 스윕,
        // output/poc/task1831/):
        // - 표1(같은 문단) 뒤 잔여 197px: 한글은 표2 전체를 다음 쪽으로
        //   (repeat_header·제목셀 유무 무관, 표가 새 페이지보다 커도 동일).
        // - 표1을 지우고 텍스트 필러로 잔여 133~880px 스윕: 전 구간 분할 —
        //   텍스트만 선행하면 잔여와 무관하게 기존대로 분할한다.
        // - 다른 문단의 선행 float 는 밀기를 유발하지 않는다: 36387040 결재문서
        //   pi=46(8×3)은 위에 pi=41/43/44 표가 있어도 한글이 p4/p5 로 분할.
        // 즉 같은 앵커 문단의 float 스택 멤버만 통째-이월 그룹으로 다뤄진다.
        // 단 상단 시작 표(18151945 별표7)와 텍스트 선행 표는 기존 분할 유지.
        let total_rows_h: f64 =
            whole_row_fit_h.iter().sum::<f64>() + cs * row_count.saturating_sub(1) as f64;
        // [Task #1853] 이월 그룹은 같은 문단의 **진짜 flow 스택 float**(자리차지·문단
        // 앵커·글자처럼 아님)로 한정한다. 선행 항목의 소스 컨트롤을 조회해
        // is_para_topbottom(!tac && TopAndBottom && vert=Para) 인 표만 선행 float 로
        // 센다. `para_index` 만 비교하면 (a) 표 자신의 tac=true 캡션 상자(156767631
        // 캡션 ci=1, 78842 캡션 ci=0)나 (b) vert=용지 페이지-절대 앵커 상자(3143097
        // pi2 의 상자 22개)까지 "선행 형제 float" 로 오분류해, 분할 가능한 본체 표를
        // 통째로 다음 쪽으로 밀어 +1쪽 회귀가 났다(#1844 서베이). 별표4(2448877)의
        // 표1 은 tac=false·TopAndBottom·vert=Para 라 그대로 인정되어 정합을 유지한다.
        let preceded_by_same_para_float = st.current_items.iter().any(|it| {
            let ci = match it {
                PageItem::Table {
                    para_index,
                    control_index,
                }
                | PageItem::PartialTable {
                    para_index,
                    control_index,
                    ..
                } if *para_index == para_idx => *control_index,
                _ => return false,
            };
            matches!(
                para.controls.get(ci),
                Some(Control::Table(t)) if is_para_topbottom_float(&t.common)
            )
        });
        if row_count > 1 && preceded_by_same_para_float {
            let remaining_now =
                (table_available - st.current_height - first_frag_overhead).max(0.0);
            if total_rows_h + caption_base_overhead > remaining_now
                && !native_picture_caption_fits_actual_footnote_boundary
            {
                self.prefill_before_deferred_table(
                    st,
                    para_idx,
                    para,
                    table,
                    paragraphs_all,
                    composed_all,
                    styles,
                );
                st.advance_column_or_new_page();
            }
        }

        // [Task #1811] visible-host RowBreak 자리차지 표가 현재 쪽에서 곧바로 부분
        // 분할되면, layout 단계의 fragment host 후처리보다 먼저 문서 흐름의 host 텍스트
        // 줄을 소비한다. 기준은 샘플명이 아니라 같은 문단의 visible text 와 RowBreak
        // 자리차지 표 속성이다.
        let host_vpos_is_cumulative = para
            .line_segs
            .iter()
            .find(|ls| !is_synthetic_line_seg(ls))
            .is_some_and(|seg| {
                seg.vertical_pos
                    > crate::renderer::px_to_hwpunit(st.layout.body_area.height, self.dpi)
            });
        let native_hwp5_host_precedes_first_fragment = st.profile.hwp5_stored_pagination_layout()
            && native_hwp5_rowbreak_host_precedes_first_fragment(para, table);
        if (st.profile.hwpx_stored_layout()
            && host_vpos_is_cumulative
            && !table.common.treat_as_char
            && is_para_topbottom_float(&table.common)
            && matches!(
                table.page_break,
                crate::model::table::TablePageBreak::RowBreak
            ))
            || native_hwp5_host_precedes_first_fragment
        {
            self.pre_emit_visible_rowbreak_host_text(st, para_idx, para, composed_all, styles);
        }

        #[cfg(not(target_arch = "wasm32"))]
        let continuation_fragment_budget = std::env::var("RHWP_2424_FRAGMENT_BUDGET")
            .ok()
            .and_then(|value| value.parse::<usize>().ok())
            .filter(|budget| *budget > 0)
            .unwrap_or(usize::MAX);
        #[cfg(target_arch = "wasm32")]
        let continuation_fragment_budget = usize::MAX;
        let prepared = BlockTableContinuationPreparedState {
            host_placement: fragment_host_placement,
            host_frame,
            row_count,
            cell_spacing: cs,
            can_intra_split,
            base_available,
            table_available,
            layout_engine,
            rowspan_touched,
            cut_row_heights: cut_row_h,
            whole_row_fit_heights: whole_row_fit_h,
            first_fragment_painted_row_footer_guard,
            caption_is_top,
            caption_overhead,
            total_rows_height: total_rows_h,
            total_footnote_height: total_footnote,
            queue_table_footnotes,
            table_footnotes: ft.table_footnotes.clone(),
            footnote_margin: fn_margin,
            host_spacing_total,
            host_spacing_before: ft.host_spacing.before,
            host_spacing_after_only: ft.host_spacing.spacing_after_only,
            terminal_nested_child_host_line_spacing: native_terminal_child_host_line_spacing(
                self.profile.get().hwp5_stored_pagination_layout(),
                table,
                self.dpi,
            ),
            strict_following_plain_text_fit: ft.strict_following_plain_text_fit,
            budget_para_start_height,
            first_fragment_actual_footnote_boundary:
                (native_picture_caption_fits_actual_footnote_boundary
                    || native_ordinary_rowbreak_rewind_uses_actual_footnote_boundary
                    || native_hwp5_internal_reset_rewind_needs_anchor_resync)
                    .then(|| {
                        st.base_available_height()
                            - st.current_footnote_height
                            - st.current_zone_y_offset
                            - st.current_bottom_fixed_exclusion
                    }),
            source_next_positive_rewind: next_rewinds_after_table && !next_starts_new_page,
            first_fragment_saved_offset: {
                let column = st.inline_flow_column();
                crate::renderer::layout::native_hwp5_internal_reset_rowbreak_first_fragment_saved_top(
                    self.profile.get().hwp5_stored_pagination_layout(),
                    para,
                    para_idx.checked_sub(1).and_then(|i| paragraphs_all.get(i)),
                    paragraphs_all.get(para_idx + 1),
                    table,
                    &column,
                    self.dpi,
                ).map(|top| top - column.y)
            },
            next_para_stored_top: paragraphs_all.get(para_idx + 1).and_then(|next| {
                let seg = next
                    .line_segs
                    .first()
                    .filter(|seg| !is_synthetic_line_seg(seg))?;
                let spacing_before = styles
                    .para_styles
                    .get(next.para_shape_id as usize)
                    .map_or(0.0, |style| style.spacing_before);
                Some((seg.vertical_pos, spacing_before))
            }),
            source_cellbreak_row_end: (self.profile.get().hwp5_stored_pagination_layout()
                && !self.profile.get().session_edited())
            .then(|| paragraphs_all.get(para_idx + 1))
            .flatten()
            .and_then(|next| {
                crate::renderer::float_placement::stored_cellbreak_fragment_row_end(
                    table, para, next,
                )
            }),
            // 표 25처럼 저장 table 높이 안에는 들어가지만 셀 원문은 그보다 훨씬 긴
            // HWP5 RowBreak 표는 PDF가 마지막 continuation 표와 URL 각주 사이에
            // 일반 40px safety margin을 두지 않는다. 이 예외는 셀 각주가 많은
            // 고정-height 표로 한정한다. 실제 FootnoteArea 높이는 queue의 composed
            // line 측정으로 계속 예약하므로 본문/각주 충돌을 허용하지 않는다.
            relax_terminal_table_footnote_fit: queue_table_footnotes
                && ft.table_footnotes.len() >= 8
                && declared_table_height > 0.0
                && total_rows_h > declared_table_height * 2.0,
        };
        let source = BlockTableContinuationSource {
            para_index: para_idx,
            control_index: ctrl_idx,
            paragraph: para,
            table,
            row_geometry_table,
            measured_table: mt,
            styles,
        };
        let placeholder = st.transfer_placeholder();
        let flow_state = std::mem::replace(st, placeholder);
        (
            BlockTableContinuationContext::new(continuation_fragment_budget, prepared, flow_state),
            source,
        )
    }
}
