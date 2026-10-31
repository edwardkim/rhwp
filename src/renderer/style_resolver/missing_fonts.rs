//! 한컴 Print로 확인한 HWPX 결측 언어 슬롯의 기본 조회 문맥.

use std::borrow::Cow;

use crate::model::{
    document::{DocInfo, Document},
    style::{CharShape, Font},
};

/// 글자모양 표가 없는 HWPX는 0번 run 참조도 해소할 수 없다. 한컴 Print는
/// 이를 10pt 기본 글자로 복구한다. 원본 IR에 속성을 추가하지 않는다.
pub(crate) fn missing_char_table(document: &Document) -> bool {
    document.layout_profile().hwpx_container() && document.doc_info.char_shapes.is_empty()
}

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
    match try_lookup_context(document, &super::storage::Unbounded) {
        Ok(lookup) => lookup,
        Err(never) => match never {},
    }
}

pub(super) fn try_lookup_context<'a, S: super::storage::Storage>(
    document: &'a Document,
    storage: &S,
) -> Result<Cow<'a, DocInfo>, S::Error> {
    if missing_char_table(document) {
        let mut info = storage.clone_doc_info(&document.doc_info)?;
        storage.push(
            &mut info.char_shapes,
            CharShape {
                base_size: 1000,
                ..Default::default()
            },
        )?;
        let extra = 7usize.saturating_sub(info.font_faces.len());
        storage.reserve(&mut info.font_faces, extra)?;
        info.font_faces.resize_with(7, Vec::new);
        for fonts in &mut info.font_faces {
            if fonts.is_empty() {
                // 결측 표의 최초 Print는 Gulim을 사용한다. Save가 나중에
                // 기록한 한컴바탕 선언으로 원래 Print를 바꾸지 않는다.
                storage.push(
                    fonts,
                    Font {
                        name: storage.string("굴림")?,
                        alt_type: 1,
                        default_name: Some(storage.string("Gulim")?),
                        ..Default::default()
                    },
                )?;
            }
        }
        return Ok(Cow::Owned(info));
    }
    let Some(name) = missing_latin_default(document) else {
        return Ok(Cow::Borrowed(&document.doc_info));
    };
    let mut info = storage.clone_doc_info(&document.doc_info)?;
    storage.reserve(&mut info.font_faces[1], 1)?;
    info.font_faces[1].push(Font {
        name: storage.string(name)?,
        alt_type: 1,
        default_name: Some(storage.string("Haansoft Batang")?),
        ..Default::default()
    });
    Ok(Cow::Owned(info))
}
