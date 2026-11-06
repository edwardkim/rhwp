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
