//! Protected / rowspan block scan step. Queries keep the page state read-only; this step owns scan-result updates.

use crate::renderer::typeset::{
    table, BlockRowScanVars, BlockTableRowScan, TypesetEngine, MIN_TOP_KEEP_PX,
};

use super::{ScanInput, ScanProgress, ScanStep};

impl TypesetEngine {
    pub(super) fn scan_rowspan_block_step(
        &self,
        input: ScanInput<'_>,
        progress: ScanProgress,
        block: table::scan::RowBlockCandidate,
        cs_before: f64,
    ) -> ScanStep {
        let ScanInput {
            st,
            layout_engine,
            mt,
            table,
            styles,
            cut_row_h,
            whole_row_fit_h,
            rowspan_touched,
            start_cut,
            straddle_page_bands,
            v,
            table_storage_declares_splits,
            ..
        } = input;
        let BlockRowScanVars {
            cursor_row,
            row_count,
            cs,
            can_intra_split,
            is_continuation,
            avail_for_rows,
            header_overhead,
            landscape_rowbreak_bleed,
            landscape_whole_row_tolerance,
            landscape_short_row_tolerance,
            landscape_short_row_max_height,
            strict_painted_bottom_fit,
            source_first_fragment_overflow_allowance,
            source_first_fragment_row_end,
            start_cut_is_block: _,
            start_row_height_override,
            ..
        } = v;
        let ScanProgress {
            mut r,
            scan,
            mut bleed_absorbed_row_height,
        } = progress;
        let BlockTableRowScan {
            source_frame_band,
            mut consumed,
            mut end_row,
            mut split_block_start,
            mut split_end_cut,
            mut split_end_limit,
            mut end_row_height_override,
        } = scan;
        let block_query = table::scan::RowBlockQuery {
            layout_engine,
            mt,
            table,
            styles,
            cut_row_h,
            rowspan_touched,
            cs,
            straddle_page_bands,
        };
        let table::scan::RowBlockCandidate {
            b_start,
            b_end,
            block_size,
            protected,
            rowbreak_rowspan_block,
            rowbreak_use_row_offsets,
            ..
        } = block;
        let keep_scanning = (|| {
            // [Task #1025] 연속분 커서가 블록 중간이면 블록 시작 컷을 적용.
            let blk_start_cut: &[usize] = if r == cursor_row { &start_cut } else { &[] };
            let block_row_offsets = block_query.row_offsets(&block, blk_start_cut);
            let block_h = block_query.required_height(&block, blk_start_cut);
            let required = consumed + cs_before + block_h;
            // The source remainder and a mixed row/block walk can sum the
            // same physical boxes in different orders. Account only for f64
            // roundoff, as the saved-span reservation does after this walk.
            let roundoff = f64::EPSILON
                * (required.abs() + avail_for_rows.abs()).max(1.0)
                * (r - cursor_row + block_size + 2) as f64;
            if required <= avail_for_rows
                || (required.is_finite()
                    && avail_for_rows.is_finite()
                    && required - avail_for_rows <= roundoff)
            {
                consumed += cs_before + block_h;
                r = b_end;
                end_row = r;
                return true;
            }
            // [Task #1025/#1086] 블록이 가용 초과 — 거대 row_span==1 셀을
            // 줄 단위로 분할 시도(블록 컷). 보호 블록은 기존처럼 fresh
            // page 에도 안 들어가는 경우만 페이지 중간에서 쪼갠다. RowBreak
            // rowspan 블록은 hard-break(vpos reset)를 만난 경우에만 중간
            // 분할을 허용해 일반 RowBreak 행 경계 정책의 blast radius 를 줄인다.
            let budget = (avail_for_rows - consumed - cs_before).max(0.0);
            let res = if rowbreak_use_row_offsets {
                layout_engine.advance_row_block_cut_with_row_offsets(
                    table,
                    b_start,
                    b_end,
                    blk_start_cut,
                    budget,
                    &block_row_offsets,
                    styles,
                )
            } else {
                layout_engine.advance_row_block_cut(
                    table,
                    b_start,
                    b_end,
                    blk_start_cut,
                    budget,
                    styles,
                )
            };
            // [Task #1025] 블록이 fresh 페이지에도 안 들어가야(진짜 page-larger)
            // 페이지 중간에서 분할한다. fresh 페이지엔 들어가면(잔여 공간만
            // 부족) 통째로 다음 페이지로 미뤄 잔여 overflow 를 피한다(기존 동작).
            // 페이지 시작 행(r==cursor_row)은 더 미룰 수 없으므로 무조건 분할.
            let genuinely_page_larger =
                block_h > (st.base_available_height() - header_overhead).max(0.0);
            // [#2097 진단] 블록 컷/이월 결정 입력 — 동작 불변.
            if std::env::var("RHWP_DIAG_SCAN").is_ok() {
                eprintln!(
                "DIAG_SCAN BLOCK_DECIDE r={} b={}..{} block_h={:.1} rest={:.1} budget={:.1} cut_h={:.1} fully={} hard={} rbrb={} pglarger={}",
                r,
                b_start,
                b_end,
                block_h,
                avail_for_rows - consumed,
                budget,
                res.consumed_height,
                res.fully_consumed,
                res.hit_hard_break,
                rowbreak_rowspan_block,
                genuinely_page_larger
            );
            }
            let cut_query = table::scan::block_fit::BlockCutQuery {
                rows: &block_query,
                block: &block,
                r,
                cursor_row,
                blk_start_cut,
                res: &res,
            };
            if let Some(last_row_band) = cut_query.source_complete_last_row_band(budget) {
                let remaining_band = budget;
                // 마지막 행만 남은 body band까지 줄인다. preceding rowspan
                // row와 spacing은 그대로 두어 renderer의 row geometry와
                // scanner의 physical fragment height가 같은 좌표계를 쓴다.
                consumed += cs_before + remaining_band;
                r = b_end;
                end_row = r;
                end_row_height_override = Some(last_row_band);
                return true;
            }
            let allow_block_split = cut_query.allows_split(genuinely_page_larger);
            // [#2097] RowBreak rowspan 블록 쪽 하단 밴드 필: plain 컷 walk 는
            // 셀-로컬 높이만 보고 행 시작 y 를 무시해, 블록 밴드가 잔여를
            // 초과해도 fully_consumed 로 오판해 분할이 기각된다 (3248363
            // b=6..8: block_h 661.8 > 잔여 540.7 인데 fully=true/cut_h 376).
            // 한글은 이 경계에서 행 오프셋 기준 밴드 컷으로 쪽을 채운다
            // (한글 PDF p2 만충 + p3 상단 셀 내용 중간 재개 실측). hard-break
            // 없는 쪽 하단 경계 한정으로 오프셋 컷을 재시도한다.
            // 내부 hard-break 없는 protected 블록(rbrb=false)도 plain 컷이
            // 기각되는 같은 경계에서 한글은 밴드를 채운다 (75544 rows 8..11:
            // block_h 420.0 > 잔여 79.2 통이월로 쪽 하단 방치 -> +1쪽, 한글
            // PDF p2 는 rows 8..9 수용 실측) — fully 오판 여부와 무관하게
            // 기각 경계 전체로 오프셋 재시도를 확장한다.
            let mut band_fill = None;
            let mut complete_row_frame = None;
            if cut_query.retries_band(allow_block_split, can_intra_split, block_h, budget) {
                let offsets = block_query.row_offsets_for_cut(&block, blk_start_cut);
                let res2 = layout_engine.advance_row_block_cut_with_row_offsets(
                    table,
                    b_start,
                    b_end,
                    blk_start_cut,
                    budget,
                    &offsets,
                    styles,
                );
                if std::env::var("RHWP_DIAG_SCAN").is_ok() {
                    eprintln!(
                    "DIAG_SCAN BLOCK_BAND? r={} b={}..{} budget={:.1} cut_h={:.1} fully={} end_cut={:?}",
                    r,
                    b_start,
                    b_end,
                    budget,
                    res2.consumed_height,
                    res2.fully_consumed,
                    res2.end_cut
                );
                }
                if !res2.fully_consumed {
                    let candidate = Some((res2, offsets));
                    let selected = table::scan::block_fragment::SelectedBlockCut::select(
                        rowbreak_use_row_offsets,
                        &res,
                        &block_row_offsets,
                        &candidate,
                    );
                    if selected.cut_res.consumed_height < MIN_TOP_KEEP_PX {
                        complete_row_frame = selected.complete_row_boundary_frame(
                            &block_query,
                            &block,
                            blk_start_cut,
                            budget,
                        );
                    }
                    if selected.cut_res.consumed_height >= MIN_TOP_KEEP_PX
                        || complete_row_frame.is_some()
                    {
                        band_fill = candidate;
                    }
                }
            }
            if can_intra_split
                && ((!res.fully_consumed && allow_block_split) || band_fill.is_some())
            {
                let selected = table::scan::block_fragment::SelectedBlockCut::select(
                    rowbreak_use_row_offsets,
                    &res,
                    &block_row_offsets,
                    &band_fill,
                );
                end_row = selected.end_row(&block);
                if let Some((height, last)) = complete_row_frame {
                    split_block_start = Some(b_start);
                    split_end_cut = selected.cut_res.end_cut.clone();
                    split_end_limit = selected.cut_res.consumed_height;
                    end_row_height_override = Some(last);
                    consumed += cs_before + height;
                    return false;
                }
                if let Some(band) = selected.complete_row_boundary_band(
                    &block_query,
                    &block,
                    blk_start_cut,
                    end_row,
                    (
                        budget,
                        st.base_available_height() - header_overhead,
                        !is_continuation && cursor_row == 0,
                        self.dpi,
                    ),
                ) {
                    // 빈 컷은 완전 행 경계다. 블록 시작 표시는 빈 공간을 다음
                    // 완전 행 상자에 합칠 소유권이며 내용 컷으로 해석하지 않는다.
                    split_block_start = Some(b_start);
                    split_end_cut.clear();
                    split_end_limit = 0.0;
                    end_row_height_override = Some(band);
                    let before_last = block_query.fragment_height(
                        &block,
                        end_row - 1,
                        blk_start_cut,
                        &selected.cut_res.end_cut,
                    ) + if end_row > b_start + 1 { cs } else { 0.0 };
                    consumed += cs_before + before_last + band;
                    return false;
                }
                split_end_cut = selected.cut_res.end_cut.clone();
                split_end_limit = selected.cut_res.consumed_height;
                split_block_start = Some(b_start);
                let split_total =
                    selected.occupied_height(&block_query, &block, blk_start_cut, end_row);
                consumed += cs_before + split_total;
                return false;
            }
            if r == cursor_row {
                // 분할 불가 — 강제 통째 배치(기존 overflow 동작 유지).
                consumed += cs_before + block_h;
                r = b_end;
                end_row = r;
                return true;
            }
            end_row = r;
            false
        })();
        let scan = BlockTableRowScan {
            source_frame_band,
            consumed,
            end_row,
            split_block_start,
            split_end_cut,
            split_end_limit,
            end_row_height_override,
        };
        ScanStep {
            progress: ScanProgress {
                r,
                scan,
                bleed_absorbed_row_height,
            },
            keep_scanning,
        }
    }
}
