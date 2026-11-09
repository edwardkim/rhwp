//! 원본 언어 슬롯과 현재 표시 프로그램의 동일성을 메트릭 요청과 함께 보존한다.

use super::{storage, DocInfo, ResolvedStyleSet, LANG_COUNT};

fn same_advances(
    a: &crate::renderer::font_metrics_data::FontMetric,
    b: &crate::renderer::font_metrics_data::FontMetric,
) -> bool {
    std::ptr::eq(a, b)
        || (a.bold == b.bold
            && a.italic == b.italic
            && a.em_size == b.em_size
            && a.latin_ranges
                .iter()
                .map(|range| (range.start, range.widths))
                .eq(b
                    .latin_ranges
                    .iter()
                    .map(|range| (range.start, range.widths)))
            && match (a.hangul, b.hangul) {
                (None, None) => true,
                (Some(a), Some(b)) => {
                    (
                        a.cho_groups,
                        a.jung_groups,
                        a.jong_groups,
                        a.cho_map,
                        a.jung_map,
                        a.jong_map,
                        a.widths,
                    ) == (
                        b.cho_groups,
                        b.jung_groups,
                        b.jong_groups,
                        b.cho_map,
                        b.jung_map,
                        b.jong_map,
                        b.widths,
                    )
                }
                _ => false,
            })
}

/// 표시 family를 인증한 뒤에도 실제 측정이 원본의 검증된 전진을 소비하는지 확인한다.
/// 같은 전진이 서로 다른 이름의 정적 표로 생성된 별칭도 보존한다.
/// 단위·스타일·문자 범위·한글 조합표가 다른 표에는 신뢰를 넘기지 않는다.
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
        same_advances(source.metric, other.metric) && source.bold_fallback == other.bold_fallback
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
