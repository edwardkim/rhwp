//! 독립 한컴 Print의 본문 합성 기울임. 실제 italic face에는 중복 적용하지 않는다.
pub(crate) use super::text_transform::SYNTHETIC_ITALIC_SKEW as SKEW_X;
// SVG/PDF의 선언과 브라우저 paint의 명시적 합성은 각 소비 경로에서 구분한다.
pub(crate) const CSS_STYLE: &str = "oblique 18.418deg";
pub(crate) fn css_prefix(italic: bool) -> &'static str {
    if italic {
        "italic "
    } else {
        ""
    }
}
