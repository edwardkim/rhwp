//! 분할 표 종료 뒤 저장 호스트 줄의 전진량을 조판과 실제 배치에서 공유한다.

use crate::model::{paragraph::Paragraph, table::Table};

/// 저장 사다리가 입증한 호스트 줄을 분할 종료 뒤 한 번 소비한다.
/// 이미 방출한 줄과 편집으로 무효화한 저장 위치는 종료 여백에 다시 더하지 않는다.
pub(crate) fn stored_unconsumed_rowbreak_host_advance_hu(
    native_stored_layout: bool,
    para: &Paragraph,
    table: &Table,
    next_para: Option<&Paragraph>,
    host_line_consumed: bool,
) -> Option<i32> {
    if host_line_consumed
        || para.stored_text_partition_is_dirty()
        || para.cell_format_vpos_dirty
        || next_para.is_some_and(|next| {
            next.stored_text_partition_is_dirty() || next.cell_format_vpos_dirty
        })
    {
        return None;
    }
    crate::renderer::float_placement::native_empty_host_rowbreak_line_advance_hu(
        native_stored_layout,
        para,
        table,
        next_para,
    )
}
