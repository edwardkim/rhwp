//! Prepare the next fragment budget at its current host origin.

use crate::renderer::typeset::{
    controls, hwpunit_to_px, is_synthetic_line_seg, nearest_saved_rowbreak_frame_row_end,
    para_has_non_whitespace_text, paragraph, partial_rowbreak_fragment_spacing_px,
    row_geometry_table, saved_rowbreak_first_fragment_flow_overflow_allowance, table,
    table_declared_object_covers_cell_row_frames, Control, TypesetEngine, TypesetState,
    SINGLE_ROW_DECLARED_TRUST_MAX_RATIO,
};

use super::super::TableContinuationCursor;
use super::{FragmentBudget, FragmentInput, FragmentProfile};

impl TypesetEngine {
    pub(super) fn prepare_table_fragment_budget(
        &self,
        st: &mut TypesetState,
        input: FragmentInput<'_>,
    ) -> FragmentBudget {
        let para_idx = input.source.para_index;
        let ctrl_idx = input.source.control_index;
        let para = input.source.paragraph;
        let table = input.source.table;
        let row_geometry_table = input.source.row_geometry_table;
        let mt = input.source.measured_table;
        let prepared = input.prepared;
        let row_count = input.prepared.row_count;
        let cs = input.prepared.cell_spacing;
        let base_available = input.prepared.base_available;
        let table_available = input.prepared.table_available;
        let cut_row_h = &input.prepared.cut_row_heights;
        let first_fragment_painted_row_footer_guard =
            input.prepared.first_fragment_painted_row_footer_guard;
        let caption_is_top = input.prepared.caption_is_top;
        let caption_overhead = input.prepared.caption_overhead;
        let total_rows_h = input.prepared.total_rows_height;
        let total_footnote = input.prepared.total_footnote_height;
        let table_footnotes = &input.prepared.table_footnotes;
        let host_spacing_before = input.prepared.host_spacing_before;
        let strict_following_plain_text_fit = input.prepared.strict_following_plain_text_fit;
        let budget_para_start_height = input.prepared.budget_para_start_height;
        let first_fragment_actual_footnote_boundary =
            input.prepared.first_fragment_actual_footnote_boundary;
        let source_next_positive_rewind = input.prepared.source_next_positive_rewind;
        let cursor_row = input.start.cursor_row;
        let is_continuation = input.start.is_continuation;
        let start_cut = &input.start.start_cut;
        let caption_extra =
            if !is_continuation && cursor_row == 0 && start_cut.is_empty() && caption_is_top {
                caption_overhead
            } else {
                0.0
            };
        // [Task #874 #9] 첫 fragment 의 page_avail 은 host_spacing.before 와
        // 문단 기준 양수 vertical_offset 를 제외해야 한다.
        // layout 은 표를 cur_h + host_spacing.before + v_offset 위치에 배치하지만,
        // typeset 의 page_avail = (table_available - cur_h) 은 두 overhead 를
        // 포함하지 않아 split 결정 시 actual 가용보다 과대 평가됨 → partial 오버플로우.
        // aift.hwp p44 pi=584: 41.6 px split_end → 실제 가용 36 px → overflow 37.6 px.
        // [#7095] 본문을 통째로 담은 1×1 RowBreak 쪽 조각은 쪽마다 바깥 여백(위·아래)을
        // 다시 열고, 비끝 조각 상자는 본문 아래 − 바깥 아래 여백 − 100HU 에서 끝난다
        // (한/글 2020 정본, PDF 쪽 척도 제거 후 두 문서 101~104HU · 돌연변이 7종에서 상수).
        // 렌더러(`table_partial.rs`)가 같은 술어로 상자를 고정하므로 예산도 같이 뺀다.
        //
        // 쪽 **상단**에서 시작하는 조각에만 쓴다. 쪽 중간에서 시작하는 첫 조각에 여백과
        // 100HU 를 빼면 컷이 한 유닛 일러져 한/글보다 쪽이 는다(80168 29쪽 pi226 · 157→158,
        // rowbreak-problem-pages 14쪽 pi16 · 18→19). 렌더러의 상자 고정 조건과 같은 축이다.
        let single_cell_page_fragment =
            crate::renderer::float_placement::native_single_cell_rowbreak_page_fragment(
                self.profile.get().hwp5_stored_pagination_layout(),
                table,
            ) && (is_continuation || st.current_height < 0.5);
        // 저장된 첫 조각의 원점은 paint와 같은 값으로 예약한다.
        // trailing trim으로 얻은 컷만 공유하고 이전 flow 원점을 유지하면
        // 실제 표 상자와 페이지 예산이 서로 다른 높이를 소비한다.
        if !is_continuation && cursor_row == 0 && start_cut.is_empty() {
            if let Some(offset) = prepared.first_fragment_saved_offset {
                st.align_flow_to(offset);
            }
        }
        let (host_before_overhead, fragment_outer_bottom_overhead) =
            partial_rowbreak_fragment_spacing_px(
                table,
                host_spacing_before,
                is_continuation,
                strict_following_plain_text_fit || single_cell_page_fragment,
                crate::renderer::float_placement::native_empty_host_cellbreak_fragment_repeats_outer_margin(
                    self.profile.get().hwp5_stored_pagination_layout(),
                    para,
                    table,
                ),
                self.dpi,
            );
        // 끝 조각의 흐름 전진에는 이 형상이 새로 연 아래 여백과 100HU 를 넣지 않는다.
        // 둘 다 비끝 조각 상자의 계약이고, 끝 조각은 내용에 맞춰 끝나 렌더러도 그 뒤에
        // 여백을 두지 않는다. 넣어 두면 쓰지 않는 자리를 예산에서 먹어 다음 내용이 밀린다.
        // - rowbreak-problem-pages 14쪽: pi13 끝 조각 뒤 pi16 이 0.2px 차로 안 들어가 18→19쪽
        // - hwpctl_API_v2.4 73쪽: pi1750 끝 조각 뒤 pi1760 13행이 74쪽으로 밀려 본문 넘침
        //   (정본은 13행을 73쪽 992.7 에 두고, 조각 아래 괘선 393.11 뒤에 여백을 두지 않는다)
        let terminal_outer_bottom_overhead = if single_cell_page_fragment {
            partial_rowbreak_fragment_spacing_px(
                table,
                host_spacing_before,
                is_continuation,
                strict_following_plain_text_fit,
                crate::renderer::float_placement::native_empty_host_cellbreak_fragment_repeats_outer_margin(
                    self.profile.get().hwp5_stored_pagination_layout(),
                    para,
                    table,
                ),
                self.dpi,
            )
            .1
        } else {
            fragment_outer_bottom_overhead
        };
        let single_cell_page_fragment_inset_px = if single_cell_page_fragment {
            hwpunit_to_px(
                crate::renderer::float_placement::SINGLE_CELL_PAGE_FRAGMENT_BOTTOM_INSET_HU,
                self.dpi,
            )
        } else {
            0.0
        };
        let fragment_outer_bottom_overhead =
            fragment_outer_bottom_overhead + single_cell_page_fragment_inset_px;
        // [#6143] 오프셋이 쪽 경계에서 이미 소진된 첫 조각은 예산에서도 빼지
        // 않는다. 앵커 문단이 이 쪽에 아무것도 내지 않았고(항목 0 · host 선방출 0)
        // 표가 쪽 최상단에서 시작하면 오프셋의 기준점(문단 자리)이 이 쪽에 없다 —
        // 앵커는 앞 쪽에 있고 오프셋은 거기서 쓰였다. 그래도 예산에서 빼면 조각이
        // 오프셋만큼 짧아져 표가 한 쪽 더 갈라진다(156555538 9쪽: page_avail
        // 990.3−1.9−554.6=433.8 → 행 1 을 20줄에서 자르고 나머지를 10쪽으로,
        // 총 18쪽. 한글은 17쪽). layout(table_partial.rs) 의 같은 게이트와
        // 대칭이어야 컷과 배치가 어긋나지 않는다(#2015 감액과 같은 이유).
        let para_offset_consumed_by_page_break = !is_continuation
            && st.current_items.is_empty()
            && st.current_height < 1.0
            && st
                .pre_emitted_host_heights
                .get(&para_idx)
                .copied()
                .unwrap_or(0.0)
                <= 0.0
            && crate::renderer::float_placement::para_offset_consumed_by_page_break(
                para,
                &table.common,
                base_available,
                self.dpi,
            );
        // [#5585 국소형 ②] `vert_rel_to=Para` 양수 오프셋은 **문단 앵커로부터의**
        // 거리다. 한 문단이 표 여러 개를 품고 그것들이 쪽을 하나씩 차지하면, 두 번째
        // 이후 표에게 그 오프셋은 이미 앞 형제가 소진한 값이다. 그런데 조판기는
        // 쪽마다 다시 물어 예산을 그만큼 깎는다.
        //
        // `02. 지표정의서- 주요정책부문` 실측 — 문단 `pi=72` 가 `11x21` 표 16개를 품는다.
        //
        // ```text
        //   vert_off  avail_for_rows   표 높이   결과
        //     0.0        742.7          734.8    통째 (ci=8)
        //    11.3        731.4          729.7    통째 (ci=1)
        //    11.3        731.4         ~733.7    분할! (ci=6·7·9·10·13)
        // ```
        //
        // 한/글 2024 실측(2-up 오라클을 논리 쪽으로 환산): 이 표들의 상단이 **15.60 /
        // 15.31pt** — 본문 상단 그대로다. rhwp 는 **22.64pt**(=+11.3px)에서 시작한다.
        // 한/글은 이 오프셋을 안 문다.
        //
        // 좁힘: **앞선 형제 표가 있는 경우만**. 표 하나짜리 문단(`issue_2287` 핀)은
        // 종전 계약을 그대로 둔다 — 넓히면 그 문서에 조각 공백화(sliver)가 생긴다.
        let has_earlier_sibling_table = para
            .controls
            .iter()
            .take(ctrl_idx)
            .any(|c| matches!(c, Control::Table(_)));
        let anchor_offset_already_spent = !is_continuation
            && st.current_items.is_empty()
            && st.current_height < 1.0
            && has_earlier_sibling_table;
        let vert_offset_overhead =
            if is_continuation || para_offset_consumed_by_page_break || anchor_offset_already_spent
            {
                0.0
            } else {
                use crate::model::shape::VertRelTo as VR3;
                let is_para_relative_table =
                    !table.common.treat_as_char && matches!(table.common.vert_rel_to, VR3::Para);
                // HwpUnit=u32 이므로 음수 (u32 wrap) 는 i32 로 캐스트 후 확인.
                let v_off_i32 = table.common.vertical_offset as i32;
                if is_para_relative_table && v_off_i32 > 0 {
                    // [#6860] `v_off` 의 기준점은 문단 상단이 아니라 **앵커 줄**(표 제어
                    // 문자가 실린 저장 줄)이다. layout 이 개체 원점을 그만큼 내리므로
                    // (`stored_float_anchor_offset_px`) 예산도 같이 내려야 컷과 배치가
                    // 어긋나지 않는다 — 안 빼면 3067979 87쪽 첫 조각이 본문을 10.3px 넘는다.
                    // 호스트가 한 줄이거나 제어 문자가 첫 줄이면 0 이라 종전과 같다.
                    let raw = hwpunit_to_px(v_off_i32, self.dpi)
                        + crate::renderer::layout::stored_float_anchor_offset_px(
                            para, table, ctrl_idx, self.dpi,
                        );
                    // [#2015] host 텍스트가 pre-emit(pre_emit_visible_rowbreak_host_text)
                    // 되어 current_height 를 para_start → para_start+host_h 로 전진시킨 경우,
                    // vert_off(para_start 기준 표 오프셋)를 그대로 빼면 host_h 만큼 이중계상되어
                    // 앵커가 body 바닥 아래로 밀린다(91.2px 오버플로우). 표의 참 오프셋은
                    // current_height 기준 (vert_off − host_h) 이므로 pre-emit host_h 만큼 감액.
                    // layout(table_partial.rs) 도 동일 감액을 적용해 typeset 컷과 정합한다.
                    let host_h = st
                        .pre_emitted_host_heights
                        .get(&para_idx)
                        .copied()
                        .unwrap_or(0.0);
                    (raw - host_h).max(0.0)
                } else {
                    0.0
                }
            };
        // [Task #1860] 빈 host out-of-flow para float(비-TAC, TopAndBottom,
        // vert=Para, v_off>0, host 텍스트 없음)는 para_start + v_off 에 배치되는
        // 개체다(#986/#1088/#157). 같은 문단의 선행 in-flow inline(예: tac 캡션 표,
        // #1855 유사입법례 표의 "참고|유사입법례" 캡션)이 current_height 를 전진시켜도,
        // float 은 para_start 기준에 놓여 세로로 겹치므로 아래로 쓰는 실가용은
        // para_start 기준이다. page_avail 이 current_height 를 빼면 선행 inline 만큼
        // 이중차감 → 분할 예산이 짧아져 RowBreak 컷이 소스 hard_break 보다 조기 발동
        // (공공데이터법 라벨 −23pt / p45 +40.8pt). 이 클래스만 current_height 대신
        // para_start_height 기준으로 예산을 잡는다.
        let is_empty_host_column_float = !is_continuation
            && !table.common.treat_as_char
            && vert_offset_overhead > 0.0
            && !para_has_non_whitespace_text(para);
        let page_avail = if is_continuation {
            // [Task #1937] 연속 페이지는 신선 full-page 를 기준으로 한다(레퍼런스
            // Paginator engine.rs:2502-2503 과 정합). table_available 은 표 *시작*
            // 페이지에서 표 전체 각주(total_footnote)를 available 에서 차감한 값이라,
            // 각주가 많은 큰 RowBreak 표(소상공인 중간보고서 pi=306: 22개 각주 820px →
            // 시작 페이지 잔여 75.8px)가 연속 페이지마다 그 좁은 잔여를 그대로 물려받아
            // 페이지당 ~1행으로 과분할된다(122행 → 188쪽). 표 각주는 첫 fragment fit
            // 판정에서만 보수적으로 예약하고, 연속 페이지는 신선 본문 가용을 쓴다.
            // zone offset·border tolerance 는 유지.
            (base_available
                - st.current_zone_y_offset
                - st.layout.pagination_tolerance_px
                - host_before_overhead
                - fragment_outer_bottom_overhead)
                .max(0.0)
        } else if let Some(footnote_boundary) = first_fragment_actual_footnote_boundary {
            // 일반 RowBreak 표의 safety budget은 첫 fragment에서 보수적으로
            // 40px를 남긴다. 다만 조판 전에 검증한 native HWP5 그림+caption
            // 2행 표는 표 전체가 기존 각주 경계 안에 들어가므로, 같은 정확한
            // 경계로 row scan을 수행해 그림 행과 caption 행을 분리하지 않는다.
            (footnote_boundary
                - st.current_height
                - caption_extra
                - host_before_overhead
                - vert_offset_overhead
                - fragment_outer_bottom_overhead)
                .max(0.0)
        } else if is_empty_host_column_float {
            // out-of-flow float 은 para_start + v_off 에 배치된다(#986/#1088/#157).
            // 같은 문단의 선행 in-flow inline(예: tac 캡션 표)이 current_height 를
            // para_start 위로 밀어도, float 은 para_start+v_off 부터 시작해 그 아래를
            // 전부 쓴다 → 예산 기준은 current_height 가 아니라 참 para_start 다. 선행
            // inline 이 없으면 budget_para_start==current_height 라 종전과 동일(#874 불변).
            //
            // 클램프: 지연(deferred) 배치가 페이지/컬럼 경계를 넘은 뒤 실행되면
            // 저장된 para_start(원 페이지 흐름 좌표)가 새 페이지의 current_height 보다
            // 커져 무효가 된다(pr-1674 pi=27 ci=1: 새 페이지 cur_h=0 에서 stale
            // para_start 583.8 차감 → 예산 −584px → 조기 분할 +1쪽). 참 para_start 는
            // 현재 흐름 높이를 초과할 수 없으므로 current_height 로 상한한다.
            (table_available
                - budget_para_start_height.min(st.current_height)
                - caption_extra
                - host_before_overhead
                - vert_offset_overhead
                - fragment_outer_bottom_overhead)
                .max(0.0)
        } else {
            (table_available
                - st.current_height
                - caption_extra
                - host_before_overhead
                - vert_offset_overhead
                - fragment_outer_bottom_overhead)
                .max(0.0)
        };
        let page_avail = if !is_continuation && start_cut.is_empty() {
            (page_avail - first_fragment_painted_row_footer_guard).max(0.0)
        } else {
            page_avail
        };
        let fragment_placement = prepared.host_placement.map(|original| {
            if !is_continuation
                && prepared.host_frame
                    == (
                        st.pages.len(),
                        st.current_column,
                        st.current_zone_y_offset.to_bits(),
                    )
            {
                original
            } else {
                // 첫 조각 전체가 이월된 경우에도 이전 frame의 거리를 재가산하지 않는다.
                crate::renderer::float_placement::ParagraphFloatPlacement {
                    flow: original.flow,
                    anchor_y: st.current_height,
                    stored_host_origin: None,
                    table_top: st.current_height + host_before_overhead,
                    occupied_bottom: st.current_height + host_before_overhead,
                }
            }
        });
        // A resolved host origin is shared with paint. Single-cell fragments
        // open their top margin here once, so the replacement budget cannot
        // silently lose the margin that table_partial would add afterwards.
        let single_cell_fragment_shape =
            crate::renderer::float_placement::native_single_cell_rowbreak_page_fragment(
                self.profile.get().hwp5_stored_pagination_layout(),
                table,
            ) && std::ptr::eq(row_geometry_table, table);
        let fragment_placement = fragment_placement.map(|mut p| {
            if single_cell_fragment_shape
                && !is_continuation
                && prepared.host_frame
                    == (
                        st.pages.len(),
                        st.current_column,
                        st.current_zone_y_offset.to_bits(),
                    )
            {
                p.table_top += hwpunit_to_px(table.outer_margin_top as i32, self.dpi);
            }
            p
        });
        let page_avail = fragment_placement.map_or(page_avail, |p| {
            let boundary = if is_continuation
                || prepared.host_frame
                    != (
                        st.pages.len(),
                        st.current_column,
                        st.current_zone_y_offset.to_bits(),
                    ) {
                st.available_height() - st.layout.pagination_tolerance_px
            } else {
                first_fragment_actual_footnote_boundary.unwrap_or(table_available)
            };
            (boundary
                - p.table_top
                - caption_extra
                - hwpunit_to_px(table.outer_margin_bottom as i32, self.dpi)
                - if !is_continuation && start_cut.is_empty() {
                    first_fragment_painted_row_footer_guard
                } else {
                    0.0
                })
            .max(0.0)
        });

        // A resolved visible-host origin replaces the default budget above;
        // retain the physical bottom inset in that replacement as well.
        // Empty-host stored cuts keep their existing advance-height budget:
        // its final line spacing is not painted content. Subtracting the box
        // inset from that mid-page advance rejects valid source units
        // (80168 157->158 pages, rowbreak-problem-pages 18->19).
        let page_avail = if let Some(p) = fragment_placement.filter(|_| single_cell_fragment_shape)
        {
            let box_bottom = crate::renderer::float_placement::single_cell_page_fragment_bottom(
                table,
                st.available_height(),
                self.dpi,
            );
            page_avail.min((box_bottom - p.table_top - caption_extra).max(0.0))
        } else {
            page_avail
        };

        // RowBreak 표의 common.height가 전체 표가 아니라 첫 physical fragment를
        // 저장할 수 있다. 저장 anchor가 현재 flow와 같고 declared bottom이 이
        // fragment bound 안에 있을 때만 source frame을 행 경계 후보로 쓴다.
        // host spacing과 paint inset은 source object 좌표가 아니므로 섞지 않는다.
        let source_first_fragment_flow_bottom = table_available;
        let scan_row_count = if !is_continuation && cursor_row == 0 && start_cut.is_empty() {
            prepared.source_cellbreak_row_end.unwrap_or(row_count)
        } else {
            row_count
        };
        let saved_first_fragment_source_frame = if !is_continuation
            && cursor_row == 0
            && start_cut.is_empty()
            && first_fragment_painted_row_footer_guard <= 0.0
            && !table.common.treat_as_char
            && matches!(
                table.page_break,
                crate::model::table::TablePageBreak::RowBreak
            )
            && row_count > 1
            && table_footnotes.is_empty()
            && table.common.height > 0
            && table.common.height <= i32::MAX as u32
            && std::ptr::eq(row_geometry_table, table)
        {
            para.line_segs
                .iter()
                .find(|seg| !is_synthetic_line_seg(seg))
                .and_then(|seg| {
                    let base = st.vpos_page_base.unwrap_or(0);
                    let anchor_hu = seg.vertical_pos.saturating_sub(base);
                    let anchor_px = hwpunit_to_px(anchor_hu, self.dpi);
                    let flow_bottom_hu =
                        anchor_hu.saturating_add(table.common.height.min(i32::MAX as u32) as i32);
                    let flow_bottom_px = hwpunit_to_px(flow_bottom_hu, self.dpi);
                    ((anchor_px - st.current_height).abs() <= 0.5
                        && flow_bottom_px <= source_first_fragment_flow_bottom + 0.5)
                        .then_some((
                            hwpunit_to_px(
                                table.common.height.min(i32::MAX as u32) as i32,
                                self.dpi,
                            ),
                            flow_bottom_px,
                        ))
                })
        } else {
            None
        };
        // [#6123] 저장 행 높이(행 안 row_span==1 셀의 최대 저장 높이). 프레임
        // 바닥이 진짜 행 경계인지 저장 좌표계에서 검산하는 데 쓴다.
        let stored_row_heights: Vec<f64> = (0..row_count)
            .map(|row| {
                table
                    .cells
                    .iter()
                    .filter(|cell| {
                        cell.row as usize == row && cell.row_span == 1 && cell.height < 0x8000_0000
                    })
                    .map(|cell| hwpunit_to_px(cell.height as i32, self.dpi))
                    .fold(0.0f64, f64::max)
            })
            .collect();
        let source_first_fragment_row_end =
            saved_first_fragment_source_frame.and_then(|(frame_height, _)| {
                nearest_saved_rowbreak_frame_row_end(
                    frame_height,
                    cut_row_h,
                    &stored_row_heights,
                    cs,
                )
            });
        // 기존 각주가 이미 이 page의 body tail을 예약했으면 object frame만으로
        // whole row를 수용할 수 없다. 그 마지막 행의 cell-unit partial cut은
        // footnote-aware row scanner가 소유한다.
        // [#5057] 이 허용치는 **저장된 첫 조각 source frame** 이 주는 것이고, 그
        // 기록은 컨테이너(HWP5 / 직접 HWPX)와 무관하게 파일에 그대로 있다. 종전에는
        // 네이티브 HWP5 프로파일에만 열려 있어, **같은 바이트**를 direct-HWPX 로 읽으면
        // 마지막 행을 못 받아 표가 쪼개졌다.
        //
        // 21484591 실측 — `META-INF/rhwp-hwp5-origin` 만 뺀 사본과의 A/B:
        //
        // ```text
        //   두 프로파일 모두  avail_for_rows = 523.4  (host_before·vert_off 동일)
        //   hwp5    r=7 에서 sfwr=true  → consumed 528.8 (5.4px 초과 수용) → 8행 전부
        //   direct  r=7 에서 sfwr=false → 7행에서 끊고 8행은 다음 단 → +1쪽
        //   한/글 2024 = 13쪽 = hwp5    (direct 는 14쪽)
        // ```
        // [#7288] 저장 프레임은 **이 내용에 대한 증거**일 때만 예산 초과를 허용한다.
        //
        // 이 허용치는 선언 개체 높이(`table.common.height`)가 만든 프레임에서 온다.
        // 선언이 실제 측정보다 크게 작으면 그 프레임은 이 내용의 기록이 아니다 —
        // 편집으로 낡았거나 애초에 다른 내용의 값이다. 그대로 믿으면 잔여 밴드에 안
        // 들어가는 행을 통째로 얹어 지면 밖으로 넘긴다.
        //
        // ```text
        //   선언 개체 높이   53.8px
        //   측정 첫 행      515.0px   ← 9.6배
        //   잔여 밴드       335.5px
        //   종전           행을 y=670..1185 에 통째로 — 본문(1009px) 밖 176px
        // ```
        //
        // 상한은 새로 만들지 않는다. 같은 «선언을 어디까지 믿나» 질문에 [#3236] 이
        // 쓰는 `SINGLE_ROW_DECLARED_TRUST_MAX_RATIO` 를 그대로 쓴다 — 측정이 선언의
        // 1.5배를 넘으면 글꼴 대체 팽창이 아니라 내용이 진짜로 큰 것이므로 선언 특례를
        // 적용하지 않는다. 프레임이 덮는다고 본 행들(`source_first_fragment_row_end`)의
        // 측정 합으로 견준다. [#5057] 의 21484591 은 선언과 측정이 맞아 넘지 않는다.
        let source_frame_declared_height_is_credible = source_first_fragment_row_end
            .zip(saved_first_fragment_source_frame)
            .is_some_and(|(row_end, (frame_height, _))| {
                let measured: f64 = cut_row_h.iter().take(row_end).sum::<f64>()
                    + cs * row_end.saturating_sub(1) as f64;
                measured <= frame_height * SINGLE_ROW_DECLARED_TRUST_MAX_RATIO
            });
        let mut source_first_fragment_overflow_allowance = saved_first_fragment_source_frame
            .filter(|_| {
                st.profile.hwp5_stored_pagination_layout() || st.profile.hwpx_stored_layout()
            })
            .filter(|_| source_frame_declared_height_is_credible)
            .filter(|_| st.current_footnote_height <= 0.0)
            .map(|(_, flow_bottom_px)| {
                saved_rowbreak_first_fragment_flow_overflow_allowance(
                    table.common.height,
                    std::ptr::eq(row_geometry_table, table),
                    flow_bottom_px,
                    source_first_fragment_flow_bottom,
                )
            })
            .unwrap_or(0.0);

        // [Task #1022] 머리행 반복 overhead — 렌더러(layout_partial_table)는
        // start_row 이전의 반복 제목행을 다시 그리므로(다중 머리행: rs>=2 헤더 셀 등),
        // 페이지네이터도 동일 제목행 전체 높이 + 각 행 뒤 cs 를 계산한다.
        // [Task #1716] 반복 대상은 **표 상단의 연속 제목행 블록**(leading_header_rows)뿐.
        // 종전엔 cursor 아래의 모든 is_header 행을 합산해, 본문 행에도 header="1" 이
        // 흩어진 표(건설공사 품질시험기준 pi=12)에서 cursor 전진 시 overhead 가 누적되어
        // 가용 높이가 0이 되고 페이지당 1행 폭주(+100쪽)가 발생했다. 렌더러(table_partial)도
        // 동일 leading_header_rows 를 사용하므로 desync(오버플로) 없음.
        let header_overhead =
            if is_continuation && mt.repeat_header && mt.has_header_cells && row_count > 1 {
                let hr: Vec<usize> = row_geometry_table
                    .leading_header_rows()
                    .into_iter()
                    .filter(|&r| r < cursor_row)
                    .collect();
                if hr.is_empty() {
                    0.0
                } else {
                    let h: f64 = hr.iter().map(|&r| cut_row_h[r]).sum();
                    h + cs * hr.len() as f64
                }
            } else {
                0.0
            };
        let avail_for_rows = {
            // [Task #1831] 단 상단에서 시작하는 첫 fragment 가 표 **전체** 기준
            // 근소(≤2px) 오차로만 넘치면 전체 배치를 허용한다 — 행높이 측정
            // 드리프트로 마지막 행/블록이 다음 쪽으로 밀리는 것을 방지. 실측:
            // 2448877 표2 = 캡션 28.7 + 행합 914.2 vs 가용 941.1 (1.8px 초과)
            // 인데 한글은 한 쪽(p2)에 통째 배치. 전체가 들어갈 때만 적용하므로
            // 분할 경계 산정에는 영향 없음.
            const WHOLE_TABLE_FIT_TOLERANCE_PX: f64 = 2.0;
            let base = (page_avail - header_overhead).max(0.0);
            if !is_continuation
                && cursor_row == 0
                && start_cut.is_empty()
                && !strict_following_plain_text_fit
                && total_rows_h > base
                && total_rows_h <= base + WHOLE_TABLE_FIT_TOLERANCE_PX
            {
                total_rows_h
            } else {
                base
            }
        };
        // 후속 host의 양수 vpos rewind는 표 continuation이 새 source page에서
        // 이어짐을 뜻한다. object가 전체 row geometry를 덮지 않을 때만 common
        // height를 첫 fragment frame으로 보고, 그 frame에 가장 가까운 행 끝에만
        // 측정 행높이와 source row boundary의 차이를 적용한다.
        if source_next_positive_rewind
            && st.current_footnote_height <= 0.0
            && !table_declared_object_covers_cell_row_frames(table, self.dpi)
        {
            if let Some(row_end) = source_first_fragment_row_end {
                let source_row_end_height = cut_row_h.iter().take(row_end).sum::<f64>()
                    + cs * row_end.saturating_sub(1) as f64;
                source_first_fragment_overflow_allowance = source_first_fragment_overflow_allowance
                    .max((source_row_end_height - avail_for_rows).max(0.0));
            }
        }

        // [Task #1046 Stage 2 진단] 첫/연속 fragment 의 가용공간 분해 — 렌더러
        // y_start 점프(vert_offset)·host_before 와의 정합 확인용. 동작 불변(게이트).
        if std::env::var("RHWP_TABLE_DRIFT").is_ok() {
            eprintln!(
                "TABLE_SPLIT_AVAIL: pi={} sec={} cursor_row={} cont={} cur_h={:.1} table_avail={:.1} caption={:.1} host_before={:.1} vert_off={:.1} outer_bottom={:.1} page_avail={:.1} header_oh={:.1} avail_for_rows={:.1} start_cut={:?}",
                para_idx, st.section_index, cursor_row, is_continuation, st.current_height,
                table_available, caption_extra, host_before_overhead, vert_offset_overhead,
                fragment_outer_bottom_overhead, page_avail, header_overhead, avail_for_rows,
                start_cut,
            );
        }

        FragmentBudget {
            caption_extra,
            host_before_overhead,
            terminal_outer_bottom_overhead,
            fragment_outer_bottom_overhead,
            vert_offset_overhead,
            page_avail,
            fragment_placement,
            scan_row_count,
            saved_first_fragment_source_frame,
            source_first_fragment_row_end,
            source_first_fragment_overflow_allowance,
            header_overhead,
            avail_for_rows,
        }
    }
}
