//! 구역 문단 처리의 finish_paragraph_anchor_state 단계. 조건·예약·발행 순서를 유지한다.
use crate::renderer::typeset::{
    compute_body_wide_top_reserve_for_para, ComposedParagraph, Control, DeferredTableFlushPoint,
    Issue2424TypesetProfile, MeasuredTable, PageDef, PageItem, Paragraph, ResolvedStyleSet,
    TypesetEngine, TypesetState,
};
impl TypesetEngine {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn finish_paragraph_anchor_state(
        &self,
        st: &mut TypesetState,
        composed: &[ComposedParagraph],
        styles: &ResolvedStyleSet,
        measured_tables: &[MeasuredTable],
        page_def: &PageDef,
        para_idx: usize,
        para: &Paragraph,
        paragraphs: &[Paragraph],
        has_table: bool,
        issue2424_ts_enabled: bool,
        issue2424_prof: &mut Issue2424TypesetProfile,
    ) {
        // [Task #1027 Stage D] 항목 배치 후 vpos 커서 prev/base 추적 (렌더러 정합).
        // 렌더러 build_single_column: 매 항목 후 prev_layout_para 갱신, 표/Shape/
        // PartialTable 배치 후 page/lazy base 무효화(LINE_SEG lh 가 개체 높이를
        // 반영 못 해 drift 유발 → 직후 paragraph 는 lazy 역산으로 재산출). 단단 전용.
        if st.col_count == 1 {
            if st.reset_vpos_after_queued_table_footnote_page {
                // RowBreak 표가 남은 cell-footnote만 든 fresh page를 만든 경우,
                // 표 host의 저장 VPOS를 이 새 page 본문에 상속하면 빈 첫 문단도
                // 표 높이만큼 forward-snap 된다. 새 page cursor를 보존하고 다음
                // 본문 문단이 자체 stored VPOS에서 시작하게 한다.
                st.reset_vpos_cursor();
                st.acknowledge_queued_footnote_reset(false);
            } else {
                st.record_previous_layout_paragraph(Some(para_idx));
                // [#2243] 저장-앵커 사다리 dirty 태깅: 저장 lineseg 없는 문단(생성기
                // 누락)은 한글이 fresh 재계산으로 키울 수 있어, 그 뒤 기계 사다리
                // v0 로의 **역스냅**은 성장분을 뭉갠다 (36398599: p33~36 누락 구간
                // fresh +4320HU → 한글 4쪽 vs 역스냅 3쪽). dirty 이후 스냅은 전방만
                // 허용한다. 합성-앵커(전면 NO_LS 문서)는 자기정합이므로 무관.
                if st.vpos_page_base_stored
                    && paragraphs.get(para_idx).is_some_and(|p| {
                        p.line_segs.first().map_or(true, |s| {
                            s.tag & crate::model::paragraph::LineSeg::TAG_IMPLEMENTATION_PROPERTY
                                != 0
                        })
                    })
                {
                    st.mark_vpos_ladder_dirty();
                }
                st.record_previous_partial_table(matches!(
                    st.current_items.last(),
                    Some(PageItem::PartialTable { .. })
                ));
                let last = st.current_items.last();
                if matches!(
                    last,
                    Some(
                        PageItem::Table { .. }
                            | PageItem::PartialTable { .. }
                            | PageItem::Shape { .. }
                    )
                ) {
                    // [#2243] 예외: 표 호스트의 **저장** lineseg lh 가 개체 높이를 온전히
                    // 포함(기계생성 결재문서: lh = 표 + outMargin)하면 저장 사다리가 표
                    // 라인을 관통해 연속이므로 base 를 유지한다. 무효화하면 후속 문단
                    // 스냅이 드리프트를 역산한 lazy base 로 고착 (36395325 p2 +10.7px,
                    // p4 +16.6px 팬텀 → sliver 쪽 +2). lh 가 개체를 못 담는 일반
                    // 케이스는 종전대로 무효화. HWPX(기계생성 결재문서 계열) 한정 —
                    // HWP5 native 는 종전 핀 보존 (issue_1418 2026_oss_rst 6쪽).
                    let host_line_covers_object = st.profile.hwpx_stored_layout()
                        && matches!(last, Some(PageItem::Table { .. }))
                        && para.line_segs.first().is_some_and(|s| {
                            s.tag & crate::model::paragraph::LineSeg::TAG_IMPLEMENTATION_PROPERTY
                                == 0
                        })
                        && {
                            // lh 는 표 + outMargin×2 까지 담아야 사다리 연속이 보장된다
                            // (lh = 표높이만인 기계 사다리는 om 만큼 역스냅 과소 —
                            // 36398599 -1쪽 회귀 차단).
                            let max_tbl_h = para
                                .controls
                                .iter()
                                .filter_map(|c| match c {
                                    Control::Table(t) => Some(
                                        t.common.height as i64
                                            + t.outer_margin_top as i64
                                            + t.outer_margin_bottom as i64,
                                    ),
                                    _ => None,
                                })
                                .max()
                                .unwrap_or(i64::MAX);
                            para.line_segs
                                .iter()
                                .map(|s| s.line_height as i64)
                                .max()
                                .unwrap_or(0)
                                >= max_tbl_h
                        };
                    if !host_line_covers_object {
                        // Para-float TopAndBottom 표 예외(렌더러 2513)는 Stage E.
                        st.record_vpos_page_origin(None);
                        st.record_vpos_lazy_origin(None);
                    }
                }
            }
        }

        // [Task #362] Square wrap 표 처리 후 wrap zone 활성화.
        // Paginator engine.rs:356-372 동일 시멘틱.
        // 후속 paragraph 가 동일 cs/sw 를 가지면 흡수.
        if has_table {
            let has_tac_block = para
                .controls
                .iter()
                .any(|c| matches!(c, Control::Table(t) if t.common.treat_as_char));
            let has_non_tac_table = !has_tac_block;
            if has_non_tac_table {
                let is_wrap_around = para.controls.iter().any(|c| {
                    if let Control::Table(t) = c {
                        matches!(t.common.text_wrap, crate::model::shape::TextWrap::Square)
                    } else {
                        false
                    }
                });
                if is_wrap_around {
                    // [Task #1745] 텍스트 혼합 anchor(첫 LINE_SEG 가 전폭 텍스트 줄)면
                    // anchor LINE_SEG 가 wrap 띠를 인코딩하지 않으므로 표 geometry 로
                    // 띠 (cs, sw) 를 도출해 후속 문단 매칭에 사용 (한글: 후속 문단을
                    // 표 옆 잔여 띠에 배치 — samples/task1745).
                    if let Some((strip_cs, strip_sw)) =
                        crate::renderer::text_anchor_square_table_strip(para).or_else(|| {
                            crate::renderer::empty_host_square_table_left_strip(
                                para,
                                st.layout.column_width_hu(),
                            )
                        })
                    {
                        st.arm_control_wrap(strip_cs, strip_sw, para_idx, false);
                    } else {
                        let anchor_cs = para.line_segs.first().map(|s| s.column_start).unwrap_or(0);
                        let anchor_sw = para
                            .line_segs
                            .first()
                            .map(|s| s.segment_width as i32)
                            .unwrap_or(0);
                        // [#1956] anchor LINE_SEG 가 단 전체 폭이면(표가 본문 폭
                        // 이상 = 옆 공간 없음) 후속 전체 폭 문단들이 전부 오매칭
                        // 되므로 arming 하지 않는다. sw=0 은 기존 sw0_match 담당.
                        let col_w_hu = st.layout.column_width_hu();
                        let band_full_width = anchor_sw > 0 && (anchor_sw - col_w_hu).abs() < 3000;
                        if !band_full_width {
                            st.arm_control_wrap(anchor_cs, anchor_sw, para_idx, false);
                        }
                    }
                }
            }
        }
        if has_table {
            let issue2424_flush_started = issue2424_ts_enabled.then(std::time::Instant::now);
            self.flush_deferred_table_controls(
                st,
                paragraphs,
                composed,
                styles,
                measured_tables,
                DeferredTableFlushPoint::AfterTableParagraph(para_idx),
            );
            Issue2424TypesetProfile::add(
                &mut issue2424_prof.deferred_flush,
                issue2424_flush_started,
            );
            // [#1955] 글뒤로/글앞으로 비-TAC 표 anchor: 후행 빈 문단 흡수 arming.
            let has_behind_float_table = para.controls.iter().any(|c| {
                matches!(c, Control::Table(t)
                        if !t.common.treat_as_char
                            && matches!(t.common.text_wrap,
                                crate::model::shape::TextWrap::BehindText
                                    | crate::model::shape::TextWrap::InFrontOfText))
            });
            // [#4514] 흡수는 표가 fragment 로 흐름을 이미 소비한 앵커(#1955 원
            // 사례 — oversized [별표] 표)에만 정당하다. #703 Shape 단축(흐름 0)
            // 앵커에서 후행 빈 문단까지 흡수하면 저장 사다리의 갭(≈표 높이)이
            // 어느 쪽에도 계상되지 않아 후속 표가 위로 붕괴한다 (sample1-repro
            // 8쪽: 표 4개 최대 555.5px 겹침). shortcut 앵커는 arming 을 생략해
            // 필러가 자기 줄 높이만큼 정상 흐름으로 전진하게 둔다. 한 문단에
            // shortcut 표와 fragment 표가 공존하는 극단 케이스도 생략 쪽을
            // 택한다(공간 이중 계상보다 유실이 드묾).
            if has_behind_float_table && st.overlay_shape_shortcut_para != Some(para_idx) {
                st.arm_behind_float_absorption(Some(para_idx));
            }
        }
        // 비-TAC Picture/Shape Square wrap: engine.rs:380-397 동일 시멘틱.
        // 그림의 첫 lineseg cs가 0일 수 있어 any_seg_matches 허용 플래그 활성화.
        let issue2424_tail_started = issue2424_ts_enabled.then(std::time::Instant::now);
        self.typeset_no_table_paragraph_tail(
            st, page_def, para, paragraphs, composed, styles, para_idx, has_table,
        );
        Issue2424TypesetProfile::add(&mut issue2424_prof.para_tail, issue2424_tail_started);

        // Task #321: col 0 처리 중 body-wide TopAndBottom 표/도형이 발견되면
        // col 1+ advance 시 적용할 current_height 시작값을 미리 등록.
        // layout의 body_wide_reserved와 동일 조건으로 detect.
        if st.col_count > 1 && st.current_column == 0 && st.pending_body_wide_top_reserve == 0.0 {
            let reserve = compute_body_wide_top_reserve_for_para(para, &st.layout, self.dpi);
            if reserve > 0.0 {
                st.reserve_body_wide_top(reserve);
            }
        }
    }
}
