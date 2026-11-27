//! 실제 셀 컷과 후속 저장 줄의 닫힌 등식으로 끝 조각의 단 축을 전달한다.

use super::FragmentInput;
use crate::renderer::typeset::{
    hwpunit_to_px, is_synthetic_line_seg, PageItem, TypesetEngine, TypesetState,
};

impl TypesetEngine {
    pub(super) fn restore_consumed_cell_prefix_axis(
        &self,
        st: &mut TypesetState,
        input: &FragmentInput<'_>,
    ) {
        let table = input.source.row_geometry_table;
        if !st.profile.hwp5_origin_hwpx()
            || st.profile.session_edited()
            || st.col_count != 1
            || st.current_zone_y_offset != 0.0
            || st.current_start_height != 0.0
            || st.current_endnote_flow
            || table.row_count != 1
            || table.col_count != 1
            || table.cells.len() != 1
            || table.caption.is_some()
            || !input.prepared.table_footnotes.is_empty()
            || input.row_cursor_is_nested
            || input.start.start_cut_is_block
        {
            return;
        }
        let [PageItem::PartialTable {
            para_index,
            control_index,
            start_row: 0,
            end_row: 1,
            is_continuation: true,
            start_cut,
            end_cut,
            is_block_split: false,
            start_cut_is_block: false,
            row_cursor_is_nested: false,
            ..
        }] = st.current_items.as_slice()
        else {
            return;
        };
        let [cut] = start_cut.as_slice() else {
            return;
        };
        if *para_index != input.source.para_index
            || *control_index != input.source.control_index
            || !end_cut.is_empty()
            || *start_cut != input.start.start_cut
        {
            return;
        }
        let prefix = input
            .prepared
            .layout_engine
            .consumed_stored_cell_frame_prefix(&table.cells[0], table, input.source.styles, *cut);
        let Some(base) = prefix else {
            return;
        };
        let Some((_, next)) = st.table_flow_successor(*para_index, input.source.paragraphs_all)
        else {
            return;
        };
        let [line] = next.line_segs.as_slice() else {
            return;
        };
        if !next.text.is_empty()
            || !next.controls.is_empty()
            || next.stored_text_partition_is_dirty()
            || next.cell_format_vpos_dirty
            || next.source_line_seg_vertical_pos.is_some()
            || is_synthetic_line_seg(line)
            || line.line_height <= 0
        {
            return;
        }
        let expected = line
            .vertical_pos
            .checked_sub(base)
            .map(|end| hwpunit_to_px(end, self.dpi));
        if std::env::var_os("RHWP_DIAG_TERMINAL_AXIS").is_some() {
            eprintln!("TERMINAL_AXIS pi={para_index} cut={cut} prefix={base} expected={expected:?} actual={}", st.current_height);
        }
        if expected.is_none_or(|end| (end - st.current_height).abs() > hwpunit_to_px(2, self.dpi)) {
            return;
        }
        st.record_paragraph_origin(crate::renderer::pagination::ParagraphOrigin {
            paragraph_index: input.source.para_index,
            spacing_before: 0.0,
            source_vpos_base: base,
            flow_start: 0.0,
            flow_end: st.current_height,
        });
        st.record_stored_column_origin(Some((base, 0.0)));
    }
}
