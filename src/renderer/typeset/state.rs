//! 조판 상태의 조회 입력과 확정 결과 반영 경계.
//! inline 흐름, 문단 fit의 1회성 보정 소비와 일반 전체/분할 배치 반영을 소유한다.
//! 지연 표 큐의 인출·복원과 배치 후 vpos 반영도 이 경계에서 수행한다.
//! 빈 호스트 float의 예산 조회와 항목·lane·흐름 확정도 담당한다.
//! 표 문단의 비표 개체 조회 입력과 항목·흐름 확정도 담당한다.
//! 배치 후 TAC 높이 보정의 사다리 상태와 최종 높이 확정도 담당한다.
//! 데코레이션 host 텍스트는 trim 상태를 바꾸지 않고 항목/전진량만 반영한다.
//! 장식 표의 Shape 발행과 이어받기 예약/현재 단 컷·앵커 확정도 담당한다.
//! 일반 표 배치 전 float 배타 영역 소비와 배치 후 지연 판단의 페이지 관측도 담당한다.
//! 표 소유 문단의 진입 진단·배타 영역 소비 후 너비·배치 전 흐름 관측도 담당한다.
//! 나머지 상태 변경은 상위 구현에 남아 있다.

use super::controls::decoration_table::OverlayContinuation;
use super::controls::deferred::DeferredTableControl;
use super::controls::empty_float::{EmptyFloatPage, EmptyFloatPlacement};
use super::controls::host_wrap::StoredHostBand;
use super::controls::shape_flow::{TableHostShapeFlow, TableHostShapePage};
use super::controls::stored_tac::{StoredTacControlPlacement, StoredTacPage};
use super::controls::table_entry::TableControlPage;
use super::controls::tac_fit::TacFitPage;
use super::controls::tac_reconcile::TacHeightPage;
use super::controls::wrap_match::WrapBand;
use super::inline_flow::plan::InlineFlowInput;
use super::paragraph::fit::saved_tail_overflow_to_fit;
use super::paragraph::overflow::OverflowPage;
use super::paragraph::placement::ParagraphFragment;
use super::paragraph::scan::LineScanPage;
use super::paragraph::split_entry::SplitEntryPage;
use super::TypesetState;
use crate::renderer::float_placement::FloatLaneSet;
use crate::renderer::inline_flow::InlineFlowPlan;
use crate::renderer::page_layout::LayoutRect;
use crate::renderer::pagination::PageItem;

impl TypesetState {
    /// 저장 어울림 자격을 비운 뒤 밴드 바닥을 흐름 높이에 반영한다.
    /// 뒤따르는 fit은 종료 후의 높이/예산을 다시 조회해야 한다.
    pub(super) fn end_following_wrap(&mut self) {
        self.wrap_around_cs = -1;
        self.wrap_around_sw = -1;
        self.wrap_around_any_seg = false;
        self.wrap_around_derived_band = false;
        self.close_square_band();
    }

    /// 꼬리 항목 → 높이 전진 → 사다리 dirty 순서를 보존한다. 쪽 전환은 호출자 책임이다.
    pub(super) fn commit_wrap_tail(
        &mut self,
        para_idx: usize,
        wrap_prefix_len: usize,
        end_line: usize,
        suffix_height: f64,
    ) {
        self.current_items.push(PageItem::PartialParagraph {
            para_index: para_idx,
            start_line: wrap_prefix_len,
            end_line,
        });
        self.current_height += suffix_height;
        self.vpos_ladder_dirty = true;
    }

    /// 저장 끝점으로 밴드를 먼저 늘린 뒤 표의 첫 조각 소유 단에 기록한다.
    pub(super) fn commit_wrap_absorption(
        &mut self,
        absorption: super::controls::wrap_absorption::WrapAbsorption,
    ) {
        if let Some(source_offset_px) = absorption.source_offset_px {
            self.extend_square_band_to_source_bottom(source_offset_px);
        }
        self.record_wrap_around_para(absorption.paragraph);
    }

    /// Square 표 옆으로 흐른 문단이 표보다 아래까지 이어지면, 그 저장 좌표의 마지막
    /// 줄까지 배제 밴드를 확장한다. 표 자체만 예약하면 전폭 복귀 뒤의 fit 경로가 그
    /// 텍스트 높이를 잃어 뒤쪽 본문을 과도하게 같은 쪽에 배치한다.
    fn extend_square_band_to_source_bottom(&mut self, source_offset_px: f64) {
        if source_offset_px <= 0.0 {
            return;
        }
        if let Some(top) = self.square_band_top {
            self.square_band_bottom = self.square_band_bottom.max(top + source_offset_px);
        }
    }

    /// [Task #1745] 흡수된 어울림 문단 기록 — 다쪽 분할 표는 첫 fragment column 에 소급.
    ///
    /// 한글은 어울림 문단을 anchor 표의 시작 쪽(첫 fragment) 옆 wrap 띠에 배치한다.
    /// RowBreak 분할 표는 흡수 시점에 첫 fragment column 이 이미 flush 되어 있으므로,
    /// 현재 column 에 anchor 의 첫 fragment(비연속 PartialTable/Table)가 없으면
    /// `pages` 에서 찾아 그 column 의 wrap_around_paras 에 push 한다.
    fn record_wrap_around_para(&mut self, wrap_para: crate::renderer::pagination::WrapAroundPara) {
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
        if !self.current_items.iter().any(is_first_fragment) {
            for page in self.pages.iter_mut() {
                for col in page.column_contents.iter_mut() {
                    if col.items.iter().any(is_first_fragment) {
                        col.wrap_around_paras.push(wrap_para);
                        return;
                    }
                }
            }
        }
        self.current_column_wrap_around_paras.push(wrap_para);
    }

    /// 활성 여부를 확인한 후, 상태 변경 없는 매칭 구간에서만 사용하는 관측값.
    pub(super) fn following_wrap_band(&self) -> WrapBand {
        WrapBand {
            cs: self.wrap_around_cs,
            sw: self.wrap_around_sw,
            anchor_para: self.wrap_around_table_para,
            any_seg: self.wrap_around_any_seg,
        }
    }

    pub(super) fn register_following_wrap_anchor(
        &mut self,
        para_idx: usize,
        anchor: crate::renderer::pagination::WrapAnchorRef,
    ) {
        self.current_column_wrap_anchors.insert(para_idx, anchor);
    }

    pub(super) fn host_wrap_column_width_hu(&self) -> i32 {
        self.layout.column_width_hu()
    }

    pub(super) fn host_wrap_needs_derived_lane(&self) -> bool {
        self.wrap_around_cs < 0
    }

    pub(super) fn arm_stored_host_wrap(&mut self, para_idx: usize, band: StoredHostBand) {
        self.wrap_around_cs = band.anchor_cs;
        self.wrap_around_sw = band.anchor_sw;
        self.wrap_around_table_para = para_idx;
        self.wrap_around_any_seg = true;
        self.wrap_around_derived_band = false;
    }

    pub(super) fn register_host_wrap_anchor(
        &mut self,
        para_idx: usize,
        anchor: crate::renderer::pagination::WrapAnchorRef,
    ) {
        self.current_column_wrap_anchors.insert(para_idx, anchor);
    }

    pub(super) fn arm_derived_host_wrap(&mut self, para_idx: usize, lane: i32) {
        self.wrap_around_cs = 0;
        self.wrap_around_sw = lane;
        self.wrap_around_table_para = para_idx;
        self.wrap_around_any_seg = false;
        self.wrap_around_derived_band = true;
    }

    /// 강제 경계 준비 뒤의 문단 흐름 힌트에 사용할 현재 상태의 profile.
    pub(super) fn paragraph_flow_profile(
        &self,
    ) -> crate::model::provenance::LayoutCompatibilityProfile {
        self.profile
    }

    /// 지연 배치는 새 문단 진입이 아니므로 float 배타 영역을 추가로 소비하지 않는다.
    pub(super) fn deferred_table_column_width(&self) -> f64 {
        self.layout
            .column_areas
            .get(self.current_column as usize)
            .map(|a| a.width)
            .unwrap_or(self.layout.body_area.width)
    }

    /// 원 배치 시점의 예산 앵커와 구분되는, 현재 지연 배치의 렌더 앵커.
    pub(super) fn deferred_table_anchor_height(&self) -> f64 {
        self.current_height
    }

    /// 표 문단 진입의 관측은 float 배타 영역 소비보다 먼저 수행한다.
    pub(super) fn trace_table_paragraph_entry(&self, para_idx: usize) {
        // [#2243 진단] 표 문단 진입 누적 — 동작 불변.
        if std::env::var("RHWP_DIAG_TAC").is_ok() {
            eprintln!(
                "DIAG_TBLP pi={} cur_h={:.1} items={} page_base={:?} anchor={:.1}",
                para_idx,
                self.current_height,
                self.current_items.len(),
                self.vpos_page_base,
                self.vpos_col_anchor,
            );
        }
    }

    /// 기존 배타 영역을 소비한 뒤 그 시점의 가용 단 너비를 반환한다.
    pub(super) fn prepare_table_paragraph_column(&mut self) -> f64 {
        self.apply_visible_float_exclusions(0.0);
        self.layout
            .column_areas
            .get(self.current_column as usize)
            .map(|a| a.width)
            .unwrap_or(self.layout.body_area.width)
    }

    /// 배치가 시작되기 전 문단 앵커와 완료 쪽 수를 함께 관측한다.
    pub(super) fn table_paragraph_flow_position(&self) -> (f64, usize) {
        (self.current_height, self.pages.len())
    }

    pub(super) fn has_fragment_queued_table_footnotes(
        &self,
        para_idx: usize,
        ctrl_idx: usize,
    ) -> bool {
        self.fragment_queued_table_footnotes
            .contains(&(para_idx, ctrl_idx))
    }

    pub(super) fn flow_table_column_top(&self) -> bool {
        self.current_height < 1.0
    }

    pub(super) fn prepare_flow_table_anchor(
        &mut self,
        natural_top_lead: Option<f64>,
        para_start_height: f64,
    ) -> f64 {
        if let Some(natural_top_lead) = natural_top_lead {
            self.apply_visible_float_exclusions_for_para_float(natural_top_lead);
            self.current_height
        } else {
            para_start_height
        }
    }

    pub(super) fn flow_table_page_count(&self) -> usize {
        self.pages.len()
    }

    pub(super) fn coanchored_table_page(&self) -> (usize, &[PageItem]) {
        (self.pages.len(), &self.current_items)
    }

    pub(super) fn decoration_table_flow_height(&self) -> f64 {
        self.current_height
    }

    pub(super) fn emit_decoration_table(&mut self, para_idx: usize, ctrl_idx: usize) {
        self.current_items.push(PageItem::Shape {
            para_index: para_idx,
            control_index: ctrl_idx,
        });
    }

    pub(super) fn finish_decoration_table(
        &mut self,
        para_idx: usize,
        ctrl_idx: usize,
        continuation: Option<OverlayContinuation>,
    ) {
        if let Some(continuation) = continuation {
            self.pending_overlay_continuations.push((
                para_idx,
                ctrl_idx,
                continuation.first_unfit,
                continuation.reserve_px,
            ));
            self.current_column_overlay_cuts
                .push((para_idx, ctrl_idx, continuation.first_unfit));
        }
        // [#4514] 흐름 소비 0 배치 — 이 앵커는 #1955 흡수 대상이 아니다.
        self.overlay_shape_shortcut_para = Some(para_idx);
    }

    pub(super) fn table_control_page(&self) -> TableControlPage {
        TableControlPage {
            has_items: !self.current_items.is_empty(),
            col_count: self.col_count,
        }
    }

    pub(super) fn commit_decoration_host_text(&mut self, fragment: ParagraphFragment) {
        self.current_items.push(fragment.item);
        self.current_height += fragment.height;
    }

    pub(super) fn tac_height_page(&self) -> TacHeightPage<'_> {
        TacHeightPage {
            profile: self.profile,
            current_height: self.current_height,
            vpos_page_base: self.vpos_page_base,
            vpos_col_anchor: self.vpos_col_anchor,
            inline_placements: &self.inline_placements,
            inline_box_flow_bottom: self.inline_box_flow_bottom,
        }
    }

    /// 두 상태 변경 사이의 기존 진단 시점을 유지한다.
    pub(super) fn commit_tac_spacing_omission(&mut self, trace: impl FnOnce()) {
        self.stored_ladder_spacing_omitted = true;
        trace();
        self.vpos_ladder_dirty = true;
    }

    pub(super) fn commit_tac_capped_bottom(&mut self, capped_bottom: f64) {
        if self.current_height > capped_bottom {
            self.current_height = capped_bottom;
        }
    }

    pub(super) fn has_spilled_page_tail_float(&self, para_idx: usize, ctrl_idx: usize) -> bool {
        self.page_tail_spilled_floats
            .contains(&(para_idx, ctrl_idx))
    }

    pub(super) fn table_host_shape_page(&self) -> TableHostShapePage {
        TableHostShapePage {
            has_items: !self.current_items.is_empty(),
            current_height: self.current_height,
        }
    }

    /// 단/쪽 전환 뒤 현재 페이지에 항목을 추가하고 기존 우선순위대로 높이를 반영한다.
    pub(super) fn commit_table_host_shape(
        &mut self,
        para_idx: usize,
        ctrl_idx: usize,
        flow: TableHostShapeFlow,
    ) {
        self.current_items.push(PageItem::Shape {
            para_index: para_idx,
            control_index: ctrl_idx,
        });
        if let Some(line_h) = flow.tac_separate_line_h {
            self.current_height += line_h;
        } else if let Some(extra) = flow.non_tac_pushdown_h {
            self.current_height += extra;
        }
    }

    /// Lane 조회용 페이지 관측값. 예산은 별도 지연 조회로 제공한다.
    pub(super) fn empty_float_page(&self) -> EmptyFloatPage<'_> {
        EmptyFloatPage {
            layout: &self.layout,
            current_column: self.current_column,
            profile: self.profile,
            current_height: self.current_height,
            current_items: &self.current_items,
        }
    }

    pub(super) fn empty_float_available_height(
        &self,
        note_content_height: f64,
        note_count: usize,
    ) -> f64 {
        let total_footnote = self.projected_footnote_height(note_content_height, note_count);
        let fn_margin = if total_footnote > 0.0 {
            self.footnote_safety_margin
        } else {
            0.0
        };

        (self.base_available_height() - total_footnote - fn_margin - self.current_zone_y_offset)
            .max(0.0)
    }

    /// 항목 추가 → lane 예약 → 흐름 높이 반영 순서를 보존한다.
    pub(super) fn commit_empty_float_table(
        &mut self,
        para_idx: usize,
        ctrl_idx: usize,
        placement: EmptyFloatPlacement,
        lanes: &mut FloatLaneSet,
    ) {
        let EmptyFloatPlacement {
            x_start,
            x_end,
            raw_top,
            reserved_height,
        } = placement;
        self.current_items.push(PageItem::Table {
            para_index: para_idx,
            control_index: ctrl_idx,
        });
        lanes.place(Some(ctrl_idx), x_start, x_end, raw_top, reserved_height);
        self.current_height = self.current_height.max(lanes.max_bottom());
    }

    /// 각주 등록 뒤 단일 단의 vpos 관측값을 기존 순서로 확정한다.
    pub(super) fn commit_deferred_table_anchor(&mut self, para_index: usize) {
        if self.col_count == 1 {
            self.vpos_prev_layout_para = Some(para_index);
            if matches!(
                self.current_items.last(),
                Some(PageItem::Table { .. } | PageItem::PartialTable { .. })
            ) {
                self.vpos_page_base = None;
                self.vpos_lazy_base = None;
                self.vpos_prev_partial_table = matches!(
                    self.current_items.last(),
                    Some(PageItem::PartialTable { .. })
                );
            }
        }
    }

    pub(super) fn has_deferred_table_controls(&self) -> bool {
        !self.deferred_table_controls.is_empty()
    }

    pub(super) fn enqueue_deferred_table_controls(&mut self, deferred: Vec<DeferredTableControl>) {
        self.deferred_table_controls.extend(deferred);
    }

    pub(super) fn take_deferred_table_controls(&mut self) -> Vec<DeferredTableControl> {
        std::mem::take(&mut self.deferred_table_controls)
    }

    /// flush에서 남은 후보를 기존 순서로 복원한다. 종전처럼 교체하며 추가 병합하지 않는다.
    pub(super) fn restore_deferred_table_controls(&mut self, remaining: Vec<DeferredTableControl>) {
        self.deferred_table_controls = remaining;
    }

    /// 일반 TAC 배치 전 판단에 필요한 읽기 전용 상태만 전달한다.
    pub(super) fn tac_fit_page(&self) -> TacFitPage<'_> {
        TacFitPage {
            profile: self.profile,
            current_height: self.current_height,
            vpos_page_base: self.vpos_page_base,
            current_items: &self.current_items,
        }
    }

    /// 저장 TAC 줄 수용에 필요한 읽기 전용 페이지 관측값.
    pub(super) fn stored_tac_page(&self) -> StoredTacPage {
        StoredTacPage {
            profile: self.profile,
            current_height: self.current_height,
            vpos_col_anchor: self.vpos_col_anchor,
            vpos_page_base: self.vpos_page_base,
            vpos_lazy_base: self.vpos_lazy_base,
            side_wrap_empty: self.side_wrap_exclusions.is_empty(),
        }
    }

    /// 확정 좌표 → 표 항목 → 흐름 끝점의 기존 적용 순서를 유지한다.
    pub(super) fn commit_stored_tac_control(
        &mut self,
        para_idx: usize,
        placement: StoredTacControlPlacement,
    ) {
        self.inline_placements
            .insert((para_idx, placement.control_index), placement.inline);
        self.current_items.push(PageItem::Table {
            para_index: para_idx,
            control_index: placement.control_index,
        });
        self.current_height = placement.end;
    }

    /// 전체 fit의 관측값을 빌린다. 가용 높이의 진단 조회는 Query의 기존 단락 위치에 남긴다.
    pub(super) fn paragraph_whole_fit_page(&self) -> super::paragraph::whole_fit::WholeFitPage<'_> {
        super::paragraph::whole_fit::WholeFitPage {
            profile: self.profile,
            omit_fresh_recalc_doc: self.omit_fresh_recalc_doc,
            col_count: self.col_count,
            current_items: &self.current_items,
            current_height: self.current_height,
            body_height: self.base_available_height(),
            visible_float_exclusions: &self.visible_float_exclusions,
            hangul2024_reclaimed: self.hangul2024_reclaimed,
        }
    }

    /// hide_empty_line 경로에 진입했을 때만 페이지별 횟수를 초기화한다.
    pub(super) fn begin_empty_paragraph_page(&mut self) {
        let current_page_idx = self.pages.len();
        if current_page_idx != self.hidden_empty_page_idx {
            self.hidden_empty_lines = 0;
            self.hidden_empty_page_idx = current_page_idx;
        }
    }

    /// guide와 앞선 빈 문단 drift 경로: 항목·높이·횟수는 변경하지 않는다.
    pub(super) fn hide_empty_paragraph(&mut self, para_idx: usize) {
        self.hidden_empty_paras.insert(para_idx);
    }

    /// 옵션에 의해 감춘 빈 문단은 횟수 → 숨김 표시 → 항목 순으로 기록한다.
    pub(super) fn commit_counted_hidden_paragraph(&mut self, para_idx: usize) {
        self.hidden_empty_lines += 1;
        self.hidden_empty_paras.insert(para_idx);
        // height=0 으로 page 진행 — fit 분기에서 추가 처리하지 않음
        self.current_items.push(PageItem::FullParagraph {
            para_index: para_idx,
        });
    }

    /// 구역 끝 안전여백/각주 예산 흡수는 항목만 남기며 숨김 표시를 추가하지 않는다.
    pub(super) fn place_unadvanced_empty_paragraph(&mut self, para_idx: usize) {
        self.current_items.push(PageItem::FullParagraph {
            para_index: para_idx,
        });
    }

    pub(super) fn paragraph_empty_tail_page(&self) -> super::paragraph::empty::EmptyTailPage<'_> {
        super::paragraph::empty::EmptyTailPage {
            col_count: self.col_count,
            current_items: &self.current_items,
            current_height: self.current_height,
            body_height: self.base_available_height(),
            current_zone_y_offset: self.current_zone_y_offset,
            current_footnote_height: self.current_footnote_height,
        }
    }

    /// 저장 꼬리가 쪽 끝을 채운 경우에만 호출한다. 판정 이후 가용 높이를 다시 조회한다.
    pub(super) fn fill_paragraph_entry_page_tail(&mut self) {
        self.current_height = self.current_height.max(self.available_height());
    }

    /// 빈 host float 뒤의 엄격 fit 자격을 기존 조건으로 한 번 소비한다.
    pub(super) fn take_strict_paragraph_fit(
        &mut self,
        para: &crate::model::paragraph::Paragraph,
    ) -> bool {
        super::take_strict_plain_text_fit_after_empty_host_float_once(
            &mut self.strict_plain_text_fit_after_empty_host_float_once,
            para,
        )
    }

    /// 구성된 줄이 없더라도 원래 FullParagraph 항목을 보존한 뒤 높이를 계산한다.
    pub(super) fn begin_empty_line_paragraph(&mut self, para_idx: usize) {
        self.current_items.push(PageItem::FullParagraph {
            para_index: para_idx,
        });
    }

    pub(super) fn paragraph_split_entry_page(&self) -> SplitEntryPage<'_> {
        SplitEntryPage {
            profile: self.profile,
            col_count: self.col_count,
            current_height: self.current_height,
            current_items: &self.current_items,
            body_height: self.base_available_height(),
            stored_ladder_spacing_omitted: self.stored_ladder_spacing_omitted,
            hangul2024_reclaimed: self.hangul2024_reclaimed,
        }
    }

    /// 호환성 재수용을 택한 빈 문단의 spill 소유만 기록한다. 쪽 전환은 별도다.
    pub(super) fn mark_blank_paragraph_spill(&mut self, para_idx: usize) {
        self.hangul2024_spill_para = Some(para_idx);
    }

    /// 넘침 판단에 필요한 읽기 전용 값만 전달한다. base 높이 조회에는 부수효과가 없다.
    pub(super) fn paragraph_overflow_page(&self) -> OverflowPage {
        OverflowPage {
            col_count: self.col_count,
            current_height: self.current_height,
            has_items: !self.current_items.is_empty(),
            body_height: self.base_available_height(),
            body_area_height: self.layout.body_area.height,
            hwp3_layout: self.profile.hwp3_layout(),
        }
    }

    /// atomic 항목을 먼저 넣고 조정자가 기존 flow 메트릭을 계산하게 한다.
    pub(super) fn begin_atomic_overflow_paragraph(&mut self, para_idx: usize) {
        self.current_items.push(PageItem::FullParagraph {
            para_index: para_idx,
        });
    }

    /// atomic 넘침은 기존 trimmed spacing을 덮지 않는다.
    pub(super) fn advance_atomic_overflow_paragraph(
        &mut self,
        advance: f64,
        total_height: f64,
        body_bottom_vpos: Option<i32>,
    ) {
        self.current_height += advance;
        self.flow_underrun += (total_height - advance).max(0.0);
        if let Some(v) = body_bottom_vpos {
            self.prev_body_bottom_vpos = Some(v);
        }
    }

    /// tail 넘침은 total_height를 그대로 전진하며 underrun을 누적하지 않는다.
    pub(super) fn commit_tail_overflow_paragraph(
        &mut self,
        para_idx: usize,
        total_height: f64,
        body_bottom_vpos: Option<i32>,
    ) {
        self.current_items.push(PageItem::FullParagraph {
            para_index: para_idx,
        });
        self.vpos_prev_trimmed_sb_px = 0.0;
        self.current_height += total_height;
        if let Some(v) = body_bottom_vpos {
            self.prev_body_bottom_vpos = Some(v);
        }
    }

    /// 항목 순서만 확정한다. 높이 계산과 진단은 이 변경 뒤 조정자가 실행한다.
    pub(super) fn insert_fitted_paragraph(&mut self, para_idx: usize, defer_preceding_float: bool) {
        let paragraph_item = PageItem::FullParagraph {
            para_index: para_idx,
        };
        if defer_preceding_float {
            // 빈 host의 양수-offset 자리차지 표는 다음 계산 본문 문단이 표 위 빈칸을
            // 채운 뒤에 그려진다. 표를 먼저 놓으면 그 본문이 표 하단으로 밀린다.
            let table_item = self
                .current_items
                .pop()
                .expect("checked trailing table item");
            self.current_items.push(paragraph_item);
            self.current_items.push(table_item);
        } else {
            self.current_items.push(paragraph_item);
        }
    }

    /// 일반 전체/빈 구성 결과 경로에서 계산한 trim, 높이, underrun, 저장 하단을 순서대로 반영한다.
    pub(super) fn apply_full_paragraph_flow(
        &mut self,
        advance: f64,
        total_height: f64,
        trimmed_spacing_before: f64,
        body_bottom_vpos: Option<i32>,
    ) {
        self.vpos_prev_trimmed_sb_px = trimmed_spacing_before;
        self.current_height += advance;
        self.flow_underrun += (total_height - advance).max(0.0);
        if let Some(v) = body_bottom_vpos {
            self.prev_body_bottom_vpos = Some(v);
        }
    }

    /// 확정된 조각을 항목 추가 → trim 초기화 → 높이 전진 순서로 반영한다.
    /// 페이지 전환과 다음 컷 선택은 조정자의 책임이다.
    pub(super) fn commit_split_paragraph_fragment(&mut self, fragment: ParagraphFragment) {
        self.current_items.push(fragment.item);
        self.vpos_prev_trimmed_sb_px = 0.0;
        self.current_height += fragment.height;
    }

    /// 저장 다단 조각은 일반 split과 달리 trim 복원값을 초기화하지 않는다.
    pub(super) fn commit_multicolumn_paragraph_fragment(&mut self, fragment: ParagraphFragment) {
        self.current_items.push(fragment.item);
        self.current_height += fragment.height;
    }

    /// [Task #2320] 마지막 단에서의 분할은 새 페이지 단 0으로 진행한다.
    /// 중간 단에서는 기존 flush → 단 증가 → 높이 초기화 순서를 유지한다.
    pub(super) fn advance_after_multicolumn_fragment(&mut self) {
        if self.current_column + 1 < self.col_count {
            self.flush_column();
            self.current_column += 1;
            self.current_height = 0.0;
        } else {
            self.advance_column_or_new_page();
        }
    }

    /// 줄 후보 계산에 필요한 값만 관측한다. 페이지 전환 뒤 다시 호출해야 한다.
    pub(super) fn paragraph_line_scan_page(&self) -> LineScanPage {
        LineScanPage {
            profile: self.profile,
            col_count: self.col_count,
            body_height: self.base_available_height(),
            current_height: self.current_height,
            has_items: !self.current_items.is_empty(),
            vpos_ladder_dirty: self.vpos_ladder_dirty,
            current_footnote_height: self.current_footnote_height,
            footnote_safety_margin: self.footnote_safety_margin,
            current_zone_y_offset: self.current_zone_y_offset,
            current_bottom_fixed_exclusion: self.current_bottom_fixed_exclusion,
        }
    }

    pub(super) fn inline_flow_column(&self) -> LayoutRect {
        let layout = self.current_zone_layout.as_ref().unwrap_or(&self.layout);
        let mut column = layout
            .column_areas
            .get(self.current_column as usize)
            .copied()
            .unwrap_or(layout.body_area);
        column.y += self.current_zone_y_offset;
        column.height = (column.height - self.current_zone_y_offset).max(0.0);
        column
    }

    pub(super) fn inline_flow_input(&self, start: f64, preceding: bool) -> InlineFlowInput<'_> {
        InlineFlowInput {
            column: self.inline_flow_column(),
            body: &self.layout.body_area,
            page_width: self.layout.page_width,
            page_height: self.layout.page_height,
            start,
            exclusions: preceding.then_some(&self.side_wrap_exclusions),
        }
    }

    /// fit을 통과한 동일 plan을 배치 metadata와 높이에 함께 반영한다.
    pub(super) fn commit_inline_flow(&mut self, para_index: usize, plan: InlineFlowPlan) {
        self.current_items
            .push(PageItem::FullParagraph { para_index });
        self.current_height = plan.end;
        self.inline_box_flow_bottom = self.inline_box_flow_bottom.max(plan.end);
        self.inline_flow_plans.insert(para_index, plan);
        self.vpos_ladder_dirty = true;
    }

    /// 다음 fit의 안전여백 면제를 한 번 소비한다. float 배제 영역 적용 전에 호출한다.
    pub(super) fn take_paragraph_safety_margin(
        &mut self,
        strict_after_empty_host_float: bool,
        prev_is_partial_table: bool,
        layout_drift_safety_px: f64,
    ) -> f64 {
        if strict_after_empty_host_float {
            // The preceding table established a hard painted-bottom flow floor, so the generic
            // tail-before-vpos-reset safety exemption must not leak across this boundary.
            self.skip_safety_margin_once = false;
        }
        if self.skip_safety_margin_once {
            self.skip_safety_margin_once = false;
            0.0
        } else if prev_is_partial_table {
            0.0
        } else if self.vpos_page_base_stored && self.vpos_page_base.is_some() {
            // [#2243] 현재 위치가 **저장** lineseg page-path 앵커로 스냅된 상태면
            // 누적 drift 가 정의상 없으므로 safety 마진을 면제한다 (#361 의
            // PartialTable 정밀 누적 면제와 동일 근거). 156631374: stored 정위치
            // 881.3 + fit 49.6 = 930.9 ≤ 933.6 인데 마진 4px 가 razor 를 기각해
            // 1쪽 문서가 2쪽으로 밀리던 회귀.
            0.0
        } else {
            layout_drift_safety_px
        }
    }

    /// float 배제 영역을 반영한 상태에서 각주 안전여백 반환 flag를 소비한다.
    pub(super) fn take_paragraph_footnote_margin_addback(
        &mut self,
        strict_after_empty_host_float: bool,
    ) -> f64 {
        // [Task #1725] tail-before-vpos-reset 문단은 각주 안전마진(보수 버퍼 40px)만 1회 되돌려
        // 본문에 유지한다. 한글 LINESEG 는 이 tail 문단을 본문(각주 위)에 배치하는데, rhwp 각주
        // 예약의 보수 버퍼가 tail 을 수 px 밀어 near-empty 페이지 over-pagination(국제고속선기준
        // 258 vs 242)을 만든다. 다음 문단이 새 페이지를 시작(vpos-reset)하므로 tail 을 현재
        // 페이지에 두는 것이 한글 정합. (실제 각주 높이는 유지 — 버퍼만 완화하여 겹침 위험 최소화;
        // 버퍼 초과분은 별도 원인이라 여기서 다루지 않는다.)
        if strict_after_empty_host_float {
            self.skip_footnote_margin_once = false;
            0.0
        } else if self.skip_footnote_margin_once {
            self.skip_footnote_margin_once = false;
            if self.current_footnote_height > 0.0 {
                self.footnote_safety_margin
            } else {
                0.0
            }
        } else {
            0.0
        }
    }

    /// 각주 보정 다음에 저장 꼬리 증거를 소비하고 순수 fit 계산으로 넘긴다.
    pub(super) fn take_paragraph_tail_overflow(
        &mut self,
        strict_after_empty_host_float: bool,
        fit_height: f64,
    ) -> f64 {
        // vpos-reset 직전 tail은 저장 line의 실제 bottom이 body 안에 있을 때만, 현재
        // flow가 그 bottom까지 닿는 정확한 차이를 1회 반영한다.
        if strict_after_empty_host_float {
            self.tail_saved_bounds_once = None;
            0.0
        } else {
            self.tail_saved_bounds_once
                .take()
                .and_then(|bounds| {
                    saved_tail_overflow_to_fit(
                        bounds,
                        self.current_height,
                        fit_height,
                        self.base_available_height(),
                        self.current_footnote_height,
                    )
                })
                .unwrap_or(0.0)
        }
    }
}
