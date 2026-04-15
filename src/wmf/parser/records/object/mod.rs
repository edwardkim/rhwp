//! Implementation of the definitions in Section 2.3.4 of the WMF
//! specifications.

mod create_brush_indirect;
mod create_font_indirect;
mod create_palette;
mod create_pattern_brush;
mod create_pen_indirect;
mod create_region;
mod delete_object;
mod dib_create_pattern_brush;
mod select_clip_region;
mod select_object;
mod select_palette;

pub use self::{
    create_brush_indirect::*, create_font_indirect::*, create_palette::*, create_pattern_brush::*,
    create_pen_indirect::*, create_region::*, delete_object::*, dib_create_pattern_brush::*,
    select_clip_region::*, select_object::*, select_palette::*,
};
