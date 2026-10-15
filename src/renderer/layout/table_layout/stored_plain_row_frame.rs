//! Source row boxes authenticate partial text frames beyond the opening fragment.

use crate::model::paragraph::LineSeg;
use crate::model::table::{Cell, Table, TablePageBreak};

use super::{LayoutEngine, ResolvedStyleSet};

impl LayoutEngine {
    /// 일반 부분 컷의 저장 글줄은 중첩 개체의 물리 조각을 입증하지 않는다.
    /// 해당 행의 실제 소유자 전부가 원본 일반 글줄 상자를 공유하는지 확인한다.
    pub(crate) fn row_has_original_plain_text_owners(
        &self,
        table: &Table,
        row: usize,
        cut: &[usize],
    ) -> bool {
        let owners: Vec<_> = table
            .cells
            .iter()
            .filter(|cell| {
                usize::from(cell.row) <= row
                    && row < usize::from(cell.row) + usize::from(cell.row_span)
            })
            .collect();
        !owners.is_empty()
            && owners.len() == cut.len()
            && owners.iter().all(|cell| {
                usize::from(cell.row) == row
                    && cell.row_span == 1
                    && self.stored_plain_text_cell_frame(table, cell)
            })
    }

    pub(crate) fn stored_span_opening_frame(
        &self,
        table: &Table,
        row: usize,
    ) -> Option<crate::renderer::float_placement::StoredSpanOpeningFrame> {
        let profile = self.profile.get();
        crate::renderer::float_placement::stored_span_opening_frame(
            table,
            row,
            profile.hwp5_stored_pagination_layout() || profile.hwpx_stored_layout(),
            profile.session_edited(),
            self.render_normalization
                .borrow()
                .table_text_reflowed(table),
        )
    }

    /// A frame with blank tail cannot identify an ordinary row cut from one
    /// cell alone. Completed siblings fit; remaining owners must agree on the
    /// same source ink boundary before restarting within their paragraph.
    pub(super) fn stored_row_cut_has_shared_ink_end(
        &self,
        table: &Table,
        cell: &Cell,
        previous: &LineSeg,
        frame: i64,
    ) -> bool {
        let padding = cell.effective_padding(&table.padding);
        let ink_end = i64::from(previous.vertical_pos)
            + i64::from(previous.line_height)
            + i64::from(padding.top)
            + i64::from(padding.bottom);
        let mut shared_boundary = false;
        for peer in table
            .cells
            .iter()
            .filter(|peer| peer.row == cell.row && peer.row_span == 1 && peer.col != cell.col)
        {
            if peer.dirty_flag
                || peer.paragraphs.iter().any(|para| {
                    !para.controls.is_empty()
                        || para.stored_text_partition_is_dirty()
                        || para.cell_format_vpos_dirty
                        || para.line_segs.is_empty()
                        || para.line_segs.iter().any(|line| {
                            line.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0
                                || line.vertical_pos < 0
                                || line.line_height <= 0
                        })
                })
            {
                return false;
            }
            let pad = peer.effective_padding(&table.padding);
            let peer_padding = i64::from(pad.top) + i64::from(pad.bottom);
            let shared = peer.paragraphs.iter().any(|para| {
                para.line_segs.windows(2).any(|pair| {
                    let (before, after) = (&pair[0], &pair[1]);
                    let prefix: Vec<_> = peer
                        .paragraphs
                        .iter()
                        .flat_map(|paragraph| &paragraph.line_segs)
                        .take_while(|line| !std::ptr::eq(*line, before))
                        .collect();
                    before.vertical_pos > 0
                        && after.vertical_pos == 0
                        && before.line_spacing >= 0
                        && after.text_start > before.text_start
                        && before.column_start == after.column_start
                        && before.segment_width > 0
                        && before.segment_width == after.segment_width
                        && prefix
                            .windows(2)
                            .all(|lines| lines[0].vertical_pos <= lines[1].vertical_pos)
                        && prefix
                            .last()
                            .is_none_or(|line| line.vertical_pos <= before.vertical_pos)
                        && i64::from(before.vertical_pos)
                            + i64::from(before.line_height)
                            + peer_padding
                            == ink_end
                        && ink_end < frame
                        && frame
                            < ink_end
                                + i64::from(before.line_spacing)
                                + i64::from(after.line_height)
                        && frame < i64::from(peer.height)
                })
            });
            if shared {
                shared_boundary = true;
                continue;
            }
            let lines: Vec<_> = peer
                .paragraphs
                .iter()
                .flat_map(|para| &para.line_segs)
                .collect();
            if lines.is_empty()
                || lines
                    .windows(2)
                    .any(|pair| pair[1].vertical_pos < pair[0].vertical_pos)
                || lines.iter().any(|line| {
                    i64::from(line.vertical_pos) + i64::from(line.line_height) + peer_padding
                        > frame
                })
            {
                return false;
            }
        }
        shared_boundary
    }

    pub(crate) fn stored_row_boxes_exceed_first_frame(&self, table: &Table) -> bool {
        if table.common.height == 0 || table.common.height > i32::MAX as u32 {
            return false;
        }
        // Rows without a single-row declaration contribute a zero lower bound.
        // A merged-only row cannot erase the independently declared other rows.
        let lower_bound: i64 = table
            .get_raw_row_heights()
            .iter()
            .filter(|&&height| height < 0x8000_0000)
            .map(|&height| i64::from(height))
            .sum();
        lower_bound + i64::from(table.cell_spacing) * i64::from(table.row_count.saturating_sub(1))
            > i64::from(table.common.height)
    }

    pub(crate) fn stored_two_line_row_cut_frame(
        &self,
        table: &Table,
        row: usize,
        styles: &ResolvedStyleSet,
    ) -> bool {
        let profile = self.profile.get();
        if !(profile.hwp5_stored_pagination_layout() || profile.hwpx_stored_layout())
            || profile.session_edited()
            || self
                .render_normalization
                .borrow()
                .table_text_reflowed(table)
            || table.common.treat_as_char
            || table.page_break != TablePageBreak::RowBreak
            || !self.stored_row_boxes_exceed_first_frame(table)
        {
            return false;
        }
        let cells: Vec<_> = table
            .cells
            .iter()
            .filter(|cell| usize::from(cell.row) == row)
            .collect();
        !cells.is_empty()
            && cells.iter().all(|cell| {
                cell.row_span == 1
                    && !cell.dirty_flag
                    && cell.paragraphs.iter().all(|para| {
                        !para.stored_text_partition_is_dirty()
                            && !para.cell_format_vpos_dirty
                            && para.controls.is_empty()
                    })
            })
            && cells.iter().any(|cell| {
                let [first, next] = cell.paragraphs.as_slice() else {
                    return false;
                };
                let ([before], [after]) = (first.line_segs.as_slice(), next.line_segs.as_slice())
                else {
                    return false;
                };
                let padding = cell.effective_padding(&table.padding);
                (before.tag | after.tag) & LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0
                    && before.vertical_pos == 0
                    && after.vertical_pos == 0
                    && before.line_height > 0
                    && after.line_height > 0
                    && before.line_spacing >= 0
                    && i64::from(before.line_height)
                        + i64::from(before.line_spacing)
                        + i64::from(after.line_height)
                        + i64::from(padding.top)
                        + i64::from(padding.bottom)
                        == i64::from(cell.height)
                    && [first, next].iter().all(|para| {
                        styles
                            .para_styles
                            .get(para.para_shape_id as usize)
                            .is_some_and(|style| {
                                style.spacing_before == 0.0 && style.spacing_after == 0.0
                            })
                    })
            })
    }
}
