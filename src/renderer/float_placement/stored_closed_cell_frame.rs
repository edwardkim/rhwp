//! 닫힌 저장 표 프레임과 앞뒤 글줄의 셀 원점·점유 끝을 공유한다.

use super::{hwpunit_to_px, stored_table_before_host_frame, StoredBlockCellFlow};
use crate::model::{control::Control, paragraph::Paragraph, table::Table};
use crate::renderer::style_resolver::ResolvedStyleSet;

/// 마지막 저장 줄의 점유 끝과 후속 문단이 같은 바깥 상자를 닫는 경우의 괘선 하단.
/// 빈 마지막 줄도 물리 공간을 소유한다. 바깥 아래 여백은 괘선 뒤에서 별도로 소비한다.
pub(super) fn terminal_cell_border_before_successor_hu(
    table: &crate::model::table::Table,
    successor: &Paragraph,
) -> Option<i32> {
    use crate::model::paragraph::LineSeg;
    let [cell] = table.cells.as_slice() else {
        return None;
    };
    let tail = cell.paragraphs.last()?;
    let line = tail.line_segs.last()?;
    let [next] = successor.line_segs.as_slice() else {
        return None;
    };
    if cell.dirty_flag
        || tail.stored_text_partition_is_dirty()
        || tail.cell_format_vpos_dirty
        || !tail.controls.is_empty()
        || line.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0
        || line.vertical_pos < 0
        || line.line_height <= 0
        || table.outer_margin_top < 0
        || table.outer_margin_bottom < 0
    {
        return None;
    }
    let padding = cell.effective_padding(&table.padding);
    if padding.top < 0 || padding.bottom < 0 {
        return None;
    }
    let outer_bottom = i64::from(line.vertical_pos)
        + i64::from(line.line_height)
        + i64::from(padding.top)
        + i64::from(padding.bottom)
        + i64::from(table.outer_margin_top)
        + i64::from(table.outer_margin_bottom);
    (outer_bottom == i64::from(next.vertical_pos))
        .then(|| {
            next.vertical_pos
                .checked_sub(i32::from(table.outer_margin_bottom))
        })
        .flatten()
}

/// 앞 글줄의 저장 전진 끝에서 표의 원본 바깥 상자를 놓았을 때 다음 글줄이
/// 바로 이어지면 표가 두 글줄 사이의 저장 공간을 이미 소비하고 있다.
/// 호스트는 개체 주소만 보유하므로 재측정한 높이로 뒤 글줄을 밀지 않는다.
pub(crate) fn stored_table_between_text_lines(
    paragraphs: &[Paragraph],
    index: usize,
    table: &Table,
    styles: &ResolvedStyleSet,
    dpi: f64,
) -> Option<(f64, f64)> {
    let previous = paragraphs.get(index.checked_sub(1)?)?;
    let host = paragraphs.get(index)?;
    let next = paragraphs.get(index + 1)?;
    let [line] = host.line_segs.as_slice() else {
        return None;
    };
    if !matches!(host.controls.as_slice(), [Control::Table(_)])
        || !host.text.is_empty()
        || !previous.controls.is_empty()
        || !next.controls.is_empty()
        || previous.text.trim().is_empty()
        || line.segment_width != 0
        || table.cells.iter().any(|cell| cell.dirty_flag)
        || table.common.vertical_offset != 0
        || table.common.height == 0
        || table.outer_margin_top < 0
        || table.outer_margin_bottom < 0
    {
        return None;
    }
    let before = previous.line_segs.last()?;
    let after = next.line_segs.first()?;
    // 앞 글줄의 저장 전진 뒤에 문단 뒤 간격도 들어간다. 뒤 문단은
    // 제어 없는 빈 글줄이어도 같은 저장 공간을 닫을 수 있다.
    let spacing_after = styles
        .para_styles
        .get(previous.para_shape_id as usize)?
        .spacing_after;
    let after_hu = (spacing_after / hwpunit_to_px(1, dpi)).round() as i32;
    let start = before
        .vertical_pos
        .checked_add(before.line_height)?
        .checked_add(before.line_spacing)?
        .checked_add(after_hu)?;
    let bottom = start
        .checked_add(i32::try_from(table.common.height).ok()?)?
        .checked_add(i32::from(table.outer_margin_top))?
        .checked_add(i32::from(table.outer_margin_bottom))?;
    (start >= before.vertical_pos && start <= line.vertical_pos && bottom == after.vertical_pos)
        .then(|| (hwpunit_to_px(start, dpi), hwpunit_to_px(bottom, dpi)))
}

/// 모든 블록 표가 앞뒤 글줄과 같은 저장 프레임을 닫을 때 좌표를 재사용한다.
pub(super) fn closed_before_host_flow(
    paragraphs: &[Paragraph],
    styles: &ResolvedStyleSet,
    heights: &[Vec<f64>],
    dpi: f64,
) -> Option<StoredBlockCellFlow> {
    let mut previous = None;
    let mut has_frame = false;
    let mut result = StoredBlockCellFlow {
        paragraph_origins: Vec::with_capacity(paragraphs.len()),
        origins: paragraphs
            .iter()
            .map(|p| vec![None; p.controls.len()])
            .collect(),
        bottom: 0.0,
        closed_before_host: true,
        table_between_text_lines: false,
    };
    for (pi, paragraph) in paragraphs.iter().enumerate() {
        if paragraph.raw_break_type != 0
            || paragraph.column_type != crate::model::paragraph::ColumnBreakType::None
        {
            return None;
        }
        let first = paragraph.line_segs.first()?;
        let source_host = hwpunit_to_px(first.vertical_pos, dpi);
        result.paragraph_origins.push(source_host);
        for line in &paragraph.line_segs {
            if previous.is_some_and(|top| line.vertical_pos < top) {
                return None;
            }
            previous = Some(line.vertical_pos);
            let end = hwpunit_to_px(line.vertical_pos.checked_add(line.line_height)?, dpi);
            result.bottom = result.bottom.max(end);
        }
        for (ci, control) in paragraph.controls.iter().enumerate() {
            let Control::Table(table) = control else {
                continue;
            };
            if let Some((anchor, bottom)) =
                stored_table_between_text_lines(paragraphs, pi, table, styles, dpi)
            {
                result.origins[pi][ci] = Some(anchor);
                result.bottom = result.bottom.max(bottom);
                result.table_between_text_lines = true;
                has_frame = true;
                continue;
            }
            let body = *heights.get(pi)?.get(ci)?
                - hwpunit_to_px(
                    i32::from(table.outer_margin_top) + i32::from(table.outer_margin_bottom),
                    dpi,
                );
            let before = styles
                .para_styles
                .get(paragraph.para_shape_id as usize)?
                .spacing_before;
            let frame = stored_table_before_host_frame(
                paragraphs,
                pi,
                table,
                ci,
                source_host - before,
                body,
                styles,
                dpi,
            )?;
            // 원점은 바깥여백·offset 앞이며 실제 배치가 그 몫을 한 번 적용한다.
            // 표 뒤의 호스트 줄과 다음 본문은 저장 좌표를 그대로 소비한다.
            result.origins[pi][ci] = Some(frame.anchor_y);
            result.bottom = result.bottom.max(frame.occupied_bottom);
            has_frame = true;
        }
    }
    has_frame.then_some(result)
}
