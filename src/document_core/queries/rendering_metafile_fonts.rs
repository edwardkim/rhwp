//! WMF가 본문에서 쓰지 않은 내장 family를 요청해도 실제 프로그램을 공급한다.

use crate::{model::document::Document, paint::metafile_fonts::MetafileFonts};

pub(super) fn with_document_fonts(base: &MetafileFonts, document: &Document) -> MetafileFonts {
    let mut fonts = base.clone();
    let mut names = Vec::new();
    for face in document.doc_info.font_faces.iter().flatten() {
        if face.is_embedded {
            if let Some(id) = face.resolved_bin_data_id {
                names.push((face.name.as_str(), face.name.as_str(), id));
            }
        }
        if let Some(substitute) = face.subst_font.as_ref().filter(|s| s.is_embedded) {
            if let Some(id) = substitute.resolved_bin_data_id {
                names.push((face.name.as_str(), substitute.face.as_str(), id));
            }
        }
    }
    let ids = names.iter().map(|(_, _, id)| *id).collect::<Vec<_>>();
    let bytes = super::load_bounded_embedded_font_bytes(
        &document.bin_data_content,
        &ids,
        super::MAX_EMBEDDED_FONT_BYTES,
        super::MAX_EMBEDDED_FONT_BYTES_PER_PAGE,
    );
    for (alias, family, id) in names {
        let Some(data) = bytes.get(&id) else {
            continue;
        };
        let Some(index) = crate::paint::resolve_embedded_font_face_index(data, family, None) else {
            continue;
        };
        let _ = fonts.register(alias, data, index);
        if alias != family {
            let _ = fonts.register(family, data, index);
        }
    }
    fonts
}
