//! 표 도메인 조정. 기존 평가·상태 쓰기 순서를 보존한다.

use crate::renderer::typeset::{
    column_def_design_spacing_px, controls, footnote_between_notes_margin_px,
    footnote_separator_overhead_px, hwpunit_to_px, is_synthetic_line_seg, para_has_visible_text,
    paragraph, row_geometry_table, state, table, BlockTableContinuationSource, ColumnDef,
    ComposedParagraph, Control, FootnoteShape, MeasuredTable, PageDef, PageLayoutInfo,
    PaginationResult, Paragraph, ResolvedStyleSet, ResumablePaginationStep,
    ResumableTablePaginationJob, TypesetEngine, TypesetState,
};

impl TypesetEngine {
    /// [#2424] 단일 문단의 마지막 대형 RowBreak 표를 shadow flow state에서 시작한다.
    ///
    /// Stage D의 첫 fast path는 편집 지연 대상이 문서의 유일한 본문 표인 형상으로
    /// 의도적으로 제한한다. 지원 범위를 벗어나면 caller가 기존 full paginate로
    /// fallback할 수 있도록 `None`을 반환한다.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn begin_resumable_table_pagination(
        &self,
        paragraphs: &[Paragraph],
        composed: &[ComposedParagraph],
        styles: &ResolvedStyleSet,
        page_def: &PageDef,
        column_def: &ColumnDef,
        section_index: usize,
        measured_tables: &[MeasuredTable],
        hide_empty_line: bool,
        profile: crate::model::provenance::LayoutCompatibilityProfile,
        skip_spacing_before_prededuct: bool,
        footnote_shape: Option<&FootnoteShape>,
        paragraph_index: usize,
        control_index: usize,
        fragment_budget: usize,
    ) -> Option<ResumableTablePaginationJob> {
        if paragraphs.len() != 1 || paragraph_index != 0 || column_def.column_count.max(1) != 1 {
            return None;
        }
        let para = paragraphs.get(paragraph_index)?;
        let table = match para.controls.get(control_index)? {
            Control::Table(table) => table,
            _ => return None,
        };
        if table.common.treat_as_char
            || !matches!(
                table.page_break,
                crate::model::table::TablePageBreak::RowBreak
            )
            || table.row_count <= 1
            || para_has_visible_text(para)
            || para.controls.iter().enumerate().any(|(index, control)| {
                index != control_index
                    && !matches!(
                        control,
                        Control::SectionDef(_) | Control::ColumnDef(_) | Control::Bookmark(_)
                    )
            })
            || para
                .controls
                .iter()
                .enumerate()
                .skip(control_index + 1)
                .any(|(_, control)| !matches!(control, Control::Bookmark(_)))
        {
            return None;
        }
        let measured_table = measured_tables.iter().find(|measured| {
            measured.para_index == paragraph_index && measured.control_index == control_index
        })?;
        if measured_table.row_heights.is_empty() {
            return None;
        }

        let layout = PageLayoutInfo::from_page_def(page_def, column_def, self.dpi);
        self.profile.set(profile);
        let default_footnote_shape = FootnoteShape::default();
        let footnote_shape = footnote_shape.unwrap_or(&default_footnote_shape);
        let mut state = TypesetState::new(
            layout,
            1,
            section_index,
            footnote_separator_overhead_px(footnote_shape, self.dpi),
            footnote_between_notes_margin_px(footnote_shape, self.dpi),
            hwpunit_to_px(3000, self.dpi),
            column_def.column_type,
        );
        state.hide_empty_line = hide_empty_line;
        state.profile = profile;
        state.has_stored_line_segs = para
            .line_segs
            .iter()
            .any(|line| !is_synthetic_line_seg(line));
        state.skip_spacing_before_prededuct = skip_spacing_before_prededuct;
        state.current_zone_design_spacing_px = column_def_design_spacing_px(column_def, self.dpi);
        state.section_has_no_footer = true;
        state.ensure_page();

        let column_width = state
            .layout
            .column_areas
            .first()
            .map(|area| area.width)
            .unwrap_or(state.layout.body_area.width);
        let formatted_para = self.format_paragraph(
            para,
            composed.get(paragraph_index),
            styles,
            Some(column_width),
        );
        let formatted_table = self.format_table(
            para,
            paragraph_index,
            control_index,
            table,
            measured_tables,
            styles,
            composed.get(paragraph_index),
            None,
            true,
        );
        let context = self.typeset_block_table_inner(
            &mut state,
            paragraph_index,
            control_index,
            para,
            table,
            &formatted_table,
            &formatted_para,
            Some(measured_table),
            styles,
            0.0,
            0.0,
            true,
            true,
            paragraphs,
            composed,
            true,
        )?;
        let mut job = ResumableTablePaginationJob {
            context,
            paragraph_index,
            control_index,
        };
        job.context.fragment_budget = fragment_budget.max(1);
        Some(job)
    }

    pub(crate) fn step_resumable_table_pagination(
        &self,
        job: &mut ResumableTablePaginationJob,
        paragraph: &Paragraph,
        table: &crate::model::table::Table,
        measured_table: &MeasuredTable,
        styles: &ResolvedStyleSet,
        fragment_budget: usize,
    ) -> ResumablePaginationStep {
        self.profile.set(job.context.flow_state.profile);
        job.context.fragment_budget = fragment_budget.max(1);
        let before = job.context.cursor.fragments_emitted;
        let source = BlockTableContinuationSource {
            para_index: job.paragraph_index,
            control_index: job.control_index,
            paragraph,
            table,
            row_geometry_table: row_geometry_table(table),
            measured_table,
            styles,
        };
        self.step_block_table_continuation(&mut job.context, source);
        ResumablePaginationStep {
            fragments_processed: job.context.cursor.fragments_emitted.saturating_sub(before),
            complete: job.context.is_complete(),
        }
    }

    pub(crate) fn finish_resumable_table_pagination(
        &self,
        job: ResumableTablePaginationJob,
        paragraphs: &[Paragraph],
        section_index: usize,
    ) -> Option<PaginationResult> {
        if !job.context.is_complete() {
            return None;
        }
        let mut state = job.context.into_flow_state();
        if !state.current_items.is_empty() {
            state.flush_column_always();
        }
        state.ensure_page();
        let (hf_entries, page_number_pos) =
            Self::collect_header_footer_controls(paragraphs, section_index);
        Self::finalize_pages(&mut state.pages, &hf_entries, &page_number_pos, paragraphs);
        Some(PaginationResult {
            pages: state.pages,
            wrap_around_paras: Vec::new(),
            hidden_empty_paras: state.hidden_empty_paras,
            pre_emitted_host_paras: state.pre_emitted_host_paras,
            pre_emitted_host_heights: state.pre_emitted_host_heights,
            endnotes: state.endnotes,
            endnote_paragraphs: state.endnote_paragraphs,
            endnote_para_sources: state.endnote_para_sources,
            endnote_between_notes_hu: state.endnote_between_notes_hu,
            endnote_separator_above_hu: state.endnote_separator_above_hu,
            endnote_separator_below_hu: state.endnote_separator_below_hu,
        })
    }

    pub(crate) fn resumable_table_target(job: &ResumableTablePaginationJob) -> (usize, usize) {
        (job.paragraph_index, job.control_index)
    }
}
