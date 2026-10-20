//! 문단 경계의 완결성을 실제 셀 유닛 소유로 판정한다.

use super::LayoutEngine;
use crate::model::paragraph::{LineSeg, NumberingMarker};
use crate::model::style::HeadType;
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
        .is_some()
    }

    /// 실제 목록의 완결 항목 사이에는 문단 내부 고아 줄이 없다.
    /// 일반 제목의 문자 모양으로 목록을 추정하지 않고 paint의 마커를 확인한다.
    pub(crate) fn row_cut_closes_independent_list_items(
        &self,
        table: &Table,
        row: usize,
        start_cut: &[usize],
        end_cut: &[usize],
        styles: &ResolvedStyleSet,
    ) -> bool {
        if !self.row_cut_completes_unlinked_plain_paragraphs(table, row, start_cut, end_cut, styles)
        {
            return false;
        }
        let mut has_item = false;
        for (index, cell_index) in Self::row_cut_cell_order(table, row).into_iter().enumerate() {
            let cell = &table.cells[cell_index];
            let units = self.cell_units(cell, table, styles);
            let end = end_cut[index];
            if end == units.len() {
                continue;
            }
            let paragraph = &cell.paragraphs[units[end - 1].para_idx];
            let style = &styles.para_styles[paragraph.para_shape_id as usize];
            let has_marker = match style.head_type {
                HeadType::Number => matches!(
                    &paragraph.numbering_marker,
                    NumberingMarker::Text(text, _) if !text.trim().is_empty()
                ),
                HeadType::Bullet => {
                    super::paragraph_layout::bullet_marker_text(style, styles).is_some()
                }
                _ => false,
            };
            if !has_marker {
                return false;
            }
            has_item = true;
        }
        has_item
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
        .is_some()
    }

    /// 진행 중인 모든 칸이 한 줄씩 두 조각으로만 나뉘는가.
    /// 긴 문단의 첫 줄 컷과, 완결한 문단 사이의 컷은 이 고아 줄 질의와 구분한다.
    pub(crate) fn row_cut_leaves_single_lines_on_both_sides(
        &self,
        table: &Table,
        row: usize,
        start_cut: &[usize],
        end_cut: &[usize],
        styles: &ResolvedStyleSet,
    ) -> bool {
        if !start_cut.is_empty()
            || !self.row_cut_respects_plain_paragraph_keeps(table, row, start_cut, end_cut, styles)
        {
            return false;
        }
        let mut splits_paragraph = false;
        for (index, cell_index) in Self::row_cut_cell_order(table, row).into_iter().enumerate() {
            let cell = &table.cells[cell_index];
            if cell.paragraphs.iter().any(|paragraph| {
                !paragraph
                    .controls
                    .iter()
                    .all(crate::renderer::composer::control_is_width_neutral_marker)
            }) {
                return false;
            }
            let units = self.cell_units(cell, table, styles);
            let Some(&end) = end_cut.get(index) else {
                return false;
            };
            if end == units.len() {
                continue;
            }
            if end != 1
                || units.len() != 2
                || units.iter().any(|unit| !unit.is_single_visible_line())
            {
                return false;
            }
            splits_paragraph |= units[0].continues_same_paragraph(&units[1]);
        }
        splits_paragraph
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
        .is_some()
    }

    /// 위 유지 계약을 만족하는 컷 가운데 진행 중인 셀 하나 이상이 문단 끝에서 끝나는가.
    /// 모든 진행 셀이 문단 안에서 끊기면 false다.
    pub(crate) fn row_cut_keeps_close_a_plain_paragraph(
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
        ) == Some(true)
    }

    /// 계약을 만족하면 `Some(문단 끝 컷 존재 여부)`, 아니면 `None`.
    fn row_cut_plain_paragraph_contract(
        &self,
        table: &Table,
        row: usize,
        start_cut: &[usize],
        end_cut: &[usize],
        styles: &ResolvedStyleSet,
        (require_complete_paragraphs, source): (bool, CutSource),
    ) -> Option<bool> {
        // 유지 속성은 실제 줄 구성의 계약이다. 저장 LineSeg가 없거나 편집으로
        // 재생성됐다는 이유로 완결한 문단 컷을 고정 px 고아 기준에 넘기지 않는다.
        let profile = self.profile.get();
        let table_reflowed = self
            .render_normalization
            .borrow()
            .table_text_reflowed(table);
        if !(profile.hwp5_stored_pagination_layout() || profile.hwpx_stored_layout())
            || table.page_break != TablePageBreak::RowBreak
            || table.common.treat_as_char
            || (matches!(source, CutSource::Stored) && table_reflowed)
        {
            return None;
        }
        let mut has_kept_cut = false;
        let mut closes_paragraph = false;
        for (index, cell_index) in Self::row_cut_cell_order(table, row).into_iter().enumerate() {
            let cell = &table.cells[cell_index];
            let units = self.cell_units(cell, table, styles);
            let &end = end_cut.get(index)?;
            if end > units.len() {
                return None;
            }
            if end == units.len() {
                continue;
            }
            let start = start_cut.get(index).copied().unwrap_or(0);
            if end <= start
                || cell.text_direction != 0
                || cell.paragraphs.iter().any(|para| {
                    !para
                        .controls
                        .iter()
                        .all(crate::renderer::composer::control_is_width_neutral_marker)
                })
            {
                return None;
            }
            let closing = &units[end - 1];
            let following = &units[end];
            let complete = closing.closes_before_next_paragraph(following);
            if !complete
                && (require_complete_paragraphs || !closing.continues_same_paragraph(following))
            {
                return None;
            }
            let paragraph = cell.paragraphs.get(closing.para_idx)?;
            if paragraph.text.trim().is_empty() {
                return None;
            }
            match source {
                CutSource::Stored => {
                    // 문단 끝 컷은 저장 글줄이 없어도 문단 소유 경계로 확정된다.
                    // 문단 안 컷의 저장 줄 인증과 별도의 Reflow 계약은 유지한다.
                    if paragraph.stored_text_partition_is_dirty()
                        || paragraph.cell_format_vpos_dirty
                        || (paragraph.line_segs.is_empty() && !complete)
                        || paragraph
                            .line_segs
                            .iter()
                            .any(|line| line.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0)
                    {
                        return None;
                    }
                }
                CutSource::Reflow => {
                    if !(table_reflowed || crate::renderer::para_has_no_stored_line_segs(paragraph))
                        || !closing.has_visible_line()
                    {
                        return None;
                    }
                }
            }
            let style = styles.para_styles.get(paragraph.para_shape_id as usize)?;
            if (complete && style.keep_with_next)
                || (!complete && (style.keep_lines || style.widow_orphan))
            {
                return None;
            }
            // 문단 유지 지시가 없는 내부 컷도 유효한 글줄 경계다.
            // 엄격한 완결 문단 질의는 위에서 내부 컷을 이미 제외했다.
            has_kept_cut = true;
            closes_paragraph |= complete;
        }
        has_kept_cut.then_some(closes_paragraph)
    }
}
