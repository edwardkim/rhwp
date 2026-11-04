//! 저장 줄 없는 셀 문단의 인라인 표 크기를 현재 줄 구성에 전달한다.

use super::{ComposedParagraph, ResolvedStyleSet, SingleLineOverflowCache};
use crate::model::{control::Control, paragraph::Paragraph, table::Table};

pub(crate) fn table_cells_have_no_stored_lines(table: &Table) -> bool {
    !table.cells.is_empty()
        && table.cells.iter().all(|cell| {
            cell.paragraphs
                .iter()
                .all(crate::renderer::para_has_no_stored_line_segs)
        })
}

/// 측정과 배치는 같은 폭·개체 크기로 기존 프레임 조판기를 호출한다.
/// 원본 모델은 유지하고, 파생 프레임에 실측 개체 높이와 줄 소속을 함께 보존한다.
#[allow(clippy::too_many_arguments)]
pub(crate) fn recompose_cell_lines_with_nested_tables(
    composed: &mut ComposedParagraph,
    source: &Paragraph,
    width: f64,
    styles: &ResolvedStyleSet,
    dpi: f64,
    stored_geometry_policy: impl Into<crate::model::provenance::StoredGeometryPolicy>,
    native_hwp5: bool,
    cache: &SingleLineOverflowCache,
    mut nested_height: impl FnMut(&Table) -> f64,
) {
    match recompose_cell_lines_with_nested_tables_with_storage(
        composed,
        source,
        width,
        styles,
        dpi,
        stored_geometry_policy,
        native_hwp5,
        cache,
        |table| Ok(nested_height(table)),
        &crate::storage::Unbounded,
    ) {
        Ok(()) => (),
        Err(never) => match never {},
    }
}

pub(crate) fn recompose_cell_lines_with_nested_tables_with_storage<S: crate::storage::Storage>(
    composed: &mut ComposedParagraph,
    source: &Paragraph,
    width: f64,
    styles: &ResolvedStyleSet,
    dpi: f64,
    stored_geometry_policy: impl Into<crate::model::provenance::StoredGeometryPolicy>,
    native_hwp5: bool,
    cache: &SingleLineOverflowCache,
    mut nested_height: impl FnMut(&Table) -> Result<f64, S::Error>,
    storage: &S,
) -> Result<(), S::Error> {
    let stored_geometry_policy = stored_geometry_policy.into();
    let original = source;
    // 표의 물리 폭이 바뀌면 이전 폭으로 저장한 host 줄도 재사용할 수 없다.
    // 모델은 보존하고 같은 셀 폭에서 글자·개체 슬롯을 함께 다시 구성한다.
    let width_projection = (!stored_geometry_policy.preserve_hwp3_origins
        && source.controls.iter().any(|control| {
            matches!(control, Control::Table(table)
                if super::inline_table_width_cache_is_stale(table))
        }))
    .then(|| {
        let mut paragraph = storage.clone_paragraph(source)?;
        paragraph.line_segs.clear();
        Ok::<_, S::Error>(paragraph)
    })
    .transpose()?;
    let source = width_projection.as_ref().unwrap_or(source);
    let stored_projection = if !stored_geometry_policy.preserve_hwp3_origins
        && !crate::renderer::para_has_no_stored_line_segs(source)
        && source.controls.iter().any(|control| {
            matches!(control, Control::Table(table) if table.common.treat_as_char)
                || matches!(control, Control::Form(form) if form.common.treat_as_char)
        }) {
        super::inline_box_rows::normalized_host_with_table_height_with_storage(
            source,
            styles,
            |table, _| {
                let measured = nested_height(table)?;
                Ok((measured.is_finite() && measured > 0.0)
                    .then(|| crate::renderer::px_to_hwpunit(measured, dpi)))
            },
            storage,
        )?
    } else {
        None
    };
    let source = stored_projection.as_ref().unwrap_or(source);
    if stored_projection.is_some() {
        let mut refreshed =
            super::compose_paragraph_in_context_with_storage(source, styles, storage)?;
        super::preserve_context_resolved_runs_with_storage(composed, &mut refreshed, storage)?;
        *composed = refreshed;
    }
    let mut projected = None;
    let mut heights = Vec::new();
    let reflowed_table_host = (!native_hwp5 || width_projection.is_some())
        && !stored_geometry_policy.preserve_hwp3_origins
        && crate::renderer::para_has_no_stored_line_segs(source)
        && source.controls.iter().any(|control| {
            matches!(control, Control::Table(table)
                if super::inline_table_needs_reflow_measurement(table))
        });
    if reflowed_table_host {
        for (index, control) in source.controls.iter().enumerate() {
            let Control::Table(table) = control else {
                continue;
            };
            if !table.common.treat_as_char {
                continue;
            }
            // 같은 줄 구성에 참여하는 저장 줄 자식도 프레임 소유자다. 일부만
            // 발행하면 셀 컷은 형제를 따로 예약하고 paint는 같은 줄에 겹쳐 그린다.
            // 자식의 내부 저장 줄은 재조판하지 않고 본체 실측만 전달한다.
            let measured = nested_height(table)?;
            let Ok(height) = u32::try_from(crate::renderer::px_to_hwpunit(measured, dpi)) else {
                continue;
            };
            let height = if super::inline_table_needs_reflow_measurement(table) {
                height
            } else {
                height.max(table.common.height)
            };
            storage.push(&mut heights, (index, height))?;
        }
        if !heights.is_empty() {
            let mut paragraph = storage.clone_paragraph(source)?;
            // 구현이 합성한 줄은 새 개체 크기를 위한 독립적인 저장 근거가 아니다.
            paragraph.line_segs.clear();
            for &(index, height) in &heights {
                if let Control::Table(table) = &mut paragraph.controls[index] {
                    table.common.height = height;
                }
            }
            projected = Some(paragraph);
        }
    }
    if let Some(paragraph) = &projected {
        // 원본에 글자가 없어도 인라인 개체 슬롯은 실제 줄을 만든다.
        // 빈 문자열을 점유 없음으로 바꾸지 않고 같은 프레임 소유자에게 전달한다.
        if let Some(rebuilt) =
            super::recompose_stored_lines_in_frame_with_known_square_band_with_storage(
                composed,
                paragraph,
                super::ParagraphBox::content_for_style(
                    width,
                    styles.para_styles.get(paragraph.para_shape_id as usize),
                    dpi,
                ),
                width,
                styles,
                dpi,
                stored_geometry_policy,
                super::line_breaking::StoredRowMissPolicy::Reflow,
                &[],
                false,
                false,
                false,
                &heights,
                storage,
            )?
        {
            *composed = rebuilt;
        }
    } else {
        super::recompose_horizontal_cell_lines_for_width_with_storage(
            composed,
            source,
            width,
            styles,
            dpi,
            stored_geometry_policy,
            native_hwp5,
            cache,
            storage,
        )?;
    }
    if let Some(paragraph) = stored_projection {
        storage.bytes(std::mem::size_of::<super::frame_geometry::FrameGeometry>())?;
        storage.bytes(
            paragraph.line_segs.len()
                * (std::mem::size_of::<crate::model::paragraph::LineSeg>()
                    + std::mem::size_of::<(u32, super::SpaceMetric)>()),
        )?;
        let geometry = composed.frame_geometry.get_or_insert_with(|| {
            std::sync::Arc::new(super::frame_geometry::FrameGeometry {
                space_metrics: paragraph
                    .line_segs
                    .iter()
                    .enumerate()
                    .map(|(index, line)| (line.text_start, paragraph.line_space_metric(index)))
                    .collect(),
                line_segs: paragraph.line_segs.clone(),
                inline_table_heights: Vec::new(),
                stored_partition: true,
            })
        });
        let geometry = std::sync::Arc::make_mut(geometry);
        // 물리 높이의 갱신만으로 프레임이 새 줄 나눔을 소유한다고 선언하지 않는다.
        for (index, (old, new)) in original
            .controls
            .iter()
            .zip(&paragraph.controls)
            .enumerate()
        {
            if let (Control::Table(old), Control::Table(new)) = (old, new) {
                if old.common.height != new.common.height {
                    geometry
                        .inline_table_heights
                        .retain(|(owner, _)| *owner != index);
                    storage.push(
                        &mut geometry.inline_table_heights,
                        (index, new.common.height),
                    )?;
                }
            }
        }
    }
    Ok(())
}
