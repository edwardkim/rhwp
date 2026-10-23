//! 한컴 Print로 확인한 HWPX 결측 언어 슬롯의 기본 조회 문맥.

use std::borrow::Cow;

use crate::model::{
    document::{DocInfo, Document},
    style::Font,
};

/// 명시한 다른 언어의 face를 비어 있는 영문 슬롯으로 복제하지 않는다.
/// 원본 0번 참조의 기본값만 제공하며 원본 IR과 직렬화는 변경하지 않는다.
pub(crate) fn missing_latin_default(document: &Document) -> Option<&'static str> {
    (document.layout_profile().hwpx_container()
        && document
            .doc_info
            .font_faces
            .get(1)
            .is_some_and(Vec::is_empty)
        && document
            .doc_info
            .char_shapes
            .iter()
            .any(|shape| shape.font_ids[1] == 0))
    .then_some("한컴바탕")
}

pub(super) fn lookup_context(document: &Document) -> Cow<'_, DocInfo> {
    let Some(name) = missing_latin_default(document) else {
        return Cow::Borrowed(&document.doc_info);
    };
    let mut info = document.doc_info.clone();
    info.font_faces[1].push(Font {
        name: name.to_string(),
        alt_type: 1,
        default_name: Some("Haansoft Batang".to_string()),
        ..Default::default()
    });
    Cow::Owned(info)
}
