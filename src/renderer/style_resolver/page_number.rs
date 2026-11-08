//! 자동 쪽번호 글자모양은 명시 스타일을 우선하고 목표 조판 세대를 따른다.

use crate::model::document::DocInfo;

/// 한컴 2020 Print는 명시 스타일이 없으면 기본 10pt 바탕을 쓴다.
/// 2024 Print는 같은 입력에서 바탕글의 글꼴·크기·색을 사용한다.
/// 쪽번호 제어를 담은 run의 글자모양은 어느 경로에서도 상속하지 않는다.
pub(super) fn resolve(doc_info: &DocInfo, hangul2024: bool) -> (Option<usize>, bool) {
    let style = doc_info
        .styles
        .iter()
        .find(|style| {
            style.local_name == "쪽 번호" || style.english_name.eq_ignore_ascii_case("Page Number")
        })
        .or_else(|| {
            if !hangul2024 {
                return None;
            }
            doc_info.styles.iter().find(|style| {
                style.local_name == "바탕글" || style.english_name.eq_ignore_ascii_case("Normal")
            })
        });
    let id = style.map(|style| style.char_shape_id as usize);
    let hft = id
        .and_then(|id| doc_info.char_shapes.get(id))
        .and_then(|shape| doc_info.font_faces.get(1)?.get(shape.font_ids[1] as usize))
        .is_some_and(|face| face.alt_type == 2);
    (id, hft)
}
