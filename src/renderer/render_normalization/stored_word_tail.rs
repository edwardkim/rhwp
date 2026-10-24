//! 줄 끝 재결합과 같은 원본 물리 프레임의 후속 원점을 함께 발행한다.
use crate::model::{
    control::Control,
    page::{ColumnDef, PageDef},
    paragraph::{ColumnBreakType, LineSeg, Paragraph},
};
use crate::renderer::{
    composer::{project_stored_soft_wraps, projects_current_marker_geometry, ParagraphBox},
    page_layout::PageLayoutInfo,
    px_to_hwpunit,
    style_resolver::ResolvedStyleSet,
};

fn uniform_column_width(page: &PageDef, columns: &ColumnDef, dpi: f64) -> Option<f64> {
    let layout = PageLayoutInfo::from_page_def(page, columns, dpi);
    let width = layout.column_areas.first()?.width;
    // 너비가 다른 단은 현재 문단의 실제 소유 단 없이 첫 단으로 추측하지 않는다.
    layout
        .column_areas
        .iter()
        .all(|column| px_to_hwpunit(column.width, dpi) == px_to_hwpunit(width, dpi))
        .then_some(width)
}

fn frame_advance(lines: &[LineSeg]) -> Option<i32> {
    let first = lines.first()?;
    let last = lines.last()?;
    last.vertical_pos
        .checked_add(last.line_height)?
        .checked_add(last.line_spacing)?
        .checked_sub(first.vertical_pos)
}

/// 셀 선언보다 긴 정상 저장 줄들도 표 전체의 선언 상자를 정확히 닫을 수 있다.
/// 온전한 원본 격자의 실제 줄 끝·안 여백으로 닫히는 행은 짧은 셀 선언만으로
/// 낡은 캐시라고 판정하지 않는다. 쪽 재시작·병합·편집 셀은 이 인증을 갖지 않는다.
fn stored_closed_table_row_frames(
    table: &crate::model::table::Table,
) -> Option<std::collections::HashMap<u16, u32>> {
    if table.common.height == 0
        || !crate::renderer::float_placement::stored_complete_column_frame(table)
    {
        return None;
    }
    let mut rows = std::collections::HashMap::<u16, u32>::new();
    for cell in &table.cells {
        if cell.row_span != 1 || cell.line_wrap != 0 {
            return None;
        }
        let padding = cell.effective_padding(&table.padding);
        if padding.top < 0 || padding.bottom < 0 {
            return None;
        }
        let ink = crate::renderer::float_placement::stored_plain_cell_ink_height(cell)?;
        let end = u32::try_from(ink)
            .ok()?
            .checked_add(padding.top as u32)?
            .checked_add(padding.bottom as u32)?;
        let height = rows.entry(cell.row).or_default();
        *height = (*height).max(cell.height).max(end);
    }
    let total = (0..table.row_count).try_fold(0_u64, |sum, row| {
        sum.checked_add(u64::from(*rows.get(&row)?))
    })?;
    let spacing = u64::try_from(table.cell_spacing)
        .ok()?
        .checked_mul(u64::from(table.row_count.saturating_sub(1)))?;
    (total.checked_add(spacing)? == u64::from(table.common.height)).then_some(rows)
}

pub(crate) fn normalize(
    original: &[Paragraph],
    styles: &ResolvedStyleSet,
    page: &PageDef,
    initial_columns: &ColumnDef,
    refresh_markers: bool,
    dpi: f64,
) -> Option<Vec<Paragraph>> {
    normalize_flow(
        original,
        styles,
        uniform_column_width(page, initial_columns, dpi),
        Some(page),
        true,
        refresh_markers,
        dpi,
    )
}

/// 문단의 새 줄 구성과 뒤 문단의 원점은 같은 흐름에서 함께 발행한다.
/// 본문은 단의 폭을, 셀은 자신의 정수 내용 상자를 소유한다.
fn normalize_flow(
    original: &[Paragraph],
    styles: &ResolvedStyleSet,
    mut width: Option<f64>,
    body_page: Option<&PageDef>,
    refresh_text: bool,
    refresh_markers: bool,
    dpi: f64,
) -> Option<Vec<Paragraph>> {
    let mut shift = 0_i32;
    let mut previous_source_vpos = None;
    let mut result = Vec::with_capacity(original.len());
    let mut changed = false;
    for source in original {
        if source.column_type != ColumnBreakType::None
            || styles
                .para_styles
                .get(source.para_shape_id as usize)
                .is_some_and(|style| style.page_break_before)
        {
            shift = 0;
        }
        for control in &source.controls {
            if let (Some(page), Control::ColumnDef(columns)) = (body_page, control) {
                width = uniform_column_width(page, columns, dpi);
                shift = 0;
            }
        }
        let mut paragraph = source.clone();
        for control in &mut paragraph.controls {
            let Control::Table(table) = control else {
                continue;
            };
            let mut row_frames = std::collections::HashMap::<u16, u32>::new();
            for cell in &table.cells {
                if cell.row_span == 1 {
                    let height = row_frames.entry(cell.row).or_default();
                    *height = (*height).max(cell.height);
                }
            }
            if let Some(closed) = stored_closed_table_row_frames(table) {
                row_frames = closed;
            }
            for cell in &mut table.cells {
                if cell.text_direction != 0 || cell.line_wrap != 0 {
                    continue;
                }
                let padding = cell.effective_padding(&table.padding);
                let Some(inner_hu) = i32::try_from(cell.width).ok().and_then(|width| {
                    width
                        .checked_sub(i32::from(padding.left))?
                        .checked_sub(i32::from(padding.right))
                }) else {
                    continue;
                };
                if inner_hu <= 0 {
                    continue;
                }
                // 괘선의 누적 px 좌표를 다시 빼면 정수 폭이 1HU 작아질 수 있다.
                // 원본의 정수 길이에서 내용 폭을 계산한 뒤 기존 셀 폭 계약에 전달한다.
                let inner = crate::renderer::composer::cell_inner_text_width(
                    crate::renderer::hwpunit_to_px(inner_hu, dpi),
                    0.0,
                    0.0,
                    dpi,
                );
                let frame = if cell.row_span == 1 {
                    row_frames.get(&cell.row).copied().unwrap_or(cell.height)
                } else {
                    cell.height
                };
                let stale = stored_text_exceeds_frame(cell, frame, &padding);
                if let Some(projected) = normalize_flow(
                    &cell.paragraphs,
                    styles,
                    Some(inner),
                    None,
                    stale,
                    refresh_markers,
                    dpi,
                ) {
                    cell.paragraphs = projected;
                    changed = true;
                }
            }
        }
        let mut origin_changed = false;
        for line in &mut paragraph.line_segs {
            let source_vpos = line.vertical_pos;
            if line.is_first_line_of_page()
                || line.is_first_line_of_column()
                || previous_source_vpos.is_some_and(|previous| source_vpos < previous)
            {
                shift = 0;
            }
            previous_source_vpos = Some(source_vpos);
            line.vertical_pos = source_vpos.checked_add(shift)?;
            origin_changed |= shift != 0;
        }
        if origin_changed {
            // 원문 문자 축은 그대로지만 물리 원점은 새 프레임 소유다.
            // 이전 원점 snapshot을 남겨 같은 파생 줄을 서로 다른 축으로
            // 판정하지 않는다. 원본 document의 snapshot은 보존된다.
            paragraph.source_line_seg_vertical_pos = None;
        }
        if let Some(width) = width {
            let style = styles.para_styles.get(paragraph.para_shape_id as usize);
            let box_ = if body_page.is_some() {
                ParagraphBox::body_for_style(width, style, dpi)
            } else {
                ParagraphBox::content_for_style(width, style, dpi)
            };
            // 현재 측정으로 더 적은 줄에 들어간다는 사실만으로 정상 저장 줄을
            // 무효화하지 않는다. 원문 문자에 없는 목록 마커는 문서 순서에서
            // 해소한 실제 기하를 갱신해야 하며, 같은 결과가 채움·높이에 참여한다.
            let spans_physical_frames =
                crate::renderer::composer::stored_rows_cross_physical_frame(&paragraph);
            let selected_program_rows =
                crate::renderer::composer::project_stored_rows_with_selected_program(
                    &paragraph, &box_, styles, dpi,
                );
            let marker_rows = || {
                if !refresh_markers
                    || !(refresh_text || spans_physical_frames)
                    || !projects_current_marker_geometry(&paragraph, &box_, styles)
                {
                    return None;
                }
                let first = paragraph.line_segs[0].vertical_pos;
                let frame = box_.frame(first);
                project_stored_soft_wraps(&paragraph, &frame, styles, dpi, false)
            };
            if let Some((lines, spaces)) = selected_program_rows.or_else(marker_rows) {
                let old_advance = frame_advance(&paragraph.line_segs)?;
                let new_advance = frame_advance(&lines)?;
                shift = shift.checked_add(new_advance.checked_sub(old_advance)?)?;
                paragraph.replace_line_segs_with_space_metrics(lines, spaces);
                changed = true;
            }
        }
        result.push(paragraph);
    }
    changed.then_some(result)
}

/// 줄 수 감소만으로 캐시를 버리지 않고 원본 저장 프레임의 실제 초과를 확인한다.
fn stored_text_exceeds_frame(
    cell: &crate::model::table::Cell,
    frame_height: u32,
    padding: &crate::model::Padding,
) -> bool {
    if cell.dirty_flag || frame_height == 0 || frame_height > i32::MAX as u32 {
        return false;
    }
    let mut end = 0_i64;
    let mut first = true;
    for para in &cell.paragraphs {
        if !para.controls.is_empty()
            || para.column_type != ColumnBreakType::None
            || para.stored_text_partition_is_dirty()
            || para.cell_format_vpos_dirty
            || para.line_segs.is_empty()
        {
            return false;
        }
        for line in &para.line_segs {
            if line.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0
                || line.line_height <= 0
                || line.line_spacing < 0
                || i64::from(line.vertical_pos) < end
                || (!first && (line.is_first_line_of_page() || line.is_first_line_of_column()))
            {
                return false;
            }
            end = i64::from(line.vertical_pos) + i64::from(line.line_height);
            first = false;
        }
    }
    !first && end + i64::from(padding.top) + i64::from(padding.bottom) > i64::from(frame_height)
}
