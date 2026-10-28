//! 실제 인라인 상자를 담지 못하는 단일 저장 줄의 공통 프레임 투영.
use crate::model::{
    control::Control, paragraph::Paragraph, provenance::LayoutCompatibilityProfile, table::Table,
};
use crate::renderer::{
    height_measurer::HeightMeasurer, px_to_hwpunit, style_resolver::ResolvedStyleSet,
};

/// 원본의 문자·개체 주소와 가로 배치는 보존한다. 줄 높이만으로 원본의 재줄바꿈을
/// 추측하지 않으며, 여러 줄·별도 캡션·고정 줄간격은 각자의 기존 소유자에게 둔다.
pub(crate) fn normalized_host(
    source: &Paragraph,
    styles: &ResolvedStyleSet,
    dpi: f64,
    profile: LayoutCompatibilityProfile,
) -> Option<Paragraph> {
    normalized_host_with_table_height(source, styles, |table, index| {
        let measured = HeightMeasurer::new(dpi)
            .with_stored_geometry_policy(profile.stored_geometry_policy())
            .with_native_hwp5(profile.native_hwp5_layout())
            .with_hwpx_stored_layout(profile.hwpx_stored_layout())
            .with_session_edited(profile.session_edited())
            .measure_table(table, 0, index, styles);
        (measured.total_height.is_finite() && measured.total_height > 0.0)
            .then(|| px_to_hwpunit(measured.total_height, dpi))
    })
}

/// 셀의 호출자는 같은 깊이·너비·호환 정책으로 이미 사용하는 표 측정기를 전달한다.
pub(super) fn normalized_host_with_table_height(
    source: &Paragraph,
    styles: &ResolvedStyleSet,
    mut measure: impl FnMut(&Table, usize) -> Option<i32>,
) -> Option<Paragraph> {
    match normalized_host_with_table_height_with_storage(
        source,
        styles,
        |table, index| Ok(measure(table, index)),
        &crate::storage::Unbounded,
    ) {
        Ok(value) => value,
        Err(never) => match never {},
    }
}
pub(crate) fn normalized_host_with_table_height_with_storage<S: crate::storage::Storage>(
    source: &Paragraph,
    styles: &ResolvedStyleSet,
    mut measure: impl FnMut(&Table, usize) -> Result<Option<i32>, S::Error>,
    storage: &S,
) -> Result<Option<Paragraph>, S::Error> {
    crate::storage::with_optional(|failure| {
        let [stored] = source.line_segs.as_slice() else {
            return None;
        };
        let style = styles.para_styles.get(source.para_shape_id as usize)?;
        if source.stored_text_partition_is_dirty()
            || source.text.contains(['\n', '\r', '\t'])
            || stored.line_height <= 0
            || stored.text_height != stored.line_height
            || stored.baseline_distance <= 0
            || stored.baseline_distance > stored.line_height
            || style.line_spacing_type != crate::model::style::LineSpacingType::Percent
            || style.vertical_alignment != 0
        {
            return None;
        }
        let mut owner = None;
        for (index, control) in source.controls.iter().enumerate() {
            if super::control_is_width_neutral_marker(control) {
                continue;
            }
            if owner.replace((index, control)).is_some() {
                return None;
            }
        }
        let (index, control) = owner?;
        let mut table_height = None;
        let height = match control {
            Control::Table(table) if table.common.treat_as_char && table.caption.is_none() => {
                let margins = i32::from(table.outer_margin_top)
                    .checked_add(i32::from(table.outer_margin_bottom))?;
                let declared = i32::try_from(table.common.height).ok()?;
                // 측정 전체를 항상 반복하지 않는다. 활성 서식보다 작은 빈 em 캐시가
                // 있거나, 선언한 상자 자체도 못 담는 줄에서만 같은 표 측정기를 쓴다.
                let stale_empty = !table.common.size_protect
                    && table.cells.iter().any(|cell| {
                        cell.paragraphs.iter().any(|paragraph| {
                            paragraph.text.is_empty()
                                && paragraph.controls.is_empty()
                                && crate::storage::optional(
                                    super::compose_paragraph_in_context_with_storage(
                                        paragraph, styles, storage,
                                    ),
                                    failure,
                                )
                                .is_none_or(|composed| composed.frame_geometry.is_some())
                        })
                    });
                let outside_row = declared.checked_add(margins)? > stored.line_height;
                let height = if !table.common.size_protect && (stale_empty || outside_row) {
                    let height =
                        crate::storage::optional(measure(table, index), failure)??.max(declared);
                    table_height = Some(u32::try_from(height).ok()?);
                    height
                } else {
                    declared
                };
                height.checked_add(margins)?
            }
            Control::Shape(shape) if shape.common().treat_as_char => {
                if crate::renderer::layout::shape_caption_for_layout(shape).is_some() {
                    return None;
                }
                // 저장 줄이 담아야 할 것은 실제 그리기에 전달하는 Common 프레임이다.
                // flow_height_hu의 흐름 상한에는 내부 그리기 좌표의 current_height도
                // 포함된다. 그 상한으로 유효한 저장 줄을 늘리면 좌표계만 큰 도형이
                // 뒤 표와 본문을 다음 쪽으로 밀어낸다.
                i32::try_from(shape.common().height)
                    .ok()?
                    .checked_add(i32::from(shape.common().margin.top))?
                    .checked_add(i32::from(shape.common().margin.bottom))?
            }
            Control::Picture(picture)
                if picture.common.treat_as_char && picture.caption.is_none() =>
            {
                super::inline_picture_size_hu(picture)
                    .1
                    .checked_add(i32::from(picture.common.margin.top))?
                    .checked_add(i32::from(picture.common.margin.bottom))?
            }
            Control::Form(form) if form.common.treat_as_char => i32::try_from(form.height)
                .ok()?
                .checked_add(i32::from(form.common.margin.top))?
                .checked_add(i32::from(form.common.margin.bottom))?,
            _ => return None,
        };
        if height <= stored.line_height {
            return None;
        }
        let mut projected = crate::storage::optional(storage.clone_paragraph(source), failure)?;
        // 일반 재조판도 사용하는 같은 줄 상자 생산자다. paint 전용 좌표 보정이나
        // clamp로 개체만 옮기지 않고 후속 문단도 이 프레임의 점유를 소비한다.
        super::line_breaking::apply_inline_control_line_height(&mut projected.line_segs[0], height);
        if let (Some(height), Control::Table(table)) =
            (table_height, &mut projected.controls[index])
        {
            table.common.height = height;
        }
        projected.source_line_seg_vertical_pos = None;
        Some(projected)
    })
}
