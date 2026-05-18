//! Implementation of the definitions in Section 2.2.2 of the WMF
//! specifications.

mod bitmap16;
mod bitmap_info_header;
mod ciexyz;
mod ciexyz_triple;
mod color_ref;
mod device_independent_bitmap;
mod log_brush;
mod log_color_space;
mod log_color_space_w;
mod palette_entry;
mod pitch_and_family;
mod point_l;
mod point_s;
mod poly_polygon;
mod rect;
mod rect_l;
mod rgb_quad;
mod rgb_triple;
mod scan;
mod size_l;

pub use self::{
    bitmap16::*, bitmap_info_header::*, ciexyz::*, ciexyz_triple::*, color_ref::*,
    device_independent_bitmap::*, log_brush::*, log_color_space::*, log_color_space_w::*,
    palette_entry::*, pitch_and_family::*, point_l::*, point_s::*, poly_polygon::*, rect::*,
    rect_l::*, rgb_quad::*, rgb_triple::*, scan::*, size_l::*,
};
use crate::wmf::imports::*;

/// Convert UTF16-LE bytes to String.
fn utf16le_bytes_to_string(bytes: &[u8]) -> Result<String, crate::wmf::parser::ParseError> {
    if !bytes.len().is_multiple_of(2) {
        return Err(crate::wmf::parser::ParseError::UnexpectedPattern {
            cause: "Byte array length must be even".to_owned(),
        });
    }

    let u16_vec = bytes
        .chunks_exact(2)
        .map(|chunk| u16::from_le_bytes(chunk.try_into().expect("should be converted")))
        .collect::<Vec<_>>();

    String::from_utf16(&u16_vec).map_err(|err| crate::wmf::parser::ParseError::UnexpectedPattern {
        cause: err.to_string(),
    })
}
