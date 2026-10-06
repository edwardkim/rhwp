//! 구역 문단 처리의 place_paragraph_flow 단계. 조건·예약·발행 순서를 유지한다.
use crate::renderer::typeset::{
    hwpunit_to_px, is_synthetic_line_seg,
    native_hwp5_circled_rowbreak_table_heading_requires_fresh_page,
    native_hwp5_first_footnote_overlap_break_line, notes,
    original_hwpx_tac_filled_page_keeps_short_trail, synth_square_wrap_rects, ComposedParagraph,
    Control, Issue2424TypesetProfile, MeasuredTable, PageDef, PageItem, Paragraph,
    ResolvedStyleSet, TypesetEngine, TypesetState,
};
impl TypesetEngine {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn place_paragraph_flow(
        &self,
        st: &mut TypesetState,
        para_idx: usize,
        para: &Paragraph,
        paragraphs: &[Paragraph],
        composed: &[ComposedParagraph],
        styles: &ResolvedStyleSet,
        measured_tables: &[MeasuredTable],
        page_def: &PageDef,
        has_table: bool,
        issue2424_ts_enabled: bool,
        issue2424_prof: &mut Issue2424TypesetProfile,
    ) -> Option<notes::footnotes::boundary::NativeHwp5FootnoteBreak> {
        let issue2424_branch_started = issue2424_ts_enabled.then(std::time::Instant::now);
        let mut native_hwp5_footnote_break = None;
        if !has_table {
            // --- 핵심: format → fits → place/split ---
            let col_w = st
                .layout
                .column_areas
                .get(st.current_column as usize)
                .map(|a| a.width)
                .unwrap_or(st.layout.body_area.width);
            // NO_LS 문서의 Square 그림 어울림 합성.
            // 저장 lineseg 가 없으면 어울림 기계(저장 cs/sw 매칭)가 서지 않아 텍스트가
            // 그림 위로 흐른다. 그림 사각형을 페이지 공간으로 기억해 두고(가로
            // 오프셋으로 앵커 단 밖에 놓이는 형상 포함), 현재 단과 교차하는 NO_LS
            // 문단을 감폭으로 측정하고 wrap_anchors 로 성형한다.
            let para_no_ls = !para.line_segs.iter().any(|seg| {
                seg.tag & crate::model::paragraph::LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0
            });
            let mut col_w = col_w;
            let synth_col_area = st
                .layout
                .column_areas
                .get(st.current_column as usize)
                .map(|a| (a.x, a.width));
            if st.profile.hwp5_stored_pagination_layout() && para_no_ls {
                if let Some((cax, _)) = synth_col_area {
                    let rects = synth_square_wrap_rects(para, cax, st.current_height, self.dpi);
                    st.extend_synthetic_wrap_rects(rects);
                }
                if let Some((cax, caw)) = synth_col_area {
                    if !st.wrap_synth_rects.is_empty() {
                        let cur = st.current_height;
                        let mut cs_px: f64 = 0.0;
                        let mut right_cut: f64 = caw;
                        for &(x0, x1, top, bottom) in &st.wrap_synth_rects {
                            // 시작 판정은 한 줄 높이(24px)까지 선행 허용 — 앵커 문단과
                            // 옆 단 텍스트의 y 가 정확히 일치하지 않는 저작 형상에서
                            // 항목 첫 줄이 미성형으로 그림과 겹치던 잔차 완화. 그보다
                            // 아래에서 시작하는 개체는 줄 단위 밴드 배제(아래 post-format
                            // 등록)가 담당한다 — 문단 전체 감폭은 첫 줄까지 밀어낸다.
                            // 소형 아이콘이 문단 첫 줄보다 아래에서 시작하면 문단
                            // 전체 감폭이 첫 줄까지 밀어낸다 — 줄 단위 밴드 배제
                            // (post-format 등록)가 담당하도록 여기서는 제외한다.
                            if bottom - top <= 40.0 && top > cur + 8.0 {
                                continue;
                            }
                            // 대형 개체는 문단 시작이 개체보다 위면 감폭하지 않는다 —
                            // 개체 위에 있는 제목 줄 문단이 개체 옆으로 밀리는 오폭 방지
                            // (선행 허용 창은 판정/그리기 눈금 오차보다 커서 위험).
                            if bottom - top > 40.0 && top > cur {
                                continue;
                            }
                            if cur + 40.0 < top || cur >= bottom - 1.0 {
                                continue;
                            }
                            let ox0 = x0.max(cax);
                            let ox1 = x1.min(cax + caw);
                            if ox1 - ox0 < 8.0 {
                                continue;
                            }
                            let left_gap = ox0 - cax;
                            let right_gap = cax + caw - ox1;
                            if right_gap >= left_gap {
                                cs_px = cs_px.max(ox1 - cax + 3.8);
                            } else {
                                right_cut = right_cut.min(left_gap - 3.8);
                            }
                        }
                        let sw_px = (right_cut - cs_px).max(0.0);
                        if (cs_px > 0.5 || right_cut < caw - 0.5) && sw_px >= caw * 0.25 {
                            st.register_following_wrap_anchor(
                                para_idx,
                                crate::renderer::pagination::WrapAnchorRef {
                                    anchor_para_index: para_idx,
                                    anchor_cs: crate::renderer::px_to_hwpunit(cs_px, self.dpi),
                                    anchor_sw: crate::renderer::px_to_hwpunit(sw_px, self.dpi),
                                    anchor_image_margin_right: 0,
                                    band_y_range: None,
                                },
                            );
                            let margin_left = styles
                                .para_styles
                                .get(para.para_shape_id as usize)
                                .map_or(0.0, |style| style.margin_left);
                            col_w = crate::renderer::synthetic_wrap_column_width(
                                col_w,
                                margin_left,
                                st.current_column_wrap_anchors.get(&para_idx),
                                self.dpi,
                            );
                        }
                    }
                }
            }
            // [#6175] devel 의 known-square-band 인자를 그대로 전달한다 —
            // 위에서 좁힌 col_w(합성 어울림 감폭)와 함께 쓴다.
            let formatted = self.format_paragraph_with_known_square_band(
                para,
                composed.get(para_idx),
                styles,
                Some(col_w),
                st.wrap_around_derived_band
                    || st.current_column_wrap_anchors.contains_key(&para_idx),
            );
            // 줄 단위 어울림 배제: 문단 시작은 개체 위이지만 뒷줄이 개체 사각형과
            // 교차하는 형상(출석부) — 문단 전체 감폭 대신 교차 y 밴드를 anchor 에
            // 실어 layout 이 교차하는 줄만 감폭한다.
            if st.profile.hwp5_stored_pagination_layout()
                && para_no_ls
                && !st.current_column_wrap_anchors.contains_key(&para_idx)
                && !st.wrap_synth_rects.is_empty()
            {
                if let Some((cax, caw)) = synth_col_area {
                    let cur = st.current_height;
                    let para_bottom = cur + formatted.height_for_fit;
                    for &(x0, x1, top, bottom) in &st.wrap_synth_rects {
                        // 소형 아이콘이 문단 첫 줄보다 아래에서 시작해 문단 안에서
                        // 끝나는 형상만 — 문단 전체 감폭 판정과 상보.
                        if bottom - top > 40.0 || top <= cur + 8.0 || top >= para_bottom {
                            continue;
                        }
                        let ox0 = x0.max(cax);
                        let ox1 = x1.min(cax + caw);
                        if ox1 - ox0 < 8.0 {
                            continue;
                        }
                        let left_gap = ox0 - cax;
                        let right_gap = cax + caw - ox1;
                        if right_gap < left_gap {
                            continue; // 우측 개체 형상은 미지원(현행 유지)
                        }
                        let cs_px = ox1 - cax + 3.8;
                        let sw_px = caw - cs_px;
                        if sw_px < caw * 0.25 {
                            continue;
                        }
                        st.register_following_wrap_anchor(
                            para_idx,
                            crate::renderer::pagination::WrapAnchorRef {
                                anchor_para_index: para_idx,
                                anchor_cs: crate::renderer::px_to_hwpunit(cs_px, self.dpi),
                                anchor_sw: crate::renderer::px_to_hwpunit(sw_px, self.dpi),
                                anchor_image_margin_right: 0,
                                band_y_range: Some((top - cur, bottom - cur)),
                            },
                        );
                        break;
                    }
                }
            }
            native_hwp5_footnote_break =
                native_hwp5_first_footnote_overlap_break_line(&st, para, &formatted, self.dpi);
            let is_last_in_section = para_idx + 1 == paragraphs.len();
            if native_hwp5_circled_rowbreak_table_heading_requires_fresh_page(
                &st, para, &formatted, paragraphs, para_idx, self.dpi,
            ) {
                st.advance_column_or_new_page();
            }
            // [Task #1027 Stage D] fit 직전 vpos 스냅으로 누적 drift 제거 (렌더러 정합).
            self.vpos_snap_current_height(
                st,
                para_idx,
                paragraphs,
                styles,
                formatted.spacing_before,
            );
            if let Some(crate::model::control::Control::Table(prev_table)) =
                st.current_items.last().and_then(|item| match item {
                    PageItem::Table {
                        para_index,
                        control_index,
                    } => paragraphs
                        .get(*para_index)
                        .and_then(|p| p.controls.get(*control_index)),
                    _ => None,
                })
            {
                let next_h = paragraphs.get(para_idx + 1).map(|next| {
                    next.line_segs
                        .iter()
                        .map(|s| {
                            hwpunit_to_px(
                                s.line_height.saturating_add(s.line_spacing) as i32,
                                self.dpi,
                            )
                        })
                        .sum::<f64>()
                });
                let remaining = st.available_height() - st.current_height;
                let table_h = hwpunit_to_px(prev_table.common.height as i32, self.dpi);
                if original_hwpx_tac_filled_page_keeps_short_trail(
                    !st.profile.hwp5_stored_pagination_layout(),
                    prev_table,
                    table_h,
                    st.layout.body_area.height,
                    remaining,
                    formatted.height_for_fit.max(formatted.total_height),
                    next_h.unwrap_or(0.0),
                ) {
                    st.advance_column_or_new_page();
                }
            }
            if !self.typeset_inline_flow(st, para_idx, para, styles, measured_tables) {
                self.typeset_paragraph(
                    st,
                    para_idx,
                    para,
                    &formatted,
                    paragraphs,
                    styles,
                    is_last_in_section,
                );
            }
            // HWPX가 수식 인라인 개체를 포함한 문단의 line_seg 높이를 실제
            // 조판보다 작게 저장하는 경우, 다음 문단의 양수 VPOS가 같은 물리
            // 쪽의 정확한 흐름 끝을 가리킨다. 일반 문단과 표에는 적용하지 않고,
            // 수식 전용 host가 같은 본문 영역의 다음 앵커를 넘겨 소비했을
            // 때에만 source flow 끝으로 되돌린다.
            let saved_equation_host_next_anchor = st.profile.hwpx_stored_layout()
                && st.col_count == 1
                && !para.controls.is_empty()
                && para
                    .controls
                    .iter()
                    .all(|ctrl| matches!(ctrl, Control::Equation(_)));
            if saved_equation_host_next_anchor {
                // 수식 host의 조판 높이를 다음 저장 VPOS로 되돌릴 근거는
                // 수식 control과 같은 위치의 source LineSeg가 서로 다른 높이를
                // 기록한 경우다. 두 높이가 일치하면 다음 앵커는 일반 본문 흐름의
                // 정상 간격이므로 이를 압축하면 후속 쪽 owner를 앞당긴다.
                let saved_equation_line_height_mismatch = para.line_segs.len()
                    == para.controls.len()
                    && para
                        .line_segs
                        .iter()
                        .zip(&para.controls)
                        .any(|(seg, ctrl)| {
                            matches!(ctrl, Control::Equation(equation)
                                if !is_synthetic_line_seg(seg)
                                    && seg.line_height > 0
                                    && equation.common.height > 0
                                    && seg.line_height != equation.common.height as i32)
                        });
                let source_host = para
                    .line_segs
                    .iter()
                    .rev()
                    .find(|seg| !is_synthetic_line_seg(seg));
                let source_next = paragraphs.get(para_idx + 1).and_then(|next| {
                    next.line_segs
                        .iter()
                        .find(|seg| !is_synthetic_line_seg(seg))
                });
                if let (Some(base), Some(source_host), Some(source_next)) =
                    (st.vpos_page_base, source_host, source_next)
                {
                    let source_host_y = st.vpos_col_anchor
                        + hwpunit_to_px(
                            (source_host.vertical_pos as i64 - base as i64) as i32,
                            self.dpi,
                        );
                    let source_next_y = st.vpos_col_anchor
                        + hwpunit_to_px(
                            (source_next.vertical_pos as i64 - base as i64) as i32,
                            self.dpi,
                        );
                    if saved_equation_line_height_mismatch
                        && source_next.vertical_pos > source_host.vertical_pos
                        && source_host_y >= 0.0
                        && source_next_y <= st.base_available_height() + 0.5
                        && st.current_height > source_next_y
                    {
                        st.align_flow_to(source_next_y);
                    }
                }
            }
        } else {
            // 표 문단: Phase 2에서 전환 예정. 현재는 기존 방식 호환용 stub.
            self.typeset_table_paragraph(
                st,
                para_idx,
                para,
                composed.get(para_idx),
                paragraphs.get(para_idx + 1),
                styles,
                measured_tables,
                page_def,
                paragraphs,
                composed,
            );
        }
        Issue2424TypesetProfile::add(
            if has_table {
                &mut issue2424_prof.table_para
            } else {
                &mut issue2424_prof.text_para
            },
            issue2424_branch_started,
        );

        native_hwp5_footnote_break
    }
}
