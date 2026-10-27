// 최신 devel의 저장 미주 잉크 수용 경로를 같은 모듈 범위에서 보존한다.
/// [#6574] 렌더 줄 하단이 단 하단을 넘어도 같은 단으로 보는 허용치(px). 좌표 반올림만 흡수한다.
const ENDNOTE_RENDER_INK_FIT_TOLERANCE_PX: f64 = 0.25;

/// [#6574] 렌더 경로 잉크 하단으로 본 문단의 현재 단 수용 여부.
pub(in crate::renderer::typeset) enum EndnoteRenderInkFit {
    /// 판정하지 않았다(빈 단·본문이 섞인 단).
    Unjudged,
    /// 문단 전체가 단 안에 그려진다.
    Fits,
    /// 앞 `n` 줄만 단 안에 그려진다.
    SplitAt(usize),
    /// 첫 줄부터 단 하단을 넘는다.
    NextColumn,
}

/// [#6574] 현재 단이 저장 사다리의 단 경계에서 시작했는지. 저장 되감김은 저장 당시 배치의 단
/// 경계라, 현재 단 시작이 그 배치와 같을 때만 다음 되감김도 같은 배치의 경계로 믿을 수 있다.
/// 첫 미주 항목이 (a) 되감기는 줄에서 나뉜 문단의 이어진 조각이거나 (b) 직전 문단 마지막 줄
/// 끝보다 앞 vpos 에서 시작하는 문단이면 경계에서 시작한 것이다. 본문이 함께 있는 단과 첫
/// 미주로 시작하는 단도 경계로 본다.
pub(in crate::renderer::typeset) fn endnote_column_starts_at_stored_break(
    st: &TypesetState,
    paragraphs: &[Paragraph],
) -> bool {
    let mut endnote_items = st
        .current_items
        .iter()
        .filter(|item| page_item_para_index(item).is_some());
    let Some(first) = endnote_items.next() else {
        return true;
    };
    let Some(pi) = page_item_para_index(first) else {
        return true;
    };
    if pi < paragraphs.len() {
        return true;
    }
    let Some(para) = paragraph_by_global_index(paragraphs, &st.endnote_paragraphs, pi) else {
        return true;
    };
    if let PageItem::PartialParagraph { start_line, .. } = first {
        if *start_line > 0 {
            return match (
                para.line_segs.get(*start_line - 1),
                para.line_segs.get(*start_line),
            ) {
                (Some(prev), Some(cur)) => cur.vertical_pos < prev.vertical_pos,
                _ => true,
            };
        }
    }
    if pi == paragraphs.len() {
        return true;
    }
    let Some(prev) = paragraph_by_global_index(paragraphs, &st.endnote_paragraphs, pi - 1) else {
        return true;
    };
    match (prev.line_segs.last(), para.line_segs.first()) {
        (Some(last), Some(cur)) => {
            cur.vertical_pos
                < last
                    .vertical_pos
                    .saturating_add(last.line_height)
                    .saturating_add(last.line_spacing)
        }
        _ => true,
    }
}

/// 같은 저장 단의 뒤 조각이 소유한 최초 줄 컷이다. 물리 fit만으로 지우지 않는다.
/// 편집 후 재조판·합성 줄·다른 시작 단과 이미 소비한 앞 조각에는 적용하지 않는다.
pub(super) fn stored_endnote_column_cut(
    st: &TypesetState,
    paragraphs: &[Paragraph],
    para_index: usize,
) -> Option<usize> {
    if st.profile.session_edited()
        || st.col_count <= 1
        || para_index < paragraphs.len()
        || st
            .reissued_endnote_rows
            .contains_key(&(para_index - paragraphs.len()))
        || !endnote_column_starts_at_stored_break(st, paragraphs)
    {
        return None;
    }
    let para = paragraph_by_global_index(paragraphs, &st.endnote_paragraphs, para_index)?;
    if para.stored_text_partition_is_dirty()
        || para.line_segs.iter().any(|line| {
            line.vertical_pos < 0
                || line.line_height < 0
                || line.tag & crate::model::paragraph::LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0
        })
    {
        return None;
    }
    let cut = para
        .line_segs
        .windows(2)
        .position(|lines| lines[1].vertical_pos < lines[0].vertical_pos)?
        + 1;
    if st.current_items.iter().any(|item| {
        matches!(item, PageItem::PartialParagraph { para_index: owner, start_line, .. }
            if *owner == para_index && *start_line >= cut)
    }) {
        return None;
    }
    Some(cut)
}

impl TypesetEngine {
    /// [#6574] 현재 단 항목 뒤에 `en_para_idx` 를 붙여 렌더 경로로 그렸을 때 글줄 잉크
    /// 하단이 단 하단(`available`)을 넘는지 판정한다. 넘으면 앞 줄부터 다시 그려 단 안에
    /// 남는 마지막 줄 수를 찾는다. 단이 비어 있으면 판정하지 않는다(어디서도 못 들어간다).
    #[allow(clippy::too_many_arguments)]
    pub(in crate::renderer::typeset) fn judge_endnote_render_ink_fit(
        &self,
        st: &TypesetState,
        paragraphs: &[Paragraph],
        styles: &ResolvedStyleSet,
        available: f64,
        en_col_w: f64,
        en_para_idx: usize,
        line_count: usize,
    ) -> EndnoteRenderInkFit {
        if st.current_items.is_empty() {
            return EndnoteRenderInkFit::Unjudged;
        }
        // 렌더는 단을 물리 본문 하단에서 끝낸다. 누계 판정용 쪽 나눔 허용치는
        // 잉크 판정에 싣지 않는다(표 판정과 같은 물리 경계).
        let available = (available - st.layout.pagination_tolerance_px).max(0.0);
        self.measure_endnote_render_ink_fit(
            st,
            paragraphs,
            styles,
            available,
            en_col_w,
            en_para_idx,
            line_count,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn measure_endnote_render_ink_fit(
        &self,
        st: &TypesetState,
        paragraphs: &[Paragraph],
        styles: &ResolvedStyleSet,
        available: f64,
        en_col_w: f64,
        en_para_idx: usize,
        line_count: usize,
    ) -> EndnoteRenderInkFit {
        let limit = available + ENDNOTE_RENDER_INK_FIT_TOLERANCE_PX;
        let ink_bottom = |item: PageItem| {
            self.simulate_endnote_column_extent(
                st,
                paragraphs,
                styles,
                available,
                en_col_w,
                Some(&item),
            )
            .and_then(|(_, ink)| ink)
        };
        let full_extent = self.simulate_endnote_column_extent(
            st,
            paragraphs,
            styles,
            available,
            en_col_w,
            Some(&PageItem::FullParagraph {
                para_index: en_para_idx,
            }),
        );
        let full = full_extent.and_then(|(_, ink)| ink);
        let fits = |ink: Option<f64>| ink.is_none_or(|bottom| bottom <= limit);
        if en_ssot_debug() {
            eprintln!(
                "EN_INKFIT pi={} avail={:.1} full_ink={:?} lines={}",
                en_para_idx, available, full, line_count
            );
        }
        if full.is_none() {
            return EndnoteRenderInkFit::Unjudged;
        }
        let stored_cut =
            stored_endnote_column_cut(st, paragraphs, en_para_idx).filter(|cut| *cut < line_count);
        // 같은 저장 단의 구성이 유지되면 물리 여유만으로 다음 단의 소속을 지우지 않는다.
        // 앞 줄의 실제 감소량이 뒤 조각의 전진을 회수한 경우만 전체 수용을 재검증한다.
        let reissued_column = stored_cut.is_some_and(|cut| {
            paragraph_by_global_index(paragraphs, &st.endnote_paragraphs, en_para_idx).is_some_and(
                |paragraph| {
                    self.endnote_column_reclaims_saved_tail(st, paragraphs, styles, paragraph, cut)
                },
            )
        });
        if en_ssot_debug() && stored_cut.is_some() {
            eprintln!(
                "EN_SAVED_CUT pi={en_para_idx} cut={stored_cut:?} reissued_column={reissued_column}"
            );
        }
        if fits(full) && (stored_cut.is_none() || reissued_column) {
            return EndnoteRenderInkFit::Fits;
        }
        // 뒤 단의 저장 소속은 전체 자연 높이의 수용과 별개다. 같은 컷의 앞
        // 조각을 실제로 재생하며, 공간이 부족하면 더 작은 앞 조각만 수용한다.
        let split_limit = stored_cut.unwrap_or(line_count.saturating_sub(1));
        for split in (1..=split_limit).rev() {
            let head = ink_bottom(PageItem::PartialParagraph {
                para_index: en_para_idx,
                start_line: 0,
                end_line: split,
            });
            if en_ssot_debug() {
                eprintln!(
                    "EN_INKFIT_SPLIT pi={} split={} head_ink={:?}",
                    en_para_idx, split, head
                );
            }
            if fits(head) {
                return EndnoteRenderInkFit::SplitAt(split);
            }
        }
        EndnoteRenderInkFit::NextColumn
    }

    /// [#6574] 현재 단 항목 뒤에 `extra_item` 을 붙여 scratch `LayoutEngine` 으로 **렌더와 같은
    /// 경로**를 한 번 태우고 `(커서 하단, 글줄 잉크 하단)` 을 돌려준다. 좌표는
    /// `current_height` 프레임이다. `extra_item` 은 전체 문단이나 앞 몇 줄(`PartialParagraph`)
    /// 이다.
    pub(in crate::renderer::typeset) fn simulate_endnote_column_extent(
        &self,
        st: &TypesetState,
        paragraphs: &[Paragraph],
        styles: &ResolvedStyleSet,
        available: f64,
        en_col_w: f64,
        extra_item: Option<&PageItem>,
    ) -> Option<(f64, Option<f64>)> {
        if st.current_items.is_empty() {
            return None;
        }
        let ssot_debug = en_ssot_debug();
        // 본문 항목이 함께 있는 단은 렌더처럼 본문·구분선부터 단 위에서 차례로 그린다 — 누계가
        // 기록한 미주 시작점(`current_start_height`)은 본문 마지막 줄 간격 등을 렌더와 다르게
        // 실어 미주 영역이 몇 px 어긋난다. 본문 항목이 없는 단만 기록된 시작점에서 그린다.
        let column_has_body_items = st
            .current_items
            .iter()
            .any(|item| page_item_para_index(item).is_some_and(|pi| pi < paragraphs.len()));
        let origin_recorded = st.current_start_height > 0.0 && !column_has_body_items;
        let is_endnote_item = |item: &&PageItem| {
            !origin_recorded || page_item_para_index(item).is_none_or(|pi| pi >= paragraphs.len())
        };
        let mut local_paras: Vec<Paragraph> = Vec::new();
        let mut local_indices: Vec<(usize, usize)> = Vec::new();
        for pi in st
            .current_items
            .iter()
            .chain(extra_item)
            .filter(is_endnote_item)
            .filter_map(page_item_para_index)
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
                PageItem::FullParagraph { para_index } => {
                    lookup_local(*para_index).map(|l| PageItem::FullParagraph { para_index: l + 1 })
                }
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
                    straddle_row_relief,
                    straddle_page_bands,
                    fragment_row_heights,
                    fragment_block_frames,
                    end_row_owns_stored_frame,
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
                    straddle_row_relief: straddle_row_relief.clone(),
                    straddle_page_bands: straddle_page_bands.clone(),
                    fragment_row_heights: fragment_row_heights.clone(),
                    fragment_block_frames: fragment_block_frames.clone(),
                    end_row_owns_stored_frame: *end_row_owns_stored_frame,
                }),
                PageItem::Shape {
                    para_index,
                    control_index,
                } => lookup_local(*para_index).map(|l| PageItem::Shape {
                    para_index: l + 1,
                    control_index: *control_index,
                }),
                // 구분선은 시작점이 기록된 단에서는 제외한다(start_height 가 그 여백을 이미
                // 반영). 시작점이 없는 단은 렌더처럼 본문·구분선부터 차례로 그린다.
                PageItem::EndnoteSeparator { .. } => (!origin_recorded).then(|| item.clone()),
            }
        };
        let extra_local = extra_item.and_then(&remap);
        let local_items: Vec<PageItem> = st
            .current_items
            .iter()
            .filter(is_endnote_item)
            .filter_map(&remap)
            .chain(extra_local)
            .collect();
        if local_items.is_empty() {
            return None;
        }
        // build_single_column 은 양수 start_height 를 무시(음수 shift 만 적용)하므로,
        // 단이 본문 아래에서 시작(start>0)하면 col_area.y 에 그 오프셋을 실어 동일 프레임에서
        // 렌더한다. 음수(vpos 되감김)는 col_area.y=0 + start_height 음수 shift 로 처리.
        let col_y = if origin_recorded {
            st.current_start_height
        } else {
            0.0
        };
        let col_area = crate::renderer::page_layout::LayoutRect {
            x: 0.0,
            y: col_y,
            width: en_col_w,
            height: (available - col_y).max(0.0),
        };
        // 로컬 문단 순서(더미 0 포함)의 미주 출처. 마지막 로컬 문단 뒤에는 원래 흐름의
        // 다음 미주 문단 출처를 이어 붙여, 렌더와 같은 "같은 미주의 다음 문단" 판정을 받게 한다.
        let endnote_source = |pi: usize| {
            pi.checked_sub(paragraphs.len())
                .and_then(|local| st.endnote_para_sources.get(local))
                .cloned()
        };
        let dummy_source = crate::renderer::pagination::EndnoteParaSource {
            section_index: usize::MAX,
            para_index: usize::MAX,
            control_index: usize::MAX,
            cell_path: Vec::new(),
            note_para_index: usize::MAX,
            tail_leading_consumed_hu: 0,
        };
        let successor_source = local_indices
            .last()
            .and_then(|(global, _)| endnote_source(global + 1));
        // 같은 단의 본문 para 는 단 앞쪽에 모인다. 그 뒤(+1 오프셋)부터 미주 출처를 매겨
        // 본문 para 가 실렌더처럼 미주 출처 없이 그려지게 한다.
        let body_prefix = local_indices
            .iter()
            .take_while(|(global, _)| *global < paragraphs.len())
            .count();
        let body_is_prefix = local_indices[body_prefix..]
            .iter()
            .all(|(global, _)| *global >= paragraphs.len());
        let source_base = if body_is_prefix { 1 + body_prefix } else { 0 };
        let sourced_locals = if body_is_prefix {
            &local_indices[body_prefix..]
        } else {
            &local_indices[..]
        };
        let local_sources: Vec<crate::renderer::pagination::EndnoteParaSource> = (!body_is_prefix)
            .then(|| dummy_source.clone())
            .into_iter()
            .chain(
                sourced_locals
                    .iter()
                    .map(|(global, _)| endnote_source(*global).unwrap_or(dummy_source.clone())),
            )
            .chain(successor_source)
            .collect();
        // 현재 단의 어울림 anchor 를 로컬 인덱스(+1)로 옮긴다. anchor 문단이 이 단에 없으면
        // 어떤 로컬 문단과도 같지 않은 값으로 둔다(레이아웃은 동일성 비교만 한다).
        let to_local = |global: usize| lookup_local(global).map(|l| l + 1);
        let placements = crate::renderer::layout::EndnoteColumnPlacements {
            wrap_anchors: st
                .current_column_wrap_anchors
                .iter()
                .filter_map(|(para, anchor)| {
                    let mut anchor = anchor.clone();
                    anchor.anchor_para_index =
                        to_local(anchor.anchor_para_index).unwrap_or(usize::MAX);
                    to_local(*para).map(|local| (local, anchor))
                })
                .collect(),
            inline_placements: st
                .inline_placements
                .iter()
                .filter_map(|((para, ctrl), v)| to_local(*para).map(|l| ((l, *ctrl), v.clone())))
                .collect(),
            inline_flow_plans: st
                .inline_flow_plans
                .iter()
                .filter_map(|(para, v)| to_local(*para).map(|l| (l, v.clone())))
                .collect(),
            stored_paragraph_flow_ends: st
                .stored_paragraph_flow_ends
                .iter()
                .filter_map(|(para, end)| to_local(*para).map(|local| (local, *end)))
                .collect(),
            host_compositions: st
                .host_compositions
                .iter()
                .filter_map(|(para, comp)| to_local(*para).map(|local| (local, comp.clone())))
                .collect(),
            stored_column_top_spacings: st
                .stored_column_top_spacings
                .iter()
                .filter_map(|(para, spacing)| to_local(*para).map(|local| (local, *spacing)))
                .collect(),
            paragraph_float_placements: st
                .paragraph_float_placements
                .iter()
                .filter_map(|((para, ctrl), v)| to_local(*para).map(|l| ((l, *ctrl), v.clone())))
                .collect(),
            float_pushed_line_spacings: st
                .float_pushed_line_spacings
                .iter()
                .filter_map(|(para, spacing)| to_local(*para).map(|local| (local, *spacing)))
                .collect(),
            page_band_flow_starts: st
                .page_band_flow_starts
                .iter()
                .filter_map(|(para, start)| to_local(*para).map(|local| (local, *start)))
                .collect(),
        };
        let scratch = crate::renderer::layout::LayoutEngine::new(self.dpi);
        self.prime_section_cell_context(&scratch);
        // 렌더 셋업과 같은 출처 프로필·정규화를 싣는다. 프로필이 기본값이면 HWP3 변환본의
        // 수식 줄 들여쓰기 배율이 두 배가 되어 이어지는 수식 줄이 좁아지고 한 줄 더 꺾인다.
        scratch.set_layout_profile(st.profile);
        scratch.set_hwp3_origin_flow_spacing_before(st.profile.hwp3_origin_flow_spacing_before());
        scratch.set_render_normalization_overlay(std::sync::Arc::clone(&self.render_normalization));
        let (bottom, ink_bottom) = scratch.measure_endnote_column_extent(
            local_items,
            &a3_paras,
            &a3_composed,
            styles,
            &col_area,
            st.current_start_height,
            st.section_index,
            (
                st.endnote_separator_above_hu,
                st.endnote_between_notes_hu,
                st.endnote_separator_below_hu,
            ),
            (source_base, &local_sources),
            placements,
            &[],
        );
        if ssot_debug {
            eprintln!(
                "EN_COLSIM start_h={:.1} avail={:.1} items={} bottom={:.1} ink={:?}",
                st.current_start_height,
                available,
                local_indices.len(),
                bottom,
                ink_bottom,
            );
        }
        Some((bottom, ink_bottom))
    }
}
