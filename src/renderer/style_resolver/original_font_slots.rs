//! 원본 언어 슬롯과 현재 표시 프로그램의 동일성을 메트릭 요청과 함께 보존한다.

use super::{storage, DocInfo, ResolvedStyleSet, LANG_COUNT};

/// 표시 family를 인증한 뒤에도 실제 측정이 원본의 검증된 표를 소비하는지 확인한다.
/// family 별칭에 별도 메트릭 키가 없어도 원본 표는 유지할 수 있지만,
/// 다른 메트릭 표가 있는 대체 프로그램에 원본의 신뢰를 넘기지 않는다.
pub(super) fn keeps_source_metric_table(
    source: &str,
    current_metric_face: &str,
    target: &str,
    bold: bool,
    italic: bool,
) -> bool {
    use crate::renderer::font_metrics_data::find_metric;
    let same = |source: &crate::renderer::font_metrics_data::MetricMatch,
                other: &crate::renderer::font_metrics_data::MetricMatch| {
        std::ptr::eq(source.metric, other.metric) && source.bold_fallback == other.bold_fallback
    };
    let (Some(source), Some(current)) = (
        find_metric(source, bold, italic),
        find_metric(current_metric_face, bold, italic),
    ) else {
        return false;
    };
    same(&source, &current)
        && find_metric(target, bold, italic).is_none_or(|target| same(&source, &target))
}

pub(super) fn refresh<S: storage::Storage>(
    doc_info: &DocInfo,
    styles: &mut ResolvedStyleSet,
    storage: &S,
) -> Result<(), S::Error> {
    let mut slots = storage.vec(styles.char_styles.len())?;
    for (style_id, style) in styles.char_styles.iter().enumerate() {
        let shape = doc_info.char_shapes.get(style_id);
        let row = std::array::from_fn::<_, LANG_COUNT, _>(|language| {
            let Some(source) = shape.and_then(|shape| {
                doc_info
                    .font_faces
                    .get(language)?
                    .get(shape.font_ids[language] as usize)
            }) else {
                return false;
            };
            let primary = style
                .font_family_for_lang(language)
                .split(',')
                .next()
                .unwrap_or("")
                .trim();
            let source = source.name.trim();
            !primary.is_empty()
                && (source.eq_ignore_ascii_case(primary)
                    || crate::renderer::installed_font_program_aliases(source)
                        .iter()
                        .any(|alias| alias.eq_ignore_ascii_case(primary)))
        });
        slots.push(row);
    }
    styles.unsubstituted_font_slots = slots;
    Ok(())
}
