//! 쪽/단의 확정·전이·초기화. 기존 실행 순서를 보존한다.
use super::data::StateView;
use crate::renderer::typeset::{
    hwpunit_to_px, ColumnContent, ColumnType, Control, PageContent, PageItem, PageLayoutInfo,
    Paragraph, ResolvedStyleSet, TypesetState,
};
impl TypesetState {
    pub(in crate::renderer::typeset) fn new(
        layout: PageLayoutInfo,
        col_count: u16,
        section_index: usize,
        footnote_separator_overhead: f64,
        footnote_between_notes_margin: f64,
        footnote_safety_margin: f64,
        column_type: ColumnType,
    ) -> Self {
        Self {
            data: StateView {
                pages: Vec::new(),
                current_items: Vec::new(),
                current_height: 0.0,
                current_start_height: 0.0,
                current_endnote_flow: false,
                column_had_compact_endnote_rewind: false,
                prev_body_bottom_vpos: None,
                flow_underrun: 0.0,
                hangul2024_reclaimed: 0.0,
                hangul2024_spill_para: None,
                stored_ladder_spacing_omitted: false,
                omit_fresh_recalc_doc: false,
                ladder_band_floor: 0.0,
                current_ladder_band_tables: Vec::new(),
                current_column: 0,
                col_count,
                layout,
                section_index,
                square_band_bottom: 0.0,
                square_band_top: None,
                current_footnote_height: 0.0,
                current_bottom_fixed_exclusion: 0.0,
                bottom_fixed_consumed_flow: 0.0,
                page_has_page_abs_top_table: false,
                defer_host_line_item_para: None,
                is_first_footnote_on_page: true,
                current_page_has_footnote_separator: false,
                footnote_separator_overhead,
                footnote_between_notes_margin,
                footnote_safety_margin,
                section_has_no_footer: false,
                current_zone_y_offset: 0.0,
                current_zone_layout: None,
                on_first_multicolumn_page: false,
                pending_body_wide_top_reserve: 0.0,
                visible_float_exclusions: Vec::new(),
                side_wrap_exclusions: std::collections::BTreeMap::new(),
                inline_placements: std::collections::HashMap::new(),
                inline_flow_plans: std::collections::HashMap::new(),
                paragraph_float_placements: std::collections::HashMap::new(),
                inline_box_flow_bottom: 0.0,
                deferred_table_controls: Vec::new(),
                deferred_next_page_square_pictures: Vec::new(),
                page_start_square_pictures: Vec::new(),
                fragment_queued_table_footnotes: std::collections::HashSet::new(),
                reset_vpos_after_queued_table_footnote_page: false,
                prefilled_paras: std::collections::HashSet::new(),
                pre_emitted_host_paras: std::collections::HashSet::new(),
                pre_emitted_host_heights: std::collections::HashMap::new(),
                skip_safety_margin_once: false,
                skip_footnote_margin_once: false,
                tail_saved_bounds_once: None,
                strict_plain_text_fit_after_empty_host_float_once: false,
                profile: Default::default(),
                has_stored_line_segs: false,
                hide_empty_line: false,
                hidden_empty_lines: 0,
                hidden_empty_page_idx: usize::MAX,
                hidden_empty_paras: std::collections::HashSet::new(),
                page_tail_spilled_floats: std::collections::HashSet::new(),
                endnotes: Vec::new(),
                endnote_paragraphs: Vec::new(),
                endnote_para_sources: Vec::new(),
                endnote_between_notes_hu: 0,
                endnote_separator_above_hu: 0,
                endnote_separator_below_hu: 0,
                wrap_around_cs: -1,
                wrap_synth_rects: Vec::new(),
                wrap_around_sw: -1,
                wrap_around_table_para: 0,
                wrap_around_any_seg: false,
                wrap_around_derived_band: false,
                behind_float_table_para: None,
                next_para_first_stored_vpos: None,
                next_para_is_empty_float_table_anchor: false,
                next_para_is_plain_text: false,
                behind_pending_absorbs: Vec::new(),
                overlay_shape_shortcut_para: None,
                pending_overlay_continuations: Vec::new(),
                current_column_overlay_continuations: Vec::new(),
                current_column_overlay_cuts: Vec::new(),
                current_column_wrap_around_paras: Vec::new(),
                current_column_wrap_anchors: std::collections::HashMap::new(),
                current_zone_column_type: column_type,
                current_zone_design_spacing_px: 0.0,
                vpos_page_base: None,
                vpos_lazy_base: None,
                vpos_page_base_stored: false,
                vpos_ladder_dirty: false,
                vpos_prev_layout_para: None,
                vpos_prev_partial_table: false,
                vpos_col_anchor: 0.0,
                skip_spacing_before_prededuct: false,
                vpos_prev_trimmed_sb_px: 0.0,
            },
        }
    }

    /// [#2424] page-flow state를 continuation context로 이동할 때 잠시 남겨둘 빈 자리.
    /// context drain 직후 원래 state로 교체되며 placeholder 자체는 조판에 사용되지 않는다.
    pub(in crate::renderer::typeset) fn transfer_placeholder(&self) -> Self {
        Self::new(
            self.data.layout.clone(),
            self.data.col_count,
            self.data.section_index,
            self.data.footnote_separator_overhead,
            self.data.footnote_between_notes_margin,
            self.data.footnote_safety_margin,
            self.data.current_zone_column_type,
        )
    }

    /// [Task #1027 Stage D] 컬럼 경계에서 vpos 스냅 상태 초기화.
    /// 렌더러 build_single_column 진입 정합: page/lazy base·prev 초기화,
    /// anchor 를 현 current_height(컬럼 시작값)로 설정.
    pub(in crate::renderer::typeset) fn reset_vpos_cursor(&mut self) {
        self.data.vpos_prev_trimmed_sb_px = 0.0;
        self.data.vpos_page_base = None;
        self.data.vpos_lazy_base = None;
        self.data.vpos_page_base_stored = false;
        self.data.vpos_ladder_dirty = false;
        self.data.vpos_prev_layout_para = None;
        self.data.vpos_prev_partial_table = false;
        self.data.vpos_col_anchor = self.data.current_height;
    }

    /// 사용 가능한 본문 높이 (각주, 존 오프셋 차감)
    pub(in crate::renderer::typeset) fn available_height(&self) -> f64 {
        let base = self.base_available_height();
        let fn_margin = if self.data.current_footnote_height > 0.0 {
            self.data.footnote_safety_margin
        } else {
            0.0
        };
        // [#2559] 한글은 꼬리말 콘텐츠가 없는 구역에서 각주가 빈 꼬리말 밴드까지
        // 사용하도록 배치한다. 밴드를 초과하는 각주 높이만 본문을 줄여야 조기
        // 개행이 누적되지 않는다. 실제 꼬리말이 있으면 점유 가능성이 있으므로
        // 보수적으로 밴드를 회수하지 않는다.
        let footnote_penalty =
            (self.data.current_footnote_height - self.footer_band_reclaim()).max(0.0);
        // [#3707 진단] 가용 높이의 차감 내역. 두 문서에서 avail 이 21.4px 다른데
        // 본문 영역은 같으므로, 어느 항목이 그 차이를 만드는지 가른다. 동작 불변.
        if std::env::var("RHWP_DIAG_AVAIL").is_ok() {
            eprintln!(
                "DIAG_AVAIL base={:.1} fn_h={:.1} reclaim={:.1} penalty={:.1} fn_margin={:.1} zone_y={:.1} bottom_fixed={:.1} → {:.1}",
                base,
                self.data.current_footnote_height,
                self.footer_band_reclaim(),
                footnote_penalty,
                fn_margin,
                self.data.current_zone_y_offset,
                self.data.current_bottom_fixed_exclusion,
                (base - footnote_penalty - fn_margin - self.data.current_zone_y_offset
                    - self.data.current_bottom_fixed_exclusion)
                    .max(0.0),
            );
        }
        (base
            - footnote_penalty
            - fn_margin
            - self.data.current_zone_y_offset
            - self.data.current_bottom_fixed_exclusion)
            .max(0.0)
    }

    /// 기본 가용 높이 (각주/존 미차감)
    pub(in crate::renderer::typeset) fn base_available_height(&self) -> f64 {
        self.data.layout.available_body_height()
    }

    /// 동일 표의 terminal fragment가 아직 current column에 있는지, 직전에 flush된
    /// page에 있는지 구분한다. `Some(true)`는 current, `Some(false)`는 flushed다.
    pub(in crate::renderer::typeset) fn native_table_host_terminal_fragment_placement(
        &self,
        para_index: usize,
        control_index: usize,
        row_count: u16,
    ) -> Option<bool> {
        let is_terminal = |item: &PageItem| match item {
            PageItem::Table {
                para_index: item_para_index,
                control_index: item_control_index,
            } => *item_para_index == para_index && *item_control_index == control_index,
            PageItem::PartialTable {
                para_index: item_para_index,
                control_index: item_control_index,
                end_row,
                end_cut,
                ..
            } => {
                *item_para_index == para_index
                    && *item_control_index == control_index
                    && *end_row == usize::from(row_count)
                    && end_cut.is_empty()
            }
            _ => false,
        };

        if self.data.current_items.iter().any(&is_terminal) {
            return Some(true);
        }
        self.data
            .pages
            .iter()
            .rev()
            .flat_map(|page| page.column_contents.iter().rev())
            .flat_map(|column| column.items.iter().rev())
            .any(is_terminal)
            .then_some(false)
    }

    /// [#4090] Square 어울림 밴드 종료 — 흐름을 밴드 바닥으로 스냅한다.
    ///
    /// 한글은 어울림 개체 옆으로 글을 흘리되 **개체 바닥까지만** 흘리고 그 아래는
    /// 전폭으로 복귀한다. 밴드 안에서는 흐름이 글줄만큼만 전진하므로, 밴드를 벗어날
    /// 때(또는 쪽이 끝날 때) 개체 높이를 반영해 끌어올려야 후속 내용이 개체 안으로
    /// 들어가지 않는다.
    pub(in crate::renderer::typeset) fn close_square_band(&mut self) {
        if self.data.square_band_bottom > 0.0 {
            self.data.current_height = self.data.current_height.max(self.data.square_band_bottom);
            self.data.square_band_bottom = 0.0;
            self.data.square_band_top = None;
        }
    }

    /// 실제로 현재 단에 방출한 그림만 등록한다. 미래/다른 쪽의 그림은 예약하지 않는다.
    pub(in crate::renderer::typeset) fn register_side_wrap_picture(
        &mut self,
        para_index: usize,
        control_index: usize,
        para: &Paragraph,
        paragraph_top: Option<f64>,
        styles: &ResolvedStyleSet,
    ) {
        let Some(Control::Picture(picture)) = para.controls.get(control_index) else {
            return;
        };
        if picture.common.vert_rel_to == crate::model::shape::VertRelTo::Para
            && paragraph_top.is_none()
        {
            // 배치 소유자가 아직 문단 원점을 전달하지 않은 경로는 원시 vpos로 추정하지 않는다.
            return;
        }
        if !self.data.current_items.iter().any(|item| {
            matches!(item, PageItem::Shape { para_index: pi, control_index: ci }
            if *pi == para_index && *ci == control_index)
        }) {
            return;
        }
        let column = self.inline_flow_column();
        let style = styles.para_styles.get(para.para_shape_id as usize);
        let left = style.map_or(0.0, |s| s.margin_left);
        let right = style.map_or(0.0, |s| s.margin_right);
        let container = crate::renderer::page_layout::LayoutRect {
            x: column.x + left,
            width: (column.width - left - right).max(0.0),
            ..column
        };
        let paper = crate::renderer::page_layout::LayoutRect {
            x: 0.0,
            y: 0.0,
            width: self.data.layout.page_width,
            height: self.data.layout.page_height,
        };
        let frame = crate::renderer::float_placement::ObjectPlacementFrame {
            container: &container,
            column: &column,
            body: &self.data.layout.body_area,
            paper: &paper,
            paragraph_y: column.y + paragraph_top.unwrap_or(0.0),
            alignment: style.map_or(crate::model::style::Alignment::Left, |s| s.alignment),
            dpi: self.data.layout.dpi,
        };
        if let Some(exclusion) = frame.picture_exclusion(picture) {
            self.data
                .side_wrap_exclusions
                .insert((para_index, control_index), exclusion);
        }
    }

    /// 현재 항목을 ColumnContent로 만들어 마지막 페이지에 push
    pub(in crate::renderer::typeset) fn flush_column(&mut self) {
        // [#4090] 쪽이 끝나면 어울림 밴드도 끝난다 — 개체 높이를 used 에 반영한다.
        self.close_square_band();
        self.data.inline_box_flow_bottom = 0.0;
        if self.data.current_items.is_empty()
            && self.data.current_column_wrap_around_paras.is_empty()
            && self.data.page_start_square_pictures.is_empty()
        {
            return;
        }
        let col_content = ColumnContent {
            column_index: self.data.current_column,
            start_height: self.data.current_start_height,
            endnote_flow: self.data.current_endnote_flow,
            items: self.take_current_page_items(),
            zone_layout: self.data.current_zone_layout.clone(),
            zone_y_offset: self.data.current_zone_y_offset,
            wrap_around_paras: std::mem::take(&mut self.data.current_column_wrap_around_paras),
            used_height: self.data.current_height,
            wrap_anchors: std::mem::take(&mut self.data.current_column_wrap_anchors),
            overlay_continuations: std::mem::take(
                &mut self.data.current_column_overlay_continuations,
            ),
            overlay_cuts: std::mem::take(&mut self.data.current_column_overlay_cuts),
            inline_placements: std::mem::take(&mut self.data.inline_placements),
            inline_flow_plans: std::mem::take(&mut self.data.inline_flow_plans),
            paragraph_float_placements: std::mem::take(&mut self.data.paragraph_float_placements),
        };
        if let Some(page) = self.data.pages.last_mut() {
            page.column_contents.push(col_content);
        } else {
            self.data
                .pages
                .push(self.new_page_content(vec![col_content]));
        }
        // [#5699 H1] 이번 단에서 발동한 사다리-미계상 표를 소속 쪽에 기록.
        if !self.data.current_ladder_band_tables.is_empty() {
            if let Some(page) = self.data.pages.last_mut() {
                page.ladder_band_tables
                    .append(&mut self.data.current_ladder_band_tables);
            } else {
                self.data.current_ladder_band_tables.clear();
            }
        }
        // [Task #1082] 단 flush 시 본문 last bottom vpos 리셋(미주 vpos-delta 시드 정합).
        self.data.prev_body_bottom_vpos = None;
        // [#2279] flow 과소 누계도 단 단위 — 리셋.
        self.data.flow_underrun = 0.0;
        // [compat 2024] hangul2024_reclaimed 는 여기서 리셋하지 않는다 —
        // flush_column 은 Square 밴드 마감 등 쪽/단 전환 없이도 불리므로,
        // 리셋은 실제 전환 지점(advance_column_or_new_page/reset_for_new_page)에서.
    }

    /// deferred Square picture는 layout의 z/order상 이 column의 첫 item이어야 하지만,
    /// typeset 중에는 height·vpos·fit에 관여하면 안 된다. 실제 column을 flush할 때만
    /// 앞에 materialize해 두 성질을 함께 보존한다.
    pub(in crate::renderer::typeset) fn take_current_page_items(&mut self) -> Vec<PageItem> {
        let mut items = std::mem::take(&mut self.data.page_start_square_pictures)
            .into_iter()
            .map(|deferred| PageItem::Shape {
                para_index: deferred.para_index,
                control_index: deferred.control_index,
            })
            .collect::<Vec<_>>();
        items.append(&mut self.data.current_items);
        items
    }

    /// 비어있어도 flush
    pub(in crate::renderer::typeset) fn flush_column_always(&mut self) {
        self.data.inline_box_flow_bottom = 0.0;
        let col_content = ColumnContent {
            column_index: self.data.current_column,
            start_height: self.data.current_start_height,
            endnote_flow: self.data.current_endnote_flow,
            items: self.take_current_page_items(),
            zone_layout: self.data.current_zone_layout.clone(),
            zone_y_offset: self.data.current_zone_y_offset,
            wrap_around_paras: std::mem::take(&mut self.data.current_column_wrap_around_paras),
            used_height: self.data.current_height,
            wrap_anchors: std::mem::take(&mut self.data.current_column_wrap_anchors),
            overlay_continuations: std::mem::take(
                &mut self.data.current_column_overlay_continuations,
            ),
            overlay_cuts: std::mem::take(&mut self.data.current_column_overlay_cuts),
            inline_placements: std::mem::take(&mut self.data.inline_placements),
            inline_flow_plans: std::mem::take(&mut self.data.inline_flow_plans),
            paragraph_float_placements: std::mem::take(&mut self.data.paragraph_float_placements),
        };
        if let Some(page) = self.data.pages.last_mut() {
            page.column_contents.push(col_content);
        } else {
            self.data
                .pages
                .push(self.new_page_content(vec![col_content]));
        }
        // [#5699 H1] 이번 단에서 발동한 사다리-미계상 표를 소속 쪽에 기록.
        if !self.data.current_ladder_band_tables.is_empty() {
            if let Some(page) = self.data.pages.last_mut() {
                page.ladder_band_tables
                    .append(&mut self.data.current_ladder_band_tables);
            } else {
                self.data.current_ladder_band_tables.clear();
            }
        }
    }

    /// 다음 단 또는 새 페이지
    /// [#6146] 저장 vpos 리셋으로 다음 쪽에 넘어가는 문단의 **자리차지 밴드**를 떠나는
    /// 쪽의 흐름 말미에 남긴다.
    ///
    /// 한글은 보도자료 꼬리의 로고 글상자처럼 쪽 말미에 걸린 비-TAC 자리차지
    /// (TopAndBottom, vert=문단) 개체를 다음 쪽으로 옮기지 않고 본문 아래 여백으로
    /// 흘려 그 쪽에 남긴다 — 한글 2024 PDF 실측 4/4 (156583583 1쪽 y 1029.4..1079.5
    /// vs 본문 하단 1039.3, 156597957·156535759·156742932 동형). 옮기면 다음 쪽
    /// 상단의 제목 표와 겹쳐 글자가 가려진다(#6146).
    ///
    /// 판별은 물리적으로 — **개체 아래끝이 용지 안에 남을 때만** 흘린다. 쪽을 넘겨야
    /// 하는 큰 개체(#1156 의 80mm 차트 OLE 계열)는 아래끝이 용지를 벗어나 제외된다.
    /// 밴드 뒤의 줄·표는 리셋대로 다음 쪽에 놓이므로 쪽수는 바뀌지 않는다.
    pub(in crate::renderer::typeset) fn spill_page_tail_floats(
        &mut self,
        para_idx: usize,
        para: &Paragraph,
    ) {
        use crate::model::shape::{TextWrap, VertRelTo};
        if self.data.current_items.is_empty() || self.data.col_count > 1 {
            return;
        }
        let dpi = self.data.layout.dpi;
        for (ctrl_idx, ctrl) in para.controls.iter().enumerate() {
            let common = match ctrl {
                Control::Picture(picture) => &picture.common,
                Control::Shape(shape) => shape.common(),
                // 표는 자기 배치 경로(place_table_with_text)가 있으므로 제외한다.
                _ => continue,
            };
            if common.treat_as_char
                || !matches!(common.text_wrap, TextWrap::TopAndBottom)
                || !matches!(common.vert_rel_to, VertRelTo::Para)
            {
                continue;
            }
            let height = hwpunit_to_px(common.height as i32, dpi)
                + hwpunit_to_px(i32::from(common.margin.bottom), dpi);
            if height <= 0.0
                || self.data.layout.body_area.y + self.data.current_height + height
                    > self.data.layout.page_height + 0.5
            {
                continue;
            }
            self.data.current_items.push(PageItem::Shape {
                para_index: para_idx,
                control_index: ctrl_idx,
            });
            self.data.current_height += height;
            self.data
                .page_tail_spilled_floats
                .insert((para_idx, ctrl_idx));
        }
    }

    pub(in crate::renderer::typeset) fn advance_column_or_new_page(&mut self) {
        self.flush_column();
        self.data.visible_float_exclusions.clear();
        if self.data.current_column + 1 < self.data.col_count {
            self.data.current_column += 1;
            // Task #321: col 0 상단의 body-wide TopAndBottom 표/도형이 차지한 높이를
            // current_height의 시작값으로 사용 (가용 공간만 줄임, zone_y_offset은 건드리지 않음).
            // layout은 body_wide_reserved로 별도 처리하므로 여기서 zone_y_offset에
            // 넣으면 double-shift가 발생.
            self.data.current_height = self.data.pending_body_wide_top_reserve;
            self.data.current_start_height = self.data.current_height;
            self.data.current_endnote_flow = false;
            // [#5699 H1] 밴드 교정 바닥은 단 흐름 좌표 — 새 단에서 리셋.
            self.data.ladder_band_floor = 0.0;
            // [compat 2024] 앵커 줄 회수 누계는 단 단위 — 새 단에서 리셋.
            if self.data.hangul2024_reclaimed > 0.0 && std::env::var("RHWP_DIAG_COMPAT24").is_ok() {
                eprintln!(
                    "DIAG_COMPAT24 clear@column-advance reclaimed={:.1}",
                    self.data.hangul2024_reclaimed
                );
            }
            self.data.hangul2024_reclaimed = 0.0;
            self.data.column_had_compact_endnote_rewind = false;
            self.reset_vpos_cursor();
        } else {
            self.push_new_page();
        }
    }

    /// 강제 새 페이지
    pub(in crate::renderer::typeset) fn force_new_page(&mut self) {
        self.flush_column();
        self.push_new_page();
    }

    /// 페이지 보장
    pub(in crate::renderer::typeset) fn ensure_page(&mut self) {
        if self.data.pages.is_empty() {
            self.data.pages.push(self.new_page_content(Vec::new()));
        }
    }

    /// 새 페이지 push + 상태 리셋
    pub(in crate::renderer::typeset) fn push_new_page(&mut self) {
        self.data
            .layout
            .apply_column_page_number(self.data.pages.len() as u32 + 1);
        self.data.pages.push(self.new_page_content(Vec::new()));
        // 합성 어울림 배제 사각형은 쪽 단위 — 새 쪽에서 비운다.
        self.data.wrap_synth_rects.clear();
        self.reset_for_new_page();
        // [#3738 Stage 22] current page의 본문/각주 흐름을 먼저 확정한 뒤, tail
        // Square picture만 새 physical page의 layout item 앞에 둔다. 즉시
        // `current_items`에 넣지 않아 다음 paragraph의 height/vpos fit은 기존과
        // 같게 유지하고, flush 때만 Shape로 materialize한다.
        for deferred in std::mem::take(&mut self.data.deferred_next_page_square_pictures) {
            // typeset_wrap_around_paragraph는 next paragraph가 page break를 일으키기
            // 전에 호출된다. 따라서 이 지점에서 다음 page column에 직접 anchor를
            // 등록해야 p1356처럼 whole-narrow paragraph도 stored cs/sw를 보존한다.
            for wrap_target_para_index in &deferred.wrap_target_para_indices {
                self.data
                    .current_column_wrap_anchors
                    .insert(*wrap_target_para_index, deferred.wrap_anchor.clone());
            }
            self.data.page_start_square_pictures.push(deferred);
        }
        // Task #321: 새 페이지에서는 body-wide top reserve 초기화
        self.data.pending_body_wide_top_reserve = 0.0;
        // [#4568] 앞 쪽에서 잘린 overlay 표의 잔여 행을 이 쪽 최상단에 이어 그린다.
        // `current_items` 가 아니라 단 전용 목록으로 넘긴다 — 흐름 항목이 아니라
        // z-layer 장식이고, 항목으로 섞으면 이 조각이 단의 첫 항목이 되어
        // `items.first()` 를 보는 휴리스틱이 조각을 본문으로 읽는다.
        let mut overlay_top_reserve = 0.0f64;
        self.data.current_column_overlay_continuations.extend(
            std::mem::take(&mut self.data.pending_overlay_continuations)
                .into_iter()
                .map(|(para_index, control_index, start_row, remaining_px)| {
                    overlay_top_reserve = overlay_top_reserve.max(remaining_px);
                    crate::renderer::pagination::OverlayContinuation {
                        para_index,
                        control_index,
                        start_row,
                        reserve_px: remaining_px,
                    }
                }),
        );
        // 잔여 높이를 새 쪽 흐름에 무조건 예약하면 안 된다 — #4514 기제에서 필러
        // 문단들이 이미 표 높이만큼 흐름 공간을 만들므로 이중 계상이 된다(실측:
        // 예약 시 48 → 56쪽, 한컴 46쪽에서 더 멀어짐). 그래서 대기열 등록부(#5792
        // 게이트)가 "뒤따르는 흐름이 자리를 만들지 못한다"고 판정한 조각만 0 아닌
        // 값을 싣는다 — 필러 형상은 종전대로 0 이라 불변이다.
        if overlay_top_reserve > 0.0 {
            self.data.current_height = self.data.current_height.max(overlay_top_reserve);
            self.data.current_start_height = self.data.current_height;
        }
    }

    pub(in crate::renderer::typeset) fn reset_for_new_page(&mut self) {
        self.data.side_wrap_exclusions.clear();
        self.data.current_column = 0;
        self.data.current_height = 0.0;
        self.data.current_start_height = 0.0;
        // [#5699 H1] 밴드 교정 바닥은 쪽 단위.
        self.data.ladder_band_floor = 0.0;
        // [compat 2024] 앵커 줄 회수 누계는 쪽 단위 — 새 쪽에서 리셋.
        if self.data.hangul2024_reclaimed > 0.0 && std::env::var("RHWP_DIAG_COMPAT24").is_ok() {
            eprintln!(
                "DIAG_COMPAT24 clear@new-page reclaimed={:.1}",
                self.data.hangul2024_reclaimed
            );
        }
        self.data.hangul2024_reclaimed = 0.0;
        self.data.current_endnote_flow = false;
        self.data.column_had_compact_endnote_rewind = false;
        self.data.current_footnote_height = 0.0;
        self.data.current_bottom_fixed_exclusion = 0.0;
        self.data.bottom_fixed_consumed_flow = 0.0;
        self.data.page_has_page_abs_top_table = false;
        self.data.is_first_footnote_on_page = true;
        self.data.current_page_has_footnote_separator = false;
        self.data.current_zone_y_offset = 0.0;
        self.data.current_zone_layout = None;
        self.data.on_first_multicolumn_page = false;
        self.data.visible_float_exclusions.clear();
        self.reset_vpos_cursor();
    }

    pub(in crate::renderer::typeset) fn apply_visible_float_exclusions(
        &mut self,
        probe_height: f64,
    ) {
        if self.data.visible_float_exclusions.is_empty() {
            return;
        }

        let use_overlap_probe = self.data.profile.hwpx_stored_layout() && probe_height > 0.0;
        self.data
            .visible_float_exclusions
            .retain(|zone| self.data.current_height < zone.bottom - 0.5);

        let mut jump_to = self.data.current_height;
        for zone in &self.data.visible_float_exclusions {
            let starts_in_zone = jump_to + 0.5 >= zone.top && jump_to < zone.bottom;
            let overlaps_zone =
                use_overlap_probe && jump_to < zone.top && jump_to + probe_height > zone.top + 0.5;
            if starts_in_zone || overlaps_zone {
                jump_to = jump_to.max(zone.bottom);
            }
        }

        if jump_to > self.data.current_height + 0.5 {
            self.data.current_height = jump_to;
        }
    }

    /// #2439: layout resolves an empty para-relative float's natural top against the visible
    /// float lane before painting it. Native pagination must consume the same lane when the
    /// upcoming natural top (flow + positive offset/outer lead) intersects it; the generic
    /// paragraph probe above intentionally does not enable native overlap probing.
    pub(in crate::renderer::typeset) fn apply_visible_float_exclusions_for_para_float(
        &mut self,
        natural_top_lead: f64,
    ) {
        if self.data.visible_float_exclusions.is_empty() || natural_top_lead <= 0.0 {
            return;
        }

        self.data
            .visible_float_exclusions
            .retain(|zone| self.data.current_height < zone.bottom - 0.5);
        let mut jump_to = self.data.current_height;
        for zone in &self.data.visible_float_exclusions {
            let natural_top = jump_to + natural_top_lead;
            if natural_top + 0.5 >= zone.top && natural_top < zone.bottom {
                jump_to = jump_to.max(zone.bottom);
            }
        }
        if jump_to > self.data.current_height + 0.5 {
            self.data.current_height = jump_to;
        }
    }

    /// [#6764] 블록 표를 이 쪽에 앉히기 전에, **다른 문단**이 남긴 자리차지 밴드를
    /// 표 높이로 한 번 짚는다.
    ///
    /// 문단 흐름용 `apply_visible_float_exclusions` 의 겹침 프로브는 HWPX 저장
    /// 프로파일 전용이라, 네이티브 HWP5 에서 `vertical_offset` 이 앵커와 표 상단
    /// 사이를 벌려 놓으면 그 틈에서 시작하는 표가 밴드를 **그냥 통과한다.**
    /// 계상은 틈 위에 남고 페인트는 밴드 아래로 가므로 분할 예산이 통째로 어긋난다
    /// (1613000-202200037 182쪽: 예산 817.6px 로 23행을 잘라 넣었는데 페인트는
    /// 용지 밖 885.6px). 표는 한 덩어리라 어긋남이 쪽 규모로 드러난다.
    ///
    /// 같은 문단이 만든 밴드는 건드리지 않는다 — co-anchored float 스택은 자기
    /// 밴드를 넘겨 짚으면 안 된다(`#1510`).
    pub(in crate::renderer::typeset) fn apply_float_band_before_block_table(
        &mut self,
        para_index: usize,
        probe_height: f64,
    ) {
        if self.data.visible_float_exclusions.is_empty() || probe_height <= 0.5 {
            return;
        }
        self.data
            .visible_float_exclusions
            .retain(|zone| self.data.current_height < zone.bottom - 0.5);

        let mut jump_to = self.data.current_height;
        for zone in &self.data.visible_float_exclusions {
            if zone.para_index == para_index {
                continue;
            }
            let starts_in_zone = jump_to + 0.5 >= zone.top && jump_to < zone.bottom;
            let crosses_zone = jump_to < zone.top && jump_to + probe_height > zone.top + 0.5;
            if starts_in_zone || crosses_zone {
                jump_to = jump_to.max(zone.bottom);
            }
        }
        if jump_to > self.data.current_height + 0.5 {
            self.data.current_height = jump_to;
        }
    }

    pub(in crate::renderer::typeset) fn new_page_content(
        &self,
        column_contents: Vec<ColumnContent>,
    ) -> PageContent {
        PageContent {
            page_index: self.data.pages.len() as u32,
            page_number: 0,
            page_number_restarted: false,
            section_index: self.data.section_index,
            layout: self.data.layout.clone(),
            column_contents,
            active_header: None,
            active_footer: None,
            page_number_pos: None,
            page_hide: None,
            footnotes: Vec::new(),
            active_master_page: None,
            extra_master_pages: Vec::new(),
            ladder_band_tables: Vec::new(),
        }
    }
}
