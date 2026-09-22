//! 확정한 항목·흐름·소유권 표시의 적용. fit 조건을 다시 계산하지 않는다.
use super::TypesetState;
use crate::renderer::typeset::{
    DeferredSquarePictureControl, EndnoteRef, HeaderFooterApply, HeaderFooterRef, PageItem,
    PaginationResult, Paragraph, VisibleFloatExclusion,
};

impl TypesetState {
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
    pub(in crate::renderer::typeset) fn mark_pre_emitted_host(&mut self, index: usize) {
        self.data.pre_emitted_host_paras.insert(index);
    }
    pub(in crate::renderer::typeset) fn record_pre_emitted_host_height(
        &mut self,
        index: usize,
        height: f64,
    ) {
        self.data.pre_emitted_host_heights.insert(index, height);
    }
    pub(in crate::renderer::typeset) fn mark_prefilled_paragraph(&mut self, index: usize) {
        self.data.prefilled_paras.insert(index);
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
    pub(in crate::renderer::typeset) fn record_vpos_lazy_origin(&mut self, value: Option<i32>) {
        self.data.vpos_lazy_base = value;
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
    pub(in crate::renderer::typeset) fn register_square_band(&mut self, top: f64, bottom: f64) {
        self.data.square_band_bottom = self.data.square_band_bottom.max(bottom);
        self.data.square_band_top = Some(top);
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
    pub(in crate::renderer::typeset) fn record_paragraph_float_placement(
        &mut self,
        key: (usize, usize),
        placement: crate::renderer::float_placement::ParagraphFloatPlacement,
    ) {
        self.data.paragraph_float_placements.insert(key, placement);
    }
    pub(in crate::renderer::typeset) fn append_endnote_paragraph(
        &mut self,
        paragraph: crate::model::paragraph::Paragraph,
    ) {
        self.data.endnote_paragraphs.push(paragraph);
    }
    pub(in crate::renderer::typeset) fn append_endnote_source(
        &mut self,
        source: crate::renderer::pagination::EndnoteParaSource,
    ) {
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
    ) {
        self.data.next_para_first_stored_vpos = vpos;
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
        PaginationResult {
            pages: self.data.pages,
            wrap_around_paras: Vec::new(),
            hidden_empty_paras: self.data.hidden_empty_paras,
            pre_emitted_host_paras: self.data.pre_emitted_host_paras,
            pre_emitted_host_heights: self.data.pre_emitted_host_heights,
            endnotes: self.data.endnotes,
            endnote_paragraphs: self.data.endnote_paragraphs,
            endnote_para_sources: self.data.endnote_para_sources,
            endnote_between_notes_hu: self.data.endnote_between_notes_hu,
            endnote_separator_above_hu: self.data.endnote_separator_above_hu,
            endnote_separator_below_hu: self.data.endnote_separator_below_hu,
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
    pub(in crate::renderer::typeset) fn apply_endnote_render_tail_spacing(
        &mut self,
        index: usize,
        spacing: i32,
    ) {
        if let Some(seg) = self
            .data
            .endnote_paragraphs
            .get_mut(index)
            .and_then(|p| p.line_segs.last_mut())
        {
            seg.line_spacing = spacing;
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
        for (&(owner, _), placement) in &self.data.paragraph_float_placements {
            if owner == para_idx
                && placement.flow == crate::renderer::float_placement::ParagraphFloatFlow::NextLine
            {
                self.data.current_height =
                    placement.paragraph_end(self.data.current_height, spacing_after);
                self.data.ladder_band_floor =
                    self.data.ladder_band_floor.max(self.data.current_height);
            }
        }
    }
}
