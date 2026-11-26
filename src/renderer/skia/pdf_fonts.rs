//! OpenType defines GSUB major version 1. HarfBuzz skips version 0 during shaping,
//! but its subsetter can reject the entire font for it. Omit that table in PDF output.
//! The file on disk, the selected glyph IDs and all embeddability flags are preserved.

use super::cache::BoundedCache;
use crate::renderer::font_paths::{MAX_NATIVE_CUSTOM_FONT_BYTES, MAX_NATIVE_CUSTOM_FONT_FILES};
use crate::renderer::text_font_presentation::{copy_outline_tables, MAX_FONT_BYTES, MAX_TABLES};
use skia_safe::{Data, FontMgr, Typeface};
use std::cell::RefCell;
use ttf_parser::{Permissions, Tag};

pub(super) struct PdfFontCache(RefCell<BoundedCache<Option<Typeface>, u32>>);

impl PdfFontCache {
    pub fn new() -> Self {
        Self(RefCell::new(BoundedCache::new(
            MAX_NATIVE_CUSTOM_FONT_FILES,
            MAX_NATIVE_CUSTOM_FONT_BYTES as usize,
        )))
    }

    pub fn for_embedding(&self, font_mgr: &FontMgr, original: &Typeface) -> Option<Typeface> {
        let id = original.unique_id();
        let mut cache = self.0.borrow_mut();
        if let Some(face) = cache.get(id) {
            return face;
        }
        let prepared = prepare(font_mgr, original);
        let size = prepared.as_ref().map_or(0, |(_, len)| *len);
        let face = prepared.map(|(face, _)| face);
        cache.insert(id, face.clone(), size);
        face
    }
}

pub(super) fn for_output(
    cache: Option<&PdfFontCache>,
    font_mgr: &FontMgr,
    face: Typeface,
) -> Typeface {
    cache
        .and_then(|cache| cache.for_embedding(font_mgr, &face))
        .unwrap_or(face)
}

fn prepare(font_mgr: &FontMgr, original: &Typeface) -> Option<(Typeface, usize)> {
    let mut header = [0u8; 4];
    if original.get_table_data(u32::from_be_bytes(*b"GSUB"), &mut header) != header.len()
        || header[..2] != [0, 0]
        || original.is_synthetic_bold()
        || original.is_synthetic_oblique()
    {
        return None;
    }
    // Read only the selected face, including nonzero TTC indices. Avoid materializing
    // an entire collection just to copy one face or reject an oversized font.
    if original.count_tables() > MAX_TABLES {
        return None;
    }
    let mut tags: Vec<_> = original.read_table_tags()?.into_iter().map(Tag).collect();
    tags.sort_unstable();
    if !tags.contains(&Tag::from_bytes(b"glyf")) || tags.contains(&Tag::from_bytes(b"fvar")) {
        return None;
    }
    let read_table = |tag: Tag| {
        (original.get_table_size(tag.0)? <= MAX_FONT_BYTES)
            .then(|| original.copy_table_data(tag.0))
            .flatten()
    };
    let rights = read_table(Tag::from_bytes(b"OS/2"))?;
    let os2 = ttf_parser::os2::Table::parse(rights.as_bytes())?;
    let fs_type = u16::from_be_bytes(rights.as_bytes().get(8..10)?.try_into().ok()?);
    // Honor even legacy programs which set these flags on OS/2 version 0 or 1.
    if fs_type & 0x0302 != 0
        || !matches!(
            os2.permissions()?,
            Permissions::Installable | Permissions::Editable | Permissions::PreviewAndPrint
        )
    {
        return None;
    }
    let program = copy_outline_tables(tags, false, &[*b"GSUB"], read_table)?;
    ttf_parser::Face::parse(&program, 0).ok()?.tables().glyf?;
    let size = program.len();
    let face = font_mgr.new_from_data(Data::new_with_owner(program), 0)?;
    (face.font_style() == original.font_style()).then_some((face, size))
}
