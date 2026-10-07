//! 현재 흐름에서 통째 표의 수용 여부와 원본 프레임을 조회한다.
//! 실제 배치와 흐름 전진은 호출자가 같은 결과로 처리한다.

use crate::renderer::typeset::{
    controls, hwpunit_to_px, is_para_topbottom_float, is_synthetic_line_seg,
    line_seg_visible_bounds_px, native_hwp5_saved_rowbreak_tail_frame_matches,
    para_has_non_whitespace_text, para_has_visible_text,
    rowbreak_table_has_internal_saved_vpos_reset, signed_hwpunit, table,
    table_declared_height_has_stored_cell_content_frame,
    table_declared_object_covers_cell_row_frames, PageItem, TypesetEngine, TypesetState,
};

use super::BlockTableInput;

pub(super) struct WholeFitInput {
    pub(super) next_starts_new_page: bool,
    pub(super) next_rewinds_after_table: bool,
    pub(super) host_spacing_total: f64,
    pub(super) table_total: f64,
    pub(super) available: f64,
    pub(super) declared_object_total: f64,
    pub(super) single_row_object_declared_fits_current: bool,
}

pub(super) struct WholeFit {
    pub(super) para_has_stored_line_seg: bool,
    pub(super) single_row_object_height_advance: Option<f64>,
    pub(super) fits_after_overlay_shapes: bool,
    pub(super) stored_rewinding_rowbreak_uses_painted_row_footprint: bool,
    pub(super) whole_fit_table_total: f64,
    pub(super) hwpx_noninline_tac_measured_fit: bool,
    pub(super) declared_table_whole_fits: bool,
    pub(super) saved_table_source_frame: Option<(f64, f64)>,
    pub(super) closed_source_frame_placement:
        Option<crate::renderer::float_placement::ParagraphFloatPlacement>,
}

impl TypesetEngine {
    /// A complete source cell can own two physical row frames even when
    /// sequential measurement would fit both. Whole placement must retain
    /// the same source boundary consumed by the row scanner and paint.
    pub(in crate::renderer::typeset) fn stored_two_line_row_frames_require_split(
        &self,
        table: &crate::model::table::Table,
        styles: &crate::renderer::style_resolver::ResolvedStyleSet,
    ) -> bool {
        if !self.profile.get().hwp5_stored_pagination_layout()
            || self.profile.get().session_edited()
            || table.common.treat_as_char
            || table.page_break != crate::model::table::TablePageBreak::RowBreak
        {
            return false;
        }
        let engine = crate::renderer::layout::LayoutEngine::new(self.dpi);
        engine.set_layout_profile(self.profile.get());
        engine.set_render_normalization_overlay(std::sync::Arc::clone(&self.render_normalization));
        (0..usize::from(table.row_count))
            .any(|row| engine.native_saved_two_line_row_frame(table, row, styles))
    }

    /// 원본 공동 앵커의 첫 수용 원점과 이월 후 소비된 오프셋을 함께 조회한다.
    /// 예약 하단과 출력 원점을 한 계획으로 반환하며, 편집·분할·절대 배치는 제외한다.
    #[allow(clippy::too_many_arguments)]
    pub(in crate::renderer::typeset) fn query_stored_whole_flow_anchor(
        &self,
        st: &TypesetState,
        para_idx: usize,
        ctrl_idx: usize,
        para: &crate::model::paragraph::Paragraph,
        table: &crate::model::table::Table,
        effective_height: f64,
    ) -> Option<crate::renderer::float_placement::ParagraphFloatPlacement> {
        use crate::renderer::float_placement as placement;
        if !(st.profile.hwp5_stored_pagination_layout() || st.profile.hwpx_stored_layout())
            || st.profile.session_edited()
            || st.col_count != 1
            || !st.current_items.is_empty()
            || st.current_height > 0.5
            || para.stored_text_partition_is_dirty()
            || para.line_segs.is_empty()
            || para.line_segs.iter().any(is_synthetic_line_seg)
            || para_has_non_whitespace_text(para)
            || !is_para_topbottom_float(&table.common)
            || !table.common.flow_with_text
            || table.common.allow_overlap
            || !matches!(table.page_break, crate::model::table::TablePageBreak::None)
            || !matches!(table.common.vert_align, crate::model::shape::VertAlign::Top)
            || table.caption.is_some()
            || self.render_normalization.table_text_reflowed(table)
        {
            return None;
        }
        let line = controls::order::stored_cross_column_flow_line(
            para,
            ctrl_idx,
            st.base_available_height(),
            self.dpi,
        );
        // 실제 앞쪽에 놓인 같은 줄의 흐름 표만 앵커 소비를 증명한다.
        // 배열상 앞 형제나 다른 저장 줄의 TAC는 아직 놓이지 않은 표를 대신하지 않는다.
        let prior_line_flow = line.is_some_and(|line| {
            // 새 쪽을 만들면 마지막 PageContent는 아직 빈 현재 쪽이다.
            // 같은 구역의 실제 소유 단이 있는 직전 쪽을 찾아야 한다.
            st.pages
                .iter()
                .rev()
                .find(|page| !page.column_contents.is_empty())
                .filter(|page| page.section_index == st.section_index)
                .is_some_and(|page| {
                    page.column_contents
                        .iter()
                        .flat_map(|column| &column.items)
                        .any(|item| {
                            matches!(item, PageItem::Table { para_index, control_index }
                            | PageItem::PartialTable { para_index, control_index, .. }
                    if *para_index == para_idx
                        && *control_index != ctrl_idx
                        && controls::order::stored_cross_column_flow_line(
                            para, *control_index, st.base_available_height(), self.dpi,
                        ) == Some(line))
                        })
                })
        });
        let consumed = prior_line_flow
            || placement::para_offset_consumed_by_page_break(
                para,
                &table.common,
                st.base_available_height(),
                self.dpi,
            );
        if line.is_none() && !consumed {
            return None;
        }
        let top = st.current_height
            + hwpunit_to_px(table.outer_margin_top as i32, self.dpi)
            + if consumed {
                0.0
            } else {
                hwpunit_to_px(signed_hwpunit(table.common.vertical_offset), self.dpi)
            };
        Some(placement::ParagraphFloatPlacement {
            flow: placement::ParagraphFloatFlow::NextLine,
            anchor_y: st.current_height,
            stored_host_origin: None,
            stored_successor_line_origin: None,
            table_left: None,
            table_top: top,
            occupied_bottom: top
                + effective_height
                + hwpunit_to_px(table.outer_margin_bottom as i32, self.dpi),
        })
    }

    /// 선방출한 Native 캡션과 첫 표 조각은 저장 문단 앵커를 함께 사용한다.
    /// 줄 전진량을 뺀 오프셋을 문단 기준 좌표로 다시 해석하지 않는다.
    pub(in crate::renderer::typeset) fn query_pre_emitted_caption_rowbreak_placement(
        &self,
        st: &TypesetState,
        para_idx: usize,
        para: &crate::model::paragraph::Paragraph,
        table: &crate::model::table::Table,
    ) -> Option<crate::renderer::float_placement::ParagraphFloatPlacement> {
        use crate::renderer::float_placement as placement;
        if !st.profile.hwp5_stored_pagination_layout()
            || st.profile.session_edited()
            || st.col_count != 1
            || para.stored_text_partition_is_dirty()
            || !crate::renderer::typeset::native_hwp5_rowbreak_host_precedes_first_fragment(
                para, table,
            )
            || !matches!(table.common.vert_align, crate::model::shape::VertAlign::Top)
            || para.line_segs.iter().any(is_synthetic_line_seg)
            || para.line_segs.windows(2).any(|pair| {
                pair[1].vertical_pos < pair[0].vertical_pos
                    || pair[1].text_start < pair[0].text_start
            })
            || !st.current_items.iter().any(|item| {
                matches!(item,
                PageItem::PartialParagraph { para_index, start_line: 0, end_line }
                    if *para_index == para_idx && *end_line == para.line_segs.len())
            })
        {
            return None;
        }
        let first = para.line_segs.iter().find(|seg| seg.line_height > 0)?;
        let last = para
            .line_segs
            .iter()
            .rev()
            .find(|seg| seg.line_height > 0)?;
        let frame_vpos = st.vpos_page_base.unwrap_or(0);
        let (anchor, _) = line_seg_visible_bounds_px(first, frame_vpos, self.dpi)?;
        let (_, end) = line_seg_visible_bounds_px(last, frame_vpos, self.dpi)?;
        if end > st.base_available_height() + 0.5 {
            return None;
        }
        // 저장 위치는 쪽 본문 기준이고 공통 계획은 현재 단 영역 기준이다.
        let anchor = anchor - st.current_zone_y_offset;
        if anchor < 0.0 {
            return None;
        }
        let top = anchor
            + hwpunit_to_px(signed_hwpunit(table.common.vertical_offset), self.dpi)
            + hwpunit_to_px(table.outer_margin_top as i32, self.dpi);
        Some(placement::ParagraphFloatPlacement {
            flow: placement::ParagraphFloatFlow::NextLine,
            anchor_y: anchor,
            stored_host_origin: Some(anchor),
            stored_successor_line_origin: None,
            table_left: None,
            table_top: top,
            occupied_bottom: top,
        })
    }

    /// 실제 조각 예산으로 저장된 닫힌 개체 프레임의 유효성을 확인한다.
    /// 통째 배치와 이월 후 스캐너 진입이 이 결과를 함께 소비한다.
    pub(in crate::renderer::typeset) fn query_closed_source_frame_placement(
        &self,
        st: &TypesetState,
        paragraphs: &[crate::model::paragraph::Paragraph],
        para_idx: usize,
        table: &crate::model::table::Table,
        effective_height: f64,
        available: f64,
    ) -> Option<crate::renderer::float_placement::ParagraphFloatPlacement> {
        if st.col_count != 1
            || st.current_height > 0.5
            || !(st.profile.hwp5_stored_pagination_layout() || st.profile.hwpx_stored_layout())
            || st.profile.session_edited()
            || self.render_normalization.table_text_reflowed(table)
        {
            return None;
        }
        let frame = crate::renderer::float_placement::stored_table_frame_with_guides(
            paragraphs, para_idx, table,
        )?;
        if (effective_height - hwpunit_to_px(table.common.height as i32, self.dpi)).abs() > 0.5 {
            return None;
        }
        let bottom = hwpunit_to_px(frame.bottom_hu, self.dpi);
        (bottom <= available).then_some(crate::renderer::float_placement::ParagraphFloatPlacement {
            flow: crate::renderer::float_placement::ParagraphFloatFlow::NextLine,
            anchor_y: 0.0,
            stored_host_origin: None,
            stored_successor_line_origin: None,
            table_left: None,
            table_top: hwpunit_to_px(frame.top_hu, self.dpi),
            occupied_bottom: bottom,
        })
    }

    /// 원본 호스트와 뒤 저장 줄이 닫는 전체 개체 프레임을 조회한다.
    /// 수용 예산 때문에 유효 원점을 버리지 않는다. 호출자가 같은 하단으로 fit을 판정한다.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn query_original_control_table_frame(
        &self,
        st: &TypesetState,
        paragraphs: &[crate::model::paragraph::Paragraph],
        para_idx: usize,
        ctrl_idx: usize,
        table: &crate::model::table::Table,
        effective_height: f64,
        host_spacing_before: f64,
    ) -> Option<crate::renderer::float_placement::ParagraphFloatPlacement> {
        if st.col_count != 1
            || !(st.profile.hwpx_stored_layout() || st.profile.hwp5_stored_pagination_layout())
            || st.profile.session_edited()
            || self.render_normalization.table_text_reflowed(table)
        {
            return None;
        }
        let para = paragraphs.get(para_idx)?;
        let next = paragraphs.get(para_idx + 1)?;
        let mut placement = crate::renderer::float_placement::stored_float_frame_before_tac_line(
            para,
            ctrl_idx,
            table,
            effective_height,
            self.dpi,
        )
        .or_else(|| {
            crate::renderer::float_placement::stored_interior_control_table_frame(
                para,
                next,
                ctrl_idx,
                table,
                effective_height,
                st.vpos_page_base.unwrap_or(0),
                self.dpi,
            )
        })
        .or_else(|| {
            crate::renderer::float_placement::stored_empty_control_table_frame(
                para,
                next,
                table,
                effective_height,
                host_spacing_before,
                // 빈 호스트의 닫힌 개체 프레임은 물리 쪽 기준 저장 좌표다.
                // 글줄 호스트의 상대 원점처럼 page base를 다시 빼지 않는다.
                0,
                self.dpi,
            )
        })
        .or_else(|| {
            crate::renderer::float_placement::stored_adjacent_line_table_frame(
                paragraphs.get(para_idx.checked_sub(1)?)?,
                para,
                next,
                table,
                effective_height,
                self.dpi,
            )
        })?;
        // 저장한 본문 좌표를 현재 단 영역 좌표로 한 번 변환한다.
        placement.anchor_y -= st.current_zone_y_offset;
        placement.table_top -= st.current_zone_y_offset;
        placement.occupied_bottom -= st.current_zone_y_offset;
        placement.stored_successor_line_origin = placement
            .stored_successor_line_origin
            .map(|origin| origin - st.current_zone_y_offset);
        (placement.table_top >= 0.0).then_some(placement)
    }

    /// 폭0 개체 앵커는 표·캡션 바깥 상자를 소유한다. 선언된 표 높이만으로
    /// 본문 예산을 넘는 캡션을 수용할 수 없다.
    pub(in crate::renderer::typeset) fn query_captioned_column_rowbreak_placement(
        &self,
        st: &TypesetState,
        para: &crate::model::paragraph::Paragraph,
        table: &crate::model::table::Table,
        host_before: f64,
        table_and_caption_height: f64,
    ) -> Option<crate::renderer::float_placement::ParagraphFloatPlacement> {
        use crate::renderer::float_placement as placement;
        let opens = placement::column_rowbreak_fragment_opens_outer_top(
            st.profile.hwpx_stored_layout(),
            (st.profile.hwpx_stored_layout() || st.profile.hwp5_stored_pagination_layout())
                .then_some(para),
            table,
            false,
            0,
            &[],
            st.current_height <= 0.5,
        );
        let signed_offset = signed_hwpunit(table.common.vertical_offset);
        // 이 계획은 문단에 고정된 개체를 앞으로 전진시킨다. 절대 기준 좌표,
        // 가운데·아래 정렬과 뒤쪽 앵커는 기존 위치 결정 경로를 유지하며,
        // 앞으로 진행하는 흐름 상자로 재해석하지 않는다.
        if !opens
            || !matches!(
                table.common.vert_rel_to,
                crate::model::shape::VertRelTo::Para
            )
            || !matches!(
                table.common.vert_align,
                crate::model::shape::VertAlign::Top | crate::model::shape::VertAlign::Inside
            )
            || signed_offset < 0
            || !placement::object_only_saved_table_anchor(para, table)
            || !table.caption.as_ref().is_some_and(|caption| {
                matches!(
                    caption.direction,
                    crate::model::shape::CaptionDirection::Top
                        | crate::model::shape::CaptionDirection::Bottom
                )
            })
        {
            return None;
        }
        let offset_consumed = st.current_items.is_empty()
            && st.current_height < 1.0
            && placement::para_offset_consumed_by_page_break(
                para,
                &table.common,
                st.base_available_height(),
                self.dpi,
            );
        let top = st.current_height
            + host_before
            + if offset_consumed {
                0.0
            } else {
                hwpunit_to_px(signed_offset, self.dpi)
            };
        Some(placement::ParagraphFloatPlacement {
            flow: placement::ParagraphFloatFlow::NextLine,
            anchor_y: st.current_height,
            stored_host_origin: None,
            stored_successor_line_origin: None,
            table_left: None,
            table_top: top,
            occupied_bottom: top
                + table_and_caption_height
                + placement::column_rowbreak_caption_outer_spacing_px(opens, para, table, self.dpi),
        })
    }

    pub(super) fn query_whole_table_fit(
        &self,
        st: &TypesetState,
        input: BlockTableInput<'_>,
        fit: WholeFitInput,
    ) -> WholeFit {
        let BlockTableInput {
            para_idx,
            ctrl_idx,
            para,
            table,
            ft,
            fmt,
            mt,
            paragraphs_all,
            ..
        } = input;
        let WholeFitInput {
            next_starts_new_page,
            next_rewinds_after_table,
            host_spacing_total,
            table_total,
            available,
            declared_object_total,
            single_row_object_declared_fits_current,
        } = fit;
        let para_has_stored_line_seg = para.line_segs.iter().any(|ls| !is_synthetic_line_seg(ls));
        let single_row_object_height_fits_current = single_row_object_declared_fits_current;
        // HWP5-origin HWPX는 저장 object 높이 기준으로는 현재 쪽에 들어가지만,
        // cell 내용 측정치는 더 큰 1행 자리차지 표를 쪽 하단까지 차지한 것으로
        // 기록한다. native HWP는 RowBreak fragment의 실제 소비량을 따라야 하므로
        // 이 HWPX 호환 보정을 적용하지 않는다.
        let single_row_object_height_advance =
            (st.profile.hwp5_origin_hwpx() && single_row_object_height_fits_current).then(|| {
                if st.current_height + table_total > available {
                    (available - st.current_height).max(0.0)
                } else {
                    declared_object_total
                }
            });

        let fits_after_overlay_shapes =
            st.current_column_has_only_overlay_shapes() && table_total <= available + 12.0;
        // [#3820] 저장된 쪽 끝 일반 RowBreak 표는 whole-fit 판단이
        // 저장 common.height(`table_total`)만 보면, renderer가 실제로 paint할 행
        // footprint보다 작게 판정해 footer 아래까지 행을 보존한다. source의 다음
        // 문단 vpos rewind가 physical fragment 경계를 명시하고, rowspan/cell-footnote가
        // 없는 ordinary-row 형상에서만 measured row footprint를 권위로 삼는다.
        // 같은 저장 경계를 가진 미편집 HWPX도 동일 계약을 소비한다. 편집·재조판
        // HWPX, 쪽 상단 표, 행 병합 및 실제 행 내부 컷은 기존 경로를 유지한다.
        let stored_rewinding_rowbreak_uses_painted_row_footprint = (st.profile.hwp5_stored_pagination_layout()
                || (st.profile.hwpx_stored_layout()
                    && !st.profile.session_edited()
                    && st.col_count == 1
                    && !self.render_normalization.table_text_reflowed(table)))
                && !table.common.treat_as_char
                && is_para_topbottom_float(&table.common)
                && matches!(
                    table.page_break,
                    crate::model::table::TablePageBreak::RowBreak
                )
                && table.row_count > 1
                && ft.table_footnotes.is_empty()
                && st.current_height >= st.base_available_height() * 0.5
                && table.cells.iter().all(|cell| cell.row_span == 1)
                // 물리 조각 경계는 뒤 호스트뿐 아니라 마지막 셀의 저장 줄에도
                // 기록될 수 있다. 둘 다 원본이 소유한 되감김이며, 셀 안 경계를
                // 무시하면 선언 높이만 보는 판단이 실제 행을 하나 더 수용한다.
                && (next_rewinds_after_table
                    || rowbreak_table_has_internal_saved_vpos_reset(table));
        let measured_row_table_height = mt.as_ref().and_then(|measured| {
            (!measured.row_heights.is_empty()).then(|| {
                measured.row_heights.iter().sum::<f64>()
                    + measured.cell_spacing * measured.row_heights.len().saturating_sub(1) as f64
            })
        });
        // 재조판한 온전한 일반 행도 실제 배치 높이로 통째 수용 여부를 정한다.
        // 원본의 저장 되감김/프레임 원점 조건과 별개이며 중첩 내용 컷은 제외한다.
        let reflowed_table_uses_painted_whole_rows = if table.row_count > 0
            && matches!(
                table.page_break,
                crate::model::table::TablePageBreak::RowBreak
            )
            && self.render_normalization.table_text_reflowed(table)
        {
            let layout_engine = crate::renderer::layout::LayoutEngine::new(self.dpi);
            layout_engine.set_layout_profile(st.profile);
            layout_engine.prime_column_layout_env(&st.layout);
            layout_engine.set_render_normalization_overlay(std::sync::Arc::clone(
                &self.render_normalization,
            ));
            (0..table.row_count as usize)
                .all(|row| layout_engine.reflowed_fragment_row_uses_measured_height(table, row))
        } else {
            false
        };
        let uses_painted_row_footprint_for_whole_fit =
            (stored_rewinding_rowbreak_uses_painted_row_footprint
                || reflowed_table_uses_painted_whole_rows)
                && measured_row_table_height
                    .is_some_and(|height| height > ft.effective_height + 0.5);
        let whole_fit_table_total = if uses_painted_row_footprint_for_whole_fit {
            table_total.max(measured_row_table_height.unwrap_or(0.0) + host_spacing_total)
        } else {
            table_total
        };
        if std::env::var("RHWP_TABLE_DRIFT").is_ok() {
            eprintln!(
                "TABLE_PAINT_FOOTPRINT pi={} native={} rewind={} measured={:.1} effective={:.1} whole_fit={}",
                para_idx,
                st.profile.hwp5_stored_pagination_layout(),
                next_rewinds_after_table,
                measured_row_table_height.unwrap_or(0.0),
                ft.effective_height,
                uses_painted_row_footprint_for_whole_fit,
            );
        }
        // [#2097/#2105] 한글의 실제 행높이 합은 저장 선언 높이와 일치한다(1730000
        // 새만금 COM 3자 비교: 저장 910.5px = 한글 910.6px vs rhwp 실측 954.1px).
        // 셀 내용 실측 팽창으로 측정 fit 이 실패해도 선언 높이가 현재 쪽에 들어가면
        // 통째 배치해 마지막 행 sliver 여분 페이지를 막는다. 쪽나눔=None(#2097)은
        // 한글이 행 컷하지 않는 표, RowBreak(#2105, 19378753 밀양시 907.7px 선언
        // vs 955.9px 실측)는 "나눔 허용"이지 강제가 아니라 선언 fit 시 한글도 나누지
        // 않는다 — 선언이 fit 하지 않는 다쪽 표의 분할 의미론은 불변. CellBreak 는
        // 셀 중간 컷 의미론이 별개라 비대상. advance 는 측정 table_total 을 유지해
        // 같은 쪽 후속 겹침을 차단한다.
        // 새 조각의 RowBreak 표는 앞 흐름이 없어 선언 높이와 충돌하지 않는다.
        // 조각 중간에서는 원본 호스트가 개체 하단을 같은 본문 안에 기록했을 때만
        // 선언을 신뢰한다. 실측 초과량만으로 글꼴 메트릭 차이와 원본 조각 경계를
        // 구분할 수 없다.
        let rowbreak_at_fragment_start = st.current_items.is_empty();
        let midpage_rowbreak_has_saved_object_bottom = para
            .line_segs
            .iter()
            .find(|ls| !is_synthetic_line_seg(ls))
            .is_some_and(|seg| {
                let base = st.vpos_page_base.unwrap_or(0);
                let v_off = signed_hwpunit(table.common.vertical_offset);
                let top_hu = seg
                    .vertical_pos
                    .saturating_add(v_off.max(0))
                    .saturating_sub(base);
                let bottom_hu =
                    top_hu.saturating_add(table.common.height.min(i32::MAX as u32) as i32);
                hwpunit_to_px(bottom_hu, self.dpi) <= available
            });
        let declared_fit_scope_ok = match table.page_break {
            crate::model::table::TablePageBreak::None => true,
            crate::model::table::TablePageBreak::RowBreak => {
                rowbreak_at_fragment_start || midpage_rowbreak_has_saved_object_bottom
            }
            crate::model::table::TablePageBreak::CellBreak => false,
        };
        // 텍스트 셀의 저장 줄 프레임이 각 선언 셀 안에 들어가고 표 개체 프레임이
        // 선언 행 형상을 소유할 때만 선언 셀 상자를 신뢰한다. 비율이나 상한만으로
        // 브라우저 측정 팽창과 실제로 큰 원본 행을 구분할 수 없다. 셀 프레임만으로도
        // 오래된 짧은 표 개체와 원본이 소유한 RowBreak 조각을 구분할 수 없다.
        let declared_excess_has_source_frame =
            table_declared_height_has_stored_cell_content_frame(table, self.dpi)
                && (!matches!(
                    table.page_break,
                    crate::model::table::TablePageBreak::RowBreak
                ) || table_declared_object_covers_cell_row_frames(table, self.dpi));
        // HWPX에서는 `treatAsChar` bit만으로 inline 표가 되지 않는다. stored-layout
        // 문서의 `flowWithText=0` 표는 block table인데, raw bit를 그대로 사용하면
        // declared whole-fit에서 제외되어 generic row cut이 저장 row height를 다시
        // 팽창시키고, 실제로 들어가는 표까지 다음 쪽으로 조기 이월한다 (#3820 p144).
        // 이 좁은 경로는 쪽나눔=None·footnote 없음·실측 높이 fit을 함께 요구한다.
        // 진짜 inline TAC와 native HWP5의 기존 정책은 `uses_tac_table_flow`에 맡긴다.
        let hwpx_noninline_tac_measured_fit = self.profile.get().hwpx_stored_layout()
            && table.common.treat_as_char
            && !self.uses_tac_table_flow(table)
            && matches!(table.page_break, crate::model::table::TablePageBreak::None)
            && ft.table_footnotes.is_empty()
            && st.current_height + ft.effective_height <= available + 0.5;
        // [편집 세션] 셀 편집으로 실측이 선언을 넘게 자란 표는 선언 기준 whole-fit
        // 이 무의미하다 — 선언으로는 "들어간다"인데 실측은 본문 하단을 넘어,
        // 표가 앞 쪽에 잘린 채 남는다(셀 Enter 재현). 실측을 fit 기준으로 써서
        // 넘치면 이월·스캔 경로로 넘긴다.
        let session_grown_measured_fit = self.profile.get().session_edited()
            && ft.effective_height > declared_object_total + 8.0;
        let declared_fit_height = if hwpx_noninline_tac_measured_fit || session_grown_measured_fit {
            ft.effective_height
        } else {
            declared_object_total
        };
        // 저장 RowBreak 개체 프레임은 본문에 들어가지만 브라우저 측정이 표를
        // 반올림 정도 아래로 둘 수 있다. 비TAC·각주 없음 형상에서는 HWP/HWPX
        // 모두 선언 프레임을 원본 소유 경계로 유지한다. 실제로 큰 표는 실측 본문이
        // 좁은 변환 경계2px를 넘으므로 행 스캐너로 간다.
        const NEAR_MEASURED_ROWBREAK_FIT_PX: f64 = 2.0;
        // 첫 논리 행에서 시작하는 세로 병합은 그 행과 다음 행을 하나의 저장
        // 밴드로 만든다. 선언 개체가 본문에 들어가는데 밴드 앞에서 나누면
        // 테두리만 있는 조각이 남는다.
        let has_leading_rowspan_band = table
            .cells
            .iter()
            .any(|cell| cell.row == 0 && cell.row_span > 1);
        let near_measured_rowbreak_fits = !table.common.treat_as_char
            && matches!(
                table.page_break,
                crate::model::table::TablePageBreak::RowBreak
            )
            && ft.table_footnotes.is_empty()
            // HWPX 저장 조판은 Native HWP5 표 선언과 별도의 페이지 프레임을
            // 보존한다. 실측이 근접한 경우의 수용은 변환 출처의 계약이다.
            // Native HWP5와 HWP5 출처 HWPX는 개체 프레임이 모든 선언 행 형상을
            // 소유함도 입증해야 한다(#5128 표174/193/203/284 통째 흡수 방지).
            && (!st.profile.hwp5_stored_pagination_layout()
                || declared_excess_has_source_frame
                || has_leading_rowspan_band)
            && declared_object_total > host_spacing_total
            && st.current_height + declared_object_total <= available
            && st.current_height + ft.effective_height
                <= available + NEAR_MEASURED_ROWBREAK_FIT_PX;
        // HWPX CELL(RowBreak) TAC 표는 일반적으로 선언 높이가 current fragment의
        // source-owned table frame이다. 단, 단일 빈 host의 유일한 non-synthetic
        // LINE_SEG가 다행 measured table보다 짧으면 그 line은 table band를 소유하지
        // 않는다. 이 예외만 declared-fit에서 제외해 measured table과 뒤 문단이
        // 같은 쪽에 겹치는 것을 막는다. 나머지 CELL/TAC 문서는 기존 declared-fit
        // 호환 경로를 유지한다.
        let hwpx_tac_cell_leftover_missing_owned_line = st.profile.hwpx_stored_layout()
            && table.common.treat_as_char
            && table.common.flow_with_text
            && matches!(
                table.page_break,
                crate::model::table::TablePageBreak::RowBreak
            )
            && para.controls.len() == 1
            && table.row_count == 3
            && table.col_count == 1
            && table.cells.len() == 3
            && !table.repeat_header
            && !para_has_visible_text(para)
            && fmt.line_heights.len() == 1
            && para.line_segs.len() == 1
            && para.line_segs.first().is_some_and(|seg| {
                !is_synthetic_line_seg(seg)
                    && hwpunit_to_px(seg.line_height, self.dpi) + 0.5 < ft.total_height
            });
        // [#6448] HWPX `pageBreak="CELL"`은 모델 RowBreak다. 글자처럼 취급 표는
        // 일반 declared-fit에서 제외되어 measured expansion으로 다음 쪽에 통째
        // 이월될 수 있다. leftover에 declaration이 들어가면 source frame을
        // 존중하되, 위의 누락 host-line 형상은 physical band 경로로 보낸다.
        let hwpx_tac_cell_leftover_declared_fits = st.profile.hwpx_stored_layout()
            && table.common.treat_as_char
            && matches!(
                table.page_break,
                crate::model::table::TablePageBreak::RowBreak
            )
            && ft.table_footnotes.is_empty()
            && declared_object_total > host_spacing_total
            && !st.current_items.is_empty()
            && st.current_height + declared_object_total <= available
            && !hwpx_tac_cell_leftover_missing_owned_line;
        let declared_table_whole_fits = near_measured_rowbreak_fits
            || hwpx_tac_cell_leftover_declared_fits
            || (!uses_painted_row_footprint_for_whole_fit
                && declared_fit_scope_ok
                // 이 HWPX 호환 경로는 선언 개체 대신 실측 표 높이를 사용한다.
                // 이 프로필에서 높이의 권위가 아닌 선언 개체에 모든 셀 행을
                // 덮도록 요구하면 실제로 들어가는 flowWithText=0 표도
                // 선언만을 이유로 행 내부에서 나뉜다.
                && (hwpx_noninline_tac_measured_fit || declared_excess_has_source_frame)
                && !ft.strict_following_plain_text_fit
                && (!table.common.treat_as_char || hwpx_noninline_tac_measured_fit)
                && declared_object_total > host_spacing_total
                && st.current_height + declared_fit_height <= available);
        // 빈 host의 단일 inline 표에서 저장 LineSeg 높이와 table common 높이가
        // 정확히 같으면, 그 LineSeg는 표의 실제 physical frame이다. 누적 측정이
        // source top을 지나쳤더라도 frame 전체가 현재 body 안에 있으면 source
        // frame이 generic table_total보다 page owner를 우선한다.
        let saved_single_inline_table_source_frame = (table.common.treat_as_char
            && table.row_count == 1
            && table.col_count == 1
            && table.cells.len() == 1
            && para.controls.len() == 1
            && !para_has_visible_text(para)
            && ft.table_footnotes.is_empty())
        .then(|| {
            let mut source_lines = para
                .line_segs
                .iter()
                .filter(|seg| !is_synthetic_line_seg(seg));
            let seg = source_lines.next()?;
            if source_lines.next().is_some()
                || seg.line_height != table.common.height.min(i32::MAX as u32) as i32
            {
                return None;
            }
            line_seg_visible_bounds_px(seg, st.vpos_page_base.unwrap_or(0), self.dpi)
        })
        .flatten()
        .filter(|(source_top, source_bottom)| {
            *source_top < st.current_height && *source_bottom <= available
        });
        // 다행 RowBreak 표는 common.height가 첫 fragment만 뜻할 수도 있다. cell
        // 내부 reset 없이 다음 host가 새 물리 page를 명시할 때만, object frame을
        // 현 page 전체를 소유한 frame으로 쓴다.
        let saved_rowbreak_object_frame = ((st.profile.hwpx_container()
            || st.profile.native_hwp5_layout())
            && !table.common.treat_as_char
            && matches!(
                table.page_break,
                crate::model::table::TablePageBreak::RowBreak
            )
            && table.row_count > 1
            && para.controls.len() == 1
            && !para_has_visible_text(para)
            && ft.table_footnotes.is_empty()
            && signed_hwpunit(table.common.vertical_offset) <= 0
            && next_starts_new_page
            // [#7336] 저장 object frame 은 **현재 쪽**의 소유권만 말한다. 선언
            // `common.height` 가 첫 조각만 뜻할 수 있다는 위 계약은 그대로 두되,
            // 실측 표가 본문 한 쪽에도 들어가지 않으면 그 표는 반드시 여러 조각으로
            // 나뉘어야 하므로 선언 frame 이 쪽 소유의 권위가 될 수 없다.
            //
            // `samples/issue7336/stored_frame_page_larger_rowbreak.hwpx` 실측:
            // 선언 frame 901.2px 로 통째 배치했는데 페인터가 3,676px 를 그려 본문
            // 아래로 2,742px 가 넘쳤고 '2-3. 추진일정' 절이 통째로 사라졌다
            // (한/글 2024 7쪽 vs rhwp 4쪽). 이 경우 종전대로 행 컷 스캐너에 맡긴다.
            && table_total - host_spacing_total <= st.base_available_height() + 0.5
            && !rowbreak_table_has_internal_saved_vpos_reset(table))
        .then(|| {
            let mut source_lines = para
                .line_segs
                .iter()
                .filter(|seg| !is_synthetic_line_seg(seg));
            let seg = source_lines.next()?;
            (source_lines.next().is_none())
                .then(|| line_seg_visible_bounds_px(seg, st.vpos_page_base.unwrap_or(0), self.dpi))
                .flatten()
        })
        .flatten()
        .and_then(|(source_top, _)| {
            let source_bottom = source_top + declared_object_total - host_spacing_total;
            let source_frame_matches_profile = if st.profile.hwpx_container() {
                source_top < st.current_height && source_bottom <= available
            } else {
                native_hwp5_saved_rowbreak_tail_frame_matches(
                    source_top,
                    source_bottom,
                    st.current_height,
                    available,
                )
            };
            source_frame_matches_profile.then_some((source_top, source_bottom))
        });
        let saved_table_source_frame =
            saved_single_inline_table_source_frame.or(saved_rowbreak_object_frame);
        let closed_source_frame_placement = self
            .query_closed_source_frame_placement(
                st,
                paragraphs_all,
                para_idx,
                table,
                ft.effective_height,
                available,
            )
            .or_else(|| {
                self.query_original_control_table_frame(
                    st,
                    paragraphs_all,
                    para_idx,
                    ctrl_idx,
                    table,
                    ft.effective_height,
                    fmt.spacing_before,
                )
            });
        WholeFit {
            para_has_stored_line_seg,
            single_row_object_height_advance,
            fits_after_overlay_shapes,
            stored_rewinding_rowbreak_uses_painted_row_footprint,
            whole_fit_table_total,
            hwpx_noninline_tac_measured_fit,
            declared_table_whole_fits,
            saved_table_source_frame,
            closed_source_frame_placement,
        }
    }
}
