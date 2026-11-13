//! 원문 글자 크기와 줄간격이 함께 입증하는 명목 줄 메트릭의 렌더 전용 투영.

use crate::model::{
    control::Control,
    paragraph::{ColumnBreakType, LineSeg, Paragraph},
    style::{BorderLineType, HeadType, LineSpacingType},
    table::Cell,
};
use crate::renderer::{
    float_placement::is_para_topbottom_float,
    px_to_hwpunit,
    style_resolver::{detect_lang_category, ResolvedStyleSet},
};

fn floating_text_host(paragraph: &Paragraph) -> bool {
    let mut floating = false;
    for control in &paragraph.controls {
        match control {
            Control::Table(table) if is_para_topbottom_float(&table.common) => floating = true,
            Control::SectionDef(_) | Control::ColumnDef(_) => {}
            _ => return false,
        }
    }
    floating
}

fn nominal_height(
    paragraph: &Paragraph,
    styles: &ResolvedStyleSet,
    dpi: f64,
    include_empty: bool,
) -> Option<i32> {
    // 여러 글자모양의 경계는 완전한 문자 주소가 있을 때만 읽는다.
    // 주소가 없는 합성 입력에서 첫 글자모양을 전체 문단의 근거로 쓰지 않는다.
    if paragraph.char_shapes.is_empty()
        || paragraph
            .char_shapes
            .windows(2)
            .any(|pair| pair[0].start_pos > pair[1].start_pos)
        || (paragraph.char_shapes.len() > 1
            && (paragraph.char_offsets.len() != paragraph.text.chars().count()
                || paragraph
                    .char_offsets
                    .windows(2)
                    .any(|pair| pair[0] >= pair[1])))
    {
        return None;
    }
    let dpi = styles.resolved_dpi.unwrap_or(dpi);
    let mut height = 0;
    let mut visible = false;
    for (position, ch) in paragraph.text.chars().enumerate() {
        if ch.is_control() || ch == '\u{FFFC}' {
            return None;
        }
        visible |= !ch.is_whitespace();
        let style = styles
            .char_styles
            .get(paragraph.char_shape_id_at(position)? as usize)?;
        let language = detect_lang_category(ch);
        if style.superscript
            || style.subscript
            || style.font_position_for_lang(language) != 0
            || style.outline_type != 0
            || style.shadow_type != 0
            || style.emboss
            || style.engrave
            || style.emphasis_dot != 0
        {
            return None;
        }
        if let Some(border) = style
            .border_fill_id
            .checked_sub(1)
            .and_then(|index| styles.border_styles.get(usize::from(index)))
        {
            if border
                .borders
                .iter()
                .any(|edge| edge.line_type != BorderLineType::None)
            {
                return None;
            }
        }
        let size = style.font_size_for_lang(language);
        if !size.is_finite() || size <= 0.0 {
            return None;
        }
        height = height.max(px_to_hwpunit(size, dpi));
    }
    // 마지막 빈 run은 보이는 글자가 없어도 문단 끝 줄 상자의 글자모양이다.
    // 재조판과 같은 원본 UTF-16 종단 주소를 소비해야 정상 저장 높이를
    // 가시 글자의 크기만으로 줄이고 셀의 기준선을 이동시키지 않는다.
    let end_size = crate::renderer::composer::paragraph_end_font_size_px(paragraph, styles);
    if end_size.is_finite() && end_size > 0.0 {
        height = height.max(px_to_hwpunit(end_size, dpi));
    }
    ((visible || include_empty) && height > 0).then_some(height)
}

fn projected_line(paragraph: &Paragraph, styles: &ResolvedStyleSet, dpi: f64) -> Option<LineSeg> {
    // 다른 문단의 편집으로 유효한 저장 줄까지 버리지 않는다.
    // 실제 편집·셀 서식 변경으로 무효화된 줄만 원래 재조판 경로에 맡긴다.
    if paragraph.stored_text_partition_is_dirty() || paragraph.cell_format_vpos_dirty {
        return None;
    }
    let [line] = paragraph.line_segs.as_slice() else {
        return None;
    };
    let style = styles.para_styles.get(paragraph.para_shape_id as usize)?;
    if !matches!(style.head_type, HeadType::None)
        || !matches!(style.line_spacing_type, LineSpacingType::Percent)
        || !style.line_spacing.is_finite()
        || style.line_spacing < 100.0
        || line.text_height <= 0
        || line.baseline_distance <= 0
        || line.baseline_distance > line.text_height
    {
        return None;
    }
    let height = nominal_height(paragraph, styles, dpi, false)?;
    // 원본 글자 크기와 원본 Percent 간격이 같은 높이를 증명해야 한다.
    // 1 HWPUNIT는 정수로 저장되는 줄간격의 반올림만 허용한다.
    let expected_spacing = (f64::from(height) * (style.line_spacing / 100.0 - 1.0)).round();
    if line.text_height <= height
        || line.line_height <= height
        || (f64::from(line.line_spacing) - expected_spacing).abs() > 1.0
    {
        return None;
    }
    let mut projected = line.clone();
    projected.line_height = height;
    projected.text_height = height;
    projected.baseline_distance = (i64::from(line.baseline_distance) * i64::from(height)
        / i64::from(line.text_height)) as i32;
    Some(projected)
}

/// 개체·장식·문단 경계 간격이 없는 한 줄 문단의 현재 스타일 메트릭.
/// 고정 간격은 줄 전체 전진이고 Percent는 명목 글자 높이에 적용한다.
fn owned_cell_line(
    paragraph: &Paragraph,
    styles: &ResolvedStyleSet,
    dpi: f64,
) -> Option<(LineSeg, bool)> {
    let [line] = paragraph.line_segs.as_slice() else {
        return None;
    };
    let style = styles.para_styles.get(paragraph.para_shape_id as usize)?;
    if !paragraph.controls.is_empty()
        || paragraph.stored_text_partition_is_dirty()
        || paragraph.cell_format_vpos_dirty
        || paragraph.column_type != ColumnBreakType::None
        || style.page_break_before
        || style.spacing_before != 0.0
        || style.spacing_after != 0.0
        || !matches!(style.head_type, HeadType::None)
        || line.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0
        || line.text_start != 0
        || line.vertical_pos < 0
        || line.segment_width <= 0
        || line.line_height != line.text_height
        || line.text_height <= 0
        || line.baseline_distance <= 0
        || line.baseline_distance > line.text_height
    {
        return None;
    }
    let height = nominal_height(paragraph, styles, dpi, true)?;
    let advance = match style.line_spacing_type {
        LineSpacingType::Percent
            if style.line_spacing.is_finite() && style.line_spacing >= 100.0 =>
        {
            let value = (f64::from(height) * style.line_spacing / 100.0).round();
            if value > f64::from(i32::MAX) {
                return None;
            }
            value as i32
        }
        LineSpacingType::Fixed if style.line_spacing.is_finite() && style.line_spacing > 0.0 => {
            px_to_hwpunit(style.line_spacing, styles.resolved_dpi.unwrap_or(dpi))
        }
        _ => return None,
    };
    if advance < height {
        return None;
    }
    // 글자 높이만으로 큰 저장 strut를 버리지 않는다. 현재 명목 크기와 문단의
    // 전진 계약이 함께 불일치할 때만 오래된 rhwp 관리 캐시로 판정한다.
    // 1 HU는 독립적으로 정수화한 높이·간격 합의 반올림 오차다.
    let stale = line.text_height > height
        && (i64::from(line.line_height) + i64::from(line.line_spacing) - i64::from(advance)).abs()
            > 1;
    let mut current = line.clone();
    current.line_height = height;
    current.text_height = height;
    current.baseline_distance = (f64::from(line.baseline_distance) * f64::from(height)
        / f64::from(line.text_height))
    .round() as i32;
    current.line_spacing = advance - height;
    Some((current, stale))
}

/// rhwp 인증 구역의 단순 텍스트 셀 캐시를 하나의 현재 줄 사다리로 투영한다.
/// 원본 IR은 보존하고, 같은 렌더 사본을 측정·분할·배치가 소비한다.
fn owned_cell_with_current_metrics<S: crate::storage::Storage>(
    cell: &Cell,
    styles: &ResolvedStyleSet,
    dpi: f64,
    storage: &S,
) -> Result<Option<Vec<Paragraph>>, S::Error> {
    if cell.dirty_flag || cell.text_direction != 0 || cell.line_wrap != 0 {
        return Ok(None);
    }
    let mut stale = false;
    let mut previous = None;
    for paragraph in &cell.paragraphs {
        let Some((_, changed)) = owned_cell_line(paragraph, styles, dpi) else {
            return Ok(None);
        };
        let top = paragraph.line_segs[0].vertical_pos;
        if previous.is_some_and(|previous| top < previous) {
            // 다음 물리 쪽으로 되감긴 줄은 전체 셀 한 프레임이 아니다.
            return Ok(None);
        }
        previous = Some(top);
        stale |= changed;
    }
    if !stale {
        return Ok(None);
    }
    let mut current_top = cell.paragraphs[0].line_segs[0].vertical_pos;
    let mut result = storage.vec(cell.paragraphs.len())?;
    for paragraph in &cell.paragraphs {
        let (mut line, _) = owned_cell_line(paragraph, styles, dpi).unwrap();
        line.vertical_pos = current_top;
        let Some(next) = current_top
            .checked_add(line.line_height)
            .and_then(|value| value.checked_add(line.line_spacing))
        else {
            return Ok(None);
        };
        current_top = next;
        let mut projected = storage.clone_paragraph(paragraph)?;
        projected.line_segs[0] = line;
        projected.source_line_seg_vertical_pos = None;
        result.push(projected);
    }
    Ok(Some(result))
}

fn project(
    paragraph: &Paragraph,
    cell: bool,
    hwpx_cells: bool,
    styles: &ResolvedStyleSet,
    dpi: f64,
) -> Option<Paragraph> {
    match project_with_storage(
        paragraph,
        cell,
        MetricScope::Render {
            hwpx_cells,
            owned_hwp5_cells: false,
        },
        styles,
        dpi,
        &crate::storage::Unbounded,
    ) {
        Ok(value) => value,
        Err(never) => match never {},
    }
}
#[derive(Clone, Copy)]
enum MetricScope {
    Render {
        hwpx_cells: bool,
        owned_hwp5_cells: bool,
    },
    ManagedCellsForExport,
}

fn project_with_storage<S: crate::storage::Storage>(
    paragraph: &Paragraph,
    cell: bool,
    scope: MetricScope,
    styles: &ResolvedStyleSet,
    dpi: f64,
    storage: &S,
) -> Result<Option<Paragraph>, S::Error> {
    let eligible = matches!(scope, MetricScope::Render { hwpx_cells, .. }
        if floating_text_host(paragraph) || (cell && hwpx_cells && paragraph.controls.is_empty()));
    let owned_hwp5_cells = matches!(
        scope,
        MetricScope::ManagedCellsForExport
            | MetricScope::Render {
                owned_hwp5_cells: true,
                ..
            }
    );
    let mut projected = eligible
        .then(|| projected_line(paragraph, styles, dpi))
        .flatten()
        .map(|line| {
            let mut result = storage.clone_paragraph(paragraph)?;
            result.line_segs[0] = line;
            Ok::<_, S::Error>(result)
        })
        .transpose()?;
    for (control_index, control) in paragraph.controls.iter().enumerate() {
        let Control::Table(table) = control else {
            continue;
        };
        for (cell_index, cell) in table.cells.iter().enumerate() {
            if cell.text_direction != 0 || cell.line_wrap != 0 {
                continue;
            }
            if owned_hwp5_cells {
                if let Some(paragraphs) =
                    owned_cell_with_current_metrics(cell, styles, dpi, storage)?
                {
                    if projected.is_none() {
                        projected = Some(storage.clone_paragraph(paragraph)?);
                    }
                    let Control::Table(table) =
                        &mut projected.as_mut().unwrap().controls[control_index]
                    else {
                        unreachable!()
                    };
                    table.cells[cell_index].paragraphs = paragraphs;
                    continue;
                }
            }
            for (paragraph_index, child) in cell.paragraphs.iter().enumerate() {
                if let Some(child) = project_with_storage(child, true, scope, styles, dpi, storage)?
                {
                    if projected.is_none() {
                        projected = Some(storage.clone_paragraph(paragraph)?);
                    }
                    let result = projected.as_mut().unwrap();
                    let Control::Table(table) = &mut result.controls[control_index] else {
                        unreachable!()
                    };
                    table.cells[cell_index].paragraphs[paragraph_index] = child;
                }
            }
        }
    }
    Ok(projected)
}

pub(crate) fn normalize(
    paragraphs: &[Paragraph],
    styles: &ResolvedStyleSet,
    dpi: f64,
    hwpx_cells: bool,
) -> Option<Vec<Paragraph>> {
    match normalize_with_storage(
        paragraphs,
        styles,
        dpi,
        hwpx_cells,
        false,
        &crate::storage::Unbounded,
    ) {
        Ok(value) => value,
        Err(never) => match never {},
    }
}
pub(crate) fn normalize_with_storage<S: crate::storage::Storage>(
    paragraphs: &[Paragraph],
    styles: &ResolvedStyleSet,
    dpi: f64,
    hwpx_cells: bool,
    owned_hwp5_cells: bool,
    storage: &S,
) -> Result<Option<Vec<Paragraph>>, S::Error> {
    normalize_in_scope(
        paragraphs,
        styles,
        dpi,
        MetricScope::Render {
            hwpx_cells,
            owned_hwp5_cells,
        },
        storage,
    )
}

/// 저장은 표시용 보정 전체가 아니라 같은 생산자의 관리 캐시만 발행한다.
pub(crate) fn managed_cells_for_export_with_storage<S: crate::storage::Storage>(
    paragraphs: &[Paragraph],
    styles: &ResolvedStyleSet,
    dpi: f64,
    storage: &S,
) -> Result<Option<Vec<Paragraph>>, S::Error> {
    normalize_in_scope(
        paragraphs,
        styles,
        dpi,
        MetricScope::ManagedCellsForExport,
        storage,
    )
}

fn normalize_in_scope<S: crate::storage::Storage>(
    paragraphs: &[Paragraph],
    styles: &ResolvedStyleSet,
    dpi: f64,
    scope: MetricScope,
    storage: &S,
) -> Result<Option<Vec<Paragraph>>, S::Error> {
    let mut result = None;
    for (index, paragraph) in paragraphs.iter().enumerate() {
        if let Some(projected) =
            project_with_storage(paragraph, false, scope, styles, dpi, storage)?
        {
            if result.is_none() {
                result = Some(
                    storage.try_collect(paragraphs.iter().map(|p| storage.clone_paragraph(p)))?,
                );
            }
            result.as_mut().unwrap()[index] = projected;
        }
    }
    Ok(result)
}
