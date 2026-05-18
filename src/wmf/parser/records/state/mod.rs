//! Implementation of the definitions in Section 2.3.5 of the WMF
//! specifications.

mod animate_palette;
mod exclude_clip_rect;
mod intersect_clip_rect;
mod move_to;
mod offset_clip_rgn;
mod offset_viewport_org;
mod offset_window_org;
mod realize_palette;
mod resize_palette;
mod restore_dc;
mod save_dc;
mod scale_viewport_ext;
mod scale_window_ext;
mod set_bk_color;
mod set_bk_mode;
mod set_layout;
mod set_map_mode;
mod set_mapper_flags;
mod set_pal_entries;
mod set_polyfill_mode;
mod set_relabs;
mod set_rop2;
mod set_stretch_blt_mode;
mod set_text_align;
mod set_text_char_extra;
mod set_text_color;
mod set_text_justification;
mod set_viewport_ext;
mod set_viewport_org;
mod set_window_ext;
mod set_window_org;

pub use self::{
    animate_palette::*, exclude_clip_rect::*, intersect_clip_rect::*, move_to::*,
    offset_clip_rgn::*, offset_viewport_org::*, offset_window_org::*, realize_palette::*,
    resize_palette::*, restore_dc::*, save_dc::*, scale_viewport_ext::*, scale_window_ext::*,
    set_bk_color::*, set_bk_mode::*, set_layout::*, set_map_mode::*, set_mapper_flags::*,
    set_pal_entries::*, set_polyfill_mode::*, set_relabs::*, set_rop2::*, set_stretch_blt_mode::*,
    set_text_align::*, set_text_char_extra::*, set_text_color::*, set_text_justification::*,
    set_viewport_ext::*, set_viewport_org::*, set_window_ext::*, set_window_org::*,
};
