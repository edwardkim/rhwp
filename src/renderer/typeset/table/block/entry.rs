//! Whole-placement attempts, preserving their ordering and short-circuit returns.

use crate::renderer::typeset::{
    controls, hwpunit_to_px, is_page_bottom_fixed_float, is_para_topbottom_float,
    is_synthetic_line_seg, line_seg_visible_bounds_px,
    native_hwp5_saved_rowbreak_tail_frame_matches, para_has_non_whitespace_text,
    para_has_visible_text, raw_table_ctrl_height_px, row_geometry_table,
    rowbreak_table_has_internal_saved_vpos_reset, saved_bounds_overlap_current_flow,
    signed_hwpunit, single_line_visible_bounds_px, table, Control, PageItem, TypesetEngine,
    TypesetState, SINGLE_ROW_DECLARED_TRUST_MAX_RATIO,
};

use super::{BlockTableInput, HostPlacementConstraint, SplitTableEntry};

impl TypesetEngine {
    pub(super) fn prepare_block_table_entry<'a>(
        &self,
        st: &mut TypesetState,
        input: BlockTableInput<'a>,
    ) -> Option<SplitTableEntry<'a>> {
        let BlockTableInput {
            para_idx,
            ctrl_idx,
            para,
            table,
            ft,
            fmt,
            mt,
            styles,
            para_start_height,
            budget_para_start_height,
            is_first_placed,
            is_last_placed,
            paragraphs_all,
            ..
        } = input;
        // [#6764] 다른 문단이 남긴 자리차지 밴드를 표 높이로 먼저 짚는다 — 예산은
        // 밴드 아래에서 시작한다. HWPX 는 문단 프로브가 이미 같은 일을 하므로 제외.
        if !st.profile.hwpx_stored_layout() {
            st.apply_float_band_before_block_table(para_idx, ft.effective_height);
        }
        // 표 내 각주를 고려한 가용 높이 계산 (Paginator engine.rs:583-586 동일)
        // [#7379] 통째/분할 **진입 판정**의 예산에는 표 자신의 각주를 미리 싣지 않는다.
        //
        // 종전에는 표 전체의 각주 높이를 `available` 에서 먼저 깎았다. 그러면 각주를 든
        // 표는 자기 각주 때문에 현재 쪽의 남은 자리가 음수가 되어, **분할을 시도조차
        // 못 하고** 통째로 다음 쪽으로 간다. 쪼개졌다면 첫 조각은 그 조각이 실제로
        // 데려가는 각주만 필요한데, 판정은 어느 조각도 쓰지 않을 예산을 요구한 것이다.
        //
        // 실측(`samples/정책연구용역사업 중간진도보고서(…).hwpx`, 한/글 정본 215쪽):
        //   문단 0.728 표(7행 · RowBreak · 자리차지 · 표 안 각주 6건)
        //     종전  total_footnote 294.0 (쪽 각주 43.4 + 표 각주 250.6)
        //           → available 622.2 < current_height 713.8  → 통째 이월
        //           → 그 쪽에 실제로 그려진 각주는 35.9px, 본문은 220px 공백
        //     이후  같은 자리에서 분할 경로로 들어간다(한/글도 이 표를 p66/p67 로 쪼갠다).
        //
        // 조각이 실제로 데려간 각주는 등록 시점에 `register_body_footnote` 가 계상하므로
        // 예약이 사라지는 것이 아니라 **판정 시점이 뒤로 밀릴 뿐**이다. samples 1,141문서
        // 전수에서 바뀐 문서는 2건이고 둘 다 개선이다(정답지 거리 개선 1 · 악화 0 ·
        // 일치 상실 0 · 넘침 −1 · 글자겹침 −179).
        let mut total_footnote = st.projected_footnote_height(0.0, 0);
        // [#1921 d=+1 / Task #1725 동형] tail-before-vpos-reset 표는 각주 안전마진
        // (보수 버퍼 40px)을 완화한다. 한글은 stored vpos 상 쪽 하단에 표+각주를
        // 여유 수 px 로 타이트하게 배치하는데(75828 pi134: 표 하단 912.2 + 각주
        // 46.1 = 958.3 ≤ 971.3), rhwp 의 보수 버퍼가 표를 다음 쪽으로 밀어
        // 이후 stored vpos=0 신호 연쇄로 문서 끝까지 +1쪽이 된다. 다음 문단의
        // stored LINE_SEG 가 새 쪽 시작(vpos≤500)을 명시할 때만 완화하므로
        // 겹침 위험은 stored 배치가 보증하는 범위로 한정된다.
        let table_anchor_vpos = para
            .line_segs
            .iter()
            .find(|ls| !is_synthetic_line_seg(ls))
            .map(|ls| ls.vertical_pos)
            .unwrap_or(0);
        let next_starts_new_page = table_anchor_vpos > 0
            && paragraphs_all
                .get(para_idx + 1)
                .and_then(|p| p.line_segs.iter().find(|ls| !is_synthetic_line_seg(ls)))
                .is_some_and(|ls| ls.vertical_pos <= 500 && ls.vertical_pos < table_anchor_vpos);
        // 저장된 다음 문단의 첫 vpos가 표 anchor보다 위로 되감기면, 이 RowBreak
        // 표의 다음 조각은 새 physical page에서 시작한다. p90 표 27처럼 한글이
        // 기존 각주 구분선 바로 위까지 마지막 온전한 행을 두는 계약을 좁혀
        // 식별하는 신호다. `<=500`인 새 쪽 상단 형상보다 넓지만, 아래의 native
        // HWP5·비-TAC·ordinary-row·기존 각주 조건과 함께만 사용한다.
        let next_rewinds_after_table = table_anchor_vpos > 0
            && paragraphs_all
                .get(para_idx + 1)
                .and_then(|p| p.line_segs.iter().find(|ls| !is_synthetic_line_seg(ls)))
                .is_some_and(|ls| ls.vertical_pos < table_anchor_vpos);
        // [#3738 Stage 15] HWP5 RowBreak 표의 cell 내부 vpos reset은 같은 row의
        // 앞부분을 현재 쪽, reset 뒤 tail을 다음 쪽에 둔 저장 경계다. 이 구조에서
        // footnote safety margin까지 유지하면 p76 표 24의 앞 3줄이 1줄로 줄어들고
        // p77에는 row 전체가 재배치되어 그림 51이 별도 page로 밀린다. native HWP5의
        // 비-TAC TopAndBottom RowBreak 표, 기존 각주, 표 자체 각주 없음, 실제 cell
        // reset이라는 네 축이 모두 있을 때만 실제 FootnoteArea 직전까지의 공간을 쓴다.
        let internal_reset_tail_uses_actual_footnote_boundary =
            st.profile.hwp5_stored_pagination_layout()
                && !table.common.treat_as_char
                && is_para_topbottom_float(&table.common)
                && matches!(
                    table.page_break,
                    crate::model::table::TablePageBreak::RowBreak
                )
                && ft.table_footnotes.is_empty()
                && st.current_footnote_height > 0.0
                && rowbreak_table_has_internal_saved_vpos_reset(table);
        let host_spacing_total = ft.host_spacing.before + ft.host_spacing.after_for_fit;
        let mut table_total = ft.effective_height + host_spacing_total;
        let native_single_rowbreak_full_table_fits_actual_footnote_boundary =
            st.profile.hwp5_stored_pagination_layout()
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
                && ft.table_footnotes.is_empty()
                && st.current_footnote_height > 0.0
                && st.current_height + table_total
                    <= (st.base_available_height() - total_footnote - st.current_zone_y_offset)
                        .max(0.0)
                        + 0.5;
        // [#3738 Stage 31] 표 27처럼 일반 행으로만 이루어진 native HWP5
        // TopAndBottom RowBreak 표는 저장된 후속 본문이 anchor보다 위로 되감길 때,
        // 마지막 온전한 행을 기존 FootnoteArea 바로 위까지 둔다. 일반 40px safety
        // margin은 p90의 30.7px relationship row를 p91로 과도하게 밀었다. 표 셀
        // 각주/rowspan/중간 reset 표는 각자 별도 계약을 가지므로 제외하고, 낮은
        // 본문 위치에서 실제 다음-쪽 되감김이 확인된 ordinary-row 형상만 첫 조각
        // scan에 실제 footnote boundary를 준다. scan은 그 경계를 넘어 행을 자르지
        // 않으므로 각주와의 물리 overlap을 허용하지 않는다.
        let native_ordinary_rowbreak_rewind_uses_actual_footnote_boundary =
            st.profile.hwp5_stored_pagination_layout()
                && !table.common.treat_as_char
                && is_para_topbottom_float(&table.common)
                && matches!(
                    table.page_break,
                    crate::model::table::TablePageBreak::RowBreak
                )
                && table.row_count > 1
                && ft.table_footnotes.is_empty()
                && st.current_footnote_height > 0.0
                && st.current_height >= st.base_available_height() * 0.5
                && table.cells.iter().all(|cell| cell.row_span == 1)
                && next_rewinds_after_table;
        let mut fn_margin = if total_footnote > 0.0 {
            if next_starts_new_page
                || internal_reset_tail_uses_actual_footnote_boundary
                || native_single_rowbreak_full_table_fits_actual_footnote_boundary
            {
                0.0
            } else {
                st.footnote_safety_margin
            }
        } else {
            0.0
        };
        let mut available =
            (st.base_available_height() - total_footnote - fn_margin - st.current_zone_y_offset)
                .max(0.0);

        let declared_object_total = raw_table_ctrl_height_px(table, self.dpi)
            .unwrap_or_else(|| hwpunit_to_px(table.common.height as i32, self.dpi).max(0.0))
            + host_spacing_total;
        let declared_empty_para_float_total = if !st.profile.hwpx_stored_layout()
            && !table.common.treat_as_char
            && is_para_topbottom_float(&table.common)
            && !para_has_visible_text(para)
            && matches!(
                table.page_break,
                crate::model::table::TablePageBreak::RowBreak
            ) {
            let declared_height = hwpunit_to_px(table.common.height as i32, self.dpi).max(0.0);
            (declared_height > 0.0).then_some(declared_height + host_spacing_total)
        } else {
            None
        };
        let mut reserve_declared_table_total = false;
        let mut native_hwp5_internal_reset_rewind_needs_anchor_resync = false;
        let mut native_hwp5_multirow_internal_reset_needs_anchor_resync = false;

        // [Task #1046 Stage 1] 표 측정 드리프트 진단: 페이지네이터 effective_height vs
        // MeasuredTable 행높이 합(+cell_spacing). RHWP_TABLE_DRIFT=1 시 출력.
        if std::env::var("RHWP_TABLE_DRIFT").is_ok() {
            let (mt_sum, mt_rows, mt_cs) = match mt {
                Some(m) => {
                    let cs_total = m.cell_spacing * (m.row_heights.len() as f64 + 1.0);
                    (
                        m.row_heights.iter().sum::<f64>() + cs_total,
                        m.row_heights.len(),
                        m.cell_spacing,
                    )
                }
                None => (f64::NAN, 0, 0.0),
            };
            eprintln!(
                "TABLE_DRIFT: pi={} sec={} eff_h={:.1} host_sp={:.1} table_total={:.1} mt_sum={:.1} mt_rows={} cs={:.1} cur_h={:.1} tac={} rows={} avail={:.1} base={:.1} fn={:.1} zone={:.1} declared={:.1} pb={:?} mt_row_heights={:?}",
                para_idx, st.section_index, ft.effective_height, host_spacing_total, table_total,
                mt_sum, mt_rows, mt_cs, st.current_height, table.common.treat_as_char, table.row_count,
                available, st.base_available_height(), total_footnote, st.current_zone_y_offset,
                declared_object_total, table.page_break,
                mt.map(|m| m.row_heights.clone()).unwrap_or_default(),
            );
        }
        // [Task #1027 Stage E1] treat_as_char 인라인 표 advance 정합.
        // 렌더러는 글자처럼취급 표를 호스트 문단의 한 LINE_SEG(line_height+line_spacing)로
        // advance 하나(=fmt.total_height), 페이지네이터는 측정된 표 effective_height 만
        // 더해 ~수십px 과소측정 → 표 이후 콘텐츠가 렌더러보다 위에 fit 판정되어 overflow
        // (Stage D 조사: p71 pi=349 +16.9px). 호스트가 표 한 줄로 구성된 경우(line==1)
        // 렌더러 advance(fmt.total_height)로 정합한다.
        if table.common.treat_as_char
            && fmt.line_heights.len() == 1
            && fmt.total_height > table_total
        {
            table_total = fmt.total_height;
        }

        // Task #321 v5: Paper-anchored TopAndBottom block 표는 절대 좌표로 그려지므로
        // cur_h advance 에 표 effective_height 를 그대로 더하면 본문 LINE_SEG vpos 와
        // mismatch (= 21_언어 page 1 col 0 의 +76 px drift). 본문 좌표계와 동기화 하기
        // 위해 host paragraph 의 first_vpos 만큼 cur_h 를 미리 jump 하고 표 advance 를
        // 본문 라인 만큼으로 축소.
        use crate::model::shape::{TextWrap, VertRelTo};
        // [#1994] Paper(용지)-앵커 부동 표는 절대 좌표로 그려지므로 flow 를 소비하지 않고 절대
        // 배치해야 한다. 기존에는 자리차지(TopAndBottom)만 이 경로를 탔으나, 글뒤로/글앞으로
        // (BehindText/InFrontOfText) Paper-앵커 표도 동일하게 절대 배치 대상이다. 특히
        // RowBreak 속성이 붙은 글뒤로 Paper-앵커 표(20200830 교회주보 pi=34 예배 스케줄,
        // vert=용지 134mm)가 이 경로를 놓치면 아래 RowBreak 분할로 빠져 흐름 상단에 컬럼분할
        // 배치되어 앞선 글뒤로 표(pi=33 교역자 명단)와 겹친다(#1994).
        let is_paper_floating_block = !table.common.treat_as_char
            && matches!(
                table.common.text_wrap,
                TextWrap::TopAndBottom | TextWrap::BehindText | TextWrap::InFrontOfText
            )
            && matches!(table.common.vert_rel_to, VertRelTo::Paper);
        // 글뒤로/글앞으로는 본문 위/아래에 겹쳐 그려지며 본문 텍스트를 밀어내지 않는다
        // (자리차지와 달리 current_height sync 로 후속 흐름을 끌어내리면 안 됨).
        let is_paper_behind_infront = !table.common.treat_as_char
            && matches!(
                table.common.text_wrap,
                TextWrap::BehindText | TextWrap::InFrontOfText
            )
            && matches!(table.common.vert_rel_to, VertRelTo::Paper);
        if is_paper_floating_block && st.current_column == 0 {
            if let Some(first_seg) = para.line_segs.first() {
                let target_y =
                    crate::renderer::hwpunit_to_px(first_seg.vertical_pos as i32, self.dpi);
                // 호스트 본문 lines + 표는 절대 좌표 → cur_h 는 first_vpos + host lines 만 진행.
                let pre_lines_h = fmt.line_advances_sum(0..fmt.line_heights.len());
                let can_sync = target_y > st.current_height && target_y + pre_lines_h <= available;
                // [Task #1858] Paper 앵커 자리차지 표는 절대좌표(table_layout vert=Paper)로
                // 그려지며 flow 를 소비하지 않는다. 첫 박스는 host vpos 로 current_height 를
                // sync 해 본문 텍스트를 정렬하지만, 같은 host 문단에 co-anchored 된 후속 Paper
                // 박스는 target_y(=동일 host vpos)가 이미 sync 된 current_height 이하라 기존
                // 가드(target_y > current_height)를 통과하지 못하고 flow 경로로 빠져 각 박스가
                // 높이를 소비 → 페이지가 폭발했다(3143097: 용지앵커 서식상자 22개가 모두 1쪽
                // 용지좌표에 있는데 한컴 1쪽 대비 rhwp 3~4쪽). 후속 co-anchored Paper 박스도
                // 절대배치(0 flow)한다. 단독 Paper 박스(선행 없음·sync 불가)는 기존대로 flow.
                let has_preceding_paper_float = para.controls.iter().take(ctrl_idx).any(|c| {
                    matches!(c, Control::Table(t)
                        if !t.common.treat_as_char
                            && matches!(t.common.text_wrap,
                                TextWrap::TopAndBottom
                                    | TextWrap::BehindText
                                    | TextWrap::InFrontOfText)
                            && matches!(t.common.vert_rel_to, VertRelTo::Paper))
                });
                // 글뒤로/글앞으로는 sync 없이도 절대배치(0 flow)한다 — RowBreak 분할·flow 배치를
                // 막아 절대 좌표(vert=용지)에 통째로 그려지게 한다.
                if can_sync || has_preceding_paper_float || is_paper_behind_infront {
                    if can_sync && !is_paper_behind_infront {
                        st.align_flow_to(target_y);
                    }
                    // table_total = 0: 표 자체는 cur_h advance 에 영향 없음 (Paper-absolute).
                    // 호스트 본문 lines 만 place_table_with_text 가 pre_height 로 추가(첫 박스만).
                    self.place_table_with_text(
                        st,
                        para_idx,
                        ctrl_idx,
                        para,
                        table,
                        fmt,
                        para_start_height,
                        0.0,
                        is_first_placed,
                        is_last_placed,
                        ft.strict_following_plain_text_fit,
                        styles,
                    );
                    return None;
                }
            }
        }

        // fits: 전체가 현재 페이지에 들어가는가?
        let is_rowbreak_para_topbottom_block = !table.common.treat_as_char
            && matches!(table.common.text_wrap, TextWrap::TopAndBottom)
            && matches!(table.common.vert_rel_to, VertRelTo::Para)
            && matches!(
                table.page_break,
                crate::model::table::TablePageBreak::RowBreak
            )
            && table.cells.iter().any(|cell| {
                cell.paragraphs.iter().any(|p| {
                    !p.text.trim().is_empty()
                        && p.controls
                            .iter()
                            .any(|c| matches!(c, crate::model::control::Control::Table(_)))
                })
            });
        if is_rowbreak_para_topbottom_block {
            if let Some(first_seg) = para.line_segs.first() {
                let target_y =
                    crate::renderer::hwpunit_to_px(first_seg.vertical_pos as i32, self.dpi);
                let previous_item_is_continued_paragraph = matches!(
                    st.current_items.last(),
                    Some(PageItem::PartialParagraph { start_line, .. }) if *start_line > 0
                );
                // 저장 줄이 표보다 **앞선** 줄일 때만 snap 한다. host 문단의 단일
                // lineseg 가 표 **아래**의 꼬리 줄(vpos ≈ 표 하단)인 문서에서 그
                // vpos 로 snap 하면 표 배치 전에 페이지가 소진돼(잔여 ≈ 꼬리 여백)
                // whole-fit 이 깨지고, 선언높이가 본문에 들어가는 표가 행 단위로
                // 과분할된다 (#6271: 1쪽 문서가 2쪽, 1쪽에는 머리 행 조각만 잔존).
                let snap_keeps_table_fitting = target_y + declared_object_total <= available;
                if !previous_item_is_continued_paragraph
                    && target_y > st.current_height
                    && target_y < available
                    && snap_keeps_table_fitting
                {
                    st.align_flow_to(target_y);
                }
            }
        }

        // [Task #1611] PAGE-앵커(vert=쪽) + valign=Bottom 자리차지 표(발신명의 footer 등)는
        // 한컴이 stored vpos 위치에 두고, 본문 누적이 그 위치+높이를 넘기면 블록을 통째로
        // 다음 쪽에 단독 배치한다. Paper-앵커(절대좌표, 위 10440)와 달리 페이지네이션에
        // 참여하므로 cur_h 를 stored vpos 로 끌어올린 뒤(본문 흐름이 vpos 보다 짧을 때) fit 을
        // 판정한다. 동기화하지 않으면 footer 가 flowed cur_h(vpos 보다 ~수십px 낮음)에 배치되어
        // page-fit 이 과소되고 footer 가 본문 페이지에 흡수된다(−1쪽 갭 요인 B).
        let is_page_bottom_topbottom_block = is_page_bottom_fixed_float(&table.common);
        if is_page_bottom_topbottom_block && st.current_column == 0 {
            if let Some(first_seg) = para.line_segs.first() {
                // 한컴은 고정크기 자리차지 블록을 **선언 높이**(common.height)로 렌더·예약한다.
                // 페이지네이터의 effective_height 는 셀 내용 기반 측정치라 선언보다 작을 수 있어
                // (footer 351.4px 선언 vs 302.3px 측정) fit 이 과소된다 → 선언 높이로 판정·예약.
                let declared_px =
                    crate::renderer::hwpunit_to_px(table.common.height as i32, self.dpi);
                let block_height = table_total.max(declared_px);
                // [Task #1658 v3] 한글 실측(stage1): 하단 고정 틀은 본문 하단에
                // 절대배치(다수 시 서로 겹침 허용)되고 본문 텍스트는 하단 배타 영역
                // (= 최대 블록 높이) 위까지만 흐른다 — flow 소비 모델이 아니다
                // (관악 36389312: 틀 2개 합 604px 가 flow 소비되면 한글 1쪽이 2쪽으로
                // over-pagination). 저장 vpos 는 하단 틀도 문서순 누적하므로, 같은
                // 페이지에 이미 예약된 틀의 소비분을 차감해 본문 텍스트 끝을 복원한다.
                // [#2098] 앵커 저장 vpos=0(쪽 기준 절대배치 산물, opengov 결재문서 계열)
                // 이면 직전 본문 문단의 저장 흐름 하단(prev_body_bottom_vpos)으로 본문
                // 끝을 복원해 fit 을 판정한다. flowed cur_h 는 누적 드리프트로 과소될
                // 수 있다 — 36387725: cur_h 578px vs 저장 640.7px → 한글은 분할(2쪽),
                // 36358528: 저장 586.1px ≤ 배타 638.8px → 한글은 흡수(1쪽).
                let anchor_vpos = first_seg.vertical_pos;
                let flow_end_vpos = if anchor_vpos <= 0 {
                    st.prev_body_bottom_vpos.unwrap_or(anchor_vpos)
                } else {
                    anchor_vpos
                };
                let target_y = crate::renderer::hwpunit_to_px(flow_end_vpos, self.dpi)
                    - st.bottom_fixed_consumed_flow;
                // [Task #1624] footer stored vpos 가 흐름 cur_h 보다 footer 한 개 높이 이상 위에
                // 있으면(본문이 짧은데 vpos 가 page-bottom 앵커/누적 노이즈), vpos 동기화는
                // 본문 직후에 들어갈 footer 를 spurious 하게 다음 쪽으로 민다(+1쪽 over-push).
                // vpos 가 흐름을 plausibly 따를 때(cur_h + block_height 이내)만 동기화한다.
                // [#2279 footer-오염] 같은 쪽에 PAGE-앵커 Top 절대배치 표가 있으면
                // 저장 누적이 절대 위치 산물로 부풀어(36496000 pi3: 저장 스텝
                // +482px vs 흐름 +143px) target_y 가 본문 끝이 아니다 — 동기화를
                // 건너뛰고 흐름 좌표로 판정한다(한글 PDF 본문 끝 ~520px = 흐름
                // 517.3px 실측 일치, stored 861.2px 는 허상. 한글 1쪽 vs +1 유령).
                // [#6535] 이 블록 **자신**이 쪽-앵커면 그 host 문단의 저장 vpos 도 절대 위치
                // 산물이다 — 위 `page_has_page_abs_top_table` 이 "같은 쪽의 **다른** 절대배치
                // 표"에 대해 편 논리와 같은 것이고, 블록 자신에게 적용하지 않을 이유가 없다.
                //
                // 실측(36404612 pi=4, `vert=쪽(0)` 발신명의 틀): 흐름 542.80px 인데 저장
                // vpos 49154 = 655.39px 로 **112.6px** 상향돼 배타 잔여 638.87 을 넘어(slack
                // -16.52) 틀이 통째로 2쪽에 단독 배치됐다 — 본문 없는 빈 쪽이 생기고 한/글은
                // 1쪽이다. 동기화를 건너뛰면 slack +96.07 로 같은 쪽에 흡수된다.
                //
                // `anchor_vpos <= 0` 인 경우는 위에서 이미 `prev_body_bottom_vpos`(직전 본문
                // 문단의 저장 흐름 하단)로 복원한 **본문 좌표**라 이 예외 대상이 아니다.
                let block_anchor_vpos_is_absolute = anchor_vpos > 0
                    && matches!(
                        table.common.vert_rel_to,
                        crate::model::shape::VertRelTo::Page
                    );
                let sync_h = if !st.page_has_page_abs_top_table
                    && !block_anchor_vpos_is_absolute
                    && target_y <= st.current_height + block_height
                {
                    st.current_height.max(target_y)
                } else {
                    st.current_height
                };
                let v_off = hwpunit_to_px(
                    signed_hwpunit(table.common.vertical_offset).max(0),
                    self.dpi,
                );
                let prospective_excl = st.current_bottom_fixed_exclusion.max(block_height + v_off);
                // available(12359)은 배타 영역 미차감 값(base - 각주 - zone)이다. 이 블록
                // 편입 후의 배타 영역(prospective_excl)을 차감해 "본문 텍스트 끝이 배타
                // 영역을 침범하는가"를 판정한다. [Issue #1920] 종전 `available +
                // current_excl - prospective_excl` 은 available 이 이미 차감됐다는 잘못된
                // 가정으로, 같은 쪽 두 번째 틀에서 기존 배타분만큼 과관용해져 한글이
                // 다음 쪽으로 넘기는 틀을 현재 쪽에 흡수했다(36373162 pi16, 2쪽→1쪽).
                let avail_after = available - prospective_excl;
                // [#2098 재보정, r12] 앵커 vpos≤0(절대배치 산물)의 저장-흐름-끝 복원
                // fit 은 경계 케이스에서 과관용 — 10k r12 재검에서 결재문서 60건이
                // 흡수돼 한글(분할)과 어긋났다. 10k 슬랙 실측: 분할 정답군 3.4~52.1px,
                // 흡수 정답군 {37.1, 39.6, 67.7, 72.9}px 로 **중첩** — 슬랙 스칼라로는
                // 완전 분리 불가(진짜 판별 신호는 후속 조사). 53px 마진은 분할군
                // 전건(≤52.1)을 한글처럼 분할하고 고슬랙 흡수(67.7/72.9)를 유지하는
                // 순최적점. 저슬랙 흡수 2건(37.1/39.6)은 기지 한계(r11 동일).
                // [#2138 재보정] warm PDF 권위 재확정: 분할 정답군 슬랙 3.4~61.3px
                // (36394733 61.3 포함 — 53px 마진이 이를 흡수해 신규 회귀), 흡수
                // 정답군 {37.1, 39.6, 67.7, 72.9}. 최적 스칼라 구간 [61.3, 67.7) 의
                // 62px 채택 — 잔여 오류는 저슬랙 흡수 2건(36358528/36477251, r11 동일
                // 기지 한계). 부수 발견: 한글 자체가 fresh-open/warm-open 에 따라
                // 같은 문서를 1쪽/2쪽으로 다르게 레이아웃(PDF 포함) — 권위 판정은
                // warm PDF 로 통일(#2138 stage1).
                // [#2279 성분②] 재구성 사다리의 host 줄박스 정합으로 본문 흐름
                // 좌표가 om_bottom(~11.4px)만큼 전진 — 압축-사다리 좌표계 기준이던
                // 62px 를 같은 폭만큼 하향(50). 코호트 재판정: 분할 정답 최대 슬랙
                // 42.5(36395825) < 50 < 흡수 정답 최소 슬랙 56.4(36376848) 로
                // 62 시절의 기지 한계(저슬랙 흡수 2건) 외 오분류 없음.
                // [#2098 마진의 적용 조건] 이 마진이 보정하는 불확실성은 "앵커 vpos≤0 이라
                // 본문 끝을 저장 vpos(prev_body_bottom_vpos)에서 **복원**했다"는 사실 자체에서
                // 온다 — 복원값이 흐름 cur_h 를 실제로 끌어올렸을 때만 판정이 복원에 의존한다
                // (36387725: cur_h 578 → 복원 640.7 로 상향, 이 상향분이 과관용의 근원).
                // 복원이 판정을 바꾸지 않았다면(sync_h == cur_h: 복원값이 흐름 이하이거나
                // #2279 처럼 동기화를 건너뛴 경우) 남은 불확실성이 없는데도 마진이 흐름 좌표
                // 기준 fit 을 일률적으로 깎아, 여유가 실재하는 쪽을 분할한다
                // (task2098/page_bottom_fixed_anchor_margin_split: cur_h == 복원 754.67,
                // 배타 잔여 800.24 → 한글 2020 정본 1쪽인데 rhwp 는 2쪽).
                // 복원이 실제로 상향한 경우에만 마진을 건다 — 코호트 재판정 신호(슬랙 스칼라)는
                // 그대로 두고 적용 범위만 좁힌다.
                let restoration_raised_fit = sync_h > st.current_height;
                // [#6535] 마진을 거는 세 번째 조건 — **흐름 좌표가 실제로 뒤처져 있을 때만**.
                //
                // 슬랙 스칼라로는 두 코호트가 갈리지 않는다는 것이 이 마진의 기지 한계였다.
                // 경합 구간을 전수 재 보니 갈리는 것은 슬랙이 아니라 `flow_underrun` 이다 —
                // 이 단의 문단 place 가 트림한 `(total_height − advance)` 누계로, 0 보다
                // 크면 `cur_h` 가 실제 내용 하단을 **과소**하게 들고 있다는 뜻이다. 마진이
                // 보정하려던 불확실성이 바로 그것이다.
                //
                //   흡수 정답(한글 1쪽, rhwp 2쪽): slack 25.0 / 25.8 / 31.6 / 35.7 / 37.8
                //                                  underrun **전부 0.00**
                //   분할 정답(한글 2쪽)          : slack 42.5  underrun **37.60**
                //
                // 슬랙은 25.0~42.5 로 완전히 겹치는데 `underrun` 은 0 vs 37.60 으로 갈린다.
                let flow_lags_behind_content = st.flow_underrun > 0.5;
                let uncertain_anchor_margin =
                    if anchor_vpos <= 0 && restoration_raised_fit && flow_lags_behind_content {
                        50.0
                    } else {
                        0.0
                    };
                // [#2279 진단] footer 흡수/분할 판정 변수 분해 — 동작 불변.
                // underrun = 이 단의 문단 place 가 트림한 (total_height − advance) 누계
                // (렌더/한글 좌표와의 발산 중 문단-sa 성분; 표 place 성분은 미포함).
                if std::env::var("RHWP_DIAG_SCAN").is_ok() {
                    eprintln!(
                        "DIAG_SCAN FOOTER pi={} anchor_vpos={} cur_h={:.2} target_y={:.2} sync_h={:.2} \
                         block_h={:.2} v_off={:.2} avail={:.2} avail_after={:.2} slack_code={:.2} margin={:.1} underrun={:.2}",
                        para_idx,
                        anchor_vpos,
                        st.current_height,
                        target_y,
                        sync_h,
                        block_height,
                        v_off,
                        available,
                        avail_after,
                        avail_after - sync_h,
                        uncertain_anchor_margin,
                        st.flow_underrun,
                    );
                }
                if sync_h + uncertain_anchor_margin <= avail_after {
                    // 현재 쪽 하단에 배치 — 본문 흐름은 vpos 동기 위치까지만 전진.
                    st.align_flow_to(sync_h);
                } else if !st.current_items.is_empty() {
                    // 배타 영역 침범 → 발신명의 블록을 통째로 다음 쪽에 단독 배치(분할 부적절).
                    if std::env::var("RHWP_DIAG_SPLITSCAN").is_ok() {
                        eprintln!(
                            "DIAG_ADVA pi={} sec={} cur_h={:.1}",
                            para_idx, st.section_index, st.current_height
                        );
                    }
                    st.advance_column_or_new_page();
                }
                let flow_before = st.current_height;
                self.place_table_with_text(
                    st,
                    para_idx,
                    ctrl_idx,
                    para,
                    table,
                    fmt,
                    para_start_height,
                    block_height,
                    is_first_placed,
                    is_last_placed,
                    ft.strict_following_plain_text_fit,
                    styles,
                );
                // 배타 모델: 블록은 flow 를 소비하지 않는다(하단 절대배치) — 소비 롤백
                // 후 하단 배타 영역으로 예약. 저장-flow 소비 누계는 후속 틀 vpos 보정용.
                let consumed = (st.current_height - flow_before).max(0.0);
                st.align_flow_to(flow_before);
                st.reserve_bottom_fixed_flow(
                    consumed,
                    st.current_bottom_fixed_exclusion.max(block_height + v_off),
                );
                // [Issue #1920] 후속 일반 문단의 vpos 캘리브레이션(vpos_snap_current_height)은
                // 저장 flow 좌표를 그대로 따라간다. 한글 저장 vpos 는 하단 고정 틀의 높이도
                // 문서순 누적하므로, 스냅이 틀 높이만큼 전진한 좌표로 이동한 뒤 배타 영역
                // 차감(available)과 이중 계산되어 문단이 다음 쪽으로 밀린다(결재문서본문
                // 36373162 pi15: 694→935px 스냅 + 247px 배타 → 한글 p1 이 p2 로, d=+1).
                // 롤백한 소비분만큼 활성 vpos base 를 전진시켜 후속 문단의 저장→flow 변환을
                // 본문 흐름 좌표계에 정렬한다(후속 틀의 target_y 보정과 동일 원리).
                if consumed > 0.0 {
                    let consumed_hu = (consumed / self.dpi * 7200.0).round() as i32;
                    if consumed_hu > 0 {
                        if let Some(base) = st.vpos_page_base {
                            st.record_vpos_page_origin(Some(base + consumed_hu));
                        } else if let Some(base) = st.vpos_lazy_base {
                            st.record_vpos_lazy_origin(Some(base + consumed_hu));
                        }
                    }
                }
                return None;
            }
        }

        // 같은 host 문단에 co-anchored 된 *후속* 자리차지(TopAndBottom, vert=문단) RowBreak
        // 표의 orphan 제어: 현재 페이지 잔여 공간엔 표 전체가 안 들어가지만 새(fresh) 페이지엔
        // 통째로 들어가면, 행 단위로 쪼개 머리 일부(예: 결재 헤더 행)만 현재 페이지에 남기지
        // 않고 표 전체를 다음 페이지로 이월한다. 한컴은 선행 자리차지 표가 페이지를 채운 뒤의
        // 후속 co-anchored 자리차지 표를 분할하지 않고 통째로 다음 페이지에 둔다(검증점검표에서
        // 결재 헤더가 본문과 다른 페이지로 분리되던 회귀).
        //
        // 단독 anchored 자리차지 표(host 의 첫/유일 float)는 본문 흐름에 따라 행 단위로 정상
        // 분할되어야 하므로(한컴 기준) 제외한다 — has_preceding_coanchored_float 로, 같은 host
        // 의 *앞선* 컨트롤에 다른 자리차지 표가 있을 때(= co-anchored 그룹의 2번째 이후)로
        // 한정한다. table_total <= available(= 한 페이지에 통째로 들어감)일 때만 이월하므로,
        // 한 페이지보다 큰 표는 이 가드를 통과하지 못하고 아래 행 분할 경로로 빠져 무한 push 가
        // 발생하지 않는다.
        let has_preceding_coanchored_float = para
            .controls
            .iter()
            .take(ctrl_idx)
            .any(|c| matches!(c, Control::Table(t) if is_para_topbottom_float(&t.common)));
        // [#2813] 빈 host 문단의 유일한 저장 앵커 줄이 float 스택 아래(현재 흐름
        // 꼬리 이후)이자 본문 안에 들어가는 위치를 인코딩하면, 한글은 이 스택과
        // 앵커 줄을 현재 쪽에 통째 배치한 것이다 — para-relative TopAndBottom
        // 스택에서 앵커 줄 vpos 는 스택 하단 이후를 기록한다(36352939: 저장 줄
        // 666.7+13.3=680.0px ≤ 본문 680.3px 로 한글 1쪽 razor-fit, rhwp 는 셀
        // 실측 팽창으로 naive fit 697.8px 실패 → 표2 이월 +1 과분할). 아래 5축
        // saved_span(개체가 줄 아래 모델)과 반대 형상이라 별도 판별자가 필요하다.
        let has_following_coanchored_float = para
            .controls
            .iter()
            .skip(ctrl_idx + 1)
            .any(|c| matches!(c, Control::Table(t) if is_para_topbottom_float(&t.common)));
        // 스택(같은 host 에 자리차지 float 표 ≥2) 형상의 임의 표에서 참 — 단일
        // float host 는 제외한다: 그 표는 한글도 행 분할하므로(issue #1488 pi=28,
        // 3쪽 분할) 통째-배치 구제 대상이 아니다.
        let host_line_trails_float_stack = is_para_topbottom_float(&table.common)
            && (has_preceding_coanchored_float || has_following_coanchored_float)
            && !para_has_non_whitespace_text(para)
            && single_line_visible_bounds_px(para, st.vpos_page_base.unwrap_or(0), self.dpi)
                .is_some_and(|bounds| {
                    // 줄이 현재 흐름 꼬리보다 자기 줄높이 이상 아래 = 앵커 줄이
                    // 스택 뒤를 인코딩(선두-줄 host 의 vpos 0/흐름-일치 저장은 제외).
                    let line_h = (bounds.1 - bounds.0).max(0.0);
                    bounds.0 > st.current_height + line_h + 16.0
                        // 일반 tail helper는 line이 현재 flow와 겹칠 때만 쓴다.
                        // 여기서는 바로 위 조건이 line이 float stack 뒤에 있음을
                        // 이미 증명하므로, 같은 physical body의 하단 안에 있는지만
                        // source frame으로 확인한다.
                        && bounds.1 <= available
                });
        // 통째-배치 구제는 스택의 2번째 이후 표(선행 co-anchored 존재)이면서
        // 표 자체가 한 쪽에 들어갈 때만 — 첫 표는 정상 fit/분할 경로를 그대로
        // 타고, 쪽보다 큰 표는 한글도 분할하므로(20320575 별표 24쪽: 통째-배치
        // 시 27→8쪽 붕괴 실측) 구제 대상이 아니다.
        // [#6795] 앞 co-anchored 표가 쪽에 걸쳐 쪼개져 **이 쪽이 그 조각으로 시작**하면,
        // 저장 앵커 줄 vpos 는 이 쪽이 아니라 문단이 시작한 쪽의 좌표다. 그 값을 근거로
        // "한글이 스택을 통째로 이 쪽에 놓았다"고 보면, 조각이 이미 차지한 자리에 뒤 표를
        // 겹쳐 놓는다(1341000-201100013 31쪽 548.0 × 401.9px, 아래 표 401.9px 소실).
        // [#3587] 이 원칙은 앞 문단의 연속 조각에도 동일하다. 재편집으로 앞 표가
        // 늘어나 다음 host가 연속 쪽으로 밀렸다면, 다음 host의 저장 앵커 역시 현재
        // 쪽의 잔여 공간을 증명하지 않는다. 소유 문단 일치가 아니라 현재 흐름 프레임의
        // 연속 조각 존재로 판정해야 뒤 스택을 앞 조각/헤더 위에 강제로 겹치지 않는다.
        let page_has_table_continuation = st.current_items.iter().any(|item| {
            matches!(
                item,
                PageItem::PartialTable {
                    is_continuation: true,
                    ..
                }
            )
        });
        // 저장 앵커는 스택 전체의 source frame이다. 셀 실측 때문에 앞 표 높이가
        // 늘어나는 #2813 구제는 보존하되, 앞 문단에 밀려 host 시작점 자체가 옮겨진
        // 경우까지 같은 frame으로 간주하지 않는다. 현재 표만 current_height에 더하면
        // 실측 팽창과 host 이동을 구분하지 못하므로 선언 스택 전체와 host 원점을 쓴다.
        let saved_stack_fits_host_origin = host_line_trails_float_stack
            && single_line_visible_bounds_px(para, st.vpos_page_base.unwrap_or(0), self.dpi)
                .is_some_and(|bounds| {
                    let declared_stack_height: f64 = para
                        .controls
                        .iter()
                        .filter_map(|control| match control {
                            Control::Table(t) if is_para_topbottom_float(&t.common) => {
                                Some(raw_table_ctrl_height_px(t, self.dpi).unwrap_or_else(|| {
                                    hwpunit_to_px(t.common.height as i32, self.dpi).max(0.0)
                                }))
                            }
                            _ => None,
                        })
                        .sum();
                    declared_stack_height > 0.0
                        && para_start_height + declared_stack_height <= bounds.0
                });
        let saved_host_line_after_stack_fits = saved_stack_fits_host_origin
            && has_preceding_coanchored_float
            && table_total <= available
            && !page_has_table_continuation;
        if std::env::var("RHWP_DIAG_2813").is_ok() {
            eprintln!(
                "DIAG_2813 pi={} ci={} float={} vis_text={} segs={} real_segs={} bounds={:?} cur_h={:.1} avail={:.1} verdict={}",
                para_idx,
                ctrl_idx,
                is_para_topbottom_float(&table.common),
                para_has_visible_text(para),
                para.line_segs.len(),
                para.line_segs.iter().filter(|ls| !is_synthetic_line_seg(ls)).count(),
                single_line_visible_bounds_px(para, st.vpos_page_base.unwrap_or(0), self.dpi),
                st.current_height,
                available,
                saved_host_line_after_stack_fits,
            );
        }
        // [#2439] 아래 orphan 가드가 새 페이지/단으로 이월한 뒤에도 원 페이지의
        // para_start_height 를 visible-float placement/exclusion 에 넘기면, 새 페이지의
        // 배타영역이 이전 페이지 시작 높이만큼 아래에서 시작한다. 후속 문단은 실제 표를
        // 건너뛰지 못하고 typeset/layout 좌표가 벌어져 본문 하단 overflow 로 이어진다.
        // 이월이 실제 발생한 경우에만 placement 기준을 fresh page-local current_height 로
        // 재설정한다. #1860 의 budget_para_start_height 는 별도 예산 계약이므로 불변이다.
        let mut placement_para_start_height = para_start_height;
        if is_para_topbottom_float(&table.common)
            && matches!(
                table.page_break,
                crate::model::table::TablePageBreak::RowBreak
            )
            && has_preceding_coanchored_float
            && !st.current_items.is_empty()
            && st.current_height + table_total > available
            && table_total <= available
            && !saved_host_line_after_stack_fits
        {
            // [#3674 진단] 통짜 이월 분기 발동 기록 — 동작 불변.
            if std::env::var("RHWP_DIAG_SPLITSCAN").is_ok() {
                eprintln!(
                    "DIAG_2439_DEFER pi={} cur_h={:.1} total={:.1} avail={:.1} coanchor={}",
                    para_idx,
                    st.current_height,
                    table_total,
                    available,
                    has_preceding_coanchored_float,
                );
            }
            if std::env::var("RHWP_DIAG_SPLITSCAN").is_ok() {
                eprintln!(
                    "DIAG_ADVB pi={} sec={} cur_h={:.1}",
                    para_idx, st.section_index, st.current_height
                );
            }
            st.advance_column_or_new_page();
            placement_para_start_height = st.current_height;
        }

        let single_row_object_declared_fits_current = !table.common.treat_as_char
            && table.row_count == 1
            && table.col_count == 1
            && table.cells.len() == 1
            && matches!(
                table.page_break,
                crate::model::table::TablePageBreak::RowBreak
            )
            && signed_hwpunit(table.common.vertical_offset) <= 0
            && !para.line_segs.is_empty()
            && !para_has_visible_text(para)
            && declared_object_total > 0.0
            && table_total > declared_object_total
            // [#3236] 선언 신뢰는 측정 초과가 폰트 대체 팽창으로 설명되는 범위까지만.
            // 실측 팽창은 인접 가드들 기준 10~20% 수준이라 1.5배를 넘는 초과는 셀
            // 내용이 진짜로 큰 것이다 — 한컴도 이 경우 쪽 경계에서 셀을 분할한다
            // (issue3236 fixture: 선언 322.6px vs 측정 910.8px(2.82배), 한컴 PDF 가
            // p2 로 셀 내용을 이어 배치). 상한 없이는 통짜 배치 후 쪽 밖 clip 으로
            // 내용이 소실된다.
            && table_total <= declared_object_total * SINGLE_ROW_DECLARED_TRUST_MAX_RATIO
            && st.current_height + declared_object_total <= available;

        let mut source_anchor_splits_here = false;
        if let Some(declared_total) = declared_empty_para_float_total {
            // 빈 host 문단의 자리차지 RowBreak 표는 렌더러가 문서에 저장된 표 선언
            // 높이를 하한으로 그린다. 저장 LineSeg가 있는 HWP5 문서는 이 선언 높이가
            // 원본 흐름 경계와 함께 쓰이므로 선언 기준으로 이월한다. LineSeg가 없는
            // HWP5-origin 계열은 셀 내용 측정 흐름을 우선하되, 측정치로는 fit 이지만
            // 선언 높이로는 현재 쪽 하단과 겹치는 경우만 선언 기준으로 이월한다.
            // 단, 1행 표의 저장 object height 는 현재 쪽 하단 tolerance 안에 맞고
            // cell 내용 측정치만 크게 나온 경우에는 한컴이 현재 쪽 하단까지 한 덩어리로
            // 배치하므로 아래 object-height fit 경로를 우선한다.
            const DECLARED_FLOAT_FIT_TOLERANCE_PX: f64 = 1.0;
            let has_internal_saved_vpos_reset = rowbreak_table_has_internal_saved_vpos_reset(table);
            let measured_fits_current = st.current_height + table_total <= available;
            let declared_overflows_current = st.current_height + declared_total > available;
            let measured_declared_excess = (table_total - declared_total).max(0.0);
            // [#5941] `anchor_delay <= measured_declared_excess` 는 둘 다 0 인 정상 형상에서
            // **부동소수점 1 ULP** 로 뒤집힌다. 실측(1130000-200900012 pi=1):
            //
            //     anchor = 42.93333333333333   cur_h = 42.93333333333334
            //     delay  = 7.105427357601002e-15   excess = 0.0   → 판정 false
            //
            // 저장 하단(881.39px)이 본문(895.73px)에 들어가는데도 표가 제 쪽으로 밀려
            // 2쪽 문서가 3쪽이 됐다(한/글 2쪽). 두 값 모두 HWPUNIT→px 나눗셈 산물이라
            // 비트 일치를 요구할 수 없다 — 픽셀 이하 오차 폭을 준다. 실제 지연(≥0.01px)
            // 은 종전대로 걸러진다.
            const ANCHOR_DELAY_FLOAT_EPS_PX: f64 = 1e-6;
            // 저장된 LineSeg와 객체 높이가 현재 쪽 본문 하단 안에 들어간다고 말하려면,
            // anchor 지연이 실제 measured excess로 설명되어야 한다.
            let saved_span = para
                .line_segs
                .iter()
                .find(|ls| !is_synthetic_line_seg(ls))
                .map(|seg| {
                    let base = st.vpos_page_base.unwrap_or(0);
                    let v_off = signed_hwpunit(table.common.vertical_offset);
                    // The stored LineSeg is the flow anchor. A positive table
                    // offset moves only the painted object top below that
                    // anchor; using the painted top as the fit anchor rejects
                    // source-owned body-top fragments by exactly that inset.
                    let anchor_hu = seg.vertical_pos.saturating_sub(base);
                    let top_hu = anchor_hu.saturating_add(v_off.max(0));
                    let bottom_hu =
                        top_hu.saturating_add(table.common.height.min(i32::MAX as u32) as i32);
                    (
                        hwpunit_to_px(anchor_hu, self.dpi),
                        hwpunit_to_px(top_hu, self.dpi),
                        hwpunit_to_px(bottom_hu, self.dpi),
                    )
                });
            let saved_object_bottom_fits_current =
                saved_span.is_some_and(|(anchor_px, _top_px, bottom_px)| {
                    let anchor_delay = (st.current_height - anchor_px).max(0.0);
                    anchor_px <= st.current_height
                        && anchor_delay <= measured_declared_excess + ANCHOR_DELAY_FLOAT_EPS_PX
                        && bottom_px <= available
                });
            // Native HWP5 can carry a complete multi-row RowBreak object frame
            // at the end of a stored page even when sequential host-spacing
            // accounting has drifted a few pixels past its saved anchor.  If the
            // object itself still ends inside the body and the next host rewinds,
            // preserve that source frame for the whole-fit path below instead of
            // deferring the table before it can resynchronize to the saved top.
            let native_hwp5_saved_rowbreak_object_frame_fits = st.profile.native_hwp5_layout()
                && !table.common.treat_as_char
                && is_para_topbottom_float(&table.common)
                && matches!(
                    table.page_break,
                    crate::model::table::TablePageBreak::RowBreak
                )
                && table.row_count > 1
                && para.controls.len() == 1
                && !para_has_visible_text(para)
                && ft.table_footnotes.is_empty()
                && signed_hwpunit(table.common.vertical_offset) <= 0
                && next_rewinds_after_table
                && !has_internal_saved_vpos_reset
                && saved_span.is_some_and(|(_anchor_px, top_px, bottom_px)| {
                    native_hwp5_saved_rowbreak_tail_frame_matches(
                        top_px,
                        bottom_px,
                        st.current_height,
                        available,
                    )
                });
            // [#2097] 저장 앵커가 현재 흐름 위치와 정합하는데 저장 하단이 쪽 본문을
            // 넘으면, 원본 한글 레이아웃은 이월이 아니라 이 지점에서 표를 분할했다
            // (2572521 pi36: 앵커 11000HU=146.7px == cur_h, 선언 839.8px 로 하단
            // 986px 초과 — 저장 p3 만충 914.7px 실측, 이월 시 7쪽으로 +1). 이
            // 형상은 선언-기준 이월을 건너뛰고 분할 경로로 보낸다.
            let saved_anchor_overlaps_current_flow = para
                .line_segs
                .iter()
                .find(|seg| !is_synthetic_line_seg(seg))
                .and_then(|seg| {
                    line_seg_visible_bounds_px(seg, st.vpos_page_base.unwrap_or(0), self.dpi)
                })
                .is_some_and(|bounds| saved_bounds_overlap_current_flow(bounds, st.current_height));
            let saved_anchor_splits_here = st.has_stored_line_segs
                // HWPX stores the source anchor as the physical fragment
                // owner. Native HWP needs the visible-bounds check because a
                // positive paint inset can put its object below that anchor.
                && (st.profile.hwpx_stored_layout() || saved_anchor_overlaps_current_flow)
                && saved_span.is_some_and(|(_anchor_px, _top_px, bottom_px)| {
                    bottom_px > available
                });
            source_anchor_splits_here = saved_anchor_splits_here;
            // [#3820 Stage 7] 표 44(pi=1778)는 앞선 표의 row-internal tail 뒤에서
            // host anchor가 흐름보다 19.1px 앞선다. 저장된 object bottom 자체는
            // 현재 body 안에 있지만, 일반 declared-height defer gate가 그 19.1px을
            // "이미 지나간 anchor"로 보고 표 전체를 다음 page로 보낸다. 한컴은 이
            // 경우 첫 fragment를 현 page에 둔 뒤 다음 page에서 이어 그린다.
            //
            // 일반 anchor tolerance를 넓히면 page-tail float를 통째로 남기는
            // document-wide 회귀가 생길 수 있다. native HWP5, non-TAC, paragraph
            // TopAndBottom, RowBreak, 다행 표, table-footnote 없음, 다음 source
            // paragraph의 vpos rewind라는 저장 계약을 모두 만족하고, saved object
            // bottom이 body 안에 드는 경우에만 declared defer를 건너뛰어 아래
            // fragment scan에 맡긴다. ordinary-row 표는 #4763처럼 delayed flow가
            // measured growth로 전부 설명될 때만 허용한다. rowspan 표는 cell 내부
            // 저장 reset이 있어 scanner가 블록 안의 실제 hard-break를 제시할 때만
            // 24px의 fragment-local anchor drift를 허용한다.
            const NATIVE_HWP5_NEAR_ANCHOR_ROWBREAK_FRAGMENT_TOLERANCE_PX: f64 = 24.0;
            let has_rowspan = table.cells.iter().any(|cell| cell.row_span > 1);
            let native_hwp5_near_anchor_rowbreak_needs_fragment_scan =
                st.profile.hwp5_stored_pagination_layout()
                    && !table.common.treat_as_char
                    && is_para_topbottom_float(&table.common)
                    && matches!(
                        table.page_break,
                        crate::model::table::TablePageBreak::RowBreak
                    )
                    && table.row_count > 1
                    && ft.table_footnotes.is_empty()
                    && (!has_rowspan || has_internal_saved_vpos_reset)
                    && next_rewinds_after_table
                    && saved_span.is_some_and(|(_anchor_px, top_px, bottom_px)| {
                        let flow_overrun = st.current_height - top_px;
                        let measured_excess = (table_total - declared_total).max(0.0);
                        let allowed_flow_overrun = if has_rowspan {
                            NATIVE_HWP5_NEAR_ANCHOR_ROWBREAK_FRAGMENT_TOLERANCE_PX
                        } else {
                            measured_excess
                        };
                        let bottom_tolerance = if has_rowspan {
                            DECLARED_FLOAT_FIT_TOLERANCE_PX
                        } else {
                            0.0
                        };
                        top_px <= st.current_height
                            && flow_overrun <= allowed_flow_overrun
                            && bottom_px <= available + bottom_tolerance
                    });
            // native HWP의 저장 object는 host LineSeg의 시작보다 약간 뒤에서 paint될 수
            // 있다. 현재 flow가 그 host line 안에 있고 object top도 같은 line 안에 있으며,
            // object bottom과 다음 source reset이 현재 물리 page를 증명하면 declared
            // overrun으로 표 전체를 이월하지 않는다. 이 경우 RowBreak scanner가 저장
            // frame의 실제 행 prefix를 현재 page owner로 확정한다.
            let native_hwp5_anchor_line_rowbreak_needs_fragment_scan =
                st.profile.hwp5_stored_pagination_layout()
                    && !table.common.treat_as_char
                    && is_para_topbottom_float(&table.common)
                    && matches!(
                        table.page_break,
                        crate::model::table::TablePageBreak::RowBreak
                    )
                    && !para_has_visible_text(para)
                    && table.row_count > 1
                    && ft.table_footnotes.is_empty()
                    && next_rewinds_after_table
                    && !rowbreak_table_has_internal_saved_vpos_reset(table)
                    && saved_span.is_some_and(|(_anchor_px, top_px, bottom_px)| {
                        bottom_px <= available
                            && para
                                .line_segs
                                .iter()
                                .find(|seg| !is_synthetic_line_seg(seg))
                                .and_then(|seg| {
                                    line_seg_visible_bounds_px(
                                        seg,
                                        st.vpos_page_base.unwrap_or(0),
                                        self.dpi,
                                    )
                                })
                                .is_some_and(|(line_top, line_bottom)| {
                                    saved_bounds_overlap_current_flow(
                                        (line_top, line_bottom),
                                        st.current_height,
                                    ) && st.current_height <= top_px
                                        && top_px <= line_bottom
                                })
                    });
            // [#3820 Stage 11] 1×1 빈-host RowBreak 표도 cell 안의 저장 vpos reset이
            // 있으면, 선언 common.height는 첫 physical fragment의 높이이고 실제 cell
            // 측정치는 다음 쪽 tail까지 합친 값이다. 앞선 out-of-flow 표의 측정 팽창이
            // current_height에 누적되면 이 표의 저장 anchor보다 아래로 흘러 declared
            // defer가 표 전체를 다음 쪽에 보내 버린다. 기준 PDF p172의 `<BTS>` 표처럼
            // reset 전 prefix와 기존 각주 사이의 fragment가 사라지고 후속 전체가 +1쪽
            // 밀리는 결과다.
            //
            // 일반 1×1 표의 anchor를 되감으면 float overlap을 만들 수 있으므로, native
            // HWP5·빈 host·비-TAC·TopAndBottom·RowBreak, cell 내부 reset, 후속 source
            // rewind, 기존 각주라는 저장 계약을 모두 요구한다. 또한 저장 객체 하단이
            // 실제 footnote boundary 안에 있고, 현재 flow의 초과분이 해당 표의
            // `measured - declared` 팽창으로 설명될 때만 anchor를 복원한다. 이 경우에만
            // 첫 fragment scan은 reset 전 cell tail을 현 페이지에 남길 수 있다.
            native_hwp5_internal_reset_rewind_needs_anchor_resync =
                st.profile.hwp5_stored_pagination_layout()
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
                    && ft.table_footnotes.is_empty()
                    && st.current_footnote_height > 0.0
                    && has_internal_saved_vpos_reset
                    && next_rewinds_after_table
                    && saved_span.is_some_and(|(_anchor_px, top_px, bottom_px)| {
                        let flow_overrun = st.current_height - top_px;
                        let measured_excess = (table_total - declared_total).max(0.0);
                        top_px <= st.current_height
                            && top_px <= available
                            && bottom_px <= available
                            && flow_overrun <= measured_excess
                    });
            // [#3931 Stage 2] 다행 RowBreak 표도 cell 문단 경계의 저장 vpos reset과
            // 후속 source 문단의 되감김이 함께 있으면, 선언 common.height는 첫
            // physical fragment의 span이고 측정 table_total은 다음 쪽 tail까지 합친
            // 높이다. 빈 host 줄과 그 spacing을 flow가 먼저 소비한 경우 current_height가
            // 저장 anchor보다 조금 아래로 밀려 declared defer가 12+4 저장 분할을
            // 통째 이월로 바꾼다.
            //
            // 일반 anchor tolerance는 넓히지 않는다. native HWP5·빈 host·비-TAC·
            // TopAndBottom·다행 RowBreak·표/현재 쪽 각주 없음·내부 reset·후속 rewind를
            // 모두 요구하고, 저장 object 하단이 실제 body 안에 있으며 flow 초과분
            // 전부가 host line + host spacing 소비로 설명될 때만 anchor를 복원한다.
            // 저장 span이 현재 flow와 이미 맞거나 declared가 넘치지 않는 경우에는
            // 동작하지 않는다.
            native_hwp5_multirow_internal_reset_needs_anchor_resync =
                st.profile.hwp5_stored_pagination_layout()
                    && !table.common.treat_as_char
                    && is_para_topbottom_float(&table.common)
                    && matches!(
                        table.page_break,
                        crate::model::table::TablePageBreak::RowBreak
                    )
                    && !para_has_visible_text(para)
                    && table.row_count > 1
                    && ft.table_footnotes.is_empty()
                    && st.current_footnote_height <= 0.5
                    && declared_overflows_current
                    && !saved_object_bottom_fits_current
                    && has_internal_saved_vpos_reset
                    && next_rewinds_after_table
                    && saved_span.is_some_and(|(anchor_px, _top_px, bottom_px)| {
                        // #4763 이후 saved span은 flow anchor와 paint top을 따로
                        // 보존한다. 다행 표 재동기화는 vertical offset이 적용되기
                        // 전의 source flow anchor를 사용한다.
                        let vertical_offset_px = hwpunit_to_px(
                            signed_hwpunit(table.common.vertical_offset).max(0),
                            self.dpi,
                        );
                        let flow_overrun = st.current_height - anchor_px;
                        let host_consumption =
                            fmt.total_height + host_spacing_total + vertical_offset_px;
                        anchor_px < st.current_height
                            && bottom_px <= available + DECLARED_FLOAT_FIT_TOLERANCE_PX
                            && flow_overrun <= host_consumption + DECLARED_FLOAT_FIT_TOLERANCE_PX
                    });
            if native_hwp5_internal_reset_rewind_needs_anchor_resync
                || native_hwp5_multirow_internal_reset_needs_anchor_resync
            {
                let (anchor_px, top_px, _) =
                    saved_span.expect("resync requires stored table anchor");
                let flow_top = if native_hwp5_multirow_internal_reset_needs_anchor_resync {
                    anchor_px
                } else {
                    top_px
                };
                st.align_flow_to(flow_top);
                placement_para_start_height = flow_top;
            }
            // [#3820 Stage 11] native HWP5의 빈-host 1×1 RowBreak 표가 자체 각주를
            // 여러 개 갖고 실제 셀 내용이 선언 높이보다 크게 자랐을 때, 첫 fragment에
            // 포함되지 않을 **자체 각주 전체**를 먼저 예약하면 선언 높이가 현재 물리
            // 본문에는 들어가도 표 전체가 다음 쪽으로 defer된다. p174 표 46은 이
            // 경우다: p174에는 표 prefix가, p175에는 continuation과 223~231 각주가
            // 있어야 한다. 이후의 `table_fn_reserved` 재스캔은 첫 fragment에 footnote
            // anchor가 없음을 확인한 경우에만 예약을 풀어 실제 첫 조각 경계를 정한다.
            // 따라서 이 분기는 그 안전한 재스캔 경로에 *진입*시키는 역할만 한다.
            let native_hwp5_own_footnote_fragment_can_start_before_reservation =
                st.profile.hwp5_stored_pagination_layout()
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
                    && ft.table_footnotes.len() >= 2
                    && st.current_footnote_height <= 0.5
                    && table_total > declared_total * SINGLE_ROW_DECLARED_TRUST_MAX_RATIO
                    && st.current_height > 0.5
                    && st.current_height + declared_total
                        <= st.base_available_height()
                            - st.current_zone_y_offset
                            - st.current_bottom_fixed_exclusion;
            if std::env::var("RHWP_DIAG_SCAN").is_ok() {
                eprintln!(
                    "DIAG_SCAN DECL_DEFER? pi={} cur_h={:.1} declared={:.1} avail={:.1} host_h={:.1} host_before={:.1} v_off={:.1} outer_top={:.1} saved={:?} bottom_fits={} splits_here={} near_anchor_fragment={} internal_reset={} next_rewind={} internal_reset_resync={} multirow_internal_reset_resync={} own_fn_fragment={}",
                    para_idx,
                    st.current_height,
                    declared_total,
                    available,
                    fmt.total_height,
                    ft.host_spacing.before,
                    hwpunit_to_px(
                        signed_hwpunit(table.common.vertical_offset).max(0),
                        self.dpi,
                    ),
                    hwpunit_to_px(table.outer_margin_top as i32, self.dpi),
                    saved_span,
                    saved_object_bottom_fits_current,
                    saved_anchor_splits_here,
                    native_hwp5_near_anchor_rowbreak_needs_fragment_scan,
                    has_internal_saved_vpos_reset,
                    next_rewinds_after_table,
                    native_hwp5_internal_reset_rewind_needs_anchor_resync,
                    native_hwp5_multirow_internal_reset_needs_anchor_resync,
                    native_hwp5_own_footnote_fragment_can_start_before_reservation,
                );
            }
            // [#2279 5축] 선언-이월의 저장 증거는 **host 문단 단위**(saved_span)로
            // 판정한다. 종전 구역 전역 st.has_stored_line_segs 는 구역 내 다른
            // 문단의 LS 만으로 no-LS host 의 RowBreak float 까지 통째 이월시켰다
            // — 한글은 이 형상(86712 pi=30: 4×3 RowBreak, saved=None, 측정 비적합
            // 980.8>971.3)을 행 분할해 현재 쪽에 머리 행들을 남긴다(p10/p11).
            // 위 주석의 원 의도("LS 없는 계열은 측정 fit 일 때만 선언 이월")와 정합.
            // native HWP의 1x1 RowBreak 표에서 실제 셀 본문이 declared object
            // height의 신뢰 상한을 넘으면, declared height로 통째 이월할 수 없다.
            // 이 형상은 현재 쪽에서 cell-unit fragment scan을 시작해야 하며, 그렇지
            // 않으면 p4처럼 첫 fragment 전체가 불필요하게 다음 쪽으로 밀린다.
            let native_hwp5_large_single_cell_rowbreak_needs_fragment_scan =
                st.profile.hwp5_stored_pagination_layout()
                    && !table.common.treat_as_char
                    && table.row_count == 1
                    && table.col_count == 1
                    && table.cells.len() == 1
                    && matches!(
                        table.page_break,
                        crate::model::table::TablePageBreak::RowBreak
                    )
                    && !para_has_visible_text(para)
                    && declared_object_total > 0.0
                    && table_total > declared_object_total * SINGLE_ROW_DECLARED_TRUST_MAX_RATIO;
            // 빈 host의 native HWP5 RowBreak 표가 셀 안에 명시적 저장 frame
            // reset을 가지면, declared object bottom만으로 통째 이월할 수 없다.
            // row-cut scanner가 source-owned frame prefix를 확정해야 한다.
            let native_hwp5_stored_rowbreak_needs_fragment_scan =
                st.profile.hwp5_stored_pagination_layout()
                    && !table.common.treat_as_char
                    && matches!(
                        table.page_break,
                        crate::model::table::TablePageBreak::RowBreak
                    )
                    && !para_has_visible_text(para)
                    && rowbreak_table_has_internal_saved_vpos_reset(table);
            if !st.current_items.is_empty()
                && !ft.strict_following_plain_text_fit
                && declared_overflows_current
                && !saved_object_bottom_fits_current
                && !saved_anchor_splits_here
                && !native_hwp5_near_anchor_rowbreak_needs_fragment_scan
                && !native_hwp5_anchor_line_rowbreak_needs_fragment_scan
                && !native_hwp5_internal_reset_rewind_needs_anchor_resync
                && !native_hwp5_multirow_internal_reset_needs_anchor_resync
                && !native_hwp5_own_footnote_fragment_can_start_before_reservation
                && !saved_host_line_after_stack_fits
                && !native_hwp5_saved_rowbreak_object_frame_fits
                && !single_row_object_declared_fits_current
                && !native_hwp5_large_single_cell_rowbreak_needs_fragment_scan
                && !native_hwp5_stored_rowbreak_needs_fragment_scan
                && (saved_span.is_some() || measured_fits_current)
                && declared_total <= available
                // [편집 세션] 이 통째-이월(keep-together)의 근거(saved_span:
                // 저장에서 한 쪽에 있었음)는 편집으로 앞 내용이 밀리면 낡는다 —
                // 한글은 RowBreak(행 경계 나눔) 표를 잔여에 행 단위로 채우고
                // 넘친 행만 다음 쪽에 둔다(셀 Enter 재현 오라클: 마지막 행만
                // 다음 쪽). 분할 스캐너가 경계를 정하게 한다.
                && !(self.profile.get().session_edited()
                    && matches!(
                        table.page_break,
                        crate::model::table::TablePageBreak::RowBreak
                    ))
            {
                if std::env::var("RHWP_DIAG_SPLITSCAN").is_ok() {
                    eprintln!(
                        "DIAG_ADVC pi={} sec={} cur_h={:.1}",
                        para_idx, st.section_index, st.current_height
                    );
                }
                st.advance_column_or_new_page();
                reserve_declared_table_total = true;
            }
        }
        if reserve_declared_table_total {
            if let Some(declared_total) = declared_empty_para_float_total {
                table_total = table_total.max(declared_total);
            }
        }

        let super::whole_fit::WholeFit {
            para_has_stored_line_seg,
            single_row_object_height_advance,
            fits_after_overlay_shapes,
            native_hwp5_rewinding_rowbreak_uses_painted_row_footprint,
            whole_fit_table_total,
            hwpx_noninline_tac_measured_fit,
            declared_table_whole_fits,
            saved_table_source_frame,
        } = self.query_whole_table_fit(
            st,
            input,
            super::whole_fit::WholeFitInput {
                next_starts_new_page,
                next_rewinds_after_table,
                host_spacing_total,
                table_total,
                available,
                declared_object_total,
                single_row_object_declared_fits_current,
            },
        );
        if let Some((source_top, _)) = saved_table_source_frame {
            st.align_flow_to(source_top);
            placement_para_start_height = source_top;
        }
        if host_line_trails_float_stack {
            // [#2813] 앵커 줄 아이템을 float 스택 뒤로 이연(한글 문서순) —
            // 스택 첫 표 배치 전에 걸려야 렌더 순서가 표→줄로 나온다.
            st.defer_host_line(Some(para_idx));
        }
        let whole_placement_height =
            if let Some((source_top, source_bottom)) = saved_table_source_frame {
                source_bottom - source_top
            } else if let Some(advance) = single_row_object_height_advance {
                advance
            } else if is_para_topbottom_float(&table.common)
                && (para_has_non_whitespace_text(para) || hwpx_noninline_tac_measured_fit)
            {
                ft.effective_height
            } else {
                table_total
            };
        let unconstrained_host_placement = para_has_non_whitespace_text(para)
            .then(|| {
                let text_origin = placement_para_start_height
                    + if placement_para_start_height > 0.0 {
                        fmt.spacing_before
                    } else {
                        0.0
                    };
                if let Some(lines) = &fmt.computed_host_lines {
                    crate::renderer::float_placement::ParagraphFloatPlacement::from_computed_host(
                        para,
                        table,
                        ctrl_idx,
                        text_origin,
                        lines,
                        whole_placement_height,
                        self.dpi,
                    )
                } else {
                    crate::renderer::float_placement::ParagraphFloatPlacement::from_stored_host(
                        para,
                        table,
                        ctrl_idx,
                        text_origin,
                        whole_placement_height,
                        self.dpi,
                    )
                }
            })
            .flatten()
            .map(|placement| {
                let placement =
                    placement.with_tail_line_space(fmt.tail_line_remaining_width, table, self.dpi);
                // A stored ladder must have a known origin in THIS column, not
                // a default zero or a base recovered from already painted nodes.
                let frame = (para.line_segs.len() == 1
                    && para.controls.len() == 1
                    && st.col_count == 1
                    && !st.vpos_ladder_dirty
                    && !st.profile.session_edited()
                    && fmt.computed_host_lines.is_none())
                .then(|| {
                    let PageItem::FullParagraph { para_index: first } = st.current_items.first()?
                    else {
                        return None;
                    };
                    let chain = paragraphs_all.get(*first..=para_idx)?;
                    let mut previous = None;
                    for host in chain {
                        if host.stored_text_partition_is_dirty() || host.line_segs.is_empty() {
                            return None;
                        }
                        for line in &host.line_segs {
                            if is_synthetic_line_seg(line)
                                || previous.is_some_and(|vpos| line.vertical_pos < vpos)
                            {
                                return None;
                            }
                            previous = Some(line.vertical_pos);
                        }
                    }
                    // A continuation has no full paragraph origin in this frame.
                    if st.current_items.iter().any(|item| {
                        matches!(item, PageItem::PartialTable { .. } | PageItem::Shape { .. })
                    }) {
                        return None;
                    }
                    Some(chain.first()?.line_segs.first()?.vertical_pos)
                })
                .flatten();
                match (frame, paragraphs_all.get(para_idx + 1)) {
                    (Some(base), Some(next)) => placement.with_stored_band_origin(
                        para,
                        next,
                        base,
                        st.vpos_col_anchor,
                        self.dpi,
                    ),
                    _ => placement,
                }
            });
        let constrain_host_placement = HostPlacementConstraint {
            table,
            has_preceding_coanchored_float,
            dpi: self.dpi,
        };
        let resolved_host_placement =
            unconstrained_host_placement.map(|p| constrain_host_placement.constrain(p, st));
        let legacy_whole_fits = st.current_height + whole_fit_table_total <= available
            || fits_after_overlay_shapes
            || single_row_object_height_advance.is_some()
            || declared_table_whole_fits
            || saved_host_line_after_stack_fits
            || saved_table_source_frame.is_some();
        // 예약 구간의 하단으로 fit을 판정한다. current_height는 앵커 줄이
        // 아니므로 여기에 표 높이만 더하면 뒤 줄의 앵커 거리가 예산에서 빠진다.
        if resolved_host_placement.map_or(legacy_whole_fits, |p| p.occupied_bottom <= available) {
            if let Some(placement) = resolved_host_placement {
                st.record_paragraph_float_placement((para_idx, ctrl_idx), placement);
            }
            // [#3674 진단] fit 분기 발동 사유 — 동작 불변.
            if std::env::var("RHWP_DIAG_SPLITSCAN").is_ok() {
                eprintln!(
                    "DIAG_FIT pi={} sec={} plain={} overlay={} single={} declared={} saved={} cur_h={:.1} total={:.1} avail={:.1}",
                    para_idx, st.section_index,
                    st.current_height + whole_fit_table_total <= available,
                    fits_after_overlay_shapes,
                    single_row_object_height_advance.is_some(),
                    declared_table_whole_fits,
                    saved_host_line_after_stack_fits,
                    st.current_height, whole_fit_table_total, available,
                );
            }
            self.place_table_with_text(
                st,
                para_idx,
                ctrl_idx,
                para,
                table,
                fmt,
                placement_para_start_height,
                whole_placement_height,
                is_first_placed,
                is_last_placed,
                ft.strict_following_plain_text_fit,
                styles,
            );
            return None;
        }

        // [Task #991] 1행짜리 글자처럼취급(treat_as_char) 표는 페이지 경계에서
        // 분할하지 않고 통째로 다음 페이지/단으로 이동한다.
        //
        // 표 분할은 행 경계 분할이 기본이고, 행 경계가 없는 1행 표는 셀 내용을
        // 페이지 중간에서 자르는 인트라-셀 분할만 가능하다. 글자처럼취급 표는
        // 본문 흐름 안의 한 글자 같은 인라인 개체이므로 인트라-셀 분할은 부적절하다
        // (한컴은 통째로 다음 페이지로 넘김). 다행(多行) tac 표는 행 경계 분할이
        // 가능하므로 기존 로직을 유지하고, 1행 tac 표만 통째 이동시킨다.
        // 한 페이지에도 안 들어가는 초대형 표는 분할 외 방법이 없으므로 폴백한다.
        if table.common.treat_as_char && table.row_count <= 1 && table_total <= available {
            if !st.current_items.is_empty() {
                st.advance_column_or_new_page();
            }
            self.place_table_with_text(
                st,
                para_idx,
                ctrl_idx,
                para,
                table,
                fmt,
                para_start_height,
                table_total,
                is_first_placed,
                is_last_placed,
                ft.strict_following_plain_text_fit,
                styles,
            );
            return None;
        }

        // MeasuredTable이 없거나 행이 없으면 강제 배치
        let mt = match mt {
            Some(m) if !m.row_heights.is_empty() => m,
            _ => {
                if !st.current_items.is_empty() {
                    st.advance_column_or_new_page();
                }
                st.append_item(PageItem::Table {
                    para_index: para_idx,
                    control_index: ctrl_idx,
                });
                st.advance_flow_by(if ft.strict_following_plain_text_fit {
                    ft.total_height
                } else {
                    ft.effective_height
                });
                if ft.strict_following_plain_text_fit && is_last_placed {
                    st.require_strict_following_text_fit();
                }
                return None;
            }
        };
        // HWPX의 빈 1×1 wrapper는 측정기와 renderer가 모두 내부 표를 행 기하로
        // 사용한다. native HWP5에서 바깥 clip/frame 소유가 필요한 경우도 실제
        // 1×1 RowBreak wrapper뿐이다. 모든 native 표에 바깥 행 기하를 적용하면
        // 일반 중첩 표의 행 cursor가 달라져 59043 pagination이 39 -> 41쪽으로
        // 회귀한다. 따라서 physical continuation 계약이 있는 wrapper로만 좁힌다.
        let effective_row_geometry_table = row_geometry_table(table);
        let hwp5_single_cell_rowbreak_wrapper = (st.profile.hwp5_stored_pagination_layout()
            || st.profile.hwp5_origin_hwpx())
            && table.row_count == 1
            && table.col_count == 1
            && matches!(
                table.page_break,
                crate::model::table::TablePageBreak::RowBreak
            )
            && effective_row_geometry_table.row_count == 1
            && effective_row_geometry_table.col_count == 1;
        let row_geometry_table = if hwp5_single_cell_rowbreak_wrapper {
            table
        } else {
            effective_row_geometry_table
        };

        let declared_table_height = (declared_object_total - host_spacing_total).max(0.0);
        let declared_table_does_not_fit_remaining =
            declared_table_height > 0.0 && st.current_height + declared_table_height > available;
        let single_cell_uses_table_padding_center = table.cells.first().is_some_and(|cell| {
            !cell.apply_inner_margin
                && matches!(
                    cell.vertical_align,
                    crate::model::table::VerticalAlign::Center
                )
        });
        if !table.common.treat_as_char
            && table.row_count == 1
            && matches!(
                table.page_break,
                crate::model::table::TablePageBreak::RowBreak
            )
            && !para_has_stored_line_seg
            && !para_has_visible_text(para)
            && single_cell_uses_table_padding_center
            && declared_table_does_not_fit_remaining
            && !st.current_items.is_empty()
            && st.current_height + table_total > available
            && table_total <= available
        {
            st.advance_column_or_new_page();
            self.place_table_with_text(
                st,
                para_idx,
                ctrl_idx,
                para,
                table,
                fmt,
                para_start_height,
                table_total,
                is_first_placed,
                is_last_placed,
                ft.strict_following_plain_text_fit,
                styles,
            );
            return None;
        }

        // [#2097] 쪽나눔=None 표는 fresh 쪽보다 커도 행 분할하지 않는다 — 한글은
        // 통째 배치 후 본문 아래(꼬리말·하단 여백)로 오버플로한다 (3023771 위촉장:
        // 선언=실측 1005px > 본문 933.5px 인 4x2/3x1 표 2건, 한글 PDF 각 1쪽 통째
        // + 하단 오버플로 실측 — rhwp 는 3조각 분할로 2→6쪽). 오버플로가 본문 하단
        // 아래 물리 슬랙(용지 경계)을 넘는 극단 형상은 미관측이라 기존 분할 폴백을
        // 유지한다(보수 가드). 판정은 host 스페이싱을 뺀 순수 표 높이 — 스페이싱
        // 포함 판정은 fresh 쪽에 들어가는 표(1220000-201800008: 표 920.8px ≤ 본문
        // 933.5px, 스페이싱 포함 934.4px)를 쪽-초과로 오판해 기존 분할(2쪽)을
        // 통째+후행 문단 밀림(3쪽)으로 회귀시킨다.
        let below_body_slack =
            (st.layout.page_height - (st.layout.body_area.y + st.layout.body_area.height)).max(0.0);
        let table_only_height = (table_total - host_spacing_total).max(0.0);
        if matches!(table.page_break, crate::model::table::TablePageBreak::None)
            && table_only_height > st.base_available_height()
            && table_only_height <= st.base_available_height() + below_body_slack
        {
            if !st.current_items.is_empty() {
                st.advance_column_or_new_page();
            }
            self.place_table_with_text(
                st,
                para_idx,
                ctrl_idx,
                para,
                table,
                fmt,
                para_start_height,
                table_total,
                is_first_placed,
                is_last_placed,
                ft.strict_following_plain_text_fit,
                styles,
            );
            return None;
        }

        Some(SplitTableEntry {
            total_footnote,
            next_starts_new_page,
            next_rewinds_after_table,
            host_spacing_total,
            table_total,
            native_ordinary_rowbreak_rewind_uses_actual_footnote_boundary,
            fn_margin,
            available,
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
        })
    }
}
