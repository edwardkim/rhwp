//! 구역 문단 처리의 prepare_paragraph_boundary 단계. 조건·예약·발행 순서를 유지한다.
use crate::renderer::typeset::{
    column_def_design_spacing_px, columndef_separator_between_floating_overlays,
    empty_table_carrier_column_break_before_page_table,
    hwp5_origin_redundant_pagehide_break_marker, native_hwp5_figure_table_overlay_guide_empty,
    para_has_visible_text, para_is_columndef_only_separator, para_is_non_tac_overlay_table_anchor,
    para_is_post_paper_page_square_table_scaffold, para_is_pre_paper_page_square_table_scaffold,
    ColumnBreakType, ColumnDef, ColumnType, Control, PageDef, PageItem, PageLayoutInfo, Paragraph,
    ResolvedStyleSet, TypesetEngine, TypesetState,
};
pub(super) struct ParagraphBoundary<'a> {
    pub para_style: Option<&'a crate::renderer::style_resolver::ResolvedParaStyle>,
    pub para_style_break: bool,
    pub force_page_break: bool,
    pub overlay_columndef_separator_break: bool,
}
impl TypesetEngine {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn prepare_paragraph_boundary<'a>(
        &self,
        st: &mut TypesetState,
        para_idx: usize,
        para: &Paragraph,
        paragraphs: &[Paragraph],
        styles: &'a ResolvedStyleSet,
        page_def: &PageDef,
        column_def: &ColumnDef,
        profile: crate::model::provenance::LayoutCompatibilityProfile,
        variant_prev_para_idx: Option<usize>,
        body_height_hu_for_variant: i32,
        force_break_before: &std::collections::HashSet<usize>,
        has_table: bool,
    ) -> Option<ParagraphBoundary<'a>> {
        // [Task #702] 새 ColumnDef 검출. shortcut.hwp p2/p3 파일/미리보기/편집 등은
        // [쪽나누기]+단정의:1단 (header) → [단나누기]+단정의:2단 (content) 패턴 사용.
        // [다단나누기] 외에도 Page/Column break 의 ColumnDef 차이도 zone 재정의 신호로 인식.
        let new_col_def_opt: Option<ColumnDef> = para.controls.iter().find_map(|c| {
            if let Control::ColumnDef(cd) = c {
                Some(cd.clone())
            } else {
                None
            }
        });
        let has_diff_col_def = new_col_def_opt
            .as_ref()
            .map(|cd| {
                cd.column_count.max(1) != st.col_count
                    || cd.column_type != st.current_zone_column_type
            })
            .unwrap_or(false);

        let is_terminal_empty_column_break = para_idx + 1 == paragraphs.len()
            && st.col_count == 1
            && para.column_type == ColumnBreakType::Column
            && !has_diff_col_def
            && para.controls.is_empty()
            && para
                .text
                .replace(|c: char| c.is_control(), "")
                .trim()
                .is_empty();
        if is_terminal_empty_column_break {
            // 1단 구역 끝의 빈 단나누기 문단은 다음 단/밴드가 없으므로
            // 별도 빈 페이지를 만들지 않는다.
            return None;
        }

        let is_overlay_guide_empty_para = st.col_count > 1
            && para.column_type == ColumnBreakType::None
            && para.controls.is_empty()
            && !para_has_visible_text(para)
            && para
                .line_segs
                .first()
                .is_some_and(|seg| seg.vertical_pos > 0)
            && (0..para_idx)
                .rev()
                .find(|&idx| {
                    paragraphs
                        .get(idx)
                        .is_some_and(|p| !p.controls.is_empty() || para_has_visible_text(p))
                })
                .and_then(|idx| paragraphs.get(idx))
                .is_some_and(para_is_non_tac_overlay_table_anchor);
        if is_overlay_guide_empty_para {
            // 글앞/글뒤 표 뒤의 빈 guide 줄은 떠 있는 개체의 위치 보조값이며,
            // 다단 flow 높이를 소비하지 않는다.
            return None;
        }

        let is_native_hwp5_figure_table_overlay_guide_empty = profile
            .hwp5_stored_pagination_layout()
            && native_hwp5_figure_table_overlay_guide_empty(para_idx, para, paragraphs);
        if is_native_hwp5_figure_table_overlay_guide_empty {
            // `그림 67` 같은 2×1 그림 표는 선언한 표 높이로 이미 flow를 예약한다.
            // 표 paint span 내부의 빈 HWP line은 다시 높이를 소비하면 안 되지만,
            // PI↔page 진단에서 문단 자체는 계속 추적 가능해야 하므로 0-height item으로
            // 남긴다.
            st.hide_empty_paragraph(para_idx);
            st.append_item(PageItem::FullParagraph {
                para_index: para_idx,
            });
            return None;
        }

        // 다단 나누기
        if para.column_type == ColumnBreakType::MultiColumn {
            self.process_multicolumn_break(st, para_idx, paragraphs, page_def);
        }

        // 단 나누기
        // [#2019 ②] 부동 개체 전용 빈 앵커 문단(별지 서식)의 단나누기(새 ColumnDef 없음)는
        // 무시한다. 흐름 텍스트가 없고 개체가 절대위치 배치이므로, 단일 단에서 단나누기를
        // 페이지 분할로 변환하면 한글과 달리 폼 요소마다 새 쪽이 생겨 과분할된다(74312).
        // 새 ColumnDef 를 동반하는 단나누기(zone 전환)는 ③(process_multicolumn_break)에서 처리.
        //
        // 추가로, 별지 서식은 폼 사이를 "빈 문단 + 단나누기 + (같은 1단) ColumnDef" 구분자로
        // 나눈다. 이 구분자는 has_diff_col_def=false(단 수 불변)라 위 branch 를 타는데, 단일
        // 단에서 페이지 분할로 변환되어 폼마다 허위 페이지가 생긴다. 텍스트 없이 ColumnDef
        // 만 든 단일 단 단나누기도 억제한다(한글은 이 구분자로 쪽을 나누지 않음).
        let columndef_only_break = para_is_columndef_only_separator(para);
        let empty_columndef_only_break =
            !has_diff_col_def && st.col_count <= 1 && columndef_only_break;
        // [#2019 v3] 별지 서식은 같은 2단 ColumnDef 를 중간중간 반복해 부동
        // overlay 글상자 묶음을 구분한다. 앞뒤가 모두 부동 overlay 앵커이면
        // 실제 텍스트 column advance 로 보지 않는다.
        let overlay_columndef_separator_break = !has_diff_col_def
            && columndef_only_break
            && columndef_separator_between_floating_overlays(para_idx, paragraphs);
        let suppress_floating_anchor_column_break = !has_diff_col_def
            && (crate::renderer::layout::para_is_floating_overlay_anchor(para)
                || empty_columndef_only_break
                || overlay_columndef_separator_break
                || (profile.hwpx_stored_layout()
                    && empty_table_carrier_column_break_before_page_table(
                        para_idx, para, paragraphs,
                    )));
        if para.column_type == ColumnBreakType::Column && !suppress_floating_anchor_column_break {
            if has_diff_col_def {
                // [Task #702] 단나누기 + 새 ColumnDef = zone 재정의 (MultiColumn 등가 처리)
                self.process_multicolumn_break(st, para_idx, paragraphs, page_def);
            } else if !st.current_items.is_empty() {
                // [Task #846] 마지막 단에서 명시적 단나누기 → 새 페이지가 아니라 같은
                // col_count 로 같은 페이지에 새 단-밴드를 시작 (들어갈 공간이 있으면). ≈ #768.
                // [Task #849] 단, 이는 "배분"(Distribute) 단에서만. "일반"(Normal/신문형)
                // 단에서 마지막 단의 단나누기는 같은 페이지 새 밴드를 만들지 않는다 (기존 동작).
                // [Task #866] shortcut.hwp 3쪽 "<편집 화면 분할에서>" pi=94 회귀 수정.
                let is_last_column = st.current_column + 1 >= st.col_count;
                if is_last_column
                    && st.col_count > 1
                    && st.current_zone_column_type == ColumnType::Distribute
                {
                    self.start_new_column_band(st, para_idx, paragraphs);
                } else {
                    st.advance_column_or_new_page();
                }
            }
        }

        // 쪽 나누기
        let force_page_break = para.column_type == ColumnBreakType::Page
            || para.column_type == ColumnBreakType::Section;
        let para_style = styles.para_styles.get(para.para_shape_id as usize);
        let para_style_break = para_style.map(|s| s.page_break_before).unwrap_or(false);

        // [Task #1007/#1035 → #1042 narrow v2] Cross-paragraph vpos reset 감지 —
        // heading paragraph (text 있음 + spacing_before ≥ 500 HU + paragraph local
        // vpos reset) 만 인정. content paragraph (spacing_before < 500) 는 skip.
        // sample16-2024 pi=162 (heading, sb=852, vpos=852) trigger ✓
        // sample16-2022 pi=87 (빈 문단, text_len=0) skip ✓
        // sample16-2022 pi=118 (content, sb=284) skip ✓
        // sample16-2022 pi=316 (content, sb=0) skip ✓
        let variant_vpos_reset_break = profile.hwp3_layout()
            && self.judge_hwp3_variant_vpos_reset_break(
                para,
                paragraphs,
                styles,
                para_idx,
                body_height_hu_for_variant,
                variant_prev_para_idx,
            );

        // [#1956] 명시적 쪽나누기 문단부터는 wrap 밴드 무효 — 새 쪽에는 anchor
        // 개체가 없으므로 후속 문단을 옆에 흡수하면 안 된다. current_items 가 비어
        // 있어 force_new_page 를 생략하는 경우에도 사용자 의도(새 쪽)는 동일하므로
        // 밴드는 해제한다. (법령안 신구조문대비표: 쪽나누기 표제가 흡수되어 pi 6쪽 이탈)
        if (force_page_break || para_style_break) && st.wrap_around_cs >= 0 {
            st.finish_stored_wrap_matching();
            st.close_square_band();
        }
        // [#1955] 명시적 쪽나누기부터는 글뒤로 표 후행 흡수도 해제 (사용자 의도 새 쪽).
        if force_page_break || para_style_break {
            st.finish_behind_float_absorption();
        }

        if (force_page_break || para_style_break || variant_vpos_reset_break)
            && !st.current_items.is_empty()
        {
            st.force_new_page();
            // [Task #702] 쪽나누기 + 새 ColumnDef = 새 페이지에서 col 정의 적용
            if has_diff_col_def {
                if let Some(cd) = &new_col_def_opt {
                    st.enter_column_definition(cd.column_count.max(1));
                    let new_layout = PageLayoutInfo::from_page_def(page_def, cd, self.dpi);
                    st.install_zone_layout(new_layout, cd.column_type);
                    // [Task #853] 새 페이지 첫 zone: 디자인 spacing /2 (위쪽 절반)만 추가.
                    // (이전 zone 은 이전 페이지에 있었으므로 아래쪽 절반은 더하지 않음.)
                    let new_ds = column_def_design_spacing_px(cd, self.dpi);
                    st.advance_zone_origin(new_ds / 2.0);
                    st.initialize_zone_spacing(new_ds);
                }
            }
        }

        // HWP5-origin 문서는 section PageHide를 연 빈 marker와, 같은 쪽 장식 host의
        // Page break를 함께 기록할 수 있다. marker의 break는 적용하되 marker
        // 자체를 배치하지 않아 host가 그 새 쪽을 바로 소유하도록 한다.
        if (profile.hwp5_stored_pagination_layout() || profile.hwpx_stored_layout())
            && hwp5_origin_redundant_pagehide_break_marker(
                para_idx,
                para,
                paragraphs,
                profile.hwpx_stored_layout(),
            )
        {
            st.hide_empty_paragraph(para_idx);
            return None;
        }

        // [Task #1046] 사후 reflow 이월: layout 에서 본문 하단 overflow 로 판정된 항목은
        // 현재 페이지에 렌더링하지 않고 다음 페이지로 넘긴다. force_break_before 에 등록된
        // para_idx 가 현재 페이지에 이미 항목이 있으면 새 페이지를 강제 (force_page_break 등가).
        // 빈 셋(reflow hint 없음)이면 무동작 → 기존 출력 불변.
        if force_break_before.contains(&para_idx) && !st.current_items.is_empty() {
            st.force_new_page();
        }

        if para_is_pre_paper_page_square_table_scaffold(para_idx, paragraphs)
            || para_is_post_paper_page_square_table_scaffold(para_idx, paragraphs)
        {
            // [#2019 v3] Paper/Page 기준 Square 표를 둘러싼 빈 스캐폴드 문단.
            // 쪽나누기/ColumnDef 효과는 위에서 적용하되, PDF 기준으로는 별도 빈 줄을
            // 렌더하거나 본문 흐름 높이를 소비하지 않는다.
            st.hide_empty_paragraph(para_idx);
            return None;
        }

        Some(ParagraphBoundary {
            para_style,
            para_style_break,
            force_page_break,
            overlay_columndef_separator_break,
        })
    }
}
