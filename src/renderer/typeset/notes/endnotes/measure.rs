//! 미주 렌더 위치·높이 Query. scratch 레이아웃을 사용하며 페이지 상태를 변경하지 않는다.

use crate::renderer::typeset::notes::endnotes::profile::{
    en_ssot_debug, en_ssot_level, EnSsotLevel,
};
use crate::renderer::typeset::{
    compose_paragraph, first_text_line, hwpunit_to_px, page_item_para_index,
    para_has_treat_as_char_picture_or_shape, paragraph_by_global_index, ComposedParagraph,
    HeightCursor, PageItem, Paragraph, ResolvedStyleSet, TypesetEngine, TypesetState,
};

impl TypesetEngine {
    /// 문단의 렌더링 높이를 계산한다 (format).
    /// [Task #1027 Stage D] 항목 fit 직전, `current_height` 를 vpos-정합 위치로 스냅한다.
    ///
    /// [Task #1363 v2 Stage 2] 미주 다단 SSOT 시뮬레이션.
    ///
    /// `st.current_items`(현재 단에 배치된 미주 항목들)를 렌더러 `build_single_column` 과
    /// 동일 경로(`HeightCursor::vpos_adjust` + line/total advances)로 재생해 단의 bottom y 를
    /// 산출한다. A2 게이트에서 `current_height` 를 이 값으로 스냅 → compute_en_metrics 의
    /// saved-delta 근사를 렌더 실측과 정합시킨다(p21 과대·p17 과소 누적 원인 제거 목표).
    /// `current_height` 상대공간(col_area_y=0, start=`current_start_height`)에서 구동.
    #[allow(clippy::too_many_arguments)]
    pub(in crate::renderer::typeset) fn simulate_endnote_column_bottom_y(
        &self,
        st: &TypesetState,
        paragraphs: &[Paragraph],
        styles: &ResolvedStyleSet,
        available: f64,
        en_col_w: f64,
        extra_para_full: Option<usize>,
        sequential_compact_rewind: bool,
    ) -> Option<f64> {
        if st.current_items.is_empty() {
            return None;
        }
        let ssot_level = en_ssot_level();
        let ssot_debug = en_ssot_debug();
        let mut local_paras: Vec<Paragraph> = Vec::new();
        let mut local_indices: Vec<(usize, usize)> = Vec::new();
        for pi in st
            .current_items
            .iter()
            .filter_map(page_item_para_index)
            .chain(extra_para_full)
        {
            if local_indices.iter().any(|(global, _)| *global == pi) {
                continue;
            }
            if let Some(p) = paragraph_by_global_index(paragraphs, &st.endnote_paragraphs, pi) {
                let local = local_paras.len();
                local_paras.push(p.clone());
                local_indices.push((pi, local));
            }
        }
        let lookup_local = |pi: usize| {
            local_indices
                .iter()
                .find_map(|(global, local)| (*global == pi).then_some(*local))
        };
        // [Task #1363 v3 옵션 3] A3: per-para 고립 측정 + HeightCursor 시뮬 대신, 단의 전 items 를
        // scratch `LayoutEngine` 으로 **1회 순차 렌더**해 정확한 단 bottom 을 읽는다. items 를
        // 로컬 0-기반 재색인해 build_single_column 경로(vpos forward-jump·trailing·text_start_line
        // 등 렌더 dispatch)를 그대로 태운다 → sim==render 구조 보장.
        // [#5886] B-level page_offcanvas 도 이 경로를 쓴다. HeightCursor 휴리스틱은
        // 저장 span/75% 되감김으로 순차 렌더(+69.7px)를 과소 계상해 알짜 풀이가
        // 용지 밖에 남았다. acc·A2 스냅은 그대로 휴리스틱.
        if ssot_level >= EnSsotLevel::A3 || sequential_compact_rewind {
            // 로컬 인덱스를 **+1 오프셋**하고 인덱스 0 에 더미 para 를 둔다. 렌더의
            // `layout_composed_paragraph` 는 `para_index == 0` + column-top + 첫 줄 vpos>0 이면
            // 절대 vpos 를 가산하는 fallback(섹션 첫 문단 제목용)이 있는데, 실제 미주 para 는
            // 큰 글로벌 인덱스라 결코 0 이 아니다. 0-기반 재색인이 이 fallback 을 잘못 발동시켜
            // 단독 측정이 폭발(35px→13721px)하므로 0 을 비워 둔다(더미는 어떤 item 도 미참조).
            let a3_paras: Vec<Paragraph> = std::iter::once(Paragraph::default())
                .chain(local_paras.iter().cloned())
                .collect();
            let a3_composed: Vec<crate::renderer::composer::ComposedParagraph> = a3_paras
                .iter()
                .map(crate::renderer::composer::compose_paragraph)
                .collect();
            let remap = |item: &PageItem| -> Option<PageItem> {
                match item {
                    PageItem::FullParagraph { para_index } => lookup_local(*para_index)
                        .map(|l| PageItem::FullParagraph { para_index: l + 1 }),
                    PageItem::PartialParagraph {
                        para_index,
                        start_line,
                        end_line,
                    } => lookup_local(*para_index).map(|l| PageItem::PartialParagraph {
                        para_index: l + 1,
                        start_line: *start_line,
                        end_line: *end_line,
                    }),
                    PageItem::Table {
                        para_index,
                        control_index,
                    } => lookup_local(*para_index).map(|l| PageItem::Table {
                        para_index: l + 1,
                        control_index: *control_index,
                    }),
                    PageItem::PartialTable {
                        para_index,
                        control_index,
                        start_row,
                        end_row,
                        is_continuation,
                        start_cut,
                        end_cut,
                        is_block_split,
                        start_cut_is_block,
                        row_cursor_is_nested,
                        end_row_height_override,
                        start_row_height_override,
                    } => lookup_local(*para_index).map(|l| PageItem::PartialTable {
                        para_index: l + 1,
                        control_index: *control_index,
                        start_row: *start_row,
                        end_row: *end_row,
                        is_continuation: *is_continuation,
                        start_cut: start_cut.clone(),
                        end_cut: end_cut.clone(),
                        is_block_split: *is_block_split,
                        start_cut_is_block: *start_cut_is_block,
                        row_cursor_is_nested: *row_cursor_is_nested,
                        end_row_height_override: *end_row_height_override,
                        start_row_height_override: *start_row_height_override,
                    }),
                    PageItem::Shape {
                        para_index,
                        control_index,
                    } => lookup_local(*para_index).map(|l| PageItem::Shape {
                        para_index: l + 1,
                        control_index: *control_index,
                    }),
                    // 구분선은 측정에서 제외(현 per-para 시뮬과 동일 — start_height 가 단 콘텐츠
                    // 시작을 이미 반영).
                    PageItem::EndnoteSeparator { .. } => None,
                }
            };
            let extra_local = extra_para_full
                .and_then(|pi| lookup_local(pi))
                .map(|l| PageItem::FullParagraph { para_index: l + 1 });
            let local_items: Vec<PageItem> = st
                .current_items
                .iter()
                .filter_map(&remap)
                .chain(extra_local)
                .collect();
            if local_items.is_empty() {
                return None;
            }
            // build_single_column 은 양수 start_height 를 무시(음수 shift 만 적용)하므로,
            // 단이 본문 아래에서 시작(start>0)하면 col_area.y 에 그 오프셋을 실어 동일 프레임에서
            // 렌더한다. 음수(vpos 되감김)는 col_area.y=0 + start_height 음수 shift 로 처리.
            let col_y = st.current_start_height.max(0.0);
            let col_area = crate::renderer::page_layout::LayoutRect {
                x: 0.0,
                y: col_y,
                width: en_col_w,
                height: (available - col_y).max(0.0),
            };
            let scratch = crate::renderer::layout::LayoutEngine::new(self.dpi);
            let bottom = scratch.measure_endnote_column_bottom(
                local_items,
                &a3_paras,
                &a3_composed,
                styles,
                &col_area,
                st.current_start_height,
                st.section_index,
                st.endnote_between_notes_hu,
            );
            if ssot_debug {
                eprintln!(
                    "EN_COLSIM start_h={:.1} avail={:.1} items={} bottom={:.1}",
                    st.current_start_height,
                    available,
                    local_indices.len(),
                    bottom,
                );
            }
            return Some(bottom);
        }
        let page_base = st
            .current_items
            .iter()
            .filter_map(page_item_para_index)
            .find_map(|pi| {
                paragraph_by_global_index(paragraphs, &st.endnote_paragraphs, pi)
                    .and_then(|p| p.line_segs.first())
                    .map(|seg| seg.vertical_pos)
            })?;
        let mut hc = HeightCursor::new(
            self.dpi,
            0.0,
            available,
            st.current_start_height,
            Some(page_base),
            st.skip_spacing_before_prededuct,
            false,
            st.current_endnote_flow && st.current_start_height < -0.5,
            st.current_endnote_flow,
        );
        hc.endnote_between_notes_hu = st.endnote_between_notes_hu;
        let mut y = st.current_start_height;
        let extra_item = extra_para_full.map(|pi| PageItem::FullParagraph { para_index: pi });
        // [#5886] 한 번 compact 되감김이 나오면 렌더처럼 나머지도 순차 적층한다.
        let mut stack_sequential = false;
        for item in st.current_items.iter().chain(extra_item.as_ref()) {
            let Some(pi) = page_item_para_index(item) else {
                continue;
            };
            let Some(local) = lookup_local(pi) else {
                continue;
            };
            // [#5886] 문단-사이 compact 되감김: 렌더는 겹치지 않고 순차 적층한다.
            // vpos_adjust 되감김 + 저장 span 은 용지 밖 풀이를 과소 계상한다.
            let compact_rewind_from_prev = sequential_compact_rewind
                && hc
                    .prev_layout_para
                    .and_then(|prev_local| {
                        let prev_bottom = local_paras.get(prev_local).and_then(|p| {
                            p.line_segs
                                .iter()
                                .map(|s| {
                                    s.vertical_pos
                                        .saturating_add(s.line_height)
                                        .saturating_add(s.line_spacing)
                                })
                                .max()
                        })?;
                        let curr_first = local_paras
                            .get(local)
                            .and_then(|p| p.line_segs.first())
                            .map(|s| s.vertical_pos)?;
                        Some(curr_first < prev_bottom)
                    })
                    .unwrap_or(false);
            if compact_rewind_from_prev {
                stack_sequential = true;
            }
            if !stack_sequential {
                y = hc.vpos_adjust(y, local, &local_paras, styles);
            }
            let item_para = &local_paras[local];
            let item_composed =
                crate::renderer::composer::compose_paragraph_in_context(item_para, styles);
            // [Task #1363 v2 Stage 3] 휴리스틱 advance 추정. 렌더러는 미주 텍스트/수식 para 를
            // **저장 line_segs**(hancom 레이아웃)로 그린다 — format_paragraph reflow(total_height)가
            // 아님. 수식 다줄 para 는 reflow 가 저장 span 보다 큼(pi=1126: 237 vs 185.8) → 단 과대.
            // 저장 line_segs vpos 범위를 advance 로 사용해 렌더와 정합. 단, **TAC 그림/도형 para**는
            // 개체 높이가 line_segs 에 없으므로(pi=1131: 빈 텍스트+309px 그림) total_height 사용.
            // 내부 vpos rewind para 는 line_segs vpos 범위가 작지만(되감김) 렌더러는 순차
            // 적층(Divergence A) → line_advances_sum 사용. (sep20/20 pi=522: saved 32.5 vs 실제 183)
            let heuristic_advance = {
                let item_fmt = self.format_endnote_paragraph(
                    item_para,
                    Some(&item_composed),
                    styles,
                    Some(en_col_w),
                );
                let internal_rewind = item_para
                    .line_segs
                    .windows(2)
                    .any(|w| w[1].vertical_pos < w[0].vertical_pos);
                let para_advance_full = if para_has_treat_as_char_picture_or_shape(item_para) {
                    item_fmt.total_height
                } else if internal_rewind || stack_sequential {
                    item_fmt.line_advances_sum(0..item_fmt.line_heights.len())
                } else {
                    let segs = &item_para.line_segs;
                    match (
                        segs.first(),
                        segs.iter()
                            .map(|s| s.vertical_pos.saturating_add(s.line_height))
                            .max(),
                    ) {
                        (Some(first), Some(bottom)) => {
                            hwpunit_to_px((bottom - first.vertical_pos).max(0), self.dpi)
                                .max(item_fmt.line_advance(0))
                        }
                        _ => item_fmt.total_height,
                    }
                };
                // 표/도형 단독 항목은 line_segs vpos 범위(저장 레이아웃 높이)로 advance.
                let saved_vpos_span = {
                    let segs = &item_para.line_segs;
                    match (
                        segs.first(),
                        segs.iter()
                            .map(|s| s.vertical_pos.saturating_add(s.line_height))
                            .max(),
                    ) {
                        (Some(first), Some(bottom)) => {
                            hwpunit_to_px((bottom - first.vertical_pos).max(0), self.dpi)
                        }
                        _ => 0.0,
                    }
                };
                match item {
                    PageItem::PartialParagraph {
                        start_line,
                        end_line,
                        ..
                    } => item_fmt.line_advances_sum(*start_line..*end_line),
                    PageItem::FullParagraph { .. } => para_advance_full,
                    PageItem::Table { .. } | PageItem::PartialTable { .. } => {
                        saved_vpos_span.max(item_fmt.total_height)
                    }
                    _ => 0.0,
                }
            };
            // [Task #1363 v3 Stage 1] A3: 휴리스틱 advance 추정 대신 scratch LayoutEngine 으로
            // para 를 실제 레이아웃해 정확한 렌더 advance 를 측정한다(렌더 권위). ssot_debug 시
            // 휴리스틱과의 diff 를 로그해 정합·drift 를 정량 확인한다.
            let advance = if ssot_level >= EnSsotLevel::A3 {
                let measured = self.measure_endnote_para_advance(
                    item_para,
                    &item_composed,
                    styles,
                    en_col_w,
                    available,
                    y,
                    item,
                    st.section_index,
                    pi,
                );
                if ssot_debug {
                    eprintln!(
                        "EN_MEASURE pi={} y_top={:.1} heuristic={:.1} measured={:.1} diff={:.1}",
                        pi,
                        y,
                        heuristic_advance,
                        measured,
                        measured - heuristic_advance,
                    );
                }
                measured
            } else {
                heuristic_advance
            };
            y += advance;
            let current_vpos_rewinds_from_prev = hc
                .prev_layout_para
                .and_then(|prev_local| {
                    let prev_first = local_paras
                        .get(prev_local)
                        .and_then(|p| p.line_segs.first())
                        .map(|seg| seg.vertical_pos)?;
                    let curr_first = local_paras
                        .get(local)
                        .and_then(|p| p.line_segs.first())
                        .map(|seg| seg.vertical_pos)?;
                    Some(curr_first < prev_first)
                })
                .unwrap_or(false);
            if matches!(item, PageItem::PartialParagraph { start_line, .. } if *start_line > 0)
                || current_vpos_rewinds_from_prev
            {
                hc.prev_layout_para = None;
                hc.vpos_page_base = None;
                hc.vpos_lazy_base = None;
            } else {
                hc.prev_layout_para = Some(local);
            }
            hc.prev_item_was_partial_table = matches!(item, PageItem::PartialTable { .. });
        }
        Some(y)
    }

    /// [Task #2079] 미주 fit 판정용 렌더-시뮬 예측 — 현재 단 items 를 저장 vpos 로
    /// 재주행해 미주 문단(en_para_idx)의 예상 시작 y 를 구한다. P6 판정 4곳 공용
    /// (종전 문자 그대로 중복 3곳 + `?` 문체 변형 1곳 dedup).
    pub(in crate::renderer::typeset) fn predict_endnote_render_y(
        &self,
        st: &TypesetState,
        paragraphs: &[Paragraph],
        styles: &ResolvedStyleSet,
        available: f64,
        en_col_w: f64,
        en_para_idx: usize,
    ) -> Option<f64> {
        let mut local_paras: Vec<Paragraph> = Vec::new();
        let mut local_indices: Vec<(usize, usize)> = Vec::new();
        for pi in st
            .current_items
            .iter()
            .filter_map(page_item_para_index)
            .chain(std::iter::once(en_para_idx))
        {
            if local_indices.iter().any(|(global, _)| *global == pi) {
                continue;
            }
            if let Some(p) = paragraph_by_global_index(paragraphs, &st.endnote_paragraphs, pi) {
                let local = local_paras.len();
                local_paras.push(p.clone());
                local_indices.push((pi, local));
            }
        }
        let lookup_local = |pi: usize, indices: &[(usize, usize)]| {
            indices
                .iter()
                .find_map(|(global, local)| (*global == pi).then_some(*local))
        };
        let first_vpos = st
            .current_items
            .iter()
            .filter_map(page_item_para_index)
            .find_map(|pi| {
                paragraph_by_global_index(paragraphs, &st.endnote_paragraphs, pi)
                    .and_then(|p| p.line_segs.first())
                    .map(|seg| seg.vertical_pos)
            });
        first_vpos.and_then(|page_base| {
            let mut hc = HeightCursor::new(
                self.dpi,
                0.0,
                available,
                st.current_start_height,
                Some(page_base),
                st.skip_spacing_before_prededuct,
                false,
                st.current_endnote_flow && st.current_start_height < -0.5,
                st.current_endnote_flow,
            );
            hc.endnote_between_notes_hu = st.endnote_between_notes_hu;
            let mut y = st.current_start_height;
            for item in &st.current_items {
                let Some(pi) = page_item_para_index(item) else {
                    continue;
                };
                let Some(local) = lookup_local(pi, &local_indices) else {
                    continue;
                };
                y = hc.vpos_adjust(y, local, &local_paras, &styles);
                let item_para = &local_paras[local];
                let item_composed =
                    crate::renderer::composer::compose_paragraph_in_context(item_para, styles);
                let item_fmt = self.format_endnote_paragraph(
                    item_para,
                    Some(&item_composed),
                    &styles,
                    Some(en_col_w),
                );
                y += match item {
                    PageItem::PartialParagraph {
                        start_line,
                        end_line,
                        ..
                    } => item_fmt.line_advances_sum(*start_line..*end_line),
                    PageItem::FullParagraph { .. } => item_fmt.total_height,
                    _ => 0.0,
                };
                let current_vpos_rewinds_from_prev = hc
                    .prev_layout_para
                    .and_then(|prev_local| {
                        let prev_first = local_paras
                            .get(prev_local)
                            .and_then(|p| p.line_segs.first())
                            .map(|seg| seg.vertical_pos)?;
                        let curr_first = local_paras
                            .get(local)
                            .and_then(|p| p.line_segs.first())
                            .map(|seg| seg.vertical_pos)?;
                        Some(curr_first < prev_first)
                    })
                    .unwrap_or(false);
                if matches!(
                    item,
                    PageItem::PartialParagraph { start_line, .. }
                        if *start_line > 0
                ) || current_vpos_rewinds_from_prev
                {
                    hc.prev_layout_para = None;
                    hc.vpos_page_base = None;
                    hc.vpos_lazy_base = None;
                } else {
                    hc.prev_layout_para = Some(local);
                }
                hc.prev_item_was_partial_table = matches!(item, PageItem::PartialTable { .. });
            }
            lookup_local(en_para_idx, &local_indices)
                .map(|local| hc.vpos_adjust(y, local, &local_paras, &styles))
        })
    }

    /// [Task #1363 v3 Stage 1] scratch `LayoutEngine` 로 미주 para 를 실제 레이아웃하여 **정확한
    /// 렌더 advance(px)** 를 측정한다. 시뮬의 휴리스틱 높이 추정(saved-vpos span / total_height /
    /// line_advances_sum)을 렌더 권위 값으로 대체하기 위한 측정 전용 경로다.
    ///
    /// 좌표는 시뮬과 동일한 **컬럼 top=0 상대 프레임**으로 구성한다(`col_area.y=0`,
    /// `y_start`=상대 y). advance(delta)는 프레임 평행이동 불변이므로 렌더 절대 좌표와 정합한다.
    /// 노드는 scratch `tree`/`col_node` 로 버려 실제 렌더에 무영향. 매 호출 `LayoutEngine::new`
    /// 로 생성하므로 numbering/overflow 등 상태도 격리된다(Stage 2 에서 실증).
    ///
    /// **알려진 fidelity 한계(Stage 1 POC)**: `bin_data_content=None` — TAC 그림 intrinsic 사이징
    /// 미반영(명시 크기 그림은 무관). `endnote_para_base` 미설정 — 미주 가상 para 판정이 false 라
    /// overflow tolerance 만 다르고 advance 에는 무영향.
    #[allow(clippy::too_many_arguments)]
    pub(in crate::renderer::typeset) fn measure_endnote_para_advance(
        &self,
        item_para: &Paragraph,
        item_composed: &ComposedParagraph,
        styles: &ResolvedStyleSet,
        en_col_w: f64,
        available: f64,
        y_start: f64,
        item: &PageItem,
        section_index: usize,
        para_index: usize,
    ) -> f64 {
        use crate::renderer::layout::{layout_rect_to_bbox, LayoutEngine};
        use crate::renderer::page_layout::LayoutRect;
        use crate::renderer::render_tree::{PageLayoutContext, RenderNode, RenderNodeType};

        // 렌더 `layout_column_item` 의 FullParagraph 텍스트 경로 정합: 실제 텍스트가 있는 para 는
        // **leading 컨트롤-전용 줄**(수식 객체마커 ￼ 등)을 건너뛰고 첫 텍스트 줄부터 그린다.
        // `composer::first_text_line` 로 판정을 공유해(#4312) 재구현 divergence(sep20/20
        // pi=936: 측정 127.7 vs 렌더 101.3)를 구조적으로 막는다. 객체-전용 para(TAC 그림 등)는
        // 0 부터(렌더도 동일). Partial 은 항목 지정 줄 범위 그대로.
        let (start_line, end_line) = match item {
            PageItem::PartialParagraph {
                start_line,
                end_line,
                ..
            } => (*start_line, *end_line),
            _ => {
                let has_real_text = item_para
                    .text
                    .chars()
                    .any(|c| c > '\u{001F}' && c != '\u{FFFC}' && !c.is_whitespace());
                let start = if has_real_text {
                    first_text_line(item_composed).unwrap_or(0)
                } else {
                    0
                };
                (start, item_composed.lines.len())
            }
        };
        let height = available.max(0.0);
        let col_area = LayoutRect {
            x: 0.0,
            y: 0.0,
            width: en_col_w,
            height,
        };
        // [#4277] 페이지네이션 측정은 paint 트리를 만들지 않는다 — 레이아웃 재귀가 실제로
        // 쓰는 건 흐름 상태(id 카운터 + 인라인 Shape 레지스트리 + 페이지 기하)뿐이므로
        // `PageLayoutContext` 만 만든다. `col_node` 는 방출된 노드를 받는 sink 로만 쓰이고
        // 높이만 읽은 뒤 버려진다.
        let scratch = LayoutEngine::new(self.dpi);
        let mut frame = PageLayoutContext::new(0, en_col_w, height);
        let col_id = frame.next_id();
        let mut col_node = RenderNode::new(
            col_id,
            RenderNodeType::Column(0),
            layout_rect_to_bbox(&col_area),
        );
        let y_after = scratch.layout_partial_paragraph(
            &mut frame,
            &mut col_node,
            item_para,
            Some(item_composed),
            styles,
            false, // 미주 scratch는 저장 LineSeg 흐름을 보존한다.
            &col_area,
            y_start,
            start_line,
            end_line,
            section_index,
            para_index,
            None, // multi_col_width_hu: 렌더 미주 body-flow 경로와 동일(None)
            None, // bin_data_content: Stage 1 POC — None
            None, // wrap_anchor: 미주 단 내부 wrap-around 없음
        );
        (y_after - y_start).max(0.0)
    }
}
