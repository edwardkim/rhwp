//! DocInfo resource-table look-up helpers.
//!
//! This module is the **only** place in `hangulang` that imports rhwp types.
//! All other modules work exclusively with the crate's own Semantic IR.
//!
//! ## Binary-data 2-hop lookup
//!
//! `Picture.image_attr.bin_data_id` (a `u16`) is the **storage ID** of the
//! embedded binary stream inside the HWP CFB container.  The parser populates
//! `Document.bin_data_content` with entries whose `id` field equals that same
//! storage ID (`BinDataContent { id: bd.storage_id, data, extension }`).
//! Therefore the lookup is a direct linear scan over `bin_data_content` for an
//! entry whose `id` matches the requested `bin_data_id`.
//!
//! Source evidence:
//! - `src/parser/mod.rs:631,1137` — `id: bd.storage_id` when building content vec
//! - `src/parser/control/shape.rs:304` — comment: "bin_data_id ← DocInfo BinData 목록의 storage_id"

use crate::model::document::{DocInfo, Document};
use crate::model::style::{CharShape, Font, ParaShape};

/// Look up a `CharShape` by its zero-based index into `DocInfo.char_shapes`.
///
/// Returns `None` if `id` is out of range.
pub fn char_shape(doc_info: &DocInfo, id: u32) -> Option<&CharShape> {
    doc_info.char_shapes.get(id as usize)
}

/// Look up a `ParaShape` by its zero-based index into `DocInfo.para_shapes`.
///
/// Returns `None` if `id` is out of range.
pub fn para_shape(doc_info: &DocInfo, id: u16) -> Option<&ParaShape> {
    doc_info.para_shapes.get(id as usize)
}

/// Resolve a binary-data ID to its decoded byte slice and file extension.
///
/// `bin_data_id` is the value stored in `Picture.image_attr.bin_data_id`.  It
/// equals the CFB storage ID used as the `id` field in `BinDataContent`.
///
/// Returns `Some((Vec<u8>, &str))` — the raw (already-decompressed) image bytes
/// and the lowercase file extension (e.g. `"jpg"`, `"png"`) — or `None` if
/// the content was not found or the data slice is empty.
///
/// The bytes are returned owned because rhwp's
/// [`BinDataBytes`](crate::model::bin_data::BinDataBytes) may resolve them lazily
/// from the source container (`BinDataBytes::Lazy`), so there is no stable borrow
/// to hand back; `BinDataBytes::load` decompresses on demand.
pub fn bin_data_bytes(document: &Document, bin_data_id: u16) -> Option<(Vec<u8>, &str)> {
    if bin_data_id == 0 {
        return None;
    }
    document
        .bin_data_content
        .iter()
        .find(|c| c.id == bin_data_id && !c.data.is_empty())
        .map(|c| (c.data.load(), c.extension.as_str()))
}

/// Look up a font name by language index and font ID.
///
/// `DocInfo.font_faces` is a `Vec<Vec<Font>>` where the outer index is the
/// language slot (0=한글, 1=영어, 2=한자, 3=일어, 4=기타, 5=기호, 6=사용자)
/// and the inner index is the font ID referenced by `CharShape.font_ids[lang]`.
///
/// Returns `None` if either index is out of range.
pub fn font_name(doc_info: &DocInfo, lang_idx: usize, font_id: u16) -> Option<&str> {
    doc_info
        .font_faces
        .get(lang_idx)
        .and_then(|faces: &Vec<Font>| faces.get(font_id as usize))
        .map(|f| f.name.as_str())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::bin_data::{BinDataBytes, BinDataContent};
    use crate::model::document::DocInfo;
    use crate::model::style::{CharShape, Font, ParaShape};

    fn make_doc_info_with_char_shapes(shapes: Vec<CharShape>) -> DocInfo {
        DocInfo {
            char_shapes: shapes,
            ..Default::default()
        }
    }

    #[test]
    fn char_shape_found() {
        let cs = CharShape {
            bold: true,
            ..Default::default()
        };
        let doc_info = make_doc_info_with_char_shapes(vec![CharShape::default(), cs]);
        assert_eq!(char_shape(&doc_info, 1).map(|s| s.bold), Some(true));
    }

    #[test]
    fn char_shape_out_of_range() {
        let doc_info = make_doc_info_with_char_shapes(vec![]);
        assert!(char_shape(&doc_info, 0).is_none());
    }

    #[test]
    fn para_shape_found() {
        let doc_info = DocInfo {
            para_shapes: vec![ParaShape::default(), ParaShape::default()],
            ..Default::default()
        };
        assert!(para_shape(&doc_info, 1).is_some());
        assert!(para_shape(&doc_info, 2).is_none());
    }

    #[test]
    fn bin_data_bytes_found() {
        let doc = Document {
            bin_data_content: vec![
                BinDataContent {
                    id: 1,
                    data: BinDataBytes::Loaded(vec![0xFF, 0xD8]),
                    extension: "jpg".to_string(),
                },
                BinDataContent {
                    id: 2,
                    data: BinDataBytes::Loaded(vec![0x89, 0x50]),
                    extension: "png".to_string(),
                },
            ],
            ..Default::default()
        };
        let (bytes, ext) = bin_data_bytes(&doc, 2).unwrap();
        assert_eq!(ext, "png");
        assert_eq!(bytes, vec![0x89u8, 0x50]);
    }

    #[test]
    fn bin_data_bytes_zero_id() {
        let doc = Document::default();
        assert!(bin_data_bytes(&doc, 0).is_none());
    }

    #[test]
    fn bin_data_bytes_empty_data_skipped() {
        let doc = Document {
            bin_data_content: vec![BinDataContent {
                id: 3,
                data: BinDataBytes::Loaded(vec![]),
                extension: "png".to_string(),
            }],
            ..Default::default()
        };
        assert!(bin_data_bytes(&doc, 3).is_none());
    }

    #[test]
    fn font_name_found() {
        let font = Font {
            name: "Nanum Gothic".to_string(),
            ..Default::default()
        };
        let doc_info = DocInfo {
            font_faces: vec![vec![font]],
            ..Default::default()
        };
        assert_eq!(font_name(&doc_info, 0, 0), Some("Nanum Gothic"));
    }

    #[test]
    fn font_name_out_of_range() {
        let doc_info = DocInfo::default();
        assert!(font_name(&doc_info, 0, 0).is_none());
    }
}
