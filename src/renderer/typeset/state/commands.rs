//! 확정한 항목·흐름·소유권 표시의 적용. fit 조건을 다시 계산하지 않는다.
use super::TypesetState;
use crate::renderer::typeset::{
    DeferredSquarePictureControl, EndnoteRef, HeaderFooterApply, HeaderFooterRef, PageItem,
    PaginationResult, Paragraph, VisibleFloatExclusion,
};

impl TypesetState {
    pub(in crate::renderer::typeset) fn replace_deferred_table_successor_start(
        &mut self,
        start: Option<usize>,
    ) -> Option<usize> {
        std::mem::replace(&mut self.data.deferred_table_successor_start, start)
    }

    pub(in crate::renderer::typeset) fn defer_table_terminal_gap(
        &mut self,
        para_index: usize,
        control_index: usize,
    ) {
        self.data.pending_table_terminal_gap = Some(super::data::DeferredTableTerminalGap {
            para_index,
            control_index,
            flow_end: self.data.current_height,
            page_index: self.data.pages.len(),
            column_index: self.data.current_column,
        });
    }

    /// 지연 표 뒤 줄은 원래 host가 아니라 현재 물리 흐름에서 증명한다.
    /// 사이 문단이 공간을 소비하거나 쪽/단이 바뀌면 그 계약은 종료한다.
    pub(in crate::renderer::typeset) fn consume_pending_table_terminal_gap(
        &mut self,
        para_index: usize,
        paragraphs: &[Paragraph],
        styles: &crate::renderer::style_resolver::ResolvedStyleSet,
        dpi: f64,
    ) {
        let Some(witness) = self.data.pending_table_terminal_gap.take() else {
            return;
        };
        if witness.page_index != self.data.pages.len()
            || witness.column_index != self.data.current_column
            || witness.flow_end != self.data.current_height
        {
            return;
        }
        let Some(para) = paragraphs.get(para_index) else {
            return;
        };
        let Some(crate::model::control::Control::Table(table)) = paragraphs
            .get(witness.para_index)
            .and_then(|host| host.controls.get(witness.control_index))
        else {
            return;
        };
        if let Some(gap) = crate::renderer::float_placement::stored_terminal_empty_line_gap(
            para,
            table,
            styles,
            self.data.profile,
            witness.flow_end,
            dpi,
        ) {
            if std::env::var_os("RHWP_DIAG_SCAN").is_some() {
                eprintln!(
                    "DIAG_SCAN DEFERRED_TERMINAL_GAP pi={} ci={} next={} flow={:.3} gap={:.3}",
                    witness.para_index, witness.control_index, para_index, witness.flow_end, gap,
                );
            }
            self.advance_flow_by(gap);
            self.record_stored_paragraph_flow_end(witness.para_index, self.data.current_height);
        } else if para.controls.is_empty() && !super::super::para_has_visible_text(para) {
            // 글자가 없다고 줄 상자를 없애지 않는다. 기존 문단 fit가 그 줄을
            // 수용하면 실제 흐름 전진으로 다음 호출에서 witness가 폐기된다.
            self.data.pending_table_terminal_gap = Some(witness);
        }
    }

    pub(in crate::renderer::typeset) fn discard_terminal_blank_only_page(
        &mut self,
        paragraphs: &[Paragraph],
    ) {
        super::finalize::discard_terminal_blank_only_page(&mut self.data.pages, paragraphs);
    }
    pub(in crate::renderer::typeset) fn finalize_pages(
        &mut self,
        entries: &[(usize, HeaderFooterRef, bool, HeaderFooterApply)],
        number: &Option<crate::model::control::PageNumberPos>,
        paragraphs: &[Paragraph],
    ) {
        super::finalize::finalize_pages(&mut self.data.pages, entries, number, paragraphs);
    }
    pub(in crate::renderer::typeset) fn append_item(&mut self, item: PageItem) {
        // 본문 표의 공통 이어받기도 등록된 미주 문단의 소속을 유지한다.
        // 단 전환은 이 표시를 초기화하므로 각 조각의 소유에서 다시 게시한다.
        if crate::renderer::typeset::page_item_para_index(&item).is_some_and(|index| {
            self.data.endnote_para_base.is_some_and(|base| {
                index >= base && index - base < self.data.endnote_paragraphs.len()
            })
        }) {
            self.data.current_endnote_flow = true;
        }
        self.data.current_items.push(item);
    }
    pub(in crate::renderer::typeset) fn remove_last_item(&mut self) -> Option<PageItem> {
        self.data.current_items.pop()
    }
    pub(in crate::renderer::typeset) fn advance_flow_by(&mut self, height: f64) {
        self.data.current_height += height;
    }
    pub(in crate::renderer::typeset) fn reclaim_flow_by(&mut self, height: f64) {
        self.data.current_height -= height;
    }
    pub(in crate::renderer::typeset) fn align_flow_to(&mut self, height: f64) {
        self.data.current_height = height;
    }
    pub(in crate::renderer::typeset) fn collect_endnote(&mut self, note: EndnoteRef) {
        self.data.endnotes.push(note);
    }
    pub(in crate::renderer::typeset) fn suppress_endnotes(&mut self) {
        self.data.endnotes.clear();
    }
    pub(in crate::renderer::typeset) fn extend_synthetic_wrap_rects(
        &mut self,
        rects: Vec<(f64, f64, f64, f64)>,
    ) {
        self.data.wrap_synth_rects.extend(rects);
    }
    pub(in crate::renderer::typeset) fn defer_behind_wrap_absorption(
        &mut self,
        para: crate::renderer::pagination::WrapAroundPara,
    ) {
        self.data.behind_pending_absorbs.push(para);
    }
    pub(in crate::renderer::typeset) fn defer_square_picture(
        &mut self,
        picture: DeferredSquarePictureControl,
    ) {
        self.data.deferred_next_page_square_pictures.push(picture);
    }
    /// 저장 프레임의 다음 쪽 소유를 현재 호스트 흐름과 분리해 보관한다.
    pub(in crate::renderer::typeset) fn defer_stored_frame(
        &mut self,
        picture: crate::renderer::typeset::DeferredStoredFrameControl,
    ) {
        let data = &mut *self.data;
        if let Some(index) = picture.following_body_para_index {
            data.deferred_source_frame_body_pages
                .insert(index, data.pages.len() + 2);
        }
        if matches!(
            picture.kind,
            super::super::DeferredStoredFrameKind::AbsoluteShape
        ) {
            // 원래 호스트에 도달하기 전 쪽 시작에서 이미 소유한 개체다.
            data.page_tail_spilled_floats
                .insert((picture.para_index, picture.control_index));
        }
        data.deferred_next_page_stored_frames.push(picture);
    }
    /// OLE를 넘기더라도 현재 단의 뒤 본문은 남은 예산을 계속 소비한다.
    pub(in crate::renderer::typeset) fn defer_table_host_ole_to_next_column(
        &mut self,
        para_idx: usize,
        ctrl_idx: usize,
        ctrl: &crate::model::control::Control,
        dpi: f64,
    ) -> bool {
        if self.data.current_column + 1 >= self.data.col_count {
            return false;
        }
        let crate::model::control::Control::Shape(shape) = ctrl else {
            return false;
        };
        let anchor_y = self
            .data
            .deferred_next_column_stored_frames
            .iter()
            .map(|frame| frame.placement.occupied_bottom)
            .fold(self.data.pending_body_wide_top_reserve, f64::max);
        let Some(placement) =
            crate::renderer::float_placement::column_ole_placement(shape, anchor_y, dpi)
        else {
            return false;
        };
        let required_height = placement.occupied_bottom - placement.anchor_y;
        if self.data.current_height < 1.0
            || self.data.current_height + required_height <= self.available_height() + 0.5
            || placement.occupied_bottom > self.available_height()
        {
            return false;
        }
        self.data.deferred_next_column_stored_frames.push(
            crate::renderer::typeset::DeferredStoredFrameControl {
                kind: crate::renderer::typeset::DeferredStoredFrameKind::Picture,
                para_index: para_idx,
                control_index: ctrl_idx,
                placement,
                following_body_para_index: None,
            },
        );
        true
    }
    pub(in crate::renderer::typeset) fn activate_next_column_stored_frames(&mut self) {
        if self.data.deferred_next_column_stored_frames.is_empty() {
            return;
        }
        for deferred in std::mem::take(&mut self.data.deferred_next_column_stored_frames) {
            self.data.current_height = self
                .data
                .current_height
                .max(deferred.placement.occupied_bottom);
            self.data.paragraph_float_placements.insert(
                (deferred.para_index, deferred.control_index),
                deferred.placement,
            );
            self.data.page_start_stored_frames.push(deferred);
        }
        self.data.current_start_height = self.data.current_height;
    }

    /// 뒤 본문이 없거나 명시적 쪽 나눔이 있어도 이월 개체의 원래 쪽 소유를 닫는다.
    pub(in crate::renderer::typeset) fn finish_next_column_stored_frames(&mut self) {
        if !self.data.deferred_next_column_stored_frames.is_empty() {
            self.advance_column_or_new_page();
            self.flush_column();
        }
    }
    pub(in crate::renderer::typeset) fn apply_deferred_source_frame_body_page(
        &mut self,
        index: usize,
    ) {
        if let Some(page) = self.data.deferred_source_frame_body_pages.remove(&index) {
            while self.data.pages.len() < page {
                self.advance_column_or_new_page();
            }
        }
    }
    pub(in crate::renderer::typeset) fn mark_pre_emitted_host(&mut self, index: usize) {
        self.data.pre_emitted_host_paras.insert(index);
    }
    pub(in crate::renderer::typeset) fn record_pre_emitted_host_height(
        &mut self,
        index: usize,
        height: f64,
        content_height: f64,
    ) {
        self.data.pre_emitted_host_heights.insert(index, height);
        self.data
            .pre_emitted_host_content_heights
            .insert(index, content_height);
    }
    pub(in crate::renderer::typeset) fn mark_prefilled_paragraph(&mut self, index: usize) {
        self.data.prefilled_paras.insert(index);
    }
    pub(in crate::renderer::typeset) fn record_prefilled_line_prefix(
        &mut self,
        index: usize,
        end_line: usize,
    ) {
        self.data.prefilled_line_prefixes.insert(index, end_line);
    }
    pub(in crate::renderer::typeset) fn take_prefilled_line_prefix(
        &mut self,
        index: usize,
    ) -> usize {
        self.data
            .prefilled_line_prefixes
            .remove(&index)
            .unwrap_or(0)
    }
    pub(in crate::renderer::typeset) fn add_visible_float_exclusion(
        &mut self,
        exclusion: VisibleFloatExclusion,
    ) {
        self.data.visible_float_exclusions.push(exclusion);
    }
    pub(in crate::renderer::typeset) fn require_strict_following_text_fit(&mut self) {
        self.data.strict_plain_text_fit_after_empty_host_float_once = true;
    }
    pub(in crate::renderer::typeset) fn mark_vpos_ladder_dirty(&mut self) {
        self.data.vpos_ladder_dirty = true;
    }
    pub(in crate::renderer::typeset) fn record_compacted_stored_rows(&mut self, delta: f64) {
        self.data.vpos_compacted_stored_delta += delta.max(0.0);
    }
    pub(in crate::renderer::typeset) fn request_vpos_reset_after_queued_footnote(&mut self) {
        self.data.reset_vpos_after_queued_table_footnote_page = true;
    }
    pub(in crate::renderer::typeset) fn finish_behind_float_absorption(&mut self) {
        self.data.behind_float_table_para = None;
    }
    pub(in crate::renderer::typeset) fn mark_compact_endnote_rewind(&mut self) {
        self.data.column_had_compact_endnote_rewind = true;
    }
    pub(in crate::renderer::typeset) fn mark_endnote_flow(&mut self) {
        self.data.current_endnote_flow = true;
    }
    pub(in crate::renderer::typeset) fn allow_tail_safety_margin_once(&mut self) {
        self.data.skip_safety_margin_once = true;
    }
    pub(in crate::renderer::typeset) fn allow_tail_footnote_margin_once(&mut self) {
        self.data.skip_footnote_margin_once = true;
    }
    pub(in crate::renderer::typeset) fn mark_page_absolute_top_table(&mut self) {
        self.data.page_has_page_abs_top_table = true;
    }
    pub(in crate::renderer::typeset) fn finish_deferred_host_line(&mut self) {
        self.data.defer_host_line_item_para = None;
    }
}

impl TypesetState {
    pub(in crate::renderer::typeset) fn reserve_saved_tail_bounds(
        &mut self,
        value: Option<(f64, f64)>,
    ) {
        self.data.tail_saved_bounds_once = value;
    }
    pub(in crate::renderer::typeset) fn acknowledge_queued_footnote_reset(&mut self, value: bool) {
        self.data.reset_vpos_after_queued_table_footnote_page = value;
    }
    pub(in crate::renderer::typeset) fn record_previous_layout_paragraph(
        &mut self,
        value: Option<usize>,
    ) {
        self.data.vpos_prev_layout_para = value;
    }
    pub(in crate::renderer::typeset) fn record_previous_partial_table(&mut self, value: bool) {
        self.data.vpos_prev_partial_table = value;
    }
    pub(in crate::renderer::typeset) fn record_vpos_snapped_flow_start(
        &mut self,
        para_idx: usize,
        y: f64,
    ) {
        self.data.vpos_snapped_flow_start = Some((para_idx, y));
    }
    pub(in crate::renderer::typeset) fn record_vpos_lazy_origin(&mut self, value: Option<i32>) {
        self.data.vpos_lazy_base = value;
    }
    pub(in crate::renderer::typeset) fn record_rejected_stored_forward_origin(
        &mut self,
        value: Option<(usize, i32)>,
    ) {
        self.data.rejected_stored_forward_origin = value;
    }
    pub(in crate::renderer::typeset) fn record_stored_column_origin(
        &mut self,
        value: Option<(i32, f64)>,
    ) {
        self.data.vpos_stored_column_origin = value;
    }
    /// 수용한 줄의 구성 결과를 같은 단의 실제 배치에 전달한다.
    pub(in crate::renderer::typeset) fn record_computed_paragraph(
        &mut self,
        index: usize,
        composed: &crate::renderer::composer::ComposedParagraph,
    ) {
        self.data
            .computed_paragraphs
            .insert(index, composed.clone());
    }
    pub(in crate::renderer::typeset) fn discard_pending_paragraph_origins(&mut self) {
        self.data.pending_paragraph_origins.clear();
    }
    /// 같은 단에서 실제로 이웃한 전체 문단만 하나의 미확정 소비로 이어 받는다.
    pub(in crate::renderer::typeset) fn queue_paragraph_origin(
        &mut self,
        paragraph_index: usize,
        spacing_before: f64,
        advance: f64,
        tolerance: f64,
    ) {
        let data = &mut *self.data;
        let continues = data.pending_paragraph_origins.last().is_some_and(|last| {
            last.paragraph_index.checked_add(1) == Some(paragraph_index)
                && (last.flow_end - data.current_height).abs() <= tolerance
                && data.current_items.len().checked_sub(2).is_some_and(|index| {
                    matches!(data.current_items.get(index), Some(PageItem::FullParagraph { para_index })
                        if *para_index == last.paragraph_index)
                })
        });
        if !continues {
            data.pending_paragraph_origins.clear();
        }
        data.pending_paragraph_origins
            .push(super::data::PendingParagraphOrigin {
                paragraph_index,
                spacing_before,
                flow_start: data.current_height,
                flow_end: data.current_height + advance,
            });
    }
    /// 후속 원본 줄이 연속 소비를 입증한 뒤에만 배치 결과로 공개한다.
    pub(in crate::renderer::typeset) fn confirm_pending_paragraph_origins(&mut self, base: i32) {
        for pending in std::mem::take(&mut self.data.pending_paragraph_origins) {
            self.record_paragraph_origin(crate::renderer::pagination::ParagraphOrigin {
                paragraph_index: pending.paragraph_index,
                spacing_before: pending.spacing_before,
                source_vpos_base: base,
                flow_start: pending.flow_start,
                flow_end: pending.flow_end,
            });
        }
    }
    /// fit 뒤 확정한 문단 소비와 원본 단 축을 함께 기록한다.
    pub(in crate::renderer::typeset) fn record_paragraph_origin(
        &mut self,
        origin: crate::renderer::pagination::ParagraphOrigin,
    ) {
        self.data
            .paragraph_origins
            .insert(origin.paragraph_index, origin);
        self.data.vpos_page_base = Some(origin.source_vpos_base);
        self.data.vpos_lazy_base = None;
        self.data.vpos_col_anchor = 0.0;
        self.data.vpos_page_base_stored = true;
        self.data.vpos_ladder_dirty = false;
    }
    pub(in crate::renderer::typeset) fn record_vpos_page_origin(&mut self, value: Option<i32>) {
        self.data.vpos_page_base = value;
    }
    pub(in crate::renderer::typeset) fn record_vpos_origin_provenance(&mut self, value: bool) {
        self.data.vpos_page_base_stored = value;
    }
    pub(in crate::renderer::typeset) fn record_vpos_ladder_validity(&mut self, value: bool) {
        self.data.vpos_ladder_dirty = value;
    }
    pub(in crate::renderer::typeset) fn record_vpos_column_anchor(&mut self, value: f64) {
        self.data.vpos_col_anchor = value;
    }
    pub(in crate::renderer::typeset) fn arm_behind_float_absorption(
        &mut self,
        value: Option<usize>,
    ) {
        self.data.behind_float_table_para = value;
    }
    pub(in crate::renderer::typeset) fn reserve_body_wide_top(&mut self, value: f64) {
        self.data.pending_body_wide_top_reserve = value;
    }
    pub(in crate::renderer::typeset) fn defer_host_line(&mut self, value: Option<usize>) {
        self.data.defer_host_line_item_para = value;
    }
    pub(in crate::renderer::typeset) fn record_inline_flow_bottom(&mut self, value: f64) {
        self.data.inline_box_flow_bottom = value;
    }
    pub(in crate::renderer::typeset) fn record_column_flow_origin(&mut self, value: f64) {
        self.data.current_start_height = value;
    }
    pub(in crate::renderer::typeset) fn record_endnote_between_margin(&mut self, value: i32) {
        self.data.endnote_between_notes_hu = value;
    }
    pub(in crate::renderer::typeset) fn record_endnote_separator_above(&mut self, value: i32) {
        self.data.endnote_separator_above_hu = value;
    }
    pub(in crate::renderer::typeset) fn record_endnote_separator_below(&mut self, value: i32) {
        self.data.endnote_separator_below_hu = value;
    }
    pub(in crate::renderer::typeset) fn finish_stored_wrap_matching(&mut self) {
        self.data.wrap_around_cs = -1;
        self.data.wrap_around_sw = -1;
        self.data.wrap_around_any_seg = false;
        self.data.wrap_around_derived_band = false;
    }
    pub(in crate::renderer::typeset) fn arm_control_wrap(
        &mut self,
        cs: i32,
        sw: i32,
        para_index: usize,
        any_segment: bool,
    ) {
        self.data.wrap_around_cs = cs;
        self.data.wrap_around_sw = sw;
        self.data.wrap_around_table_para = para_index;
        self.data.wrap_around_any_seg = any_segment;
        self.data.wrap_around_derived_band = false;
    }
    pub(in crate::renderer::typeset) fn protect_current_ladder_floor(&mut self) {
        self.data.ladder_band_floor = self.data.ladder_band_floor.max(self.data.current_height);
    }
    /// Keep the source host origin separate from the object's physical box.
    /// Following stored paragraph offsets use this origin; closing flow uses
    /// the occupied bottom supplied by the accepted object placement.
    pub(in crate::renderer::typeset) fn register_square_band(
        &mut self,
        source_origin: f64,
        bottom: f64,
    ) {
        self.data.square_band_bottom = self.data.square_band_bottom.max(bottom);
        self.data.square_band_top = Some(source_origin);
    }
    pub(in crate::renderer::typeset) fn record_ladder_band_table(&mut self, key: (usize, usize)) {
        self.data.current_ladder_band_tables.push(key);
    }
    pub(in crate::renderer::typeset) fn record_inline_placement(
        &mut self,
        key: (usize, usize),
        placement: crate::renderer::float_placement::InlineBoxPlacement,
    ) {
        self.data.inline_placements.insert(key, placement);
    }
    /// [#7330] 자리차지 표가 민 host 첫 글줄의 앞 간격을 흐름에 한 번 소비하고 기록한다.
    /// 배치는 이 기록을 확정 앞 간격으로 받아 같은 값을 소비한다.
    pub(in crate::renderer::typeset) fn consume_float_pushed_line_spacing(
        &mut self,
        para_idx: usize,
        spacing: f64,
    ) {
        self.data.current_height += spacing;
        self.data
            .float_pushed_line_spacings
            .insert(para_idx, spacing);
    }
    /// [#7677] 흐름이 이미 전진한 몫(마지막 표 조각 뒤 바깥 아래 여백)을 host 첫 글줄의 확정
    /// 앞 간격으로 **기록만** 한다 — 배치는 조각 하단에서 그만큼 내려 같은 자리에 줄을 둔다.
    pub(in crate::renderer::typeset) fn record_float_pushed_line_spacing(
        &mut self,
        para_idx: usize,
        spacing: f64,
    ) {
        self.data
            .float_pushed_line_spacings
            .insert(para_idx, spacing);
    }
    pub(in crate::renderer::typeset) fn record_paragraph_float_placement(
        &mut self,
        key: (usize, usize),
        placement: crate::renderer::float_placement::ParagraphFloatPlacement,
    ) {
        self.data.paragraph_float_placements.insert(key, placement);
    }
    /// fit이 수용한 저장 문단 끝을 paint에 그대로 전달한다.
    pub(in crate::renderer::typeset) fn record_stored_paragraph_flow_end(
        &mut self,
        para_idx: usize,
        end: f64,
    ) {
        self.data.stored_paragraph_flow_ends.insert(para_idx, end);
    }

    pub(in crate::renderer::typeset) fn record_stored_column_top_spacing(
        &mut self,
        para_idx: usize,
        frame: crate::renderer::float_placement::StoredColumnTopSpacing,
    ) {
        self.data.vpos_page_base = Some(frame.source_origin_hu);
        self.data.vpos_lazy_base = None;
        self.data.vpos_col_anchor = self.data.current_height;
        self.data.vpos_stored_column_origin =
            Some((frame.source_origin_hu, self.data.current_height));
        self.data.stored_column_top_spacings.insert(para_idx, frame);
    }
    /// 뒤 줄간격과 문단 아래 여백을 제외한 확정 본문 하단을 보존한다.
    pub(in crate::renderer::typeset) fn record_paragraph_content_bottom(
        &mut self,
        key: (usize, usize),
        content_height: f64,
        flow_advance: f64,
    ) {
        let data = &mut *self.data;
        data.paragraph_fragment_content_bottoms.insert(
            key,
            super::paragraph_fragment::ParagraphFragmentBounds {
                content_bottom: data.current_height + content_height,
                flow_end: data.current_height + flow_advance,
                stored_frame: None,
            },
        );
    }
    pub(in crate::renderer::typeset) fn append_endnote_paragraph(
        &mut self,
        paragraph: crate::model::paragraph::Paragraph,
    ) {
        self.data.endnote_paragraphs.push(paragraph);
    }
    pub(in crate::renderer::typeset) fn begin_endnote_paragraphs(&mut self, base: usize) {
        self.data.endnote_para_base = Some(base);
    }
    pub(in crate::renderer::typeset) fn record_reissued_endnote_rows(
        &mut self,
        local_index: usize,
        tail_leading_consumed_hu: i32,
    ) {
        self.data
            .reissued_endnote_rows
            .insert(local_index, tail_leading_consumed_hu);
    }
    pub(in crate::renderer::typeset) fn append_endnote_measured_table(
        &mut self,
        table: crate::renderer::height_measurer::MeasuredTable,
    ) {
        self.data.endnote_measured_tables.push(table);
    }
    pub(in crate::renderer::typeset) fn append_endnote_source(
        &mut self,
        mut source: crate::renderer::pagination::EndnoteParaSource,
    ) {
        source.tail_leading_consumed_hu = self
            .data
            .reissued_endnote_rows
            .get(&self.data.endnote_para_sources.len())
            .copied()
            .unwrap_or(0);
        self.data.endnote_para_sources.push(source);
    }
    pub(in crate::renderer::typeset) fn mark_fragment_footnotes_queued(
        &mut self,
        key: (usize, usize),
    ) {
        self.data.fragment_queued_table_footnotes.insert(key);
    }
}

impl TypesetState {
    pub(in crate::renderer::typeset) fn initialize_source(
        &mut self,
        hide_empty_line: bool,
        profile: crate::model::provenance::LayoutCompatibilityProfile,
        has_stored_line_segs: bool,
        skip_spacing_before_prededuct: bool,
    ) {
        self.data.hide_empty_line = hide_empty_line;
        self.data.profile = profile;
        self.data.has_stored_line_segs = has_stored_line_segs;
        self.data.skip_spacing_before_prededuct = skip_spacing_before_prededuct;
    }
    pub(in crate::renderer::typeset) fn classify_omitted_spacing(&mut self, omitted: bool) {
        self.data.omit_fresh_recalc_doc = omitted;
    }
    pub(in crate::renderer::typeset) fn mark_stored_spacing_omitted(&mut self) {
        self.data.stored_ladder_spacing_omitted = true;
    }
    pub(in crate::renderer::typeset) fn initialize_zone_spacing(&mut self, spacing: f64) {
        self.data.current_zone_design_spacing_px = spacing;
    }
    pub(in crate::renderer::typeset) fn record_footer_presence(&mut self, no_footer: bool) {
        self.data.section_has_no_footer = no_footer;
    }
    pub(in crate::renderer::typeset) fn observe_following_paragraph(
        &mut self,
        vpos: Option<i32>,
        empty_float: bool,
        plain_text: bool,
        lane_probe: Option<crate::renderer::float_placement::StoredLineLaneProbe>,
    ) {
        self.data.next_para_first_stored_vpos = vpos;
        self.data.next_para_lane_probe = lane_probe;
        self.data.next_para_is_empty_float_table_anchor = empty_float;
        self.data.next_para_is_plain_text = plain_text;
    }
    pub(in crate::renderer::typeset) fn enter_column_definition(&mut self, count: u16) {
        self.data.col_count = count;
    }
    pub(in crate::renderer::typeset) fn install_zone_layout(
        &mut self,
        layout: crate::renderer::page_layout::PageLayoutInfo,
        column_type: crate::model::page::ColumnType,
    ) {
        self.data.current_zone_layout = Some(layout.clone());
        self.data.layout = layout;
        self.data.current_zone_column_type = column_type;
    }
    /// 명시적으로 만든 새 쪽의 첫 영역과 편집용 쪽 정보에 같은 단 배치를 발행한다.
    /// 쪽 중간 영역 전환은 install_zone_layout을 사용해 기존 쪽 정보를 보존한다.
    pub(in crate::renderer::typeset) fn install_page_start_layout(
        &mut self,
        layout: crate::renderer::page_layout::PageLayoutInfo,
        column_type: crate::model::page::ColumnType,
    ) {
        if let Some(page) = self.data.pages.last_mut() {
            page.layout = layout.clone();
        }
        self.install_zone_layout(layout, column_type);
    }
    pub(in crate::renderer::typeset) fn advance_zone_origin(&mut self, height: f64) {
        self.data.current_zone_y_offset += height;
    }
    pub(in crate::renderer::typeset) fn align_zone_origin(&mut self, height: f64) {
        self.data.current_zone_y_offset = height;
    }
    pub(in crate::renderer::typeset) fn restart_zone_columns(&mut self) {
        self.data.current_column = 0;
        self.data.current_height = 0.0;
        self.data.on_first_multicolumn_page = true;
    }
    pub(in crate::renderer::typeset) fn reserve_bottom_fixed_flow(
        &mut self,
        consumed: f64,
        exclusion: f64,
    ) {
        self.data.bottom_fixed_consumed_flow += consumed;
        self.data.current_bottom_fixed_exclusion = exclusion;
    }
    pub(in crate::renderer::typeset) fn prepend_endnotes(
        &mut self,
        mut notes: Vec<crate::renderer::pagination::EndnoteRef>,
    ) {
        notes.append(&mut self.data.endnotes);
        self.data.endnotes = notes;
    }
    pub(in crate::renderer::typeset) fn record_current_footnote(
        &mut self,
        note: crate::renderer::pagination::FootnoteRef,
    ) {
        if let Some(page) = self.data.pages.last_mut() {
            page.footnotes.push(note);
        }
    }
    pub(in crate::renderer::typeset) fn mark_deferred_hwpx_note_body(&mut self) {
        self.data.deferred_hwpx_note_body = true;
    }
    pub(in crate::renderer::typeset) fn shift_endnote_render_lines(
        &mut self,
        index: usize,
        delta: i32,
    ) {
        if let Some(render_para) = self.data.endnote_paragraphs.get_mut(index) {
            for ls in &mut render_para.line_segs {
                ls.vertical_pos += delta;
            }
        }
    }
    pub(in crate::renderer::typeset) fn retract_endnote_render_lines(
        &mut self,
        index: usize,
        delta: i32,
    ) {
        if let Some(render_para) = self.data.endnote_paragraphs.get_mut(index) {
            for ls in &mut render_para.line_segs {
                ls.vertical_pos -= delta;
            }
        }
    }
    pub(in crate::renderer::typeset) fn attach_pending_behind_absorptions(&mut self) {
        for wrap_para in std::mem::take(&mut self.data.behind_pending_absorbs) {
            let anchor = wrap_para.table_para_index;
            let is_first_fragment = |it: &PageItem| match it {
                PageItem::Table { para_index, .. } => *para_index == anchor,
                PageItem::PartialTable {
                    para_index,
                    is_continuation,
                    ..
                } => *para_index == anchor && !*is_continuation,
                _ => false,
            };
            let mut attached = false;
            'outer: for page in self.data.pages.iter_mut() {
                for col in page.column_contents.iter_mut() {
                    if col.items.iter().any(is_first_fragment) {
                        col.wrap_around_paras.push(wrap_para.clone());
                        attached = true;
                        break 'outer;
                    }
                }
            }
            if !attached {
                // anchor 미발견(예: 표가 배치 제외) — 마지막 단에 부착해 문단 누락 방지
                if let Some(col) = self
                    .data
                    .pages
                    .last_mut()
                    .and_then(|p| p.column_contents.last_mut())
                {
                    col.wrap_around_paras.push(wrap_para);
                }
            }
        }
    }
    pub(in crate::renderer::typeset) fn into_result(
        self,
    ) -> crate::renderer::pagination::PaginationResult {
        let data = self.data.value;
        PaginationResult {
            pages: data.pages,
            wrap_around_paras: Vec::new(),
            hidden_empty_paras: data.hidden_empty_paras,
            pre_emitted_host_paras: data.pre_emitted_host_paras,
            pre_emitted_host_heights: data.pre_emitted_host_heights,
            pre_emitted_host_content_heights: data.pre_emitted_host_content_heights,
            endnotes: data.endnotes,
            endnote_paragraphs: data.endnote_paragraphs,
            endnote_para_sources: data.endnote_para_sources,
            endnote_measured_tables: data.endnote_measured_tables,
            endnote_between_notes_hu: data.endnote_between_notes_hu,
            endnote_separator_above_hu: data.endnote_separator_above_hu,
            endnote_separator_below_hu: data.endnote_separator_below_hu,
        }
    }
}

impl TypesetState {
    pub(in crate::renderer::typeset) fn record_reclaimed_host_spacing(&mut self, height: f64) {
        self.data.hangul2024_reclaimed += height;
    }
    pub(in crate::renderer::typeset) fn invalidate_vpos_after_clearance(&mut self, clearance: f64) {
        self.data.vpos_ladder_dirty |= clearance > 0.0;
    }
    fn owns_endnote_tail(&self, paragraph_index: usize, line_count: usize) -> bool {
        self.data.current_items.iter().any(|item| match item {
            PageItem::FullParagraph { para_index } => *para_index == paragraph_index,
            PageItem::PartialParagraph {
                para_index,
                end_line,
                ..
            } => *para_index == paragraph_index && *end_line == line_count,
            _ => false,
        })
    }
    /// 미주 사이 0 경계는 앞 미주의 내용 높이를 보존하고 마지막 줄 간격만 회수한다.
    /// 앞 단의 tail이나 아직 마지막 줄을 소비하지 않은 조각은 변경하지 않는다.
    pub(in crate::renderer::typeset) fn reclaim_endnote_tail_spacing(
        &mut self,
        index: usize,
        paragraph_index: usize,
        dpi: f64,
    ) -> i32 {
        let Some(paragraph) = self.data.endnote_paragraphs.get(index) else {
            return 0;
        };
        let Some(line) = paragraph.line_segs.last() else {
            return 0;
        };
        let spacing = line.line_spacing.max(0);
        if spacing == 0 || !self.owns_endnote_tail(paragraph_index, paragraph.line_segs.len()) {
            return 0;
        }
        if let Some(line) = self.data.endnote_paragraphs[index].line_segs.last_mut() {
            line.line_spacing = 0;
        }
        self.data.current_height -= crate::renderer::hwpunit_to_px(spacing, dpi);
        spacing
    }
    pub(in crate::renderer::typeset) fn apply_endnote_tail_spacing(
        &mut self,
        index: usize,
        paragraph_index: usize,
        spacing: i32,
        dpi: f64,
    ) -> f64 {
        // 내용 높이에 옮겨 담은 leading을 새 미주 간격으로 다시 예약하지 않는다.
        // source vpos의 전체 전진은 보존하며 실제 남은 빈 밴드만 발행한다.
        let consumed = self
            .data
            .reissued_endnote_rows
            .get(&index)
            .copied()
            .unwrap_or(0);
        let spacing = spacing.saturating_sub(consumed).max(0);
        let Some(paragraph) = self.data.endnote_paragraphs.get_mut(index) else {
            return 0.0;
        };
        let line_count = paragraph.line_segs.len();
        let Some(line) = paragraph.line_segs.last_mut() else {
            return 0.0;
        };
        let added = spacing.saturating_sub(line.line_spacing).max(0);
        line.line_spacing = spacing;
        // 간격은 완성된 마지막 줄이 소유한다. 앞 단의 tail이나 현재 단의
        // 미완성 조각에 같은 공간을 다시 청구하지 않는다.
        let owns_tail = self.owns_endnote_tail(paragraph_index, line_count);
        if owns_tail {
            let reserved = crate::renderer::hwpunit_to_px(added, dpi);
            self.data.current_height += reserved;
            reserved
        } else {
            0.0
        }
    }
    pub(in crate::renderer::typeset) fn append_routed_item(
        &mut self,
        page_idx: usize,
        col_idx: usize,
        item: PageItem,
    ) {
        if let Some(col) = self
            .data
            .pages
            .get_mut(page_idx)
            .and_then(|p| p.column_contents.get_mut(col_idx))
        {
            col.items.push(item);
        } else {
            self.data.current_items.push(item);
        }
    }
    pub(in crate::renderer::typeset) fn finish_paragraph_float_flow(
        &mut self,
        para_idx: usize,
        spacing_after: f64,
    ) {
        let data = &mut *self.data;
        for (&(owner, _), placement) in &data.paragraph_float_placements {
            if owner == para_idx
                && matches!(
                    placement.flow,
                    crate::renderer::float_placement::ParagraphFloatFlow::NextLine
                        | crate::renderer::float_placement::ParagraphFloatFlow::EmptyHost
                )
            {
                data.current_height = placement.paragraph_end(data.current_height, spacing_after);
                data.ladder_band_floor = data.ladder_band_floor.max(data.current_height);
            }
        }
    }
}
