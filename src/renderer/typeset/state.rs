//! 조판 상태의 조회 입력과 확정 결과 반영 경계.
//! inline 흐름, 문단 fit의 1회성 보정 소비와 일반 전체/분할 배치 반영을 소유한다.
//! 나머지 상태 변경은 상위 구현에 남아 있다.

use super::inline_flow::plan::InlineFlowInput;
use super::paragraph::fit::saved_tail_overflow_to_fit;
use super::paragraph::placement::ParagraphFragment;
use super::paragraph::scan::LineScanPage;
use super::TypesetState;
use crate::renderer::inline_flow::InlineFlowPlan;
use crate::renderer::page_layout::LayoutRect;
use crate::renderer::pagination::PageItem;

impl TypesetState {
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

    /// 같은 전체 배치 경로에서 계산한 trim, 높이, underrun, 저장 하단을 순서대로 반영한다.
    pub(super) fn apply_fitted_paragraph_flow(
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
