//! 원본 언어 슬롯과 현재 표시 프로그램의 동일성을 메트릭 요청과 함께 보존한다.

use super::{storage, DocInfo, ResolvedStyleSet, LANG_COUNT};

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::style::{CharShape, Font};
    use crate::renderer::style_resolver::ResolvedCharStyle;

    #[test]
    fn another_sources_display_family_does_not_authorize_original_truetype() {
        let names = ["RHWP Original Program A", "RHWP Original Program B"];
        let info = DocInfo {
            font_faces: (0..LANG_COUNT)
                .map(|_| {
                    names
                        .iter()
                        .map(|name| Font {
                            name: (*name).into(),
                            ..Default::default()
                        })
                        .collect()
                })
                .collect(),
            char_shapes: (0..2)
                .map(|id| CharShape {
                    font_ids: [id; LANG_COUNT],
                    ..Default::default()
                })
                .collect(),
            ..Default::default()
        };
        let mut styles = ResolvedStyleSet {
            char_styles: (0..2)
                .map(|_| ResolvedCharStyle {
                    font_family: names[0].into(),
                    font_families: vec![names[0].into(); LANG_COUNT],
                    ..Default::default()
                })
                .collect(),
            ..Default::default()
        };
        refresh(&info, &mut styles, &storage::Unbounded).unwrap();
        for language in 0..LANG_COUNT {
            assert!(styles.requests_original_truetype(0, language));
            assert!(
                !styles.requests_original_truetype(1, language),
                "같은 표시 family도 원본이 다른 슬롯의 자연 전진을 인증하지 않는다"
            );
        }
    }
}
