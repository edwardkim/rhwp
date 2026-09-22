//! 구역 수명과 원본 문단 순서의 조정. 실제 상태 쓰기는 state가 소유한다.
use crate::renderer::typeset::{
    column_def_design_spacing_px, footnote_between_notes_margin_px, footnote_separator_overhead_px,
    hwpunit_to_px, is_synthetic_line_seg, ladder_spacing_omitted_signature,
    para_has_non_whitespace_text, para_is_empty_topbottom_table_anchor, table, ColumnDef,
    ComposedParagraph, DeferredTableFlushPoint, EndnoteDeferral, FootnoteShape,
    Issue2424TypesetProfile, MeasuredTable, PageDef, PageLayoutInfo, PaginationResult, Paragraph,
    ResolvedStyleSet, TypesetEngine, TypesetState, MIN_TOP_KEEP_PX,
};
mod boundary;
mod columns;
mod controls;
mod entry;
mod finalize;
mod flow;
mod heading;
mod post_flow;
mod stored_reset;
mod tail;
mod vpos;
impl TypesetEngine {
    pub(in crate::renderer::typeset) fn run_section(
        &self,
        paragraphs: &[Paragraph],
        composed: &[ComposedParagraph],
        styles: &ResolvedStyleSet,
        page_def: &PageDef,
        column_def: &ColumnDef,
        section_index: usize,
        measured_tables: &[MeasuredTable],
        hide_empty_line: bool,
        profile: crate::model::provenance::LayoutCompatibilityProfile,
        skip_spacing_before_prededuct: bool,
        hwp3_origin_page_tolerance: bool,
        footnote_shape: Option<&FootnoteShape>,
        endnote_shape: Option<&FootnoteShape>,
        force_break_before: &std::collections::HashSet<usize>,
        endnote_deferral: EndnoteDeferral<'_>,
    ) -> PaginationResult {
        // [#2424 프로파일] paginate_pass 와 같은 env var 로 하위 단계 게이트.
        #[cfg(not(target_arch = "wasm32"))]
        let issue2424_ts_enabled =
            std::env::var("RHWP_2424_PROFILE").is_ok_and(|value| !value.is_empty() && value != "0");
        #[cfg(target_arch = "wasm32")]
        let issue2424_ts_enabled = false;
        let issue2424_ts_started = issue2424_ts_enabled.then(std::time::Instant::now);
        let mut issue2424_prof = Issue2424TypesetProfile::default();

        let layout = PageLayoutInfo::from_page_def(page_def, column_def, self.dpi);
        // [#2403] 소스분기 프로파일 — 엔진(Cell)과 state 에 한 번에 배선.
        self.profile.set(profile);
        // [#5854] 통짜 합성 LINE_SEG 사다리 판정 — 구역당 한 번.
        self.uniform_filler_ladder
            .set(crate::renderer::stored_line_ladder_is_uniform_filler(
                paragraphs, styles,
            ));
        // [#6175] 용지/쪽 기준 어울림 개체의 폭과 세로 band - 구역당 한 번.
        *self.float_carve_evidence.borrow_mut() =
            crate::renderer::float_placement::paper_or_page_float_carve_evidence(paragraphs);
        let col_count = column_def.column_count.max(1);
        let default_footnote_shape = FootnoteShape::default();
        let footnote_shape = footnote_shape.unwrap_or(&default_footnote_shape);
        let footnote_separator_overhead = footnote_separator_overhead_px(footnote_shape, self.dpi);
        let footnote_between_notes_margin =
            footnote_between_notes_margin_px(footnote_shape, self.dpi);
        let footnote_safety_margin = hwpunit_to_px(3000, self.dpi);
        // [Task #1007] variant cross-paragraph vpos reset THRESHOLD 계산용 body height (HU)
        let body_height_hu_for_variant: i32 = if profile.hwp3_layout() {
            page_def.height.saturating_sub(
                page_def
                    .margin_top
                    .saturating_add(page_def.margin_bottom)
                    .saturating_add(page_def.margin_header)
                    .saturating_add(page_def.margin_footer),
            ) as i32
        } else {
            0
        };
        // [Task #1007] 이전 paragraph 인덱스 (variant vpos reset 감지용)
        let mut variant_prev_para_idx: Option<usize> = None;

        let mut st = TypesetState::new(
            layout,
            col_count,
            section_index,
            footnote_separator_overhead,
            footnote_between_notes_margin,
            footnote_safety_margin,
            column_def.column_type,
        );
        st.initialize_source(
            hide_empty_line,
            profile,
            paragraphs
                .iter()
                .any(|p| p.line_segs.iter().any(|ls| !is_synthetic_line_seg(ls))),
            skip_spacing_before_prededuct,
        );
        // [#2279 OMIT-eager] 기계생성 압축 ladder 문서군 사전 판별 — lazy 판별
        // (#2383, 빈 host 표 성장 시점)보다 앞서 페이지말 fit 규칙에 적용되도록
        // 구역 시작 시점에 확정한다. lazy 경로는 사전 스캔 미검출 형상의
        // fallback 으로 유지. HWP3-origin 계열(변환본·page tolerance 문서)은
        // 변환기 좌표 관례가 달라 저장 흐름 신뢰가 정답 — 대상이 아니다
        // (sample16 64→65 +1 실측, #2158 핀).
        if profile.hwpx_stored_layout()
            && !profile.hwp3_layout()
            && !profile.hwp3_native_layout()
            && !hwp3_origin_page_tolerance
        {
            st.classify_omitted_spacing(ladder_spacing_omitted_signature(
                paragraphs, styles, self.dpi,
            ));
            if std::env::var("RHWP_DIAG_OMITSIG").is_ok() {
                eprintln!(
                    "DIAG_OMITSIG sec={} eager={}",
                    section_index, st.omit_fresh_recalc_doc
                );
            }
            // 사전 판별 문서군은 #2391 분할 진입 full-advance 계보도 동일하게
            // 적용 대상 — lazy 판별(#2383)의 상위 집합으로 승격만 한다.
            if st.omit_fresh_recalc_doc {
                st.mark_stored_spacing_omitted();
            }
        }
        st.initialize_zone_spacing(column_def_design_spacing_px(column_def, self.dpi));

        // 머리말/꼬리말/쪽 번호/새 번호/감추기 컨트롤 수집
        let (hf_entries, page_number_pos) =
            Self::collect_header_footer_controls(paragraphs, section_index);
        // [#2559] 조건부(Even/Odd)까지 포함해 어떤 꼬리말이라도 정의돼 있으면
        // 밴드가 점유될 수 있다. 완전히 비어 있는 구역에서만 각주 회수를 허용한다.
        st.record_footer_presence(!hf_entries.iter().any(|(_, _, is_header, _)| !is_header));
        // [#2668 진단] 밴드 회수 판정 관측 — 동작 불변.
        // "꼬리말 위치 쪽 번호(PageNumberPos)가 밴드를 점유하므로 회수 대상에서 빼야 한다"는
        // 판별자 가설을 이 출력으로 반증했다(회귀군·개선군 양쪽에 고루 존재).
        // 경위와 실측은 mydocs/report/task_2668_footnote_band_measurement.md 참조.
        if std::env::var("RHWP_DIAG_FBAND").is_ok() {
            eprintln!(
                "DIAG_FBAND sec={} footer_ctrl={} pnum_pos={:?} reclaim={}",
                section_index,
                hf_entries.iter().any(|(_, _, is_header, _)| !is_header),
                page_number_pos.as_ref().map(|p| p.position),
                st.section_has_no_footer
            );
        }

        // 가시 콘텐츠(텍스트 또는 컨트롤 보유)를 가진 마지막 문단 인덱스. 이 뒤의 빈 문단들은
        // 문서 말미의 trailing 빈 문단이라, co-anchored 자리차지 표가 페이지를 채운 경우에 한해
        // 현재 page 잔여를 초과하면 새 page 를 만들지 않고 흡수한다(아래 trailing-empty 가드).
        let last_content_para_idx: Option<usize> = paragraphs
            .iter()
            .enumerate()
            .rev()
            .find(|(_, p)| !(p.text.is_empty() && p.controls.is_empty()))
            .map(|(i, _)| i);

        if let Some(started) = issue2424_ts_started {
            issue2424_prof.setup = started.elapsed();
        }
        let issue2424_loop_started = issue2424_ts_enabled.then(std::time::Instant::now);

        for (para_idx, para) in paragraphs.iter().enumerate() {
            // [Task #1753] 지연 이월 표 직전에 선행 채움(prefill)으로 이미 배치된 문단 스킵.
            if st.prefilled_paras.contains(&para_idx) {
                continue;
            }
            // 후속 본문이 사이에 없는 새 표 문단은 이전 묶음의 일부가 아니다.
            // 이전 문단의 후행 표를 먼저 완료한다. 새 문단의 명시적 쪽/단 나눔과
            // 저장 vpos를 적용하기 전이어야 이전 표가 새 문단 뒤로 밀리지 않는다.
            // 제목/본문이 사이에 있는 float 흐름은 기존 후행 flush 계약을 유지한다.
            if st.has_deferred_table_controls() && self.paragraph_has_table(para) {
                self.flush_deferred_table_controls(
                    &mut st,
                    paragraphs,
                    composed,
                    styles,
                    measured_tables,
                    DeferredTableFlushPoint::BeforeTableParagraph(para_idx),
                );
            }
            // [#6132] 저장 vpos 가 쪽 본문을 넘고 바로 다음 문단이 되감기면,
            // 한글은 이 문단부터 다음 쪽에 둔 것이다. 다만 그 형상만으로는 부족하다 —
            // 같은 형상이 문단을 쪽 안에 그대로 두는 문서들에도 흔하게 나온다
            // (실측 후보: 2025 행정업무편람 26곳 · 2070 시장구조조사 13곳 ·
            // 2019 벤처투자 3곳 · hwp3-sample16 10곳). 그래서 세 신호를 **함께**
            // 요구한다.
            //
            // 156482639 7쪽: pi=102 '참고3' 표 vpos=73760 은 본문 73,335HU(977.8px)를
            // 넘고 pi=103 이 3790 으로 되감긴다. 한글은 둘 다 8쪽 첫머리에 두는데
            // rhwp 는 7쪽 잔여(974.5px)에 욱여넣어 8쪽이 56.8px 비었다.
            //
            // 기존 #3837 되감김 규칙은 되감긴 **다음** 문단(pi=103)에만 걸리고, 그마저
            // 그 문단이 잔여에 들어가면 분할 루프가 그대로 현재 쪽에 놓는다. 넘긴
            // 주체인 pi=102 자신은 표 경로라 그 판정을 아예 지나지 않는다.
            if st.col_count == 1 && !st.current_items.is_empty() {
                let own_stored_vpos = para
                    .line_segs
                    .iter()
                    .find(|seg| !is_synthetic_line_seg(seg))
                    .map(|seg| seg.vertical_pos);
                let next_stored_vpos = paragraphs
                    .get(para_idx + 1)
                    .and_then(|next| {
                        next.line_segs
                            .iter()
                            .find(|seg| !is_synthetic_line_seg(seg))
                    })
                    .map(|seg| seg.vertical_pos);
                if let (Some(own), Some(next)) = (own_stored_vpos, next_stored_vpos) {
                    // ① **표를 단 문단**만. #3837 되감김 규칙이 닿지 못하는 계보가
                    //    정확히 이것이고(표 경로는 그 판정을 지나지 않는다), 위 네
                    //    문서의 후보 52곳 중 표를 단 문단은 sample16 한 곳뿐이다.
                    let hosts_table = para
                        .controls
                        .iter()
                        .any(|c| matches!(c, crate::model::control::Control::Table(_)));
                    // ② 저장 자리가 본문 바닥을 **근소하게** 넘을 것. 크게 넘는 사다리는
                    //    쪽 리셋이 아니라 구역 누적 좌표계라 판정의 전제가 깨진다
                    //    (sample16 pi=738: 3567.4px / 가용 971.3px).
                    let own_px = hwpunit_to_px(own, self.dpi);
                    let body_bottom = st.base_available_height();
                    let overflows_body_narrowly =
                        own > 0 && own_px > body_bottom && own_px - body_bottom <= MIN_TOP_KEEP_PX;
                    // ③ 이 쪽에 조각이 될 만한 잔여가 남아 있을 것. 잔여가 그보다 작으면
                    //    통상 fit 이 어차피 다음 쪽으로 넘긴다 — 거기서 또 끊으면 빈 쪽이
                    //    하나 더 생긴다(3075729 #1880: 잔여 10.4px, 13쪽 → 14쪽).
                    let page_room_left = body_bottom - st.current_height;
                    if hosts_table
                        && overflows_body_narrowly
                        && next < own
                        && page_room_left > MIN_TOP_KEEP_PX
                    {
                        if std::env::var("RHWP_DIAG_VPOS_OVF").is_ok() {
                            eprintln!(
                                "[VPOS_OVF] pi={} own={} ({:.1}px) next={} avail={:.1} cur_h={:.1} items={}",
                                para_idx,
                                own,
                                own_px,
                                next,
                                body_bottom,
                                st.current_height,
                                st.current_items.len()
                            );
                        }
                        st.advance_column_or_new_page();
                    }
                }
            }
            // [#4533 HWP3] 자리차지 밴드 비예약 판별용 — 표 경로 포함 전 문단 공통.
            st.observe_following_paragraph(
                paragraphs.get(para_idx + 1).and_then(|p| {
                    p.line_segs.first().map(|seg| {
                        p.source_line_seg_vertical_pos
                            .as_ref()
                            .and_then(|source| source.first().copied())
                            .unwrap_or(seg.vertical_pos)
                    })
                }),
                paragraphs
                    .get(para_idx + 1)
                    .is_some_and(|p| para_is_empty_topbottom_table_anchor(p)),
                paragraphs
                    .get(para_idx + 1)
                    .is_some_and(|p| para_has_non_whitespace_text(p) && p.controls.is_empty()),
            );
            if std::env::var("RHWP_FLOW_DBG").is_ok() {
                eprintln!(
                    "FLOW_DBG pi={} page={} cur_h={:.1}",
                    para_idx,
                    st.pages.len(),
                    st.current_height
                );
            }
            // 표 컨트롤 감지
            let has_table = self.paragraph_has_table(para);
            if std::env::var("RHWP_DIAG_FLOW").is_ok() {
                eprintln!("DIAG_ROUTE pi={} has_table={}", para_idx, has_table);
            }

            let Some(boundary) = self.prepare_paragraph_boundary(
                &mut st,
                para_idx,
                para,
                paragraphs,
                styles,
                page_def,
                column_def,
                profile,
                variant_prev_para_idx,
                body_height_hu_for_variant,
                force_break_before,
                has_table,
            ) else {
                continue;
            };

            self.apply_stored_paragraph_boundary(
                &mut st,
                para_idx,
                para,
                paragraphs,
                styles,
                has_table,
                hwp3_origin_page_tolerance,
                profile,
                body_height_hu_for_variant,
                measured_tables,
                &boundary,
            );

            if self.absorb_section_tail(
                &mut st,
                para_idx,
                para,
                paragraphs,
                styles,
                has_table,
                last_content_para_idx,
                boundary.para_style_break,
                boundary.force_page_break,
            ) {
                continue;
            }

            // [Task #362] 어울림(Square wrap) 표 옆 paragraph 흡수.
            // Paginator engine.rs:288-320 동일 시멘틱.
            // 직전에 처리한 Square wrap 표의 (cs, sw) 와 동일한 LINE_SEG 를 가진
            // 후속 paragraph 는 표 옆에 배치되므로 height 소비 없이 wrap_around_paras 에 기록.
            let issue2424_wrap_started = issue2424_ts_enabled.then(std::time::Instant::now);
            let issue2424_wrap_absorbed = self.typeset_wrap_around_paragraph(
                &mut st,
                para,
                paragraphs,
                para_idx,
                has_table,
                page_def,
                composed.get(para_idx),
                styles,
            );
            Issue2424TypesetProfile::add(&mut issue2424_prof.wrap_around, issue2424_wrap_started);
            if issue2424_wrap_absorbed {
                continue;
            }

            st.ensure_page();

            self.keep_heading_with_following_block(&mut st, para_idx, para, paragraphs);

            let picture_host_origin = (st.pages.len(), st.current_column, st.current_height);
            let native_hwp5_footnote_break = self.place_paragraph_flow(
                &mut st,
                para_idx,
                para,
                paragraphs,
                composed,
                styles,
                measured_tables,
                page_def,
                has_table,
                issue2424_ts_enabled,
                &mut issue2424_prof,
            );

            self.finish_paragraph_anchor_state(
                &mut st,
                composed,
                styles,
                measured_tables,
                page_def,
                para_idx,
                para,
                paragraphs,
                has_table,
                issue2424_ts_enabled,
                &mut issue2424_prof,
            );

            self.place_paragraph_controls(
                &mut st,
                para_idx,
                para,
                paragraphs,
                styles,
                section_index,
                has_table,
                picture_host_origin,
                native_hwp5_footnote_break,
            );

            // [Task #1007] variant vpos reset 감지용 prev_para_idx 갱신
            variant_prev_para_idx = Some(para_idx);
        }

        let issue2424_loop_elapsed = issue2424_loop_started
            .map(|started| started.elapsed())
            .unwrap_or_default();
        let issue2424_inloop_flush = issue2424_prof.deferred_flush.0;

        // source tail에 뒤따르는 본문이 없어 자연 page break가 일어나지 않은 경우에도,
        // deferred Square picture는 현재 쪽 FootnoteArea에 겹치지 않고 독립한 다음 physical
        // page를 가져야 한다. 일반 흐름을 되감지 않고 picture PageItem만 drain한다.
        if !st.deferred_next_page_square_pictures.is_empty() {
            st.force_new_page();
        }

        // [미주 배치 — Hancom EndnoteEndOfSection/EndnoteEndOfDocument]
        let issue2424_endnote_started = issue2424_ts_enabled.then(std::time::Instant::now);
        self.typeset_section_endnotes(
            &mut st,
            paragraphs,
            composed,
            styles,
            section_index,
            page_def,
            measured_tables,
            endnote_shape,
            &endnote_deferral,
        );
        if let Some(started) = issue2424_endnote_started {
            issue2424_prof.endnotes = started.elapsed();
        }

        // 마지막 항목 처리
        let issue2424_final_flush_started = issue2424_ts_enabled.then(std::time::Instant::now);
        self.flush_deferred_table_controls(
            &mut st,
            paragraphs,
            composed,
            styles,
            measured_tables,
            DeferredTableFlushPoint::SectionEnd,
        );
        Issue2424TypesetProfile::add(
            &mut issue2424_prof.deferred_flush,
            issue2424_final_flush_started,
        );
        if !st.current_items.is_empty() {
            st.flush_column_always();
        }
        st.ensure_page();

        // [#1955] 글뒤로 표 후행 빈 문단 보류 흡수 부착 — 페이지 확정 후
        // anchor 표의 첫 fragment 가 있는 단에 소급 기록 (한글: 글뒤로 표는
        // 플로우를 소비하지 않으므로 후행 문단이 anchor 쪽에 남음).
        st.attach_pending_behind_absorptions();

        // 한컴은 문서의 마지막에 남은 빈 문단 묶음 때문에 별도 빈 쪽을 만들지
        // 않는다. 일반 흐름에서는 이 문단들이 앞 쪽의 tail로 흡수되지만, 큰 표의
        // 마지막 fragment 뒤에서는 저장 vpos가 다음 쪽을 가리킬 수 있다. 그 경우
        // rhwp가 100HU짜리 빈 line-seg만 담은 페이지를 확정하면 실제 출력보다 한
        // 쪽이 늘어난다 (#3637 HWP 2020 oracle 31쪽 → 32쪽). 명시적인 page/section
        // break나 가시 컨트롤은 보존하고, 정말 빈 문단만 있는 마지막 쪽만 버린다.
        st.discard_terminal_blank_only_page(paragraphs);

        // 페이지 번호 + 머리말/꼬리말 할당
        st.finalize_pages(&hf_entries, &page_number_pos, paragraphs);

        if let Some(started) = issue2424_ts_started {
            let total = started.elapsed();
            let loop_other = issue2424_loop_elapsed
                .saturating_sub(issue2424_prof.wrap_around.0)
                .saturating_sub(issue2424_prof.text_para.0)
                .saturating_sub(issue2424_prof.table_para.0)
                .saturating_sub(issue2424_inloop_flush)
                .saturating_sub(issue2424_prof.para_tail.0);
            let post = total
                .saturating_sub(issue2424_prof.setup)
                .saturating_sub(issue2424_loop_elapsed)
                .saturating_sub(issue2424_prof.endnotes);
            eprintln!(
                "RHWP_2424_TYPESET_PROFILE sec={} pages={} paras={} total_ms={:.2} \
                 setup={:.2} loop={:.2} [wrap={:.2}/{} text={:.2}/{} table={:.2}/{} \
                 flush={:.2}/{} tail={:.2}/{} other={:.2}] endnotes={:.2} post={:.2}",
                section_index,
                st.pages.len(),
                paragraphs.len(),
                Issue2424TypesetProfile::ms(total),
                Issue2424TypesetProfile::ms(issue2424_prof.setup),
                Issue2424TypesetProfile::ms(issue2424_loop_elapsed),
                Issue2424TypesetProfile::ms(issue2424_prof.wrap_around.0),
                issue2424_prof.wrap_around.1,
                Issue2424TypesetProfile::ms(issue2424_prof.text_para.0),
                issue2424_prof.text_para.1,
                Issue2424TypesetProfile::ms(issue2424_prof.table_para.0),
                issue2424_prof.table_para.1,
                Issue2424TypesetProfile::ms(issue2424_prof.deferred_flush.0),
                issue2424_prof.deferred_flush.1,
                Issue2424TypesetProfile::ms(issue2424_prof.para_tail.0),
                issue2424_prof.para_tail.1,
                Issue2424TypesetProfile::ms(loop_other),
                Issue2424TypesetProfile::ms(issue2424_prof.endnotes),
                Issue2424TypesetProfile::ms(post),
            );
        }

        st.into_result()
    }
}
