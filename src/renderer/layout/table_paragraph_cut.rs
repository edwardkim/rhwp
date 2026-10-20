//! 문단 경계의 완결성을 실제 셀 유닛 소유로 판정한다.

use super::LayoutEngine;
use crate::model::paragraph::LineSeg;
use crate::model::table::{Table, TablePageBreak};
use crate::renderer::style_resolver::ResolvedStyleSet;

#[derive(Clone, Copy)]
enum CutSource {
    Stored,
    Reflow,
}

impl LayoutEngine {
    /// 완결한 독립 문단을 다음 문단과 나누는 컷에는 문단 내부 고아 줄이 없다.
    /// 진행 중인 모든 셀의 컷이 문단 경계여야 하며, 이어붙임 지시는 보존한다.
    /// 실제 컷 높이·패딩·병합 셀 점유의 예산 검사는 호출자가 계속 수행한다.
    pub(crate) fn row_cut_completes_unlinked_plain_paragraphs(
        &self,
        table: &Table,
        row: usize,
        start_cut: &[usize],
        end_cut: &[usize],
        styles: &ResolvedStyleSet,
    ) -> bool {
        self.row_cut_plain_paragraph_contract(
            table,
            row,
            start_cut,
            end_cut,
            styles,
            (true, CutSource::Stored),
        )
    }

    /// 수용한 글줄의 컷은 이웃 셀의 실제 문단 유지 지시를 함께 확인한다.
    /// 내부 컷에는 실제 widow/orphan·keep_lines, 문단 끝에는 keep_with_next를 적용한다.
    pub(crate) fn row_cut_respects_plain_paragraph_keeps(
        &self,
        table: &Table,
        row: usize,
        start_cut: &[usize],
        end_cut: &[usize],
        styles: &ResolvedStyleSet,
    ) -> bool {
        self.row_cut_plain_paragraph_contract(
            table,
            row,
            start_cut,
            end_cut,
            styles,
            (false, CutSource::Stored),
        )
    }

    /// 재조판한 일반 텍스트의 온전한 글줄도 같은 문단 유지 지시를 따른다.
    /// 저장 사다리 인증을 대체하지 않으며 실제 높이·예산 검사는 호출자가 유지한다.
    pub(crate) fn reflow_row_cut_respects_plain_paragraph_keeps(
        &self,
        table: &Table,
        row: usize,
        start_cut: &[usize],
        end_cut: &[usize],
        styles: &ResolvedStyleSet,
    ) -> bool {
        self.row_cut_plain_paragraph_contract(
            table,
            row,
            start_cut,
            end_cut,
            styles,
            (false, CutSource::Reflow),
        )
    }

    fn row_cut_plain_paragraph_contract(
        &self,
        table: &Table,
        row: usize,
        start_cut: &[usize],
        end_cut: &[usize],
        styles: &ResolvedStyleSet,
        (require_complete_paragraphs, source): (bool, CutSource),
    ) -> bool {
        let profile = self.profile.get();
        let table_reflowed = self
            .render_normalization
            .borrow()
            .table_text_reflowed(table);
        if !(profile.hwp5_stored_pagination_layout() || profile.hwpx_stored_layout())
            || profile.session_edited()
            || table.page_break != TablePageBreak::RowBreak
            || table.common.treat_as_char
            || (matches!(source, CutSource::Stored) && table_reflowed)
        {
            return false;
        }
        let mut has_kept_cut = false;
        for (index, cell_index) in Self::row_cut_cell_order(table, row).into_iter().enumerate() {
            let cell = &table.cells[cell_index];
            let units = self.cell_units(cell, table, styles);
            let Some(&end) = end_cut.get(index) else {
                return false;
            };
            if end > units.len() {
                return false;
            }
            if end == units.len() {
                continue;
            }
            let start = start_cut.get(index).copied().unwrap_or(0);
            if end <= start
                || cell.dirty_flag
                || cell.text_direction != 0
                || cell.paragraphs.iter().any(|para| !para.controls.is_empty())
            {
                return false;
            }
            let closing = &units[end - 1];
            let following = &units[end];
            let complete = closing.closes_before_next_paragraph(following);
            if !complete
                && (require_complete_paragraphs || !closing.continues_same_paragraph(following))
            {
                return false;
            }
            let Some(paragraph) = cell.paragraphs.get(closing.para_idx) else {
                return false;
            };
            if paragraph.text.trim().is_empty() {
                return false;
            }
            match source {
                CutSource::Stored => {
                    if paragraph.stored_text_partition_is_dirty()
                        || paragraph.cell_format_vpos_dirty
                        || paragraph.line_segs.is_empty()
                        || paragraph
                            .line_segs
                            .iter()
                            .any(|line| line.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0)
                    {
                        return false;
                    }
                }
                CutSource::Reflow => {
                    if !(table_reflowed || crate::renderer::para_has_no_stored_line_segs(paragraph))
                        || !closing.has_visible_line()
                    {
                        return false;
                    }
                }
            }
            let Some(style) = styles.para_styles.get(paragraph.para_shape_id as usize) else {
                return false;
            };
            if (complete && style.keep_with_next)
                || (!complete && (style.keep_lines || style.widow_orphan))
            {
                return false;
            }
            // 문단 유지 지시가 없는 내부 컷도 유효한 글줄 경계다.
            // 엄격한 완결 문단 질의는 위에서 내부 컷을 이미 제외했다.
            has_kept_cut = true;
        }
        has_kept_cut
    }
}
