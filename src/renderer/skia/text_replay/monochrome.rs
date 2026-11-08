//! 일반 텍스트만 기본 윤곽 사본으로 열며 strict 컬러 자원은 관여하지 않는다.

use skia_safe::{font_arguments::VariationPosition, Data, FontArguments, FontMgr, Typeface};
use std::{cell::RefCell, collections::HashMap};

const MAX_FACES: usize = 256;
const MAX_BYTES: usize = 64 * 1024 * 1024;

#[derive(Default)]
struct Cache {
    faces: HashMap<u32, Option<Typeface>>,
    bytes: usize,
}

thread_local! {
    static FACES: RefCell<Cache> = RefCell::new(Cache::default());
}

pub(super) fn text_typeface(font_mgr: &FontMgr, original: &Typeface) -> Option<Typeface> {
    let id = original.unique_id();
    FACES.with(|cache| {
        let mut cache = cache.borrow_mut();
        if let Some(face) = cache.faces.get(&id) {
            return face.clone();
        }
        let prepared = prepare(font_mgr, original);
        let bytes = prepared.as_ref().map_or(0, |(_, bytes)| *bytes);
        if cache.faces.len() >= MAX_FACES || cache.bytes.saturating_add(bytes) > MAX_BYTES {
            cache.faces.clear();
            cache.bytes = 0;
        }
        let face = prepared.map(|(face, _)| face);
        cache.bytes += bytes;
        cache.faces.insert(id, face.clone());
        face
    })
}

fn prepare(font_mgr: &FontMgr, original: &Typeface) -> Option<(Typeface, usize)> {
    original.get_table_size(u32::from_be_bytes(*b"COLR"))?;
    let (bytes, index) = original.to_existing_font_data()?;
    let program = crate::renderer::text_font_presentation::ordinary_text_font_program(
        bytes.as_bytes(),
        u32::try_from(index).ok()?,
    )?;
    let retained = program.len();
    let mut face = font_mgr.new_from_data(Data::new_copy(&program), Some(0))?;
    // GSUB/GPOS와 같은 가변 인스턴스도 원래 선택을 유지한다.
    let coordinates = original.variation_design_position().unwrap_or_default();
    if !coordinates.is_empty() {
        let mut arguments = FontArguments::new().set_variation_design_position(VariationPosition {
            coordinates: &coordinates,
        });
        arguments.set_collection_index(0);
        face = face.clone_with_arguments(&arguments)?;
    }
    Some((face, retained))
}
