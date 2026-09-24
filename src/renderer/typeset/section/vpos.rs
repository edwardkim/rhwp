//! 구역 vpos 책임. 기존 조건과 호출 순서를 보존한다.
use crate::renderer::typeset::{
    stored_ladder_encodes_spacing_before, HeightCursor, Paragraph, ResolvedStyleSet, TypesetEngine,
    TypesetState,
};
impl TypesetEngine {
    /// 렌더러 `build_single_column` 의 inter-item VPOS_CORR(Stage C `HeightCursor::vpos_adjust`)
    /// 를 페이지네이터에서도 적용해, 단락마다 `+= total_height` 로 누적된 sb·trailing_ls
    /// drift 를 다음 항목 진입 시 제거한다(렌더러와 동일 측정). 단단(col_count==1) 전용 —
    /// 다단/flow-around 는 Stage E.
    ///
    /// HeightCursor 는 `current_height` 상대공간(col_area_y=0)에서 구동한다.
    pub(in crate::renderer::typeset) fn vpos_snap_current_height(
        &self,
        st: &mut TypesetState,
        para_idx: usize,
        paragraphs: &[Paragraph],
        styles: &ResolvedStyleSet,
        spacing_before_px: f64,
    ) {
        if st.col_count != 1 {
            return; // 다단은 Stage E
        }
        // 컬럼 첫 항목: anchor + page_base 확립 (렌더러 2186/2216 정합).
        // items.first() 의 vpos 를 page_base 로, 현 current_height 를 anchor 로 둔다.
        if st.current_items.is_empty() {
            st.record_vpos_column_anchor(st.current_height);
            st.record_vpos_page_origin(
                paragraphs
                    .get(para_idx)
                    .and_then(|p| p.line_segs.first())
                    .map(|s| s.vertical_pos),
            );
            st.record_vpos_lazy_origin(None);
            // [#2243] 저장 여부 태깅 — dirty 역스냅 금지 판단용.
            st.record_vpos_origin_provenance(
                paragraphs
                    .get(para_idx)
                    .and_then(|p| p.line_segs.first())
                    .is_some_and(|s| {
                        s.tag & crate::model::paragraph::LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0
                    }),
            );
            st.record_vpos_ladder_validity(false);
        }
        let mut hc = HeightCursor {
            dpi: self.dpi,
            col_area_y: 0.0,
            col_area_height: st.base_available_height(),
            col_anchor_y: st.vpos_col_anchor,
            vpos_page_base: st.vpos_page_base,
            vpos_lazy_base: st.vpos_lazy_base,
            prev_layout_para: st.vpos_prev_layout_para,
            prev_item_was_partial_table: st.vpos_prev_partial_table,
            skip_spacing_before_prededuct: st.skip_spacing_before_prededuct,
            trimmed_prev_spacing_before_px: st.vpos_prev_trimmed_sb_px,
            allow_vpos_rewind: false,
            allow_start_height_backtrack: false,
            suppress_large_forward_jump: false,
            suppress_hwpx_stale_forward: st.profile.hwpx_stored_layout(),
            uniform_filler_ladder: self.uniform_filler_ladder.get(),
            endnote_between_notes_hu: 0,
            prev_item_content_bottom_y: None,
            last_compacted_endnote_title_gap: false,
            min_flow_floor: f64::MIN,
            session_edited: self.profile.get().session_edited(),
        };
        let mut y = hc.vpos_adjust(st.current_height, para_idx, paragraphs, styles);
        // [#5699 H1] 저장 사다리가 자리차지 표 밴드를 계상하지 않은 문서: 흐름이
        // 계상 교정으로 확보한 표 밴드 위로 저장 vpos 스냅으로 되감기지 못한다.
        if y < st.ladder_band_floor {
            y = st.ladder_band_floor;
        }
        // 렌더의 min_flow_floor는 floor뿐 아니라 직전 순차 cursor도 보호한다.
        // 회피한 표 이후 분할기만 저장 vpos로 되감으면 쪽 fit와 출력이 갈라진다.
        if !st.inline_placements.is_empty()
            || !st.inline_flow_plans.is_empty()
            || st
                .paragraph_float_placements
                .iter()
                .any(|(&(owner, _), placement)| {
                    owner < para_idx
                        && placement.flow
                            == crate::renderer::float_placement::ParagraphFloatFlow::NextLine
                })
        {
            y = y.max(st.current_height);
        }
        // [#2243] dirty 저장-앵커 사다리의 역스냅 금지 — 저장 lineseg 누락 문단의
        // fresh 재계산 성장분을 낡은 기계 v0 가 되돌리지 못하게 한다(전방만 허용).
        // [#2279 OMIT-sa] spacing-누락 문서군은 합성(비저장) base 사다리도 동일 —
        // 사다리 자체가 spacing 을 누락하므로 dirty 성장분의 역스냅을 base 저장
        // 여부와 무관하게 금지한다 (36392557 p4: pi61 저장 anchor 로의 역스냅이
        // sa 성장분 +20.1px 를 되감아 페이지 경계를 한 문단 뒤로 밀던 형상).
        if st.vpos_ladder_dirty
            && (st.vpos_page_base_stored || st.omit_fresh_recalc_doc)
            && y < st.current_height
        {
            y = st.current_height;
        }
        // [#2279 OMIT-sa] 합성(비저장) lineseg 경계의 후방 스냅량이 직전 문단의
        // spacing_after 와 일치(±2px)하면 합성 사다리의 sa-누락이다 — 한글
        // fresh 는 sa 를 가산하므로 되감지 않는다 (36392557 p4 pi42/43/44
        // sa 6.7px ×3, 한글 PDF 절대좌표 +20.1px 실측). OMIT 문서군 + 합성
        // seg 직전 문단 한정. 이후 저장 anchor 역스냅이 성장분을 되돌리지
        // 못하게 dirty 처리(위 #2243 확장과 짝).
        if st.omit_fresh_recalc_doc && y < st.current_height && para_idx > 0 {
            let prev = &paragraphs[para_idx - 1];
            let prev_synthetic = !prev.line_segs.is_empty()
                && prev.line_segs.iter().all(|s| {
                    s.tag & crate::model::paragraph::LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0
                });
            let prev_sa = styles
                .para_styles
                .get(prev.para_shape_id as usize)
                .map(|s| s.spacing_after)
                .unwrap_or(0.0);
            if prev_synthetic && prev_sa > 0.5 && (st.current_height - y - prev_sa).abs() < 2.0 {
                y = st.current_height;
                st.mark_vpos_ladder_dirty();
            }
        }
        // [#2279 ladder-sb] 후방 스냅량이 이 문단의 spacing_before 와 정확히
        // 일치(±2px)하고, **저장 ladder 스텝 자체가 sb 를 누락**했으면 생성기
        // ladder 의 sb-누락이다 — 한글 fresh 는 sb 를 가산하므로(서브픽셀
        // 하니스 실측: 36398709 문단당 −5.0pt 균일 = sb 6.7px) 스냅으로 되감지
        // 않는다. 스텝-검사 없이 ±2px 우연 일치만으로 스킵하면 sb-포함 정상
        // ladder 에서 이중 가산(+1쪽 과다, issue_1853 실측 반증)이 난다.
        if st.profile.hwpx_stored_layout()
            && spacing_before_px > 0.5
            && y < st.current_height
            && (st.current_height - y - spacing_before_px).abs() < 2.0
        {
            y = st.current_height;
        }
        // [#6031] sb-누락 ladder 는 경계 하나가 아니라 **문서 전체 서명**이다 —
        // 위 ±2px 일치 스킵만으로는 누락분이 다음 sb=0 경계의 후방 스냅으로
        // 흘러가 되감긴다(3249937 p3: pi=36 스킵분 −6.7px 가 pi=37 스냅으로
        // 제거, 쪽 말미 누적 +53.3px 를 typeset 만 안 본다). 렌더는 후방 스냅
        // 상한(8px)으로 이 되감김을 거부하므로 판정 좌표와 배치 좌표가 갈라져
        // 쪽-말미 줄이 본문 하단 밖에 그려진다(#5801 코어). 경계 검사
        // (`stored_ladder_encodes_spacing_before`)가 누락을 확정하면 남은 열의
        // 후방 스냅·트림을 dirty 로 철회해 두 좌표계를 일치시킨다 — 한글
        // fresh 도 sb 를 가산한 흐름으로 쪽을 끊는다(p3 말미 810.7px 실측
        // = 한글 809.7pt 정합, 꼬리 '바.' 줄은 한글도 4쪽 첫 줄).
        if st.profile.hwpx_stored_layout()
            && !st.profile.hwp3_layout()
            && spacing_before_px > 5.0
            && !st.vpos_ladder_dirty
            && !stored_ladder_encodes_spacing_before(
                paragraphs,
                para_idx,
                spacing_before_px,
                self.dpi,
            )
        {
            st.mark_vpos_ladder_dirty();
            if y < st.current_height {
                y = st.current_height;
            }
        }
        // [#2243 진단] snap 입출력 — 동작 불변.
        if std::env::var("RHWP_DIAG_SNAPALL").is_ok() {
            eprintln!(
                "DIAG_SNAPALL pi={} y_in={:.1} y_out={:.1} base={:?} lazy={:?} anchor={:.1} dirty={}",
                para_idx,
                st.current_height,
                y,
                st.vpos_page_base,
                hc.vpos_lazy_base,
                st.vpos_col_anchor,
                st.vpos_ladder_dirty,
            );
        }
        if std::env::var("RHWP_DIAG_TAC").is_ok() && (y - st.current_height).abs() > 0.05 {
            eprintln!(
                "DIAG_SNAP pi={} y_in={:.1} y_out={:.1} page_base={:?} lazy={:?} anchor={:.1}",
                para_idx,
                st.current_height,
                y,
                st.vpos_page_base,
                hc.vpos_lazy_base,
                st.vpos_col_anchor,
            );
        }
        // lazy_base 는 지연 산출 시 갱신될 수 있으므로 회수.
        st.record_vpos_lazy_origin(hc.vpos_lazy_base);
        st.align_flow_to(y);
    }
}
