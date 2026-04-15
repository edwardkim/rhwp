use crate::wmf::{imports::*, parser::*};

#[derive(Clone, Debug, snafu::prelude::Snafu)]
pub enum PlayError {
    #[snafu(display("failed to generate: {cause}"))]
    FailedGenerate { cause: String },
    #[snafu(display("invalid brush: {cause}"))]
    InvalidBrush { cause: String },
    #[snafu(display("invalid record: {cause}"))]
    InvalidRecord { cause: String },
    #[snafu(display("unexpected graphics object: {cause}"))]
    UnexpectedGraphicsObject { cause: String },
    #[snafu(display("unknown: {cause}"))]
    Unknown { cause: String },
}

pub trait Player: Sized {
    /// Call after converting to write output.
    fn generate(self) -> Result<Vec<u8>, PlayError>;

    // .
    // .
    // Functions to handle Bitmap Record
    // .
    // .

    /// Render [`META_BITBLT`](crate::wmf::parser::META_BITBLT) record.
    fn bit_blt(self, record_number: usize, record: META_BITBLT) -> Result<Self, PlayError>;
    /// Render [`META_DIBBITBLT`](crate::wmf::parser::META_DIBBITBLT) record.
    fn device_independent_bitmap_bit_blt(
        self,
        record_number: usize,
        record: META_DIBBITBLT,
    ) -> Result<Self, PlayError>;
    /// Render [`META_DIBSTRETCHBLT`](crate::wmf::parser::META_DIBSTRETCHBLT) record.
    fn device_independent_bitmap_stretch_blt(
        self,
        record_number: usize,
        record: META_DIBSTRETCHBLT,
    ) -> Result<Self, PlayError>;
    /// Render [`META_SETDIBTODEV`](crate::wmf::parser::META_SETDIBTODEV) record.
    fn set_device_independent_bitmap_to_dev(
        self,
        record_number: usize,
        record: META_SETDIBTODEV,
    ) -> Result<Self, PlayError>;
    /// Render [`META_STRETCHBLT`](crate::wmf::parser::META_STRETCHBLT) record.
    fn stretch_blt(self, record_number: usize, record: META_STRETCHBLT) -> Result<Self, PlayError>;
    /// Render [`META_STRETCHDIB`](crate::wmf::parser::META_STRETCHDIB) record.
    fn stretch_device_independent_bitmap(
        self,
        record_number: usize,
        record: META_STRETCHDIB,
    ) -> Result<Self, PlayError>;

    // .
    // .
    // Functions to handle Control Record
    // .
    // .

    /// Render [`META_EOF`](crate::wmf::parser::META_EOF) record.
    fn eof(self, record_number: usize, record: META_EOF) -> Result<Self, PlayError>;
    /// Render [`MetafileHeader`](crate::wmf::parser::MetafileHeader) record.
    fn header(self, record_number: usize, header: MetafileHeader) -> Result<Self, PlayError>;

    // .
    // .
    // Functions to handle Drawing Record
    // .
    // .

    /// Render [`META_ARC`](crate::wmf::parser::META_ARC) record.
    fn arc(self, record_number: usize, record: META_ARC) -> Result<Self, PlayError>;
    /// Render [`META_CHORD`](crate::wmf::parser::META_CHORD) record.
    fn chord(self, record_number: usize, record: META_CHORD) -> Result<Self, PlayError>;
    /// Render [`META_ELLIPSE`](crate::wmf::parser::META_ELLIPSE) record.
    fn ellipse(self, record_number: usize, record: META_ELLIPSE) -> Result<Self, PlayError>;
    /// Render [`META_EXTFLOODFILL`](crate::wmf::parser::META_EXTFLOODFILL) record.
    fn ext_flood_fill(
        self,
        record_number: usize,
        record: META_EXTFLOODFILL,
    ) -> Result<Self, PlayError>;
    /// Render [`META_EXTTEXTOUT`](crate::wmf::parser::META_EXTTEXTOUT) record.
    fn ext_text_out(self, record_number: usize, record: META_EXTTEXTOUT)
        -> Result<Self, PlayError>;
    /// Render [`META_FILLREGION`](crate::wmf::parser::META_FILLREGION) record.
    fn fill_region(self, record_number: usize, record: META_FILLREGION) -> Result<Self, PlayError>;
    /// Render [`META_FLOODFILL`](crate::wmf::parser::META_FLOODFILL) record.
    fn flood_fill(self, record_number: usize, record: META_FLOODFILL) -> Result<Self, PlayError>;
    /// Render [`META_FRAMEREGION`](crate::wmf::parser::META_FRAMEREGION) record.
    fn frame_region(
        self,
        record_number: usize,
        record: META_FRAMEREGION,
    ) -> Result<Self, PlayError>;
    /// Render [`META_INVERTREGION`](crate::wmf::parser::META_INVERTREGION) record.
    fn invert_region(
        self,
        record_number: usize,
        record: META_INVERTREGION,
    ) -> Result<Self, PlayError>;
    /// Render [`META_LINETO`](crate::wmf::parser::META_LINETO) record.
    fn line_to(self, record_number: usize, record: META_LINETO) -> Result<Self, PlayError>;
    /// Render [`META_PAINTREGION`](crate::wmf::parser::META_PAINTREGION) record.
    fn paint_region(
        self,
        record_number: usize,
        record: META_PAINTREGION,
    ) -> Result<Self, PlayError>;
    /// Render [`META_PATBLT`](crate::wmf::parser::META_PATBLT) record.
    fn pat_blt(self, record_number: usize, record: META_PATBLT) -> Result<Self, PlayError>;
    /// Render [`META_PIE`](crate::wmf::parser::META_PIE) record.
    fn pie(self, record_number: usize, record: META_PIE) -> Result<Self, PlayError>;
    /// Render [`META_POLYLINE`](crate::wmf::parser::META_POLYLINE) record.
    fn polyline(self, record_number: usize, record: META_POLYLINE) -> Result<Self, PlayError>;
    /// Render [`META_POLYGON`](crate::wmf::parser::META_POLYGON) record.
    fn polygon(self, record_number: usize, record: META_POLYGON) -> Result<Self, PlayError>;
    /// Render [`META_POLYPOLYGON`](crate::wmf::parser::META_POLYPOLYGON) record.
    fn poly_polygon(
        self,
        record_number: usize,
        record: META_POLYPOLYGON,
    ) -> Result<Self, PlayError>;
    /// Render [`META_RECTANGLE`](crate::wmf::parser::META_RECTANGLE) record.
    fn rectangle(self, record_number: usize, record: META_RECTANGLE) -> Result<Self, PlayError>;
    /// Render [`META_ROUNDRECT`](crate::wmf::parser::META_ROUNDRECT) record.
    fn round_rect(self, record_number: usize, record: META_ROUNDRECT) -> Result<Self, PlayError>;
    /// Render [`META_SETPIXEL`](crate::wmf::parser::META_SETPIXEL) record.
    fn set_pixel(self, record_number: usize, record: META_SETPIXEL) -> Result<Self, PlayError>;
    /// Render [`META_TEXTOUT`](crate::wmf::parser::META_TEXTOUT) record.
    fn text_out(
        self,
        record_number: usize,
        record: META_TEXTOUT,
    ) -> Result<Self, crate::wmf::converter::PlayError>;

    // .
    // .
    // Functions to handle Object Record
    // .
    // .

    /// Render [`META_CREATEBRUSHINDIRECT`](crate::wmf::parser::META_CREATEBRUSHINDIRECT) record.
    fn create_brush_indirect(
        self,
        record_number: usize,
        record: META_CREATEBRUSHINDIRECT,
    ) -> Result<Self, PlayError>;
    /// Render [`META_CREATEFONTINDIRECT`](crate::wmf::parser::META_CREATEFONTINDIRECT) record.
    fn create_font_indirect(
        self,
        record_number: usize,
        record: META_CREATEFONTINDIRECT,
    ) -> Result<Self, PlayError>;
    /// Render [`META_CREATEPALETTE`](crate::wmf::parser::META_CREATEPALETTE) record.
    fn create_palette(
        self,
        record_number: usize,
        record: META_CREATEPALETTE,
    ) -> Result<Self, PlayError>;
    /// Render [`META_CREATEPATTERNBRUSH`](crate::wmf::parser::META_CREATEPATTERNBRUSH) record.
    fn create_pattern_brush(
        self,
        record_number: usize,
        record: META_CREATEPATTERNBRUSH,
    ) -> Result<Self, PlayError>;
    /// Render [`META_CREATEPENINDIRECT`](crate::wmf::parser::META_CREATEPENINDIRECT)
    /// record.
    fn create_pen_indirect(
        self,
        record_number: usize,
        record: META_CREATEPENINDIRECT,
    ) -> Result<Self, PlayError>;
    /// Render [`META_CREATEREGION`](crate::wmf::parser::META_CREATEREGION) record.
    fn create_region(
        self,
        record_number: usize,
        record: META_CREATEREGION,
    ) -> Result<Self, crate::wmf::converter::PlayError>;
    /// Render [`META_DELETEOBJECT`](crate::wmf::parser::META_DELETEOBJECT) record.
    fn delete_object(
        self,
        record_number: usize,
        record: META_DELETEOBJECT,
    ) -> Result<Self, PlayError>;
    /// Render [`META_DIBCREATEPATTERNBRUSH`](crate::wmf::parser::META_DIBCREATEPATTERNBRUSH) record.
    fn create_device_independent_bitmap_pattern_brush(
        self,
        record_number: usize,
        record: META_DIBCREATEPATTERNBRUSH,
    ) -> Result<Self, PlayError>;
    /// Render [`META_SELECTCLIPREGION`](crate::wmf::parser::META_SELECTCLIPREGION)
    /// record.
    fn select_clip_region(
        self,
        record_number: usize,
        record: META_SELECTCLIPREGION,
    ) -> Result<Self, PlayError>;
    /// Render [`META_SELECTOBJECT`](crate::wmf::parser::META_SELECTOBJECT) record.
    fn select_object(
        self,
        record_number: usize,
        record: META_SELECTOBJECT,
    ) -> Result<Self, PlayError>;
    /// Render [`META_SELECTPALETTE`](crate::wmf::parser::META_SELECTPALETTE) record.
    fn select_palette(
        self,
        record_number: usize,
        record: META_SELECTPALETTE,
    ) -> Result<Self, PlayError>;

    // .
    // .
    // Functions to handle State Record
    // .
    // .

    /// Render [`META_ANIMATEPALETTE`](crate::wmf::parser::META_ANIMATEPALETTE)
    /// record.
    fn animate_palette(
        self,
        record_number: usize,
        record: META_ANIMATEPALETTE,
    ) -> Result<Self, PlayError>;
    /// Render [`META_EXCLUDECLIPRECT`](crate::wmf::parser::META_EXCLUDECLIPRECT)
    /// record.
    fn exclude_clip_rect(
        self,
        record_number: usize,
        record: META_EXCLUDECLIPRECT,
    ) -> Result<Self, PlayError>;
    /// Render [`META_INTERSECTCLIPRECT`](crate::wmf::parser::META_INTERSECTCLIPRECT)
    /// record.
    fn intersect_clip_rect(
        self,
        record_number: usize,
        record: META_INTERSECTCLIPRECT,
    ) -> Result<Self, PlayError>;
    /// Render [`META_MOVETO`](crate::wmf::parser::META_MOVETO) record.
    fn move_to(self, record_number: usize, record: META_MOVETO) -> Result<Self, PlayError>;
    /// Render [`META_OFFSETCLIPRGN`](crate::wmf::parser::META_OFFSETCLIPRGN) record.
    fn offset_clip_region(
        self,
        record_number: usize,
        record: META_OFFSETCLIPRGN,
    ) -> Result<Self, PlayError>;
    /// Render [`META_OFFSETVIEWPORTORG`](crate::wmf::parser::META_OFFSETVIEWPORTORG)
    /// record.
    fn offset_viewport_origin(
        self,
        record_number: usize,
        record: META_OFFSETVIEWPORTORG,
    ) -> Result<Self, PlayError>;
    /// Render [`META_OFFSETWINDOWORG`](crate::wmf::parser::META_OFFSETWINDOWORG)
    /// record.
    fn offset_window_origin(
        self,
        record_number: usize,
        record: META_OFFSETWINDOWORG,
    ) -> Result<Self, PlayError>;
    /// Render [`META_REALIZEPALETTE`](crate::wmf::parser::META_REALIZEPALETTE)
    /// record.
    fn realize_palette(
        self,
        record_number: usize,
        record: META_REALIZEPALETTE,
    ) -> Result<Self, PlayError>;
    /// Render [`META_RESIZEPALETTE`](crate::wmf::parser::META_RESIZEPALETTE) record.
    fn resize_palette(
        self,
        record_number: usize,
        record: META_RESIZEPALETTE,
    ) -> Result<Self, PlayError>;
    /// Render [`META_RESTOREDC`](crate::wmf::parser::META_RESTOREDC) record.
    fn restore_device_context(
        self,
        record_number: usize,
        record: META_RESTOREDC,
    ) -> Result<Self, PlayError>;
    /// Render [`META_SAVEDC`](crate::wmf::parser::META_SAVEDC) record.
    fn save_device_context(
        self,
        record_number: usize,
        record: META_SAVEDC,
    ) -> Result<Self, PlayError>;
    /// Render [`META_SCALEVIEWPORTEXT`](crate::wmf::parser::META_SCALEVIEWPORTEXT)
    /// record.
    fn scale_viewport_ext(
        self,
        record_number: usize,
        record: META_SCALEVIEWPORTEXT,
    ) -> Result<Self, PlayError>;
    /// Render [`META_SCALEWINDOWEXT`](crate::wmf::parser::META_SCALEWINDOWEXT)
    /// record.
    fn scale_window_ext(
        self,
        record_number: usize,
        record: META_SCALEWINDOWEXT,
    ) -> Result<Self, PlayError>;
    /// Render [`META_SETBKCOLOR`](crate::wmf::parser::META_SETBKCOLOR) record.
    fn set_bk_color(self, record_number: usize, record: META_SETBKCOLOR)
        -> Result<Self, PlayError>;
    /// Render [`META_SETBKMODE`](crate::wmf::parser::META_SETBKMODE) record.
    fn set_bk_mode(self, record_number: usize, record: META_SETBKMODE) -> Result<Self, PlayError>;
    /// Render [`META_SETLAYOUT`](crate::wmf::parser::META_SETLAYOUT) record.
    fn set_layout(self, record_number: usize, record: META_SETLAYOUT) -> Result<Self, PlayError>;
    /// Render [`META_SETMAPMODE`](crate::wmf::parser::META_SETMAPMODE) record.
    fn set_map_mode(self, record_number: usize, record: META_SETMAPMODE)
        -> Result<Self, PlayError>;
    /// Render [`META_SETMAPPERFLAGS`](crate::wmf::parser::META_SETMAPPERFLAGS)
    /// record.
    fn set_mapper_flags(
        self,
        record_number: usize,
        record: META_SETMAPPERFLAGS,
    ) -> Result<Self, PlayError>;
    /// Render [`META_SETPALENTRIES`](crate::wmf::parser::META_SETPALENTRIES) record.
    fn set_pal_entries(
        self,
        record_number: usize,
        record: META_SETPALENTRIES,
    ) -> Result<Self, PlayError>;
    /// Render [`META_SETPOLYFILLMODE`](crate::wmf::parser::META_SETPOLYFILLMODE)
    /// record.
    fn set_polyfill_mode(
        self,
        record_number: usize,
        record: META_SETPOLYFILLMODE,
    ) -> Result<Self, PlayError>;
    /// Render [`META_SETRELABS`](crate::wmf::parser::META_SETRELABS) record.
    fn set_relabs(self, record_number: usize, record: META_SETRELABS) -> Result<Self, PlayError>;
    /// Render [`META_SETROP2`](crate::wmf::parser::META_SETROP2) record.
    fn set_raster_operation(
        self,
        record_number: usize,
        record: META_SETROP2,
    ) -> Result<Self, PlayError>;
    /// Render [`META_SETSTRETCHBLTMODE`](crate::wmf::parser::META_SETSTRETCHBLTMODE)
    /// record.
    fn set_stretch_blt_mode(
        self,
        record_number: usize,
        record: META_SETSTRETCHBLTMODE,
    ) -> Result<Self, PlayError>;
    /// Render [`META_SETTEXTALIGN`](crate::wmf::parser::META_SETTEXTALIGN) record.
    fn set_text_align(
        self,
        record_number: usize,
        record: META_SETTEXTALIGN,
    ) -> Result<Self, PlayError>;
    /// Render [`META_SETTEXTCHAREXTRA`](crate::wmf::parser::META_SETTEXTCHAREXTRA)
    /// record.
    fn set_text_char_extra(
        self,
        record_number: usize,
        record: META_SETTEXTCHAREXTRA,
    ) -> Result<Self, PlayError>;
    /// Render [`META_SETTEXTCOLOR`](crate::wmf::parser::META_SETTEXTCOLOR) record.
    fn set_text_color(
        self,
        record_number: usize,
        record: META_SETTEXTCOLOR,
    ) -> Result<Self, PlayError>;
    /// Render [`META_SETTEXTJUSTIFICATION`](crate::wmf::parser::META_SETTEXTJUSTIFICATION) record.
    fn set_text_justification(
        self,
        record_number: usize,
        record: META_SETTEXTJUSTIFICATION,
    ) -> Result<Self, PlayError>;
    /// Render [`META_SETVIEWPORTEXT`](crate::wmf::parser::META_SETVIEWPORTEXT)
    /// record.
    fn set_viewport_ext(
        self,
        record_number: usize,
        record: META_SETVIEWPORTEXT,
    ) -> Result<Self, PlayError>;
    /// Render [`META_SETVIEWPORTORG`](crate::wmf::parser::META_SETVIEWPORTORG)
    /// record.
    fn set_viewport_origin(
        self,
        record_number: usize,
        record: META_SETVIEWPORTORG,
    ) -> Result<Self, PlayError>;
    /// Render [`META_SETWINDOWEXT`](crate::wmf::parser::META_SETWINDOWEXT) record.
    fn set_window_ext(
        self,
        record_number: usize,
        record: META_SETWINDOWEXT,
    ) -> Result<Self, PlayError>;
    /// Render [`META_SETWINDOWORG`](crate::wmf::parser::META_SETWINDOWORG) record.
    fn set_window_origin(
        self,
        record_number: usize,
        record: META_SETWINDOWORG,
    ) -> Result<Self, PlayError>;

    // .
    // .
    // Functions to handle Escape Record
    // .
    // .

    /// Render [`META_ESCAPE`](crate::wmf::parser::META_ESCAPE) record.
    fn escape(self, record_number: usize, record: META_ESCAPE) -> Result<Self, PlayError>;
}
