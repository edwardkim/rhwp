//! 구역 columns 책임. 기존 조건과 호출 순서를 보존한다.
use crate::renderer::typeset::{
    column_def_design_spacing_px, hwpunit_to_px, paper_overlay_object_bottom_abs_px,
    ColumnBreakType, Control, PageDef, PageItem, PageLayoutInfo, Paragraph, TypesetEngine,
    TypesetState,
};
impl TypesetEngine {
    pub(in crate::renderer::typeset) fn process_multicolumn_break(
        &self,
        st: &mut TypesetState,
        para_idx: usize,
        paragraphs: &[Paragraph],
        page_def: &PageDef,
    ) {
        st.flush_column();

        // [Task #874 Case 5] leaving zone 의 height 계산 시 마지막 라인의 trailing
        // line_spacing 을 제외한다. zone 간 gap 은 design_spacing/2 + solo_zone_pad 가
        // 이미 담당하므로 vpos_zone_height 에 trailing_ls 까지 더하면 이중 가산.
        // 한컴 PDF 측정 (shortcut.hwp 1쪽): 본문 첫 줄 top 195.3 px (Hancom) vs 210.7 px
        // (rhwp pre) = +15.4 px (≈11.5pt) 넓다. 제목 paragraph 의 trailing_ls 16 px 이
        // vpos_zone_height 에 포함되어 다음 zone(헤더 띠 + 본문)을 일괄 16 px 하향.
        // pi=80 (21_언어_기출_편집가능본 test_544) 회귀 없음 — pi=80 은 zone 내부 box
        // 인접 paragraph 로 trailing_ls 가 layout 의 y_offset 에서 포함됨 (이 변경은 zone
        // 전환 시의 vpos_zone_height 만 수정).
        let vpos_zone_height = if para_idx > 0 {
            let mut max_vpos_end: i32 = 0;
            let mut prev_is_floating_anchor = false;
            for prev_idx in (0..para_idx).rev() {
                if let Some(last_seg) = paragraphs[prev_idx].line_segs.last() {
                    let vpos_end = last_seg.vertical_pos.saturating_add(last_seg.line_height);
                    if vpos_end > max_vpos_end {
                        max_vpos_end = vpos_end;
                    }
                    prev_is_floating_anchor =
                        crate::renderer::layout::para_is_floating_overlay_anchor(
                            &paragraphs[prev_idx],
                        );
                    break;
                }
            }
            // [#2019 부분 완화] leaving zone 높이는 "이 페이지에서 콘텐츠가 내려간 높이"여야
            // 한다. 별지 서식처럼 stored vpos 가 섹션 누적 좌표인 문서에서는 max_vpos_end 가
            // 페이지 높이를 크게 넘어 zone 전환마다 새 페이지가 생기므로, 우선 흐름 누적값
            // (st.current_height, page-상대)을 대신 쓴다. 이 역시 완전한 한글 모델은 아니며,
            // Paper 앵커 object extent 기반 page-local 계산으로 교체되어야 한다.
            let max_vpos_px = hwpunit_to_px(max_vpos_end, self.dpi);
            if max_vpos_end > 0
                && !prev_is_floating_anchor
                && max_vpos_px <= st.layout.available_body_height()
            {
                // 사다리는 **한글의 쪽 경계** 기준이라, 한글이 이미 쪽을 끊은 자리에서는
                // 직전 문단의 vpos 가 다음 쪽 상단 값이 되어 이 쪽이 실제로 소비한 높이를
                // 크게 밑돈다. 그러면 아래 Task #853 가드가 "아직 여유 있다" 로 오판해
                // 새 zone 을 같은 쪽에 얹는다 (2990099 주파수 분배표 115쪽: 사다리 76px
                // vs 실소비 867.4px — zone 두 개가 1657.3px 로 본문 876.9px 에 겹쳐
                // 745.7px 이 쪽 밖으로 나갔다). 흐름 누적은 이 쪽에 실제로 놓인 양이므로
                // 둘 중 큰 값이 이 쪽의 소비량이다.
                max_vpos_px.max(st.current_height)
            } else {
                st.current_height
            }
        } else {
            st.current_height
        };
        // [Task #853] zone 전환 시 디자인 spacing(1단 ColumnDef 의 `간격`)을 세로 간격으로:
        // (이전 zone 디자인 spacing /2) + (새 zone 디자인 spacing /2) 를 더한다.
        // shortcut.hwp 1쪽: 제목 zone(0mm) → 헤더 띠 zone(10mm) → 본문 zone(2단, 0)
        //   → 제목↔헤더 = 5mm, 헤더↔본문 = 5mm (한컴 PDF 정합).
        let new_ds = paragraphs[para_idx]
            .controls
            .iter()
            .find_map(|c| {
                if let Control::ColumnDef(cd) = c {
                    Some(column_def_design_spacing_px(cd, self.dpi))
                } else {
                    None
                }
            })
            .unwrap_or(0.0);
        // [Task #866] 직전 zone 의 마지막 paragraph 가 wrap=위아래 인 글자처럼-취급 표(헤더 띠)를
        // 보유하고 그 zone 의 1단 ColumnDef 간격이 0 이면, 한컴은 표 band 높이(표 본체 +
        // outer_margin top/bottom)만큼을 표 아래에 추가로 비워둔다(한컴 PDF 측정:
        // shortcut.hwp 2·3쪽 헤더 띠 하단↔본문 ~28~33px). ColumnDef 간격>0 인 헤더 띠(1쪽
        // 등)는 그 간격이 이미 zone 사이 여백이 되므로 제외.
        // [Task #874 Stage 2] design_spacing 조건을 ≤ 1mm(=3.8px) 까지 인정. 페이지 break 후
        // current_zone_design_spacing_px 가 stale state 로 1mm 남은 경우 (shortcut.hwp 6쪽
        // pi=210 '도구' 헤더띠 zone cd 가 pi=209 cd=1mm 인 케이스) 도 헤더띠 leaving 으로 식별.
        let tac_band_extra: f64 = if st.current_zone_design_spacing_px < 4.0 {
            (0..para_idx)
                .rev()
                .find(|&i| !paragraphs[i].line_segs.is_empty())
                .and_then(|pi| {
                    paragraphs[pi].controls.iter().find_map(|c| match c {
                        Control::Table(t)
                            if t.common.treat_as_char
                                && matches!(
                                    t.common.text_wrap,
                                    crate::model::shape::TextWrap::TopAndBottom
                                ) =>
                        {
                            Some(
                                hwpunit_to_px(t.common.height as i32, self.dpi)
                                    + hwpunit_to_px(t.outer_margin_top as i32, self.dpi)
                                    + hwpunit_to_px(t.outer_margin_bottom as i32, self.dpi),
                            )
                        }
                        _ => None,
                    })
                })
                .unwrap_or(0.0)
        } else {
            0.0
        };
        // [Task #866 v2 Stage 2/4] zone 전환 시 추가 세로 여백.
        // (1) 1단/간격=0 zone(헤더 띠 / `<...>` 소제목) 진입·이탈: +1500 HU(=20px).
        //     shortcut.hwp 4쪽 `개체 모양 복사`↔`<스타일에서>`, 6쪽 `도구`↔`맞춤법 검사` 등.
        // (2) [단나누기](ColumnBreakType::Column) 로 시작하는 새 zone: +1500 HU(=20px).
        //     배분 다단 zone 의 마지막 컬럼 [단나누기] = 같은 ColumnDef 로 새 밴드 → 한컴 PDF
        //     상 이전 밴드와 ~한 본문 줄 간격(shortcut.hwp 3쪽 `화면 확대 100%`↔`<편집 화면
        //     분할에서>`). Stage 1 의 Distribute 마지막 컬럼 라우팅과 정합.
        let entering_solo_zero = paragraphs[para_idx].controls.iter().any(|c| {
            matches!(c,
            Control::ColumnDef(cd) if cd.column_count.max(1) <= 1 && cd.spacing == 0)
        });
        let leaving_solo_zero = st.col_count <= 1 && st.current_zone_design_spacing_px < 0.5;
        // [Task #866 v3 Stage 1] 헤더 띠 zone (TAC wrap=TopAndBottom 표) 의 leaving 은
        // `tac_band_extra` 가 이미 표 band 높이만큼 패딩을 추가하므로 `solo_zone_pad` 를 또
        // 더하면 한컴 PDF 대비 본문 첫 줄이 ~13pt 더 아래로 밀려 사용자 "넓다" 피드백 발생.
        // tac_band_extra>0 == 헤더 띠 leaving 케이스 → solo_zone_pad 의 leaving 분기 제외.
        let leaving_is_header_band = leaving_solo_zero && tac_band_extra > 0.5;
        let column_break_new_band = paragraphs[para_idx].column_type == ColumnBreakType::Column;
        let solo_zone_pad = if entering_solo_zero
            || (leaving_solo_zero && !leaving_is_header_band)
            || column_break_new_band
        {
            hwpunit_to_px(1200, self.dpi)
        } else {
            0.0
        };
        let candidate_offset = st.current_zone_y_offset
            + vpos_zone_height
            + tac_band_extra
            + st.current_zone_design_spacing_px / 2.0
            + new_ds / 2.0
            + solo_zone_pad;
        let paper_overlay_tail_overflows_page = paragraphs[para_idx].column_type
            == ColumnBreakType::Column
            && Self::upcoming_paper_overlay_tail_overflows_page(
                para_idx, paragraphs, &st.layout, self.dpi,
            );

        // [Task #853] 새 zone 이 현재 페이지 하단 가까이(여유 ≲ 헤더 띠 1개 높이)에서 시작하면
        // 그 zone 의 콘텐츠(헤더 띠 ~47px 또는 본문 줄들)가 body 하단을 넘어 렌더되므로 다음
        // 페이지로 넘긴다. (shortcut.hwp 3쪽~6쪽 — 다단 zone 다수 누적 시 잔여 콘텐츠가
        // 본문영역을 넘어 바닥 여백에 그려지던 결함)
        let one_line = hwpunit_to_px(1500, self.dpi);
        if paper_overlay_tail_overflows_page
            || candidate_offset > st.layout.available_body_height() - 4.0 * one_line
        {
            st.push_new_page();
            // 새 페이지 첫 zone: 새 zone 디자인 spacing /2 만 (이전 zone 은 이전 페이지).
            st.align_zone_origin(new_ds / 2.0);
        } else {
            st.align_zone_origin(candidate_offset);
        }
        st.initialize_zone_spacing(new_ds);
        st.restart_zone_columns();

        for ctrl in &paragraphs[para_idx].controls {
            if let Control::ColumnDef(cd) = ctrl {
                st.enter_column_definition(cd.column_count.max(1));
                let new_layout = PageLayoutInfo::from_page_def(page_def, cd, self.dpi);
                // [Task #702] 새 zone 의 ColumnType 반영. Distribute(배분) 단에서
                // 짧은 컬럼 vpos-reset 검출 임계값 완화용.
                st.install_zone_layout(new_layout, cd.column_type);
                break;
            }
        }
    }
    /// [#2019 v3] 명시 단나누기 뒤에 이어지는 Paper-overlay footer band 가
    /// body frame 하단을 넘고, 곧 명시 쪽나누기가 이어지면 그 band 는 현재 쪽
    /// 꼬리에 억지로 붙이지 않고 다음 물리 쪽에 배치한다.
    ///
    /// 74312 별지 서식의 처리절차 하단 꼬리는 Paper 절대좌표(용지 기준)라
    /// 흐름 높이를 거의 소비하지 않지만, 한컴 PDF 는 body 하단 아래쪽 항목
    /// 일부를 다음 쪽 footer 로 넘긴다. 일반 텍스트/표 band 로 확장하지 않도록
    /// "전부 빈 부동 overlay 앵커 + 직후 Page/Section break" 에만 한정한다.
    pub(in crate::renderer::typeset) fn upcoming_paper_overlay_tail_overflows_page(
        para_idx: usize,
        paragraphs: &[Paragraph],
        layout: &PageLayoutInfo,
        dpi: f64,
    ) -> bool {
        let body_bottom_abs = layout.body_area.y + layout.body_area.height;
        let mut saw_overlay = false;
        let mut overflows = false;
        let mut idx = para_idx + 1;

        while let Some(para) = paragraphs.get(idx) {
            if idx > para_idx + 1
                && (para.column_type != ColumnBreakType::None
                    || para
                        .controls
                        .iter()
                        .any(|ctrl| matches!(ctrl, Control::ColumnDef(_))))
            {
                let follows_explicit_page_break = matches!(
                    para.column_type,
                    ColumnBreakType::Page | ColumnBreakType::Section
                );
                return saw_overlay && overflows && follows_explicit_page_break;
            }

            if para.text.trim().is_empty() && para.controls.is_empty() {
                idx += 1;
                continue;
            }
            if !crate::renderer::layout::para_is_floating_overlay_anchor(para) {
                return false;
            }

            saw_overlay = true;
            if paper_overlay_object_bottom_abs_px(para, dpi)
                .is_some_and(|bottom| bottom > body_bottom_abs + 1.0)
            {
                overflows = true;
            }
            idx += 1;
        }

        false
    }
    /// [Task #846] 마지막 단에서 명시적 단나누기(`ColumnBreakType::Column`, 새 ColumnDef 없음)
    /// 를 만났을 때: 새 페이지가 아니라 같은 col_count 로 같은 페이지에 새 단-밴드를 시작한다
    /// (≈ 닫힌 #768). 단, 새 밴드가 본문에 들어갈 공간(이 문단 첫 줄)이 없으면 새 페이지로 넘긴다.
    /// 규칙: `누적_밴드_높이 + 현_밴드_높이(= max(컬럼별 채움)) < 본문_높이` 이면 새 밴드, 아니면 새 페이지.
    pub(in crate::renderer::typeset) fn start_new_column_band(
        &self,
        st: &mut TypesetState,
        para_idx: usize,
        paragraphs: &[Paragraph],
    ) {
        st.flush_column();

        // 새 밴드로 들어갈 콘텐츠에 떠다니는(글자처럼 취급이 아닌) 개체가 있으면
        // 같은 페이지에 밴드를 만들지 않고 새 페이지로 넘긴다.
        if Self::upcoming_band_has_floating_object(para_idx, paragraphs) {
            st.push_new_page();
            return;
        }

        // 방금 닫힌 밴드의 높이 = 그 밴드 각 단의 마지막 문단 vpos_end 중 최댓값.
        let zone_off = st.current_zone_y_offset;
        let mut band_height_px = 0.0_f64;
        if let Some(page) = st.pages.last() {
            for cc in page.column_contents.iter().rev() {
                if cc.zone_y_offset != zone_off {
                    break;
                }
                let last_para_idx = cc.items.iter().rev().find_map(|it| match it {
                    PageItem::FullParagraph { para_index }
                    | PageItem::PartialParagraph { para_index, .. }
                    | PageItem::Table { para_index, .. }
                    | PageItem::PartialTable { para_index, .. }
                    | PageItem::Shape { para_index, .. } => Some(*para_index),
                    PageItem::EndnoteSeparator { .. } => None,
                });
                if let Some(pi) = last_para_idx {
                    if let Some(seg) = paragraphs.get(pi).and_then(|p| p.line_segs.last()) {
                        let v = hwpunit_to_px(
                            seg.vertical_pos
                                .saturating_add(seg.line_height)
                                .saturating_add(seg.line_spacing),
                            self.dpi,
                        );
                        if v > band_height_px {
                            band_height_px = v;
                        }
                    }
                }
            }
        }
        if band_height_px <= 0.0 {
            band_height_px = st.current_height;
        }

        let first_line_h = paragraphs
            .get(para_idx)
            .and_then(|p| p.line_segs.first())
            .map(|s| hwpunit_to_px(s.line_height.saturating_add(s.line_spacing), self.dpi))
            .filter(|h| *h > 0.0)
            .unwrap_or(1.0);
        let room_after_band = st.available_height() - band_height_px;

        if room_after_band >= first_line_h {
            st.advance_zone_origin(band_height_px);
            st.restart_zone_columns();
        } else {
            st.push_new_page();
        }
    }
    /// 명시적 단나누기 다음 밴드(= `para_idx` 부터 다음 나누기/새 ColumnDef 직전까지)에
    /// 떠다니는 개체(글자처럼 취급이 아닌 표/그림/그리기 개체)가 있는지.
    pub(in crate::renderer::typeset) fn upcoming_band_has_floating_object(
        para_idx: usize,
        paragraphs: &[Paragraph],
    ) -> bool {
        for (offset, p) in paragraphs[para_idx..].iter().enumerate() {
            if offset > 0
                && (p.column_type != ColumnBreakType::None
                    || p.controls
                        .iter()
                        .any(|c| matches!(c, Control::ColumnDef(_))))
            {
                break;
            }
            for ctrl in &p.controls {
                let floating = match ctrl {
                    Control::Table(t) => !t.common.treat_as_char,
                    Control::Shape(s) => !s.common().treat_as_char,
                    Control::Picture(pic) => !pic.common.treat_as_char,
                    _ => false,
                };
                if floating {
                    return true;
                }
            }
        }
        false
    }
}
