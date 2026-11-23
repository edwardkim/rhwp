//! 검증된 원본 TTF의 줄 소유와 대체 프로그램의 저장 줄 재현을 분리한다.

use super::*;

#[derive(Debug, Clone)]
pub(crate) enum SourceProgramRowOwner {
    /// 유형을 명시한 HWPX TTF와 변경 추적 0x08이 없는 현재 프로그램 조판.
    DeclaredTrueType,
    /// 유형 미상의 HWP5 등은 대체 프로그램이 저장 줄 전체를 재현해야 한다.
    StoredFallback(Box<ResolvedCharStyle>),
    /// 언어 슬롯에 실제 대체 프로그램과 그 자연 전진을 명시했다.
    ScopedNaturalReplacement,
}

pub(super) fn refresh_with_storage<S: storage::Storage>(
    doc_info: &DocInfo,
    styles: &mut ResolvedStyleSet,
    declared_hwpx_ttf: bool,
    environment: Option<&super::super::font_environment::FontEnvironment>,
    storage: &S,
) -> Result<(), S::Error> {
    let trace = std::env::var("RHWP_DIAG_SOURCE_PROGRAM").ok();
    styles.source_program_projection_slots = Default::default();
    // #8639의 양방향 한컴 Print와 구판/현재 HWPX의 독립 0→8 Print 대조:
    // 0x08이면 같은 가로 상자의 저장 줄을 유지한다. 대체 선언의 유무가
    // HWPX의 명시적 TTF를 이 정책에서 제외하는 조건은 아니다.
    let keep_stored_rows = doc_info.track_change_flags() & TRACK_CHANGE_KEEPS_STORED_ROWS != 0;
    if keep_stored_rows && !environment.is_some_and(|env| env.has_scoped_natural_replacements()) {
        return Ok(());
    }
    let dpi = styles.resolved_dpi.unwrap_or(96.0);
    let mut fallback_char_styles = None;
    let build_fallback = || -> Result<Vec<ResolvedCharStyle>, S::Error> {
        let mut fallback_info = storage.clone_doc_info(doc_info)?;
        for fonts in &mut fallback_info.font_faces {
            for font in fonts {
                if let Some(substitute) = font.subst_font.take() {
                    if substitute.font_type == 1 && !substitute.face.trim().is_empty() {
                        font.name = substitute.face;
                    }
                }
            }
        }
        storage.try_collect(
            fallback_info
                .char_shapes
                .iter()
                .map(|shape| try_resolve_single_char_style(shape, &fallback_info, dpi, storage)),
        )
    };
    for (shape_id, style) in styles.char_styles.iter().enumerate() {
        let Some(shape) = doc_info.char_shapes.get(shape_id) else {
            continue;
        };
        for language in 0..LANG_COUNT {
            let Some(font) = doc_info
                .font_faces
                .get(language)
                .and_then(|fonts| fonts.get(shape.font_ids[language] as usize))
            else {
                continue;
            };
            let primary = primary_font_name(style.font_family_for_lang(language));
            if !font.is_embedded
                && style.font_families_natural_program_metrics[language]
                && environment.is_some_and(|env| {
                    env.has_scoped_natural_replacement(
                        &font.name,
                        language,
                        shape.font_ids[language],
                    )
                })
            {
                storage.insert_map(
                    std::sync::Arc::make_mut(&mut styles.source_program_projection_slots),
                    (shape_id as u32, language),
                    SourceProgramRowOwner::ScopedNaturalReplacement,
                )?;
                continue;
            }
            if trace
                .as_ref()
                .is_some_and(|filter| filter == "all" || font.name.contains(filter))
            {
                eprintln!("SOURCE_PROGRAM shape={shape_id} lang={language} declared_hwpx_ttf={declared_hwpx_ttf} source={:?} primary={primary:?} alt={} subst={:?} metric={:?} original_glyph={:?} trusted={} verified={} source_ttf={}", font.name, font.alt_type, font.subst_font.as_ref().map(|subst| (subst.font_type, &subst.face)), style.metric_face_for_lang(language), style.original_glyph_metric_face_for_lang(language), style.font_metric_trusted_for_lang(language), super::super::layout::verified_source_projection_face(primary, style.bold, style.italic), style.source_ttf_for_lang(language));
            }
            // 원본을 열 때의 캐시 보존은 명시적으로 바꾼 슬롯의 자연 전진
            // 선택과 별도다. 나머지 슬롯은 변경 추적의 원래 정책을 따른다.
            if keep_stored_rows
                || font.alt_type != 1
                || font.is_embedded
                || font.native_program.is_some()
                || (primary != font.name.trim()
                    && !super::super::installed_font_program_aliases(font.name.trim())
                        .contains(&primary))
                || style.metric_face_for_lang(language).is_some()
                || style
                    .original_glyph_metric_face_for_lang(language)
                    .is_some()
                || !(style.font_metric_trusted_for_lang(language)
                    || super::super::layout::verified_source_projection_face(
                        primary,
                        style.bold,
                        style.italic,
                    ))
            {
                continue;
            }
            let owner = if declared_hwpx_ttf && style.source_ttf_for_lang(language) {
                // 명시적 유형이 없는 FACE_NAME 하위 비트를 이 소유권으로 승격하지 않는다.
                SourceProgramRowOwner::DeclaredTrueType
            } else {
                if !font.subst_font.as_ref().is_some_and(|substitute| {
                    substitute.font_type == 1
                        && !substitute.face.trim().is_empty()
                        && substitute.face.trim() != primary
                }) {
                    continue;
                }
                if fallback_char_styles.is_none() {
                    fallback_char_styles = Some(build_fallback()?);
                }
                let Some(fallback) = fallback_char_styles
                    .as_ref()
                    .and_then(|values| values.get(shape_id))
                else {
                    continue;
                };
                storage.bytes(std::mem::size_of::<ResolvedCharStyle>())?;
                SourceProgramRowOwner::StoredFallback(Box::new(crate::storage::try_clone(
                    fallback, storage,
                )?))
            };
            storage.insert_map(
                std::sync::Arc::make_mut(&mut styles.source_program_projection_slots),
                (shape_id as u32, language),
                owner,
            )?;
        }
    }
    Ok(())
}
