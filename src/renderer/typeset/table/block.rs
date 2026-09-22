//! Block-table orchestration: entry/whole placement, split preparation, then continuation.

use crate::renderer::typeset::{
    row_geometry_table, table, BlockTableContinuationContext, ComposedParagraph,
    FormattedParagraph, FormattedTable, MeasuredTable, Paragraph, ResolvedStyleSet, TypesetEngine,
    TypesetState,
};

use host_placement::HostPlacementConstraint;

mod entry;
mod host_placement;
mod prepare;
mod whole_fit;

#[derive(Clone, Copy)]
struct BlockTableInput<'a> {
    para_idx: usize,
    ctrl_idx: usize,
    para: &'a Paragraph,
    table: &'a crate::model::table::Table,
    ft: &'a FormattedTable,
    fmt: &'a FormattedParagraph,
    mt: Option<&'a MeasuredTable>,
    styles: &'a ResolvedStyleSet,
    para_start_height: f64,
    budget_para_start_height: f64,
    is_first_placed: bool,
    is_last_placed: bool,
    paragraphs_all: &'a [Paragraph],
    composed_all: &'a [ComposedParagraph],
}

/// Values that survive whole-placement attempts. No page state is copied here.
struct SplitTableEntry<'a> {
    total_footnote: f64,
    next_starts_new_page: bool,
    next_rewinds_after_table: bool,
    host_spacing_total: f64,
    table_total: f64,
    native_ordinary_rowbreak_rewind_uses_actual_footnote_boundary: bool,
    fn_margin: f64,
    available: f64,
    declared_object_total: f64,
    native_hwp5_internal_reset_rewind_needs_anchor_resync: bool,
    placement_para_start_height: f64,
    source_anchor_splits_here: bool,
    native_hwp5_rewinding_rowbreak_uses_painted_row_footprint: bool,
    unconstrained_host_placement: Option<crate::renderer::float_placement::ParagraphFloatPlacement>,
    constrain_host_placement: HostPlacementConstraint<'a>,
    mt: &'a MeasuredTable,
    row_geometry_table: &'a crate::model::table::Table,
    declared_table_height: f64,
}

impl TypesetEngine {
    #[allow(clippy::too_many_arguments)]
    pub(in crate::renderer::typeset) fn typeset_block_table(
        &self,
        st: &mut TypesetState,
        para_idx: usize,
        ctrl_idx: usize,
        para: &Paragraph,
        table: &crate::model::table::Table,
        ft: &FormattedTable,
        fmt: &FormattedParagraph,
        mt: Option<&MeasuredTable>,
        styles: &ResolvedStyleSet,
        para_start_height: f64,
        budget_para_start_height: f64,
        is_first_placed: bool,
        is_last_placed: bool,
        paragraphs_all: &[Paragraph],
        composed_all: &[ComposedParagraph],
    ) {
        #[cfg(not(target_arch = "wasm32"))]
        let issue2424_profile_enabled =
            std::env::var("RHWP_2424_PROFILE").is_ok_and(|value| !value.is_empty() && value != "0");
        #[cfg(target_arch = "wasm32")]
        let issue2424_profile_enabled = false;
        let issue2424_started = issue2424_profile_enabled.then(std::time::Instant::now);
        let issue2424_pages_before = st.pages.len();

        self.typeset_block_table_inner(
            st,
            para_idx,
            ctrl_idx,
            para,
            table,
            ft,
            fmt,
            mt,
            styles,
            para_start_height,
            budget_para_start_height,
            is_first_placed,
            is_last_placed,
            paragraphs_all,
            composed_all,
            false,
        );

        if let Some(started) = issue2424_started {
            eprintln!(
                "RHWP_2424_BLOCK_TABLE_PROFILE section={} para={} control={} rows={} elapsed_ms={:.3} pages_added={}",
                st.section_index,
                para_idx,
                ctrl_idx,
                table.row_count,
                started.elapsed().as_secs_f64() * 1000.0,
                st.pages.len().saturating_sub(issue2424_pages_before),
            );
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(in crate::renderer::typeset) fn typeset_block_table_inner(
        &self,
        st: &mut TypesetState,
        para_idx: usize,
        ctrl_idx: usize,
        para: &Paragraph,
        table: &crate::model::table::Table,
        ft: &FormattedTable,
        fmt: &FormattedParagraph,
        mt: Option<&MeasuredTable>,
        styles: &ResolvedStyleSet,
        para_start_height: f64,
        // [Task #1860] 예산 전용 참 para_start(문단의 참 시작 흐름 높이). 대부분
        // para_start_height 와 같으나, 지연 co-anchored 배치에선 para_start_height 가
        // 라이브 current_height(선행 캡션 반영)라 out-of-flow float 예산이 이중차감된다.
        // 렌더 위치는 para_start_height, 빈-host float 예산만 이 값을 쓴다.
        budget_para_start_height: f64,
        is_first_placed: bool,
        is_last_placed: bool,
        // [Task #1753] 지연 이월 직전 후속 문단 prefill 용 전체 슬라이스.
        paragraphs_all: &[Paragraph],
        composed_all: &[ComposedParagraph],
        suspend_before_drain: bool,
    ) -> Option<BlockTableContinuationContext> {
        let input = BlockTableInput {
            para_idx,
            ctrl_idx,
            para,
            table,
            ft,
            fmt,
            mt,
            styles,
            para_start_height,
            budget_para_start_height,
            is_first_placed,
            is_last_placed,
            paragraphs_all,
            composed_all,
        };
        let entry = self.prepare_block_table_entry(st, input)?;
        let (context, source) = self.prepare_block_table_continuation(st, input, entry);
        if suspend_before_drain {
            return Some(context);
        }
        self.drain_block_table_continuation(st, context, source);
        if ft.strict_following_plain_text_fit && is_last_placed {
            st.require_strict_following_text_fit();
        }
        None
    }
}
