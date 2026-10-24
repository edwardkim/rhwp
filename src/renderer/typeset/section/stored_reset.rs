//! 구역 문단 처리의 apply_stored_paragraph_boundary 단계. 조건·예약·발행 순서를 유지한다.
use crate::renderer::typeset::{
    hwpunit_to_px, is_synthetic_line_seg, native_near_top_reset_exceeds_remaining,
    near_top_reset_page_is_barely_started, page_holds_only_fresh_table_continuation,
    page_item_para_index, para_has_visible_text, para_hosts_page_anchored_block,
    para_is_page_bottom_fixed_table_anchor, stored_vpos_top_collision, ColumnBreakType, ColumnType,
    Control, MeasuredTable, Paragraph, ResolvedStyleSet, TypesetEngine, TypesetState,
};
impl TypesetEngine {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn apply_stored_paragraph_boundary(
        &self,
        st: &mut TypesetState,
        para_idx: usize,
        para: &Paragraph,
        paragraphs: &[Paragraph],
        styles: &ResolvedStyleSet,
        has_table: bool,
        hwp3_origin_page_tolerance: bool,
        profile: crate::model::provenance::LayoutCompatibilityProfile,
        body_height_hu_for_variant: i32,
        measured_tables: &[MeasuredTable],
        boundary: &super::entry::ParagraphBoundary<'_>,
    ) {
        let reset_consumed_by_empty_prefix = st.consume_saved_empty_run_reset(para_idx, paragraphs);
        let para_style = boundary.para_style;
        let overlay_columndef_separator_break = boundary.overlay_columndef_separator_break;
        // Task #321: 문단간 vpos-reset 기반 강제 분할
        // HWP LINE_SEG의 vertical_pos는 페이지 내 흐름 y 좌표.
        // 현재 문단 first_vpos=0이고 직전 문단이 같은 단에 있으며 last_vpos가 충분히 큰 경우,
        // HWP가 pi 경계에서 페이지/단 분할을 의도한 것 → 강제 분할.
        // [Task #362] wrap-around zone 활성 중에는 vpos-reset 가드 무시 (기존).
        // [Task #724] vpos-reset trigger 발동 시 wrap_around 강제 종료 (신규):
        // HWP5 변환본 case 에서 paragraph 442/443 wrap_around 매칭 후 후속 paragraph
        // (예: 599) vpos=0 시점에도 wrap_around active 유지되어 페이지 분할 위반 →
        // vpos-reset trigger 시 wrap_around 강제 종료 + advance_column_or_new_page.
        if para_idx > 0 && !st.current_items.is_empty() {
            let prev_para = &paragraphs[para_idx - 1];
            // 재조판 줄의 vpos는 문단 내부 좌표다. 문단 사이의 쪽/단 경계는
            // 양쪽이 실제 저장 줄일 때만 같은 쪽 좌표계의 되감김으로 읽는다.
            let curr_first_vpos = para
                .line_segs
                .first()
                .filter(|s| !is_synthetic_line_seg(s))
                .map(|s| s.vertical_pos);
            // 누락 줄을 재구성한 본문 뒤에도 다음 원본 프레임은 남아 있다.
            // 자기 문단 위 간격에서 재시작하는 미편집 저장 본문을 연결한다.
            // 합성 줄 자체의 vpos를 쪽 경계 신호로 읽지는 않는다.
            let reconstructed_source_vpos = curr_first_vpos.and_then(|cv| {
                let spacing_before = para_style
                    .map(|style| (style.spacing_before * 7200.0 / self.dpi).round() as i32)?;
                if profile.session_edited()
                    || st.col_count != 1
                    || !(profile.hwp5_stored_pagination_layout() || profile.hwpx_stored_layout())
                    || para.stored_text_partition_is_dirty()
                    || para.cell_format_vpos_dirty
                    || !para.controls.is_empty()
                    || !para_has_visible_text(para)
                    || para.column_type != ColumnBreakType::None
                    || cv <= 0
                    || spacing_before <= 0
                    || (i64::from(cv) - i64::from(spacing_before)).abs() > 1
                    || para.line_segs.iter().any(|line| {
                        is_synthetic_line_seg(line) || line.line_height <= 0
                    })
                {
                    return None;
                }
                crate::renderer::typeset::paragraph::stored_lines::stored_vpos_before_reconstructed_text(
                    &st.current_items, paragraphs, para_idx,
                ).filter(|pv| cv < *pv)
            });
            let prev_last_vpos = prev_para
                .line_segs
                .last()
                .filter(|s| !is_synthetic_line_seg(s))
                .map(|s| s.vertical_pos)
                .or(reconstructed_source_vpos);
            if let (Some(cv), Some(pv)) = (curr_first_vpos, prev_last_vpos) {
                // 저장 쪽 경계는 그 쪽의 줄 구성에 의존한다. 현재 쪽에서
                // 재조판 뒤 빈 저장 줄의 연속성만으로 옛 경계를 복구하지 않는다.
                // 실제 흐름 끝이 저장 끝과 일치하는 경계는 공통 판정에서 보존한다.
                // 명시적 쪽/단 나눔은 entry 단계에서 처리한다.
                let stored_boundary_invalidated_by_reflow =
                    reconstructed_source_vpos.is_none()
                    && crate::renderer::typeset::paragraph::stored_lines::stored_page_boundary_invalidated_by_reflow(
                        profile, st.col_count, &st.current_items, paragraphs,
                        para_idx, st.current_height, self.dpi,
                    ) || crate::renderer::typeset::paragraph::stored_lines::stored_frame_prefix_consumed_before_page(
                        st, paragraphs, para_idx, styles,
                    );
                // 현재 문단의 vpos가 직전 문단의 마지막 vpos보다 작은 경우 — 컬럼/페이지 reset 시그널.
                // - 단일 단: cv == 0 만 인정 (Task #321 보수적 기준 유지).
                //   단일 단에서 cv != 0 의 cv < pv 는 partial-table split 의 LAYOUT 잔재로
                //   해석되어야 함 (issue #418 / hwpspec pi=78→pi=79).
                // - 다단 Normal (NEWSPAPER): cv != 0 도 인정 (Task #470). pv > 5000 임계값 유지.
                // - 다단 Distribute (BalancedNewspaper): 짧은 컬럼 (3+3 분배 등) 에서 pv 가
                //   임계값 미달일 수 있어 pv > 0 으로 완화 (Task #702, shortcut 지우기 6항목 정합).
                //   단일 단/Normal 다단은 영향 없음.
                let is_distribute = st.col_count > 1
                    && matches!(st.current_zone_column_type, ColumnType::Distribute);
                // [Task #853] Distribute 다단의 "1줄짜리 컬럼" 케이스: 직전 문단이
                // 단 1줄(예: vpos=0)이고 현재 문단도 vpos=0 이면 `cv < pv` 가 0<0 으로
                // 거짓이라 컬럼 전환을 못 잡았다(shortcut.hwp 스타일/속성 섹션). 직전 문단의
                // vpos+line_height(=콘텐츠 끝)를 기준으로 비교하면 정상 흐름(cv=pv_end+ls≥pv_end)
                // 은 영향 없고 reset(cv≪pv_end)만 잡힌다.
                let prev_vpos_end = prev_para
                    .line_segs
                    .last()
                    .map(|s| s.vertical_pos.saturating_add(s.line_height))
                    .unwrap_or(pv);
                // [Task #1086 Stage 3] HWP3-origin page tolerance 대상 문서는
                // 새 페이지 첫 문단을 vpos=0 이 아니라 200/500HU 근방으로
                // 인코딩하기도 한다(hwpspec.hwp s2:pi=89, pi=104). 단일 단에서
                // 모든 cv<pv 를 reset 으로 보면 일반 직접 작성 HWP(2022년
                // 국립국어원), partial-table 직후 정상 흐름(hwpspec pi=78→79),
                // 표 host 문단(hwpspec pi=57)을 깨므로, 비영 near-top reset 은
                // 직전 문단이 페이지 하단부에 있고 대상이 텍스트/그림-only 문단일
                // 때만 인정한다.
                // 그림만 든 빈 문단은 한컴이 조금 더 일찍 새 페이지로 넘기는 케이스
                // (hwpspec.hwp s3:pi=93)가 있어 표/텍스트보다 낮은 하단 기준을 쓴다.
                let shape_only_para = para.text.trim().is_empty()
                    && !para.controls.is_empty()
                    && para
                        .controls
                        .iter()
                        .all(|c| matches!(c, Control::Picture(_) | Control::Shape(_)));
                let has_table_control =
                    para.controls.iter().any(|c| matches!(c, Control::Table(_)));
                let near_page_top_reset = hwp3_origin_page_tolerance
                    && cv > 0
                    && ((shape_only_para && cv <= 200 && prev_vpos_end > 52_000)
                        || (!shape_only_para
                            && !has_table_control
                            && cv <= 500
                            && prev_vpos_end > 60_000));
                let para_sb_hu_for_reset = para_style
                    .map(|s| (s.spacing_before * 7200.0 / 96.0) as i32)
                    .unwrap_or(0);
                // 저장 HWP5에서 인라인 묶음 도형만 든 제목 문단은 쪽 첫 줄의
                // vpos를 0 대신 작은 양수로 남길 수 있다. 보통의 shape-only
                // 문단은 쪽-앵커/어울림 좌표일 수 있어 위의 일반 near-top
                // 분기에서 제외하지만, 인라인 도형 한 개가 직전 쪽 하단에서
                // 되감기고 다음 본문 문단도 그 새 좌표에서 이어지면 물리 쪽
                // 경계다. 이 경계를 놓치면 제목 도형만 앞 쪽의 본문 밖에
                // 그려지고 이어지는 본문은 다음 쪽으로 넘어간다.
                let native_inline_shape_reset = !hwp3_origin_page_tolerance
                    && profile.hwp5_stored_pagination_layout()
                    && !profile.session_edited()
                    && shape_only_para
                    && para.controls.len() == 1
                    && matches!(&para.controls[0], Control::Shape(shape) if shape.common().treat_as_char)
                    && cv > 0
                    && cv == para_sb_hu_for_reset
                    && stored_inline_shape_restarts_column(
                        prev_para,
                        para,
                        paragraphs.get(para_idx + 1),
                        crate::renderer::px_to_hwpunit(st.layout.body_area.height, self.dpi),
                    )
                    && !para_is_page_bottom_fixed_table_anchor(para)
                    && !para_hosts_page_anchored_block(para);
                // [#1921 d=-1] 네이티브 HWP5/HWPX 의 비영 near-top reset:
                // 한글은 새 쪽 첫 줄의 stored vpos 를 0 이 아니라 해당 문단의
                // spacing_before 로 기록하기도 한다 (조문별/규제영향분석서 클래스,
                // 예: 86034 pi24 vpos=500 = sb 6.7px). cv==0 전용 트리거가 이를
                // 놓쳐 한 쪽을 과적한다. cv≈sb(±150HU) + 쪽 하단(prev_vpos_end
                // > 60_000) + 텍스트 전용 문단으로 한정해 partial-table 잔재
                // (#418)·표 host(#1086 주석) 케이스와 구분한다.
                let native_near_top_reset = !hwp3_origin_page_tolerance
                        && cv > 0
                        // [#2136] 상한 2000→2500: sb=5000유닛(=2500HU) 문단의 저장
                        // 리셋(cv=2500=sb 정확 일치)이 500HU 차로 배제되어 측정 fit
                        // 과적(148753276 pi46: used 942px > body 933.6px, 한글 p5).
                        // #1750 split-precheck 상한(2500)과 정합. sb 일치 ±150 조건이
                        // 유지되어 오발동 억제.
                        && cv <= 2500
                        && para_sb_hu_for_reset > 0
                        && (cv - para_sb_hu_for_reset).abs() <= 150
                        && !shape_only_para
                        && !has_table_control
                        && para_has_visible_text(para)
                        && prev_vpos_end > 60_000
                        // [#5941 축 B] `#5921` 의 완화("이번 쪽 잔여에 들어가면 저장
                        // near-top 리셋을 버린다")를 **네이티브 HWP5 저장 조판 프로파일의
                        // 이미 찬 쪽**에는 걸지 않는다. 그 프로파일에서 저장 사다리는 작성
                        // 엔진의 쪽 배분 자체이고, "이번 잔여에 들어간다" 는 사실이 그
                        // 기록을 이기지 못한다.
                        //
                        // 실측 — 완화가 필요한 문서는 전부 HWPX 컨테이너이거나, HWP5 라도
                        // **쪽이 사실상 비어 있다**:
                        //
                        // ```text
                        //   neartop_reset_sb2500.hwpx        hwpx   채움  2%   #5921 원 픽스처
                        //   issue1880_anchor_stack_sb…hwpx   hwpx   채움 96%   #6063 핀
                        //   hwp3-sample16-hwp5.hwpx          hwpx   채움 70~100%  off_canvas 래칫
                        //   issue5701 …tac_reset_tail.hwp    hwp5   채움  6%   ← 빈 쪽 갈래
                        //   1480000-201900698.hwp            hwp5   채움 74~100% ← 여기만 완화 금지
                        // ```
                        //
                        // 그 마지막 문서는 완화 때문에 리셋 3개가 지워져 202 → 200 이 됐고
                        // 한/글 2024(205)에서 더 멀어졌다(`#5941` 축 B bisect: `12074fbea`).
                        && ((profile.hwp5_stored_pagination_layout()
                            && !near_top_reset_page_is_barely_started(
                                st.current_height,
                                st.available_height(),
                            ))
                            // 실제 흐름이 저장 끝까지 왔다면 HWPX도 같은 쪽
                            // 경계를 소비한다. 잔여 fit만으로 이를 덮지 않는다.
                            || (!profile.session_edited()
                                && crate::renderer::typeset::stored_rewind_boundary_matches_current_flow(
                                    paragraphs, para_idx, &st.current_items, st.current_height, self.dpi,
                                ))
                            || native_near_top_reset_exceeds_remaining(
                                para,
                                para_sb_hu_for_reset,
                                st.current_height,
                                st.available_height(),
                                self.dpi,
                            ));
                let next_heading_after_top_content_reset =
                    paragraphs.get(para_idx + 1).is_some_and(|next_para| {
                        let next_sb_hu = styles
                            .para_styles
                            .get(next_para.para_shape_id as usize)
                            .map(|ps| (ps.spacing_before * 7200.0 / 96.0) as i32)
                            .unwrap_or(0);
                        next_para.line_segs.len() >= 2
                            && next_para
                                .line_segs
                                .first()
                                .filter(|ls| !is_synthetic_line_seg(ls))
                                .is_some_and(|ls| ls.vertical_pos > 0 && ls.vertical_pos <= 2500)
                            && next_para.controls.is_empty()
                            && para_has_visible_text(next_para)
                            && next_sb_hu >= 500
                    });
                let hwp3_content_vpos_zero_reset = profile.hwp3_layout()
                    && st.col_count == 1
                    && cv == 0
                    && prev_vpos_end > body_height_hu_for_variant * 70 / 100
                    && para_sb_hu_for_reset < 250
                    && para.controls.is_empty()
                    && para_has_visible_text(para)
                    && next_heading_after_top_content_reset;
                // [#5907] 앞뒤 문단이 둘 다 stored vpos 0 을 주장하는 충돌.
                // `pv > 5000` 기준은 "직전 문단이 쪽 하단부에 있었다"를 전제하는데,
                // 한/글이 짧은 문단 하나만 올리고 쪽을 넘긴 경우 pv 는 0 이라 침묵한다.
                // 같은 단 기하 + 어울림 개체 없음 + 문단 자체 나누기 없음으로 좁힌다.
                let stored_top_collision_reset = st.col_count == 1
                    && cv == 0
                    && para.column_type == ColumnBreakType::None
                    && st.wrap_around_cs < 0
                    && !para_is_page_bottom_fixed_table_anchor(para)
                    && stored_vpos_top_collision(prev_para, para);
                // 좌표 축을 닫은 완전한 TAC 표도 원본 쪽 경계는 보존한다.
                // 부분 이어받기는 이 단에 원본 전체 줄을 소비한 증거가 아니다.
                let inline_frame_page_restart = st.col_count == 1
                    && !profile.session_edited()
                    && profile.hwp5_stored_pagination_layout()
                    && matches!(st.current_items.last(), Some(crate::renderer::typeset::PageItem::Table {
                        para_index, control_index: 0,
                    }) if *para_index == para_idx - 1)
                    && crate::renderer::stored_page_origin::stored_inline_frame_restarts_page(
                        prev_para,
                        para,
                        paragraphs.get(para_idx + 1),
                        styles,
                        self.dpi,
                    );
                let source_margin_restart = st.col_count == 1
                    && !profile.session_edited()
                    && (profile.hwp5_stored_pagination_layout() || profile.hwpx_stored_layout())
                    && st.vpos_page_base.is_some_and(|origin| {
                        crate::renderer::float_placement::stored_margin_page_restart(
                            prev_para,
                            para,
                            paragraphs.get(para_idx + 1),
                            styles,
                            origin,
                            st.current_height - st.vpos_col_anchor,
                            self.dpi,
                        )
                    });
                // 이미 검증한 그림 행이 재시작 원점을 소유한다. 앞 본문이 실제
                // 저장 끝까지 도달했을 때만 그 행과 본문을 같은 새 쪽에 발행한다.
                let stored_picture_group_reset = st.col_count == 1
                    && !profile.session_edited()
                    && (profile.hwp5_stored_pagination_layout() || profile.hwpx_stored_layout())
                    && cv > 0
                    && cv < pv
                    && !para.stored_text_partition_is_dirty()
                    && !prev_para.stored_text_partition_is_dirty()
                    && para.controls.is_empty()
                    && st.has_pending_picture_group_reset(hwpunit_to_px(cv, self.dpi), self.dpi)
                    && prev_para.line_segs.last().is_some_and(|line| {
                        let end = i64::from(line.vertical_pos)
                            + i64::from(line.line_height)
                            + i64::from(line.line_spacing)
                            - i64::from(st.vpos_page_base.unwrap_or(0));
                        (end as f64 * self.dpi / 7200.0 - st.current_height).abs()
                            <= self.dpi / 7200.0
                    });
                let trigger = if st.col_count > 1 {
                    if is_distribute {
                        cv < prev_vpos_end && prev_vpos_end > 0
                    } else {
                        cv < pv && pv > 5000
                    }
                } else {
                    // 직전 줄이 쪽 위쪽에서 시작해도(한 줄짜리 쪽 높이 TAC 표 등)
                    // 그 줄의 끝이 쪽 하단부면 cv==0 은 쪽 경계다. 시작 vpos(pv)만
                    // 보면 이 경계를 놓쳐 다음 문단을 같은 쪽에 겹쳐 놓는다.
                    // 단 pv==0 이면 저장 vpos 가 없는(모두 0) 문단일 수 있어 제외한다.
                    (cv == 0
                            && pv > 0
                            && prev_vpos_end > 5000
                            && !hwp3_content_vpos_zero_reset
                            && !para_is_page_bottom_fixed_table_anchor(para)
                            // [#6535 잔여] 쪽-앵커 블록의 vpos=0 은 절대배치 산물이지
                            // 쪽 리셋 신호가 아니다.
                            && !para_hosts_page_anchored_block(para))
                        || near_page_top_reset
                        // The accepted column origin can survive a broken
                        // source ladder. Its dirty/provenance flag controls
                        // backward snapping, not ownership of a carried frame.
                        || (native_near_top_reset
                            && !st
                                .vpos_page_base
                                .is_some_and(|origin| cv < origin))
                        || native_inline_shape_reset
                        || stored_top_collision_reset
                        || source_margin_restart
                        || inline_frame_page_restart
                        || stored_picture_group_reset
                        || reconstructed_source_vpos.is_some()
                };
                // [#2279 OMIT-fit] spacing-누락 문서군에서 fresh 재계산이 직전
                // 쪽에서 밀어낸 빈 문단만 담긴 쪽에는 저장 리셋 경계를 적용하지
                // 않는다 — 한글 fresh 는 그 빈 문단을 다음 쪽 상단에 두고
                // 본문을 이어 붙인다 (36392557 pi14+pi15: 한글 PDF p3 상단
                // 36px = pi14 자리, 단독 쪽 아님).
                let omit_pushed_empty_page = st.omit_fresh_recalc_doc
                    && st.current_items.iter().all(|item| {
                        page_item_para_index(item)
                            .and_then(|idx| paragraphs.get(idx))
                            .is_some_and(|p| p.controls.is_empty() && p.text.trim().is_empty())
                    });
                // [compat 2024] (Δ1) 이번 단에서 자리차지 표 앵커 줄을 회수
                // 했거나 앞선 경계를 이미 덮어 저장 경계가 한 발씩 어긋난
                // 상태이고, 이 문단의 **첫 줄**이 잔여 공간에 들어가면 저장
                // vpos 리셋(=2022 조판의 쪽 경계)을 존중하지 않는다 — 문단이
                // 통째로 못 들어가면 일반 분할 경로가 첫 줄부터 채운다(한글
                // 2024 의 fresh 흐름과 같은 결정). 회수도 선행 덮음도 없는
                // 문서/쪽에서는 종전 동작 그대로.
                let hangul2024_refit =
                    st.profile.hangul2024_layout() && st.hangul2024_reclaimed > 0.0 && {
                        // 빈 문단(텍스트·컨트롤 없음)은 한글이 쪽 하단
                        // 여백으로 흘려도 되는 존재라 need=0, 실문단은 첫 줄
                        // (통째로 못 들어가면 일반 분할이 첫 줄부터 채운다 —
                        // 한글 2024 fresh 와 같은 결정). 회귀를 만들던 것은
                        // 이 완화가 아니라 sticky 연쇄였음(라운드 B/C 회귀값
                        // 동일로 판별).
                        let need: f64 = if !para_has_visible_text(para) && para.controls.is_empty()
                        {
                            0.0
                        } else {
                            para.line_segs
                                .first()
                                .map(|s| {
                                    hwpunit_to_px(
                                        s.line_height.saturating_add(s.line_spacing),
                                        self.dpi,
                                    )
                                })
                                .unwrap_or(0.0)
                        };
                        st.current_height + need <= st.available_height() + st.hangul2024_reclaimed
                    };
                if std::env::var("RHWP_DIAG_COMPAT24").is_ok() && trigger {
                    eprintln!(
                        "DIAG_COMPAT24 reset-trigger pi={para_idx} refit={hangul2024_refit} \
                             cur={:.1} reclaimed={:.1}",
                        st.current_height, st.hangul2024_reclaimed,
                    );
                }
                if trigger
                    && hangul2024_refit
                    && !para_has_visible_text(para)
                    && para.controls.is_empty()
                {
                    st.mark_blank_paragraph_spill(para_idx);
                }
                // [#5919] [#2019 v3] 로 명시 단나누기를 억제한 ColumnDef-only
                // overlay 구분자의 lineseg vertpos=0 은 쪽/단 경계 신호가 아니라
                // 마커 문단의 미설정값이다. 이 경계를 저장 vpos 리셋으로 다시
                // 읽으면 억제했던 단나누기가 되살아나 표 격자와 본문을 서로
                // 다른 허위 쪽으로 갈라 놓는다(74312 12쪽).
                // [편집 세션] 저장 vpos 리셋은 저장 시점의 쪽 경계 신호라
                // 편집으로 앞 내용이 늘거나 줄면 낡은 좌표다. 다만 무조건
                // 무시하면 이 쪽 잔여가 몇십 px 뿐일 때 다음 쪽에서 시작하던
                // 표의 머리 행 조각이 잔여에 낑겨 앞 문단과 겹친다(한글
                // 오라클: 잔여가 작으면 표는 통째로 다음 쪽 유지). 그래서
                // 표 host 문단은 잔여에 표 머리(첫 두 행)가 실제로 들어갈
                // 때만 낡은 경계를 무시하고 fresh fit 에 맡긴다.
                let session_stale_reset_override =
                    self.profile.get().session_edited() && !st.current_items.is_empty() && {
                        let first_table_head_px =
                            para.controls
                                .iter()
                                .enumerate()
                                .find_map(|(ci, c)| match c {
                                    Control::Table(_) => measured_tables
                                        .iter()
                                        .find(|m| m.para_index == para_idx && m.control_index == ci)
                                        .map(|m| m.row_heights.iter().take(2).sum::<f64>()),
                                    _ => None,
                                });
                        // 표 없는 문단의 저장 리셋은 단/쪽 경계 인코딩
                        // (#2299 shortcut.hwp: 리셋 76곳 = 다단 밴드)일 수
                        // 있어 존중한다 — 무시 대상은 편집으로 성장하는 표
                        // host 문단의 낡은 경계뿐이다.
                        match first_table_head_px {
                            Some(head) => st.current_height + head <= st.available_height() + 0.5,
                            None => false,
                        }
                    };
                // Plain-text near-top source resets were evaluated against the
                // accepted stored origin above. Explicit zero/shape/column
                // boundaries retain their separate ownership contracts.
                let trigger = trigger
                    && !reset_consumed_by_empty_prefix
                    && !st.stored_paragraph_boundary_already_open(paragraphs, para_idx)
                    && !stored_boundary_invalidated_by_reflow
                    && !omit_pushed_empty_page
                    && !hangul2024_refit
                    && !overlay_columndef_separator_break
                    && !session_stale_reset_override;
                if trigger {
                    // [Task #724] wrap_around active 시 강제 종료 — anchor cs=0
                    // (HWP5 변환본 caption-style) 한정. 일반 wrap_around (anchor cs>0)
                    // 는 기존 동작 (Task #362 vpos-reset 무시) 유지.
                    if st.wrap_around_cs == 0 {
                        st.finish_stored_wrap_matching();
                        st.close_square_band();
                    }
                    if st.wrap_around_cs < 0 {
                        // [#5918] 저장 리셋(cv==0/near-top)이 가리키는 쪽 경계를
                        // block-table continuation 꼬리 조각이 이미 열어 놨으면
                        // 이중으로 쪽을 넘기지 않는다. RowBreak 표의 마지막 조각은
                        // 저장 경계와 무관하게 "앞 조각이 못 들어간" 시점에 새 쪽
                        // 상단에 배출되는데, 그 새 쪽이 곧 저장 사다리의 리셋 지점과
                        // 같은 물리 경계인 경우가 있다(sample1-repro pi=578 꼬리 조각
                        // 쪽 == pi=608 vpos=0 리셋 쪽). 이때 리셋이 한 번 더
                        // advance하면 조각만 남은 근빈 쪽이 생기고 뒤따르는 표가
                        // 남은 공간으로 흐르지 못한다. 현재 쪽이 continuation
                        // 조각(들)과 빈 필러 문단만 담고 있을 때만 건너뛴다 — 실
                        // 내용이 함께 놓인 쪽의 저장 경계는 그대로 존중한다.
                        if !page_holds_only_fresh_table_continuation(&st, paragraphs) {
                            // [#6146] 저장 리셋은 이 문단의 **줄**이 다음 쪽이라는
                            // 신호지 자리차지 밴드까지 옮기라는 신호가 아니다.
                            // 한글은 밴드를 떠나는 쪽의 흐름 말미에 남기고 본문 아래
                            // 여백으로 흘린다.
                            st.spill_page_tail_floats(para_idx, para);
                            st.advance_column_or_new_page();
                        }
                    }
                }
            }
        }
    }
}

/// 인라인 도형은 자기 저장 줄 상자를 소유한다. 이전 쪽의 남은 물리 높이에
/// 그 상자가 들어가지 않고 뒤 줄이 새 축에서 이어질 때 저장 재시작을 보존한다.
fn stored_inline_shape_restarts_column(
    previous: &Paragraph,
    current: &Paragraph,
    next: Option<&Paragraph>,
    body_height_hu: i32,
) -> bool {
    let Some(next) = next else {
        return false;
    };
    let (Some(last), [line], Some(after)) = (
        previous.line_segs.last(),
        current.line_segs.as_slice(),
        next.line_segs.first(),
    ) else {
        return false;
    };
    if [previous, current, next]
        .iter()
        .any(|para| para.stored_text_partition_is_dirty() || para.cell_format_vpos_dirty)
        || [last, line, after]
            .iter()
            .any(|seg| is_synthetic_line_seg(seg) || seg.line_height <= 0 || seg.line_spacing < 0)
        || body_height_hu <= 0
    {
        return false;
    }
    let previous_end =
        i64::from(last.vertical_pos) + i64::from(last.line_height) + i64::from(last.line_spacing);
    let current_end =
        i64::from(line.vertical_pos) + i64::from(line.line_height) + i64::from(line.line_spacing);
    line.vertical_pos < last.vertical_pos
        && i64::from(after.vertical_pos) >= current_end
        && i64::from(after.vertical_pos) < previous_end
        && previous_end + i64::from(line.line_height) + i64::from(line.line_spacing)
            > i64::from(body_height_hu)
}
