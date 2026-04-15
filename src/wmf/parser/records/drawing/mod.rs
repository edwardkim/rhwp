//! Implementation of the definitions in Section 2.3.3 of the WMF
//! specifications.

mod arc;
mod chord;
mod ellipse;
mod ext_flood_fill;
mod ext_text_out;
mod fill_region;
mod flood_fill;
mod frame_region;
mod invert_region;
mod line_to;
mod paint_region;
mod pat_blt;
mod pie;
mod poly_line;
mod poly_polygon;
mod polygon;
mod rectangle;
mod round_rect;
mod set_pixel;
mod text_out;

pub use self::{
    arc::*, chord::*, ellipse::*, ext_flood_fill::*, ext_text_out::*, fill_region::*,
    flood_fill::*, frame_region::*, invert_region::*, line_to::*, paint_region::*, pat_blt::*,
    pie::*, poly_line::*, poly_polygon::*, polygon::*, rectangle::*, round_rect::*, set_pixel::*,
    text_out::*,
};
