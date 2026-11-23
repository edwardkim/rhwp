//! 닫힌 저장 행에서 그림 글줄의 내용 상자와 셀 안 여백을 구분한다.

use crate::model::{
    control::Control,
    paragraph::{ColumnBreakType, Paragraph},
    shape::VertAlign,
    table::{Cell, Table, TablePageBreak, VerticalAlign},
};

/// 빈 저장 줄을 흡수한 단독 그림 띠는 그림 자체의 높이를 점유한다.
pub(crate) fn stored_picture_only_cell_height_hu(cell: &Cell) -> Option<i32> {
    let [para] = cell.paragraphs.as_slice() else {
        return None;
    };
    let [Control::Picture(picture)] = para.controls.as_slice() else {
        return None;
    };
    let common = &picture.common;
    let height = i32::try_from(common.height).ok()?;
    (height > 0
        && !cell.dirty_flag
        && para.column_type == ColumnBreakType::None
        && !para.stored_text_partition_is_dirty()
        && !para.cell_format_vpos_dirty
        && common.flow_with_text
        && !common.allow_overlap
        && common.vertical_offset == 0
        && common.vert_align == VertAlign::Top
        && common.margin.top == 0
        && common.margin.bottom == 0
        && picture.caption.is_none()
        && crate::renderer::empty_host_line_absorbed_by_topbottom_float(para, common))
    .then_some(height)
}

/// 앞 설명, 그림, 뒤 설명으로 닫히는 저장 표의 물리 행 높이.
/// 그림 호스트의 폭 없는 저장 줄과 뒤 설명에 같은 선행 전진이 기록된다.
/// 개체 선언을 정확히 닫는 경우에만 전진을 뒤 설명이 소유하고 그림 행은
/// 그림과 안 여백만 소비한다.
pub(crate) fn stored_captioned_picture_row_heights(table: &Table) -> Option<[i32; 3]> {
    if table.row_count != 3
        || table.col_count != 1
        || table.cells.len() != 3
        || table.cell_spacing != 0
        || !super::is_para_topbottom_float(&table.common)
        || table.layout_page_break() != TablePageBreak::RowBreak
        || !super::stored_complete_column_frame(table)
    {
        return None;
    }
    let caption = table.cell_at(0, 0)?;
    let picture = table.cell_at(1, 0)?;
    let source = table.cell_at(2, 0)?;
    let [host] = picture.paragraphs.as_slice() else {
        return None;
    };
    let [caption_para] = caption.paragraphs.as_slice() else {
        return None;
    };
    let [source_para] = source.paragraphs.as_slice() else {
        return None;
    };
    let trailing_origin = source_para.line_segs.first()?.vertical_pos;
    if trailing_origin <= 0
        || host.line_segs.first()?.vertical_pos != trailing_origin
        || caption_para.text.trim().is_empty()
        || source_para.text.trim().is_empty()
        || [caption, picture, source].iter().any(|cell| {
            cell.row_span != 1
                || cell.text_direction != 0
                || cell.vertical_align != VerticalAlign::Center
        })
    {
        return None;
    }
    let caption_padding = caption.effective_vertical_padding_hu(&table.padding);
    let picture_padding = picture.effective_vertical_padding_hu(&table.padding);
    let source_padding = source.effective_vertical_padding_hu(&table.padding);
    let picture_height =
        stored_picture_only_cell_height_hu(picture)?.checked_add(picture_padding)?;
    if i32::try_from(caption.height).ok()? != caption_padding
        || i32::try_from(source.height).ok()? != source_padding
        || i32::try_from(picture.height).ok()? >= picture_height
    {
        return None;
    }
    let rows = [
        super::stored_plain_cell_ink_height(caption)?.checked_add(caption_padding)?,
        picture_height,
        super::stored_plain_cell_ink_height_from(source, trailing_origin)?
            .checked_add(source_padding)?,
    ];
    (rows.iter().copied().map(i64::from).sum::<i64>() == i64::from(table.common.height))
        .then_some(rows)
}

/// 줄 구성과 같은 HWPUNIT 그림 점유 높이가 실제 저장 줄을 증명할 때만 수용한다.
/// 원래 전체 표를 닫는지, 편집·합성 줄인지 등은 공통 행 생산자가 추가 확인한다.
pub(super) fn controls_preserve_stored_picture_lines(para: &Paragraph) -> bool {
    // MultiColumn은 한 단 정의에도 붙는 IR 상태다. 실제 단/쪽 나눔과
    // 다단 배치는 이 단일 저장 행 인증에서 소비하지 않는다.
    let single_column = para
        .controls
        .iter()
        .any(|control| matches!(control, Control::ColumnDef(column) if column.column_count == 1));
    if para.column_type != ColumnBreakType::None
        && !(para.column_type == ColumnBreakType::MultiColumn && single_column)
    {
        return false;
    }
    let Some(owners) = super::stored_control_line_indices(para) else {
        return false;
    };
    if owners.len() != para.controls.len() {
        return false;
    }
    let mut occupied = vec![0; para.line_segs.len()];
    let mut has_picture = false;
    for (control, owner) in para.controls.iter().zip(owners) {
        let picture = match control {
            // 단 정의는 그림 줄 높이를 점유하는 개체가 아니다.
            Control::ColumnDef(column) if column.column_count == 1 => continue,
            Control::Picture(picture)
                if picture.common.treat_as_char && picture.caption.is_none() =>
            {
                picture
            }
            _ => return false,
        };
        let (width, height) = crate::renderer::composer::inline_picture_occupied_size_hu(picture);
        if width <= 0 || height <= 0 {
            return false;
        }
        let Some(line_height) = occupied.get_mut(owner) else {
            return false;
        };
        *line_height = (*line_height).max(height);
        has_picture = true;
    }
    has_picture
        && occupied
            .iter()
            .zip(&para.line_segs)
            .all(|(height, line)| *height == 0 || *height == line.line_height)
}
