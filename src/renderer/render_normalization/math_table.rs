//! 수식 줄의 저장 높이에서 파생된 표 프레임과 호스트 전진을 함께 재발행한다.

use crate::model::{control::Control, paragraph::Paragraph, table::Table};
use crate::renderer::style_resolver::ResolvedStyleSet;

/// 단일 저장 줄 셀의 내용 높이 계약. 병합 행·재줄바꿈·중첩 개체는 별도 계약이다.
fn stored_cell_height(cell: &crate::model::table::Cell) -> Option<i32> {
    use crate::model::paragraph::LineSeg;
    if cell.text_direction != 0 || cell.line_wrap != 0 || cell.paragraphs.len() != 1 {
        return None;
    }
    let paragraph = &cell.paragraphs[0];
    if paragraph.stored_text_partition_is_dirty()
        || paragraph.line_segs.len() != 1
        || paragraph
            .controls
            .iter()
            .any(|control| !matches!(control, Control::Equation(equation) if equation.common.treat_as_char))
    {
        return None;
    }
    let line = &paragraph.line_segs[0];
    (line.vertical_pos == 0
        && line.line_height > 0
        && line.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0)
        .then_some(line.line_height)
}

fn content_rows(table: &Table) -> Option<Vec<i32>> {
    match content_rows_with_storage(table, &crate::storage::Unbounded) {
        Ok(value) => value,
        Err(never) => match never {},
    }
}

pub(crate) fn content_rows_with_storage<S: crate::storage::Storage>(
    table: &Table,
    storage: &S,
) -> Result<Option<Vec<i32>>, S::Error> {
    crate::storage::with_optional(|failure| {
        let mut rows =
            crate::storage::optional(storage.filled(usize::from(table.row_count), 0), failure)?;
        if rows.is_empty() || table.col_count == 0 {
            return None;
        }
        for cell in &table.cells {
            if cell.row_span != 1 || cell.col_span == 0 {
                return None;
            }
            let height = stored_cell_height(cell)?;
            let padding = cell.effective_vertical_padding_hu(&table.padding);
            let declared = i32::try_from(cell.height).ok()?;
            let required = declared.max(height.checked_add(padding)?);
            let row = rows.get_mut(usize::from(cell.row))?;
            *row = (*row).max(required);
        }
        rows.iter().all(|height| *height > 0).then_some(rows)
    })
}

fn total_height(table: &Table, rows: &[i32]) -> Option<i32> {
    rows.iter().try_fold(
        i32::from(table.cell_spacing).checked_mul(rows.len().saturating_sub(1) as i32)?,
        |total, height| total.checked_add(*height),
    )
}

fn normalize_host(paragraph: &mut Paragraph, styles: &ResolvedStyleSet) -> Option<i32> {
    match normalize_host_with_storage(paragraph, styles, &crate::storage::Unbounded) {
        Ok(value) => value,
        Err(never) => match never {},
    }
}

pub(crate) fn normalize_host_with_storage<S: crate::storage::Storage>(
    paragraph: &mut Paragraph,
    styles: &ResolvedStyleSet,
    storage: &S,
) -> Result<Option<i32>, S::Error> {
    crate::storage::with_optional(|failure| {
        if !paragraph.text.trim().is_empty()
            || paragraph.stored_text_partition_is_dirty()
            || paragraph.controls.len() != 1
            || paragraph.line_segs.len() != 1
        {
            return None;
        }
        let Control::Table(table) = &paragraph.controls[0] else {
            return None;
        };
        if !table.common.treat_as_char
            || table.common.size_protect
            || table.common.locked
            || table.caption.is_some()
        {
            return None;
        }
        let old_height = i32::try_from(table.common.height).ok()?;
        let old_total = total_height(
            table,
            &crate::storage::optional(content_rows_with_storage(table, storage), failure)??,
        )?;
        let host = &paragraph.line_segs[0];
        let margins = i32::from(table.common.margin.top) + i32::from(table.common.margin.bottom);
        // 1 HWPUNIT는 정수 저장 반올림이다. 사용자가 추가한 프레임 공간이나
        // 별도 호스트 높이는 셀 내용에서 파생됐다고 추정하지 않는다.
        if (old_total - old_height).abs() > 1
            || (host.line_height - old_height.checked_add(margins)?).abs() > 1
            || host.text_height != host.line_height
            || host.line_height <= 0
            || host.tag & crate::model::paragraph::LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0
        {
            return None;
        }
        let mut derived = crate::storage::optional(storage.clone_table(table.as_ref()), failure)?;
        let mut changed = false;
        for cell in &mut derived.cells {
            if let Some(projected) =
            crate::storage::optional(crate::renderer::composer::stored_row_projection::project_equation_rows_with_storage(
                &cell.paragraphs,
                styles,storage), failure)?
        {
            cell.paragraphs = projected;
            changed = true;
        }
        }
        if !changed {
            return None;
        }
        let new_height = total_height(
            &derived,
            &crate::storage::optional(content_rows_with_storage(&derived, storage), failure)??,
        )?;
        let delta = new_height.checked_sub(old_height)?;
        if delta == 0 || new_height <= 0 {
            return None;
        }
        derived.common.height = u32::try_from(new_height).ok()?;
        let host = &mut paragraph.line_segs[0];
        let new_line_height = host.line_height.checked_add(delta)?;
        if new_line_height <= 0 {
            return None;
        }
        host.baseline_distance = (i64::from(host.baseline_distance) * i64::from(new_line_height)
            / i64::from(host.line_height)) as i32;
        host.line_height = new_line_height;
        host.text_height = new_line_height;
        crate::storage::optional(storage.bytes(std::mem::size_of_val(&derived)), failure)?;
        paragraph.controls[0] = Control::Table(Box::new(derived));
        Some(delta)
    })
}

fn has_math_table(paragraphs: &[Paragraph]) -> bool {
    paragraphs.iter().any(|paragraph| {
        paragraph.controls.iter().any(|control| match control {
            Control::Table(table) => {
                table.common.treat_as_char
                    && table.cells.iter().any(|cell| {
                        cell.paragraphs.iter().any(|paragraph| {
                            paragraph
                                .controls
                                .iter()
                                .any(|control| matches!(control, Control::Equation(_)))
                        })
                    })
            }
            Control::Endnote(note) => has_math_table(&note.paragraphs),
            Control::Footnote(note) => has_math_table(&note.paragraphs),
            _ => false,
        })
    })
}

pub(crate) fn normalize(
    original: &[Paragraph],
    styles: &ResolvedStyleSet,
) -> Option<Vec<Paragraph>> {
    match normalize_with_storage(original, styles, &crate::storage::Unbounded) {
        Ok(value) => value,
        Err(never) => match never {},
    }
}

pub(crate) fn normalize_with_storage<S: crate::storage::Storage>(
    original: &[Paragraph],
    styles: &ResolvedStyleSet,
    storage: &S,
) -> Result<Option<Vec<Paragraph>>, S::Error> {
    crate::storage::with_optional(|failure| {
        if !has_math_table(original) {
            return None;
        }
        let mut result = crate::storage::optional(
            storage.try_collect(
                original
                    .iter()
                    .map(|paragraph| storage.clone_paragraph(paragraph)),
            ),
            failure,
        )?;
        let mut shift = 0_i32;
        let mut previous_vpos = None;
        let mut changed = false;
        for paragraph in &mut result {
            for control in &mut paragraph.controls {
                let children = match control {
                    Control::Endnote(note) => &mut note.paragraphs,
                    Control::Footnote(note) => &mut note.paragraphs,
                    _ => continue,
                };
                if let Some(projected) = crate::storage::optional(
                    normalize_with_storage(children, styles, storage),
                    failure,
                )? {
                    *children = projected;
                    changed = true;
                }
            }
            for line in &mut paragraph.line_segs {
                let source_vpos = line.vertical_pos;
                if previous_vpos.is_some_and(|previous| source_vpos < previous) {
                    shift = 0;
                }
                previous_vpos = Some(source_vpos);
                line.vertical_pos = source_vpos.saturating_add(shift);
            }
            if let Some(delta) = crate::storage::optional(
                normalize_host_with_storage(paragraph, styles, storage),
                failure,
            )? {
                shift = shift.saturating_add(delta);
                changed = true;
            }
        }
        changed.then_some(result)
    })
}
