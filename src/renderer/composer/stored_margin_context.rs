//! 구판 HWP5→HWPX가 다른 여백 단위로 옮긴 문단의 저장 줄 문맥.

use super::{line_breaking, ParagraphBox, ResolvedStyleSet};
use crate::model::paragraph::{ColumnBreakType, LineSeg, Paragraph};

/// 구판 writer가 캐시에 사용한 이전 저장 단위를 공통 여백 해석기로 복원한다.
/// px 값을 배로 늘리면 하위 단위 비트가 CHAR인 음수 값의 의미가 달라진다.
pub(crate) fn previous_margin_styles(
    doc_info: &crate::model::document::DocInfo,
    current: &ResolvedStyleSet,
    dpi: f64,
) -> Option<ResolvedStyleSet> {
    match previous_margin_styles_with_storage(doc_info, current, dpi, &crate::storage::Unbounded) {
        Ok(value) => value,
        Err(never) => match never {},
    }
}
pub(crate) fn previous_margin_styles_with_storage<S: crate::storage::Storage>(
    doc_info: &crate::model::document::DocInfo,
    current: &ResolvedStyleSet,
    dpi: f64,
    storage: &S,
) -> Result<Option<ResolvedStyleSet>, S::Error> {
    crate::storage::with_optional(|failure| {
        if doc_info.para_shapes.len() != current.para_styles.len() {
            return None;
        }
        let mut previous =
            crate::storage::optional(crate::storage::try_clone(current, storage), failure)?;
        for (raw, style) in doc_info.para_shapes.iter().zip(&mut previous.para_styles) {
            let px = |value: i32| {
                value.checked_mul(2).map(|value| {
                    crate::renderer::style_resolver::paragraph_margin_hwpunit(value, doc_info) * dpi
                        / crate::renderer::HWPUNIT_PER_INCH
                })
            };
            style.margin_left = px(raw.margin_left)?;
            style.margin_right = px(raw.margin_right)?;
            style.indent = px(raw.indent)?;
        }
        Some(previous)
    })
}

fn current_rows(
    source: &Paragraph,
    width: f64,
    styles: &ResolvedStyleSet,
    dpi: f64,
    top: i32,
    body: bool,
) -> Option<(Vec<LineSeg>, Vec<(u32, super::SpaceMetric)>)> {
    match current_rows_with_storage(
        source,
        width,
        styles,
        dpi,
        top,
        body,
        &crate::storage::Unbounded,
    ) {
        Ok(value) => value,
        Err(never) => match never {},
    }
}
fn current_rows_with_storage<S: crate::storage::Storage>(
    source: &Paragraph,
    width: f64,
    styles: &ResolvedStyleSet,
    dpi: f64,
    top: i32,
    body: bool,
    storage: &S,
) -> Result<Option<(Vec<LineSeg>, Vec<(u32, super::SpaceMetric)>)>, S::Error> {
    crate::storage::with_optional(|failure| {
        let mut input = crate::storage::optional(storage.clone_paragraph(source), failure)?;
        input.line_segs.clear();
        let style = styles.para_styles.get(source.para_shape_id as usize);
        let box_ = if body {
            ParagraphBox::body_for_style(width, style, dpi)
        } else {
            ParagraphBox::content_for_style(width, style, dpi)
        };
        let mut frame = box_.frame(top);
        let rows = crate::storage::optional(
            line_breaking::layout_paragraph_in_frame_with_storage(
                &input, &mut frame, styles, dpi, storage,
            ),
            failure,
        )??;
        Some((
            rows,
            crate::storage::optional(frame.try_project_space_metrics_since(0, storage), failure)?,
        ))
    })
}

/// 호출자가 구판 marker·XML 여백 단위를 확인한 뒤에만 사용하는 문맥 인증이다.
/// 현재 폭에 더 적은 줄이 들어간다는 사실만으로 정상 저장 캐시를 버리지 않는다.
/// 이전 2배 여백으로 구성한 모든 줄 경계가 원본과 정확히 같아야 한다.
fn has_previous_margin_partition(
    source: &Paragraph,
    width: f64,
    styles: &ResolvedStyleSet,
    previous: &ResolvedStyleSet,
    dpi: f64,
    body: bool,
) -> bool {
    match has_previous_margin_partition_with_storage(
        source,
        width,
        styles,
        previous,
        dpi,
        body,
        &crate::storage::Unbounded,
    ) {
        Ok(value) => value,
        Err(never) => match never {},
    }
}
fn has_previous_margin_partition_with_storage<S: crate::storage::Storage>(
    source: &Paragraph,
    width: f64,
    styles: &ResolvedStyleSet,
    previous: &ResolvedStyleSet,
    dpi: f64,
    body: bool,
    storage: &S,
) -> Result<bool, S::Error> {
    let Some(style) = styles.para_styles.get(source.para_shape_id as usize) else {
        return Ok(false);
    };
    if style.indent == 0.0 && style.margin_left == 0.0 && style.margin_right == 0.0 {
        return Ok(false);
    }
    let starts = |rows: &[LineSeg]| storage.collect(rows.iter().map(|line| line.text_start));
    let source_starts = storage
        .collect((0..source.line_segs.len()).map(|index| source.line_seg_text_start(index)))?;
    let Some((current, _)) =
        current_rows_with_storage(source, width, styles, dpi, 0, body, storage)?
    else {
        return Ok(false);
    };
    if starts(&current)? == source_starts {
        return Ok(false);
    }
    let previous_style = previous.para_styles.get(source.para_shape_id as usize);
    let previous_box = if body {
        ParagraphBox::body_for_style(width, previous_style, dpi)
    } else {
        ParagraphBox::content_for_style(width, previous_style, dpi)
    };
    if !line_breaking::stored_rows_reproduce_frame_expectation_with_storage(
        source,
        &mut previous_box.frame(0),
        previous,
        dpi,
        storage,
    )? {
        return Ok(false);
    }
    match current_rows_with_storage(source, width, previous, dpi, 0, body, storage)? {
        Some((rows, _)) => Ok(starts(&rows)? == source_starts),
        None => Ok(false),
    }
}

fn is_plain_original(para: &Paragraph, styles: &ResolvedStyleSet) -> bool {
    para.controls.is_empty()
        && !para.text.is_empty()
        && !para.text.contains(['\n', '\t'])
        && para.column_type == ColumnBreakType::None
        && !para.stored_text_partition_is_dirty()
        && !para.cell_format_vpos_dirty
        && !para.line_segs.is_empty()
        && para.line_segs.iter().all(|line| {
            line.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0
                && line.is_first_segment()
                && line.is_last_segment()
        })
        && styles
            .para_styles
            .get(para.para_shape_id as usize)
            .is_some_and(|style| {
                !style.page_break_before
                    && style.head_type == crate::model::style::HeadType::None
                    && style.spacing_before == 0.0
                    && style.spacing_after == 0.0
            })
}

/// 단 경계·어울림을 별도로 소유하지 않는 본문도 같은 문맥 인증을 사용한다.
pub(crate) fn project_body(
    source: &Paragraph,
    width: f64,
    styles: &ResolvedStyleSet,
    previous: &ResolvedStyleSet,
    dpi: f64,
) -> Option<Paragraph> {
    match project_body_with_storage(
        source,
        width,
        styles,
        previous,
        dpi,
        &crate::storage::Unbounded,
    ) {
        Ok(value) => value,
        Err(never) => match never {},
    }
}
pub(crate) fn project_body_with_storage<S: crate::storage::Storage>(
    source: &Paragraph,
    width: f64,
    styles: &ResolvedStyleSet,
    previous: &ResolvedStyleSet,
    dpi: f64,
    storage: &S,
) -> Result<Option<Paragraph>, S::Error> {
    crate::storage::with_optional(|failure| {
        if !is_plain_original(source, styles)
            || super::stored_rows_cross_physical_frame(source)
            || !crate::storage::optional(
                has_previous_margin_partition_with_storage(
                    source, width, styles, previous, dpi, true, storage,
                ),
                failure,
            )?
        {
            return None;
        }
        let (rows, spaces) = crate::storage::optional(
            current_rows_with_storage(
                source,
                width,
                styles,
                dpi,
                source.line_segs.first()?.vertical_pos,
                true,
                storage,
            ),
            failure,
        )??;
        let mut paragraph = crate::storage::optional(storage.clone_paragraph(source), failure)?;
        paragraph.replace_line_segs_with_space_metrics(rows, spaces);
        Some(paragraph)
    })
}

/// 바뀐 가로 문맥의 줄과 뒤 문단의 세로 원점을 같은 셀 흐름에서 발행한다.
/// 개체·명시적 쪽/단·문단 간격의 별도 소유가 있으면 이 단순 흐름은 맡지 않는다.
pub(crate) fn project_cell(
    original: &[Paragraph],
    width: f64,
    styles: &ResolvedStyleSet,
    previous: &ResolvedStyleSet,
    dpi: f64,
) -> Option<Vec<Paragraph>> {
    match project_cell_with_storage(
        original,
        width,
        styles,
        previous,
        dpi,
        &crate::storage::Unbounded,
    ) {
        Ok(value) => value,
        Err(never) => match never {},
    }
}
pub(crate) fn project_cell_with_storage<S: crate::storage::Storage>(
    original: &[Paragraph],
    width: f64,
    styles: &ResolvedStyleSet,
    previous: &ResolvedStyleSet,
    dpi: f64,
    storage: &S,
) -> Result<Option<Vec<Paragraph>>, S::Error> {
    crate::storage::with_optional(|failure| {
        if original.first()?.line_segs.first()?.vertical_pos != 0
            || original.iter().any(|para| !is_plain_original(para, styles))
            || !original.iter().any(|para| {
                crate::storage::optional(
                    has_previous_margin_partition_with_storage(
                        para, width, styles, previous, dpi, false, storage,
                    ),
                    failure,
                )
                .unwrap_or(true)
            })
        {
            return None;
        }
        let mut cursor = 0_i32;
        let mut result = crate::storage::optional(storage.vec(original.len()), failure)?;
        for source in original {
            let (rows, spaces) = crate::storage::optional(
                current_rows_with_storage(source, width, styles, dpi, cursor, false, storage),
                failure,
            )??;
            let last = rows.last()?;
            cursor = last
                .vertical_pos
                .checked_add(last.line_height)?
                .checked_add(last.line_spacing)?;
            let mut paragraph = crate::storage::optional(storage.clone_paragraph(source), failure)?;
            paragraph.replace_line_segs_with_space_metrics(rows, spaces);
            crate::storage::optional(storage.push(&mut result, paragraph), failure)?;
        }
        Some(result)
    })
}
