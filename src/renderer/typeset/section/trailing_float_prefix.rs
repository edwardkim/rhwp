//! Resolve a stored float reset after a terminal table fragment and empty guides.

use super::{MeasuredTable, Paragraph, ResolvedStyleSet, TypesetEngine, TypesetState};
use crate::model::{control::Control, paragraph::ColumnBreakType, table::Table};
use crate::renderer::float_placement::{
    stored_complete_table_source_is_current, stored_empty_control_table_frame,
    stored_plain_table_source_is_current, ParagraphFloatPlacement,
};
use crate::renderer::pagination::{PageItem, ParagraphOrigin};
use crate::storage::Storage;
use std::collections::HashSet;

impl TypesetEngine {
    /// The closed title frames are already on the outgoing page. Start the
    /// following body at the part of its saved axis those frames did not consume.
    pub(super) fn try_place_trailing_float_prefix<S: Storage>(
        &self,
        st: &mut TypesetState,
        paragraphs: &[Paragraph],
        first_index: usize,
        styles: &ResolvedStyleSet,
        measured_tables: &[MeasuredTable],
        force_break_before: &HashSet<usize>,
        storage: &S,
    ) -> Result<Option<usize>, S::Error> {
        let Some(plan) = self.trailing_float_prefix(
            st,
            paragraphs,
            first_index,
            styles,
            measured_tables,
            force_break_before,
        ) else {
            return Ok(None);
        };

        let body_index = first_index + plan.headings.len();
        for (offset, placement) in plan.headings.into_iter().enumerate() {
            accept_frame(st, first_index + offset, placement, storage)?;
        }
        st.advance_column_or_new_page_with_storage(storage)?;
        st.record_paragraph_origin_with_storage(
            ParagraphOrigin {
                paragraph_index: body_index,
                spacing_before: 0.0,
                source_vpos_base: plan.consumed_source,
                flow_start: 0.0,
                flow_end: plan.body.occupied_bottom,
            },
            storage,
        )?;
        st.record_stored_column_origin(Some((plan.consumed_source, 0.0)));
        accept_frame(st, body_index, plan.body, storage)?;
        st.record_previous_layout_paragraph(Some(body_index));
        Ok(Some(body_index))
    }

    fn trailing_float_prefix(
        &self,
        st: &TypesetState,
        paragraphs: &[Paragraph],
        first_index: usize,
        styles: &ResolvedStyleSet,
        measured_tables: &[MeasuredTable],
        force_break_before: &HashSet<usize>,
    ) -> Option<TrailingFloatPrefix> {
        if !st.profile.hwp5_stored_pagination_layout()
            || st.profile.session_edited()
            || st.col_count != 1
            || st.current_zone_y_offset != 0.0
            || st.current_endnote_flow
            || st.current_footnote_height != 0.0
            || !st.current_height.is_finite()
            || st.has_deferred_table_controls()
        {
            return None;
        }

        let first = paragraphs.get(first_index)?;
        let second = paragraphs.get(first_index + 1)?;
        let body = paragraphs.get(first_index + 2)?;
        let following = paragraphs.get(first_index + 3)?;
        let first_origin = stored_line_origin(first)?;
        let second_origin = stored_line_origin(second)?;
        let consumed_source = stored_line_origin(body)?;
        if second_origin <= 0 || second_origin >= first_origin || consumed_source <= second_origin {
            return None;
        }

        let preceding = st.current_items.as_slice();
        let [PageItem::PartialTable {
            para_index: terminal,
            control_index,
            end_row,
            is_continuation: true,
            end_cut,
            ..
        }, guides @ ..] = preceding
        else {
            return None;
        };
        let Control::Table(terminal_table) =
            paragraphs.get(*terminal)?.controls.get(*control_index)?
        else {
            return None;
        };
        if guides.is_empty()
            || !end_cut.is_empty()
            || *end_row != usize::from(terminal_table.row_count)
            || terminal.checked_add(guides.len() + 1)? != first_index
        {
            return None;
        }
        let mut source = first_origin;
        for (n, item) in guides.iter().rev().enumerate() {
            let PageItem::FullParagraph { para_index } = item else {
                return None;
            };
            if para_index.checked_add(n + 1)? != first_index {
                return None;
            }
            let guide = paragraphs.get(*para_index)?;
            let start = stored_line_origin(guide)?;
            let line = &guide.line_segs[0];
            if !guide.controls.is_empty()
                || !unforced_style(guide, styles)
                || line.line_spacing < 0
                || start
                    .checked_add(line.line_height)?
                    .checked_add(line.line_spacing)?
                    != source
            {
                return None;
            }
            source = start;
        }

        let first_table = source_table(first)?;
        let second_table = source_table(second)?;
        let body_table = source_table(body)?;
        if first_table.row_count != 1
            || second_table.row_count != 1
            || body_table.row_count < 2
            || !stored_plain_table_source_is_current(first_table)
            || !stored_plain_table_source_is_current(second_table)
            || !stored_complete_table_source_is_current(body_table)
            || first_table.common.width != second_table.common.width
            || second_table.common.width != body_table.common.width
            || !following.controls.is_empty()
            || stored_line_origin(following).is_none()
            || (first_index..first_index + 3).any(|index| force_break_before.contains(&index))
        {
            return None;
        }

        let frame = |index, host: &Paragraph, next, table, frame_vpos| {
            if !unforced_style(host, styles) {
                return None;
            }
            let measured = measured_tables
                .iter()
                .find(|measured| measured.para_index == index && measured.control_index == 0)?;
            stored_empty_control_table_frame(
                host,
                next,
                table,
                measured.total_height,
                0.0,
                frame_vpos,
                self.dpi,
                false,
            )
        };
        let first = frame(first_index, first, second, first_table, 0)?;
        let second = frame(first_index + 1, second, body, second_table, 0)?;
        let body = frame(
            first_index + 2,
            body,
            following,
            body_table,
            consumed_source,
        )?;
        if first.anchor_y != 0.0 || body.anchor_y != 0.0 {
            return None;
        }

        let available = st.base_available_height();
        let headings_end = st.current_height + second.occupied_bottom;
        if headings_end > available
            || headings_end + body.occupied_bottom <= available
            || body.occupied_bottom > available
        {
            return None;
        }
        Some(TrailingFloatPrefix {
            headings: [
                place_after(first, st.current_height),
                place_after(second, st.current_height),
            ],
            body,
            consumed_source,
        })
    }
}

struct TrailingFloatPrefix {
    headings: [ParagraphFloatPlacement; 2],
    body: ParagraphFloatPlacement,
    consumed_source: i32,
}

fn stored_line_origin(paragraph: &Paragraph) -> Option<i32> {
    let [line] = paragraph.line_segs.as_slice() else {
        return None;
    };
    (paragraph.column_type == ColumnBreakType::None
        && paragraph.text.is_empty()
        && !paragraph.stored_text_partition_is_dirty()
        && !paragraph.cell_format_vpos_dirty
        && paragraph.source_line_seg_vertical_pos.is_none()
        && !super::is_synthetic_line_seg(line)
        && line.line_height > 0)
        .then_some(line.vertical_pos)
}

fn unforced_style(paragraph: &Paragraph, styles: &ResolvedStyleSet) -> bool {
    styles
        .para_styles
        .get(paragraph.para_shape_id as usize)
        .is_some_and(|style| {
            !style.page_break_before && style.spacing_before == 0.0 && style.spacing_after == 0.0
        })
}

fn source_table(paragraph: &Paragraph) -> Option<&Table> {
    let [Control::Table(table)] = paragraph.controls.as_slice() else {
        return None;
    };
    (!table.common.allow_overlap
        && table.common.vertical_offset == 0
        && table.layout_page_break() == crate::model::table::TablePageBreak::RowBreak)
        .then_some(table.as_ref())
}

fn place_after(mut frame: ParagraphFloatPlacement, offset: f64) -> ParagraphFloatPlacement {
    frame.anchor_y += offset;
    frame.table_top += offset;
    frame.occupied_bottom += offset;
    frame.stored_successor_line_origin = Some(frame.occupied_bottom);
    frame
}

fn accept_frame<S: Storage>(
    st: &mut TypesetState,
    para_index: usize,
    placement: ParagraphFloatPlacement,
    storage: &S,
) -> Result<(), S::Error> {
    st.record_paragraph_float_placement_with_storage((para_index, 0), placement, storage)?;
    st.append_item_with_storage(
        PageItem::Table {
            para_index,
            control_index: 0,
        },
        storage,
    )?;
    st.align_flow_to(placement.occupied_bottom);
    Ok(())
}
