//! 미주 렌더 위치·높이 Query. scratch 레이아웃을 사용하며 페이지 상태를 변경하지 않는다.

use crate::renderer::typeset::notes::endnotes::profile::{
    en_ssot_debug, en_ssot_level, EnSsotLevel,
};
use crate::renderer::typeset::{
    compose_paragraph, endnote_first_local_paras, first_text_line, hwpunit_to_px,
    page_item_para_index, para_has_treat_as_char_picture_or_shape, paragraph_by_global_index,
    ComposedParagraph, HeightCursor, MeasuredTable, PageItem, Paragraph, ResolvedStyleSet,
    TypesetEngine, TypesetState,
};

/// 같은 단 배치에서 생산한 흐름 예약 끝과 실제 줄·내용의 점유 끝이다.
pub(super) struct EndnoteColumnGeometry {
    pub(super) flow_bottom: f64,
    pub(super) occupied_bottom: Option<f64>,
}

include!("stored_ink_fit.inc.rs");

/// 한 문단의 fit 판정들이 같은 단을 다시 그리는 호출은 몇 호출 안에서 되풀이된다.
const ENDNOTE_COLUMN_REPLAY_MEMO_LEN: usize = 8;

/// scratch 단 측정 키에 문단 입력을 싣는다. `NumberingMarker` 는 `Debug` 로 값을
/// 드러내지 않으므로 번호 문자열과 머리 속성을 따로 싣는다. 표 칸·주석·글상자 안
/// 문단의 번호는 `{paragraph:?}` 에도 드러나지 않는다. 번호는 조판 전에 정해지고
/// 조판은 `line_segs` 만 고치므로, 구역 진입 때 비우는 이 재사용 안에서 같은 문단의
/// 중첩 번호는 바뀌지 않는다.
fn push_replay_paragraph_key(key: &mut String, index: usize, paragraph: &Paragraph) {
    use crate::model::paragraph::NumberingMarker;
    use std::fmt::Write;
    let marker = match &paragraph.numbering_marker {
        NumberingMarker::Unresolved => None,
        NumberingMarker::Absent => Some(None),
        NumberingMarker::Text(text, head) => Some(Some((text, head))),
    };
    let _ = write!(key, "|{index}:{marker:?}:{paragraph:?}");
}

impl TypesetEngine {
    /// scratch 단 측정은 새 `LayoutEngine` 으로 입력만 소비하는 순수 계산이다. `key` 가
    /// 그 입력 전부를 담으므로 같은 키의 최근 결과를 그대로 돌려준다. 키가 입력을
    /// 빠뜨리면 다른 측정에 같은 결과를 돌려주므로, 검증을 켜면 재사용할 때마다 다시
    /// 측정해 비트 단위로 비교한다.
    fn replay_endnote_column(
        &self,
        key: String,
        measure: impl FnOnce() -> (f64, Option<f64>),
    ) -> (f64, Option<f64>) {
        let hit = self
            .endnote_column_replays
            .borrow()
            .iter()
            .find_map(|(cached, result)| (*cached == key).then_some(*result));
        if let Some(result) = hit {
            if self.endnote_replay_verify {
                let bits =
                    |(bottom, ink): (f64, Option<f64>)| (bottom.to_bits(), ink.map(f64::to_bits));
                let fresh = measure();
                assert_eq!(
                    bits(fresh),
                    bits(result),
                    "미주 단 측정 키가 scratch 입력을 빠뜨렸다: {fresh:?} != {result:?}"
                );
            }
            return result;
        }
        let result = measure();
        let mut replays = self.endnote_column_replays.borrow_mut();
        if replays.len() >= ENDNOTE_COLUMN_REPLAY_MEMO_LEN {
            replays.pop_front();
        }
        replays.push_back((key, result));
        result
    }

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
        self.simulate_endnote_column_with_tail(
            st,
            paragraphs,
            styles,
            available,
            en_col_w,
            extra_para_full,
            sequential_compact_rewind,
            None,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub(in crate::renderer::typeset) fn simulate_endnote_column_with_tail(
        &self,
        st: &TypesetState,
        paragraphs: &[Paragraph],
        styles: &ResolvedStyleSet,
        available: f64,
        en_col_w: f64,
        extra_para_full: Option<usize>,
        sequential_compact_rewind: bool,
        future: Option<(&[Paragraph], i32, bool)>,
    ) -> Option<f64> {
        self.simulate_endnote_column_with_item(
            st,
            paragraphs,
            styles,
            available,
            en_col_w,
            extra_para_full.map(|para_index| PageItem::FullParagraph { para_index }),
            sequential_compact_rewind,
            future,
            &[],
            false,
        )
    }

    /// 분할 후보도 실제 방출할 PartialParagraph로 측정한다.
    /// 이미 배치한 그림 프레임은 현재 단의 paint 문맥 그대로 소비한다.
    #[allow(clippy::too_many_arguments)]
    pub(in crate::renderer::typeset) fn simulate_endnote_column_with_item(
        &self,
        st: &TypesetState,
        paragraphs: &[Paragraph],
        styles: &ResolvedStyleSet,
        available: f64,
        en_col_w: f64,
        extra_item: Option<PageItem>,
        sequential_compact_rewind: bool,
        future: Option<(&[Paragraph], i32, bool)>,
        measured_tables: &[MeasuredTable],
        occupied_only: bool,
    ) -> Option<f64> {
        let use_occupied = occupied_only
            || future.is_some()
            || matches!(extra_item, Some(PageItem::PartialParagraph { .. }));
        let geometry = self.measure_endnote_column_with_item(
            st,
            paragraphs,
            styles,
            available,
            en_col_w,
            extra_item,
            sequential_compact_rewind,
            future,
            measured_tables,
        )?;
        if use_occupied {
            geometry.occupied_bottom
        } else {
            Some(geometry.flow_bottom)
        }
    }

    /// fit 후보가 줄 뒤 물리 간격도 예약해야 할 때 같은 생산 결과를 소비한다.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn measure_endnote_column_with_item(
        &self,
        st: &TypesetState,
        paragraphs: &[Paragraph],
        styles: &ResolvedStyleSet,
        available: f64,
        en_col_w: f64,
        extra_item: Option<PageItem>,
        sequential_compact_rewind: bool,
        future: Option<(&[Paragraph], i32, bool)>,
        measured_tables: &[MeasuredTable],
    ) -> Option<EndnoteColumnGeometry> {
        self.measure_endnote_column_with_items(
            st,
            paragraphs,
            styles,
            available,
            en_col_w,
            extra_item.as_slice(),
            sequential_compact_rewind,
            future,
            measured_tables,
        )
    }

    /// fit 후보의 문단·표·그림을 최종 방출과 같은 순서로 측정한다.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn measure_endnote_column_with_items(
        &self,
        st: &TypesetState,
        paragraphs: &[Paragraph],
        styles: &ResolvedStyleSet,
        available: f64,
        en_col_w: f64,
        extra_items: &[PageItem],
        sequential_compact_rewind: bool,
        future: Option<(&[Paragraph], i32, bool)>,
        measured_tables: &[MeasuredTable],
    ) -> Option<EndnoteColumnGeometry> {
        if st.current_items.is_empty() {
            return None;
        }
        // 앞 미주에서 방출한 표 조각도 최종 paint와 같은 행 측정을 받는다.
        // caller가 본문 측정만 전달하는 준비 조회에서는 소유 원장을 보충한다.
        let mut measured_tables = std::borrow::Cow::Borrowed(measured_tables);
        for table in &st.endnote_measured_tables {
            if !measured_tables.iter().any(|measured| {
                measured.para_index == table.para_index
                    && measured.control_index == table.control_index
            }) {
                measured_tables.to_mut().push(table.clone());
            }
        }
        let measured_tables = measured_tables.as_ref();
        let ssot_level = en_ssot_level();
        let ssot_debug = en_ssot_debug();
        let mut local_paras: Vec<Paragraph> = Vec::new();
        let mut local_indices: Vec<(usize, usize)> = Vec::new();
        for pi in st
            .current_items
            .iter()
            .filter_map(page_item_para_index)
            .chain(extra_items.iter().filter_map(page_item_para_index))
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
            // 실제 paint와 같은 전역 문단 식별자를 보존한다. 로컬 재색인하면
            // 본문/미주 구분, 원본 소유자와 다음 문단의 trailing 간격이 달라진다.
            let future_count = future.map_or(0, |(tail, _, _)| tail.len());
            let future_base = paragraphs.len() + st.endnote_paragraphs.len();
            let mut paint_paras = vec![Paragraph::default(); future_base + future_count];
            for (global, local) in &local_indices {
                paint_paras[*global] = local_paras[*local].clone();
            }
            // 구분선도 실제 흐름 항목이다. 제거하면 본문 후행 간격 회수와
            // 첫 미주 원점이 최종 배치와 달라진다.
            let mut local_items = st.current_items.clone();
            local_items.extend_from_slice(extra_items);
            if local_items.is_empty() {
                return None;
            }
            let mut sources = st.endnote_para_sources.clone();
            if let Some((tail, endnote_start, compact_spacing)) = future {
                // 표/float/글상자는 이 scratch의 실제 측정 문맥이 부족하므로
                // 원래 저장 단 경계를 유지한다. TAC 그림의 명시 크기는 동일 경로로 측정한다.
                let supported = |paragraph: &Paragraph| {
                    paragraph.controls.iter().all(|control| match control {
                        crate::model::control::Control::Table(_)
                        | crate::model::control::Control::Shape(_) => false,
                        crate::model::control::Control::Picture(picture) => {
                            picture.common.treat_as_char
                                && picture.common.width > 0
                                && picture.common.height > 0
                        }
                        _ => true,
                    })
                };
                if local_paras.iter().any(|p| !supported(p))
                    || tail.iter().any(|p| {
                        !supported(p)
                            || p.column_type != crate::model::paragraph::ColumnBreakType::None
                    })
                {
                    return None;
                }
                let owner = sources.last()?.clone();
                for (offset, paragraph) in tail.iter().enumerate() {
                    let para_index = future_base + offset;
                    paint_paras[para_index] = super::row_projection::render_paragraph(
                        paragraph,
                        endnote_start,
                        compact_spacing,
                    );
                    let mut source = owner.clone();
                    source.note_para_index += offset + 1;
                    // 임시 측정도 해당 문단이 실제로 투영한 간격만 소비한다.
                    // 앞 문단의 소유 주소를 복제하면서 그 간격까지 상속하지 않는다.
                    source.tail_leading_consumed_hu = st
                        .reissued_endnote_rows
                        .get(&(para_index - paragraphs.len()))
                        .copied()
                        .unwrap_or(0);
                    sources.push(source);
                    local_items.push(PageItem::FullParagraph { para_index });
                    for (control_index, control) in paragraph.controls.iter().enumerate() {
                        if matches!(control, crate::model::control::Control::Picture(_)) {
                            local_items.push(PageItem::Shape {
                                para_index,
                                control_index,
                            });
                        }
                    }
                }
            }
            // 아직 emit하지 않은 다음 내부 문단도 원본에서 확인한다. 같은 미주의
            // 연속 여부는 현재 항목 목록의 끝과 무관하므로 paint의 소유자 계약을
            // 유지해야 한다. 가상 본문이나 저장 LineSeg를 새로 만들지 않는다.
            if let Some(last) = sources.last().cloned() {
                let parent = paragraphs
                    .get(last.para_index)
                    .and_then(|p| p.controls.get(last.control_index));
                if let Some(crate::model::control::Control::Endnote(note)) = parent {
                    if last.note_para_index + 1 < note.paragraphs.len() {
                        let mut next = last;
                        next.note_para_index += 1;
                        next.tail_leading_consumed_hu = st
                            .reissued_endnote_rows
                            .get(&sources.len())
                            .copied()
                            .unwrap_or(0);
                        sources.push(next);
                    }
                }
            }
            // flush 뒤 실제 paint에 전달되는 단 문맥을 그대로 재생한다.
            // 빈 inline 계획이나 가상 용지 원점은 이미 배치한 표의 높이와
            // 이어지는 미주 원점을 다시 계산하게 하므로 사용할 수 없다.
            let column = crate::renderer::pagination::ColumnContent {
                column_index: st.current_column,
                start_height: st.current_start_height,
                endnote_flow: st.current_endnote_flow,
                items: local_items,
                zone_layout: st.current_zone_layout.clone(),
                zone_y_offset: st.current_zone_y_offset,
                wrap_around_paras: st.current_column_wrap_around_paras.clone(),
                used_height: st.current_height,
                wrap_anchors: st.current_column_wrap_anchors.clone(),
                overlay_continuations: st.current_column_overlay_continuations.clone(),
                overlay_cuts: st.current_column_overlay_cuts.clone(),
                inline_placements: st.inline_placements.clone(),
                inline_flow_plans: st.inline_flow_plans.clone(),
                stored_paragraph_flow_ends: st.stored_paragraph_flow_ends.clone(),
                host_compositions: st.host_compositions.clone(),
                stored_column_top_spacings: st.stored_column_top_spacings.clone(),
                float_pushed_line_spacings: st.float_pushed_line_spacings.clone(),
                page_band_flow_starts: st.page_band_flow_starts.clone(),
                paragraph_float_placements: st.paragraph_float_placements.clone(),
                paragraph_origins: st.paragraph_origins.clone(),
                computed_paragraphs: st.computed_paragraphs.clone(),
            };
            let mut col_area = st.inline_flow_column();
            col_area.width = en_col_w;
            col_area.height = available;
            let mut page = st
                .pages
                .last()
                .cloned()
                .unwrap_or_else(|| st.new_page_content(Vec::new()));
            // flush_column과 같이 현재 단의 후보 항목을 페이지에도 게시한다.
            // 빈 float host의 줄 예약은 페이지가 소유한 Shape 항목을 확인한다.
            // 단만 전달하면 실제 개체가 있는데도 일반 빈 문단으로 측정한다.
            page.column_contents.push(column.clone());
            // `paint_paras` 는 아래 문단 외에는 기본 문단이다. 아래 scratch에 새 입력을
            // 넘기면 이 키에도 싣는다.
            let mut key = format!(
                "bottom|{:p}|{:?}|{}|{}|{:?}|{:?}|{:?}|{:?}|{:?}",
                styles,
                st.profile,
                paragraphs.len(),
                paint_paras.len(),
                (
                    st.endnote_separator_above_hu,
                    st.endnote_between_notes_hu,
                    st.endnote_separator_below_hu,
                ),
                col_area,
                sources,
                measured_tables,
                page,
            );
            for index in local_indices
                .iter()
                .map(|(global, _)| *global)
                .chain(future_base..paint_paras.len())
            {
                push_replay_paragraph_key(&mut key, index, &paint_paras[index]);
            }
            let (flow_bottom, occupied_bottom) = self.replay_endnote_column(key, || {
                let paint_composed: Vec<_> = paint_paras
                    .iter()
                    .map(|p| crate::renderer::composer::compose_paragraph_in_context(p, styles))
                    .collect();
                let scratch = crate::renderer::layout::LayoutEngine::new(self.dpi);
                scratch.set_layout_profile(st.profile);
                scratch.set_render_normalization_overlay(std::sync::Arc::clone(
                    &self.render_normalization,
                ));
                scratch.set_endnote_para_sources(paragraphs.len(), &sources);
                scratch.set_endnote_shape_margins_hu(
                    st.endnote_separator_above_hu,
                    st.endnote_between_notes_hu,
                    st.endnote_separator_below_hu,
                );
                scratch.measure_endnote_column_bottom(
                    &column,
                    &page,
                    &paint_paras,
                    &paint_composed,
                    styles,
                    &col_area,
                    st.endnote_between_notes_hu,
                    measured_tables,
                )
            });
            // 실제 배치는 용지 좌표, typeset 예산은 현재 단 원점 기준이다.
            let flow_bottom = flow_bottom - col_area.y;
            let occupied_bottom = occupied_bottom.map(|bottom| bottom - col_area.y);
            if ssot_debug {
                eprintln!(
                    "EN_COLSIM start_h={:.1} avail={:.1} items={} bottom={:.1}",
                    st.current_start_height,
                    available,
                    local_indices.len(),
                    flow_bottom,
                );
            }
            return Some(EndnoteColumnGeometry {
                flow_bottom,
                occupied_bottom,
            });
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
        hc.endnote_note_first_paras =
            endnote_first_local_paras(paragraphs.len(), &st.endnote_para_sources, &local_indices);
        hc.stored_equation_reflow = !st.profile.session_edited()
            && (st.profile.hwp5_container() || st.profile.hwpx_stored_layout());
        let mut y = st.current_start_height;
        // [#5886] 한 번 compact 되감김이 나오면 렌더처럼 나머지도 순차 적층한다.
        let mut stack_sequential = false;
        for item in st.current_items.iter().chain(extra_items.iter()) {
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
                hc.prev_reissued_equation_rows =
                    crate::renderer::composer::stored_row_projection::reissued_equation_rows(
                        item_para,
                        &item_composed,
                    );
            }
            hc.prev_item_was_partial_table = matches!(item, PageItem::PartialTable { .. });
        }
        Some(EndnoteColumnGeometry {
            flow_bottom: y,
            occupied_bottom: Some(y),
        })
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
            hc.endnote_note_first_paras = endnote_first_local_paras(
                paragraphs.len(),
                &st.endnote_para_sources,
                &local_indices,
            );
            hc.stored_equation_reflow = !st.profile.session_edited()
                && (st.profile.hwp5_container() || st.profile.hwpx_stored_layout());
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
                    hc.prev_reissued_equation_rows =
                        crate::renderer::composer::stored_row_projection::reissued_equation_rows(
                            item_para,
                            &item_composed,
                        );
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
        self.prime_section_cell_context(&scratch);
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
            None,
        );
        (y_after - y_start).max(0.0)
    }
}

#[cfg(test)]
mod replay_memo_tests {
    use super::*;
    use crate::model::paragraph::{MarkerHead, NumberingMarker};

    #[test]
    fn same_column_input_reuses_scratch_measure_until_evicted() {
        let mut engine = TypesetEngine::new(96.0);
        engine.endnote_replay_verify = false;
        let calls = std::cell::Cell::new(0);
        let measure = |bottom: f64| {
            calls.set(calls.get() + 1);
            (bottom, Some(bottom - 1.0))
        };
        let replay = |key: &str, bottom: f64| {
            engine.replay_endnote_column(key.to_string(), || measure(bottom))
        };
        assert_eq!(replay("column", 10.0), (10.0, Some(9.0)));
        assert_eq!(replay("column", 99.0), (10.0, Some(9.0)));
        assert_eq!(calls.get(), 1);
        for index in 0..ENDNOTE_COLUMN_REPLAY_MEMO_LEN {
            replay(&format!("other-{index}"), 1.0);
        }
        assert_eq!(replay("column", 20.0), (20.0, Some(19.0)));
        assert_eq!(calls.get(), ENDNOTE_COLUMN_REPLAY_MEMO_LEN + 2);
    }

    #[test]
    #[should_panic(expected = "미주 단 측정 키가 scratch 입력을 빠뜨렸다")]
    fn verify_rejects_reuse_when_key_misses_an_input() {
        let mut engine = TypesetEngine::new(96.0);
        engine.endnote_replay_verify = true;
        engine.replay_endnote_column("column".to_string(), || (10.0, None));
        engine.replay_endnote_column("column".to_string(), || (10.0, None));
        engine.replay_endnote_column("column".to_string(), || (11.0, None));
    }

    #[test]
    fn numbering_marker_hidden_from_debug_still_separates_keys() {
        let key = |marker: NumberingMarker| {
            let paragraph = Paragraph {
                numbering_marker: marker,
                ..Default::default()
            };
            let mut key = String::new();
            push_replay_paragraph_key(&mut key, 0, &paragraph);
            key
        };
        let text = |number: &str| NumberingMarker::Text(number.into(), MarkerHead::default());
        assert_ne!(key(text("1.")), key(text("2.")));
        assert_ne!(
            key(NumberingMarker::Unresolved),
            key(NumberingMarker::Absent)
        );
        assert_eq!(key(text("1.")), key(text("1.")));
    }
}
