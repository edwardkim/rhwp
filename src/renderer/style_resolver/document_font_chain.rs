//! 원본 글꼴 레코드와 쉼표로 구분하는 내부 표시 체인의 경계를 보존한다.

use super::{storage, FontNameDecision};
use crate::model::style::Font;
use crate::renderer::font_environment::FontEnvironment;

/// 미해소된 원본 이름 안의 쉼표를 독립적인 paint 후보로 해석하지 않는다.
/// 같은 레코드의 유효한 TTF 대체를 측정과 paint의 공통 첫 face로 전달한다.
pub(super) fn prefer_declared_single_face<S: storage::Storage>(
    font: &Font,
    decision: &mut FontNameDecision,
    storage: &S,
) -> Result<(), S::Error> {
    if font.is_embedded
        || font.alt_type != 1
        || !font
            .subst_font
            .as_ref()
            .is_some_and(|substitute| substitute.font_type == 1 && !substitute.is_embedded)
        || !decision
            .normalized_face
            .as_deref()
            .is_some_and(|face| face.contains(','))
    {
        return Ok(());
    }
    let Some(substitute) = decision
        .subst_font
        .as_deref()
        .filter(|face| FontEnvironment::valid_name(face))
    else {
        return Ok(());
    };
    let normalized = storage.string(substitute)?;
    let displayed = storage.string(substitute)?;
    decision.normalized_face = Some(normalized);
    decision.css_family_chain.clear();
    storage.push(&mut decision.css_family_chain, displayed)?;
    Ok(())
}
