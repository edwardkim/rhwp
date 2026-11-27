//! 원본 중첩 표의 줄 수용과 비끝 조각의 페이지 테두리.

use super::{
    row_physical_band::{CutHeight, RowCutCapacity},
    LayoutEngine, NestedTableCut, ResolvedStyleSet, RowCutResult,
};
use crate::model::{
    control::Control,
    paragraph::{LineSeg, Paragraph},
    shape::VertAlign,
    table::{Cell, Table, TablePageBreak, VerticalAlign},
};
use crate::renderer::{
    float_placement::{
        is_para_topbottom_float, native_single_cell_rowbreak_page_fragment,
        single_cell_page_fragment_bottom,
    },
    hwpunit_to_px,
};

pub(crate) struct StoredNestedPageBand {
    pub(in crate::renderer::layout) last_row_height: f64,
    pub(crate) remaining_child_row: (usize, f64),
}

impl LayoutEngine {
    fn stored_nested_page_owner<'a>(
        &self,
        parent: &'a Table,
        row: usize,
    ) -> Option<(&'a Cell, &'a Paragraph, &'a Table)> {
        let profile = self.profile.get();
        let [cell] = parent.cells.as_slice() else {
            return None;
        };
        let [host] = cell.paragraphs.as_slice() else {
            return None;
        };
        let [Control::Table(child)] = host.controls.as_slice() else {
            return None;
        };
        let [line] = host.line_segs.as_slice() else {
            return None;
        };
        if row != 0
            || !native_single_cell_rowbreak_page_fragment(
                profile.hwp5_stored_pagination_layout(),
                parent,
            )
            || profile.session_edited()
            || cell.dirty_flag
            || cell.vertical_align != VerticalAlign::Top
            || super::super::para_has_non_whitespace_text(host)
            || host.stored_text_partition_is_dirty()
            || host.cell_format_vpos_dirty
            || line.vertical_pos != 0
            || line.line_height <= 0
            || line.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0
            || child.row_count <= 1
            || child.layout_page_break() != TablePageBreak::RowBreak
            || child.cell_spacing != 0
            || [parent, child.as_ref()].iter().any(|table| {
                table.caption.is_some()
                    || !is_para_topbottom_float(&table.common)
                    || !table.common.flow_with_text
                    || table.common.allow_overlap
                    || table.common.vert_align != VertAlign::Top
                    || table.common.vertical_offset != 0
                    || self
                        .render_normalization
                        .borrow()
                        .table_text_reflowed(table)
            })
        {
            return None;
        }
        Some((cell, host, child))
    }

    /// 쪽을 가르는 자식 행의 실제 줄만 수용한다. 저장 첫 프레임의 빈
    /// 끝은 원래 줄 소속의 증거이며 새 조각의 최소 필요 높이가 아니다.
    pub(crate) fn stored_nested_page_band_at_height(
        &self,
        parent: &Table,
        row: usize,
        start: &[usize],
        end: &[usize],
        parent_height: f64,
        remaining_child_row: Option<(usize, f64)>,
        styles: &ResolvedStyleSet,
    ) -> Option<StoredNestedPageBand> {
        let (cell, _, child) = self.stored_nested_page_owner(parent, row)?;
        let [end] = end else { return None };
        let first = start.first().copied().unwrap_or(0);
        let split = self.nested_table_split_from_cut_units(cell, parent, styles, first, *end, 0)?;
        let cut = split.recursive_cut?;
        let last = cut.end_row.checked_sub(1)?;
        if cut.is_block_split || cut.start_cut_is_block || cut.end_cut.is_empty() {
            return None;
        }
        let row_start = if cut.start_row == last {
            cut.start_cut.as_slice()
        } else {
            &[]
        };
        if self.row_cut_consumes_all_ordinary_units(child, last, &cut.end_cut, styles) {
            return None;
        }

        let (_, _, pad_top, pad_bottom) =
            self.resolve_stored_nested_cell_padding(cell, parent, styles);
        let child_bottom =
            single_cell_page_fragment_bottom(child, parent_height - pad_bottom, self.dpi);
        let child_top = pad_top + hwpunit_to_px(i32::from(child.outer_margin_top), self.dpi);
        let prefix_cut = NestedTableCut {
            start_row: cut.start_row,
            end_row: last,
            start_cut: cut.start_cut.clone(),
            end_cut: Vec::new(),
            is_block_split: false,
            start_cut_is_block: false,
        };
        let prefix = self.nested_cut_physical_height(child, &prefix_cut, styles)
            + if cut.start_row < last {
                self.stored_child_row_remainder_delta(child, &cut, remaining_child_row, styles)
            } else {
                0.0
            };
        let last_row_height = child_bottom - child_top - prefix;
        let minimum = self.row_cut_height_in_source_band(
            child,
            last,
            row_start,
            &cut.end_cut,
            styles,
            CutHeight::Occupied,
            true,
        );
        // 저장 경계는 정수 HWPUNIT의 틈까지 구분한다. 픽셀 환산과
        // 누적 덧셈에서 생긴 부동소수점 오차만 허용한다.
        let rounding = 8.0 * f64::EPSILON * parent_height.max(1.0);
        if minimum <= 0.0 || last_row_height + rounding < minimum {
            return None;
        }
        let source_height = remaining_child_row
            .filter(|(row, _)| *row == last && cut.start_row == last)
            .map_or_else(
                || self.declared_row_frame_height(child, last),
                |(_, height)| height,
            );
        Some(StoredNestedPageBand {
            last_row_height,
            remaining_child_row: (last, (source_height - last_row_height).max(0.0)),
        })
    }

    fn stored_child_row_remainder_delta(
        &self,
        child: &Table,
        cut: &NestedTableCut,
        remaining_child_row: Option<(usize, f64)>,
        styles: &ResolvedStyleSet,
    ) -> f64 {
        let Some((row, height)) = remaining_child_row else {
            return 0.0;
        };
        if row != cut.start_row || cut.start_cut.is_empty() || cut.start_cut_is_block {
            return 0.0;
        }
        let content = self.row_cut_content_height(child, row, &cut.start_cut, &[], styles);
        (height - content).max(0.0)
    }

    pub(crate) fn stored_nested_page_parent_floor(
        &self,
        parent: &Table,
        start: &[usize],
        remaining_child_row: Option<(usize, f64)>,
        styles: &ResolvedStyleSet,
    ) -> Option<f64> {
        remaining_child_row?;
        let (cell, _, child) = self.stored_nested_page_owner(parent, 0)?;
        let first = start.first().copied()?;
        let units = self.cell_units(cell, parent, styles);
        let split =
            self.nested_table_split_from_cut_units(cell, parent, styles, first, units.len(), 0)?;
        let cut = split.recursive_cut?;
        let delta = self.stored_child_row_remainder_delta(child, &cut, remaining_child_row, styles);
        (delta > 0.0).then(|| self.row_cut_content_height(parent, 0, start, &[], styles) + delta)
    }

    pub(crate) fn advance_stored_nested_page_band(
        &self,
        parent: &Table,
        row: usize,
        start: &[usize],
        parent_height: f64,
        remaining_child_row: Option<(usize, f64)>,
        styles: &ResolvedStyleSet,
    ) -> Option<(RowCutResult, f64)> {
        let (cell, _, _) = self.stored_nested_page_owner(parent, row)?;
        let (_, _, pad_top, pad_bottom) =
            self.resolve_stored_nested_cell_padding(cell, parent, styles);
        let budget = (parent_height - pad_top - pad_bottom).max(0.0);
        let mut cut = self.advance_row_cut_inner(
            parent,
            row,
            start,
            budget,
            styles,
            RowCutCapacity::Physical {
                deltas: &[0.0],
                allow_stored_tail: true,
            },
        );
        if cut.fully_consumed {
            return None;
        }
        let first = start.first().copied().unwrap_or(0);
        let units = self.cell_units(cell, parent, styles);
        while let Some(next) = cut.end_cut.first().copied() {
            if next < first
                || !units
                    .get(next)
                    .is_some_and(|unit| !unit.hard_break_before && !unit.page_frame_reset_before)
                || self
                    .stored_nested_page_band_at_height(
                        parent,
                        row,
                        start,
                        &[next + 1],
                        parent_height,
                        remaining_child_row,
                        styles,
                    )
                    .is_none()
            {
                break;
            }
            cut.end_cut[0] += 1;
            cut.hit_hard_break = false;
        }
        while cut.end_cut.first().is_some_and(|end| *end > first) {
            if self
                .stored_nested_page_band_at_height(
                    parent,
                    row,
                    start,
                    &cut.end_cut,
                    parent_height,
                    remaining_child_row,
                    styles,
                )
                .is_some()
            {
                cut.consumed_height = budget;
                cut.physical_height = Some(parent_height);
                return Some((cut, budget));
            }
            let end = cut.end_cut[0];
            if self
                .nested_table_split_from_cut_units(cell, parent, styles, first, end, 0)?
                .recursive_cut?
                .end_cut
                .is_empty()
            {
                break;
            }
            cut.end_cut[0] -= 1;
            cut.hit_hard_break = false;
        }
        None
    }
}
