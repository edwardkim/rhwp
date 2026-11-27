use crate::model::{
    control::{Control, FieldType},
    paragraph::{LineSeg, Paragraph},
    raw_provenance::record_digest,
    table::{Table, TablePageBreak},
};

/// 평문 셀의 저장 줄을 재사용할 자격만 확인한다.
/// 각 배치 계획은 원점·높이·분할 경계를 별도로 확인한다.
pub(crate) fn stored_plain_table_source_is_current(table: &Table) -> bool {
    stored_table_source_is_current(table, false)
}

/// 전체 표를 한 상자로 배치할 때 중첩 표까지 원본인지 확인한다.
/// 경계를 나눌 필요가 없는지는 호출자가 실측 높이와 저장 후속 줄로 확인한다.
pub(crate) fn stored_complete_table_source_is_current(table: &Table) -> bool {
    stored_table_source_is_current(table, true)
}

fn stored_table_source_is_current(table: &Table, include_nested_tables: bool) -> bool {
    table
        .raw_ctrl_seal
        .is_some_and(|sealed| sealed == record_digest(&table.common))
        && table.caption.is_none()
        && !table.cells.is_empty()
        && table.cells.iter().all(|cell| {
            !cell.dirty_flag
                && !cell.paragraphs.is_empty()
                && cell.paragraphs.iter().all(|paragraph| {
                    stored_cell_lines_are_current(paragraph)
                        && paragraph.controls.iter().all(|control| match control {
                            Control::Table(nested) if include_nested_tables => {
                                super::is_para_topbottom_float(&nested.common)
                                    && nested.common.flow_with_text
                                    && !nested.common.allow_overlap
                                    && (nested.common.vertical_offset as i32) >= 0
                                    && nested.common.height > 0
                                    && nested.layout_page_break() == TablePageBreak::RowBreak
                                    && stored_complete_table_source_is_current(nested)
                            }
                            Control::Field(field) if field.field_type == FieldType::Hyperlink => {
                                true
                            }
                            _ => {
                                include_nested_tables
                                    && crate::renderer::cell_control_is_text_metadata(control)
                            }
                        })
                })
        })
}

fn stored_cell_lines_are_current(paragraph: &Paragraph) -> bool {
    !paragraph.stored_text_partition_is_dirty()
        && !paragraph.cell_format_vpos_dirty
        && !paragraph.line_segs.is_empty()
        && paragraph.line_segs.iter().all(|line| {
            line.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0
                && line.vertical_pos >= 0
                && line.line_height > 0
                && line.line_spacing >= 0
        })
        && paragraph
            .source_line_seg_vertical_pos
            .as_ref()
            .is_none_or(|source| {
                source
                    .iter()
                    .copied()
                    .eq(paragraph.line_segs.iter().map(|line| line.vertical_pos))
            })
}
