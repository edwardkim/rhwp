/// The RecordType Enumeration defines the types of records that can be used in
/// WMF metafiles.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, strum::FromRepr, strum::EnumIter)]
#[repr(u16)]
pub enum RecordType {
    /// This record specifies the end of the file, the last record in the
    /// metafile.
    META_EOF = 0x0000,
    /// This record maps entries from the logical palette that is defined in
    /// the playback device context to the system palette.
    META_REALIZEPALETTE = 0x0035,
    /// This record defines red green blue (RGB) color values in a range of
    /// entries in the logical palette that is defined in the playback device
    /// context.
    META_SETPALENTRIES = 0x0037,
    /// This record defines the background raster operation mix mode in the
    /// playback device context. The background mix mode is the mode for
    /// combining pens, text, hatched brushes, and interiors of filled objects
    /// with background colors on the output surface.
    META_SETBKMODE = 0x0102,
    /// This record defines the mapping mode in the playback device context.
    /// The mapping mode defines the unit of measure used to transform
    /// page-space coordinates into coordinates of the output device, and also
    /// defines the orientation of the device's x and y axes.
    META_SETMAPMODE = 0x0103,
    /// This record defines the foreground raster operation mix mode in the
    /// playback device context. The foreground mix mode is the mode for
    /// combining pens and interiors of filled objects with foreground colors
    /// on the output surface.
    META_SETROP2 = 0x0104,
    /// This record is undefined and MUST be ignored.
    META_SETRELABS = 0x0105,
    /// This record defines polygon fill mode in the playback device context
    /// for graphics operations that fill polygons.
    META_SETPOLYFILLMODE = 0x0106,
    /// This record defines the bitmap stretching mode in the playback device
    /// context.
    META_SETSTRETCHBLTMODE = 0x0107,
    /// This record defines inter-character spacing for text justification in
    /// the playback device context. Spacing is added to the white space
    /// between each character, including break characters, when a line of
    /// justified text is output.
    META_SETTEXTCHAREXTRA = 0x0108,
    /// This record restores the playback device context from a previously
    /// saved device context.
    META_RESTOREDC = 0x0127,
    /// This record redefines the size of the logical palette that is defined
    /// in the playback device context.
    META_RESIZEPALETTE = 0x0139,
    /// This record defines a brush with a pattern specified by a
    /// device-independent bitmap (DIB).
    META_DIBCREATEPATTERNBRUSH = 0x0142,
    /// This record defines the layout orientation in the playback device
    /// context. (Windows NT 3.1, Windows NT 3.51, Windows 95, Windows NT 4.0,
    /// Windows 98, and Windows Millennium Edition: This record type is not
    /// supported.)
    META_SETLAYOUT = 0x0149,
    /// This record sets the background color in the playback device context to
    /// a specified color, or to the nearest physical color if the device
    /// cannot represent the specified color.
    META_SETBKCOLOR = 0x0201,
    /// This record defines the text color in the playback device context.
    META_SETTEXTCOLOR = 0x0209,
    /// This record moves the viewport origin in the playback device context by
    /// using specified horizontal and vertical offsets.
    META_OFFSETVIEWPORTORG = 0x0211,
    /// This record draws a line from the output position that is defined in
    /// the playback device context up to, but not including, a specified
    /// point.
    META_LINETO = 0x0213,
    /// This record sets the output position in the playback device context to
    /// a specified point.
    META_MOVETO = 0x0214,
    /// This record moves the clipping region that is defined in the playback
    /// device context by specified offsets.
    META_OFFSETCLIPRGN = 0x0220,
    /// This record fills a region by using a specified brush.
    META_FILLREGION = 0x0228,
    /// This record defines the algorithm that the font mapper uses when it
    /// maps logical fonts to physical fonts.
    META_SETMAPPERFLAGS = 0x0231,
    /// This record specifies the logical palette in the playback device
    /// context.
    META_SELECTPALETTE = 0x0234,
    /// This record paints a polygon consisting of two or more vertices
    /// connected by straight lines. The polygon is outlined by using the pen
    /// and filled by using the brush and polygon fill mode; these are defined
    /// in the playback device context.
    META_POLYGON = 0x0324,
    /// This record draws a series of line segments by connecting the points in
    /// a specified array.
    META_POLYLINE = 0x0325,
    /// This record defines the amount of space to add to break characters in a
    /// string of justified text.
    META_SETTEXTJUSTIFICATION = 0x020A,
    /// This record defines the output window origin in the playback device
    /// context.
    META_SETWINDOWORG = 0x020B,
    /// This record defines the horizontal and vertical extents of the output
    /// window in the playback device context.
    META_SETWINDOWEXT = 0x020C,
    /// This record defines the viewport origin in the playback device context.
    META_SETVIEWPORTORG = 0x020D,
    /// This record defines the horizontal and vertical extents of the viewport
    /// in the playback device context.
    META_SETVIEWPORTEXT = 0x020E,
    /// This record moves the output window origin in the playback device
    /// context by using specified horizontal and vertical offsets.
    META_OFFSETWINDOWORG = 0x020F,
    /// This record scales the horizontal and vertical extents of the output
    /// window that is defined in the playback device context by using the
    /// ratios formed by specified multiplicands and divisors.
    META_SCALEWINDOWEXT = 0x0410,
    /// This record scales the horizontal and vertical extents of the viewport
    /// that is defined in the playback device context by using the ratios
    /// formed by specified multiplicands and divisors.
    META_SCALEVIEWPORTEXT = 0x0412,
    /// This record sets the clipping region that is defined in the playback
    /// device context to the existing clipping region minus a specified
    /// rectangle.
    META_EXCLUDECLIPRECT = 0x0415,
    /// This record sets the clipping region that is defined in the playback
    /// device context to the intersection of the existing clipping region and
    /// a specified rectangle.
    META_INTERSECTCLIPRECT = 0x0416,
    /// This record defines an ellipse. The center of the ellipse is the center
    /// of a specified bounding rectangle. The ellipse is outlined by using the
    /// pen and is filled by using the brush; these are defined in the playback
    /// device context.
    META_ELLIPSE = 0x0418,
    /// This record fills an area of the display surface with the brush that is
    /// defined in the playback device context.
    META_FLOODFILL = 0x0419,
    /// This record defines a border around a specified region by using a
    /// specified brush.
    META_FRAMEREGION = 0x0429,
    /// This record redefines entries in the logical palette that is defined in
    /// the playback device context.
    META_ANIMATEPALETTE = 0x0436,
    /// This record outputs a character string at a specified location using
    /// the font, background color, and text color; these are defined in the
    /// playback device context.
    META_TEXTOUT = 0x0521,
    /// This record paints a series of closed polygons. Each polygon is
    /// outlined by using the pen and filled by using the brush and polygon
    /// fill mode; these are defined in the playback device context. The
    /// polygons drawn in this operation can overlap.
    META_POLYPOLYGON = 0x0538,
    /// This record fills an area with the brush that is defined in the
    /// playback device context.
    META_EXTFLOODFILL = 0x0548,
    /// This record paints a rectangle. The rectangle is outlined by using the
    /// pen and filled by using the brush; these are defined in the playback
    /// device context.
    META_RECTANGLE = 0x041B,
    /// This record sets the pixel at specified coordinates to a specified
    /// color.
    META_SETPIXEL = 0x041F,
    /// This record draws a rectangle with rounded corners. The rectangle is
    /// outlined by using the current pen and filled by using the current
    /// brush.
    META_ROUNDRECT = 0x061C,
    /// This record paints the specified rectangle by using the brush that is
    /// currently selected into the playback device context. The brush color
    /// and the surface color or colors are combined using the specified raster
    /// operation.
    META_PATBLT = 0x061D,
    /// This record saves the playback device context for later retrieval.
    META_SAVEDC = 0x001E,
    /// This record draws a pie-shaped wedge bounded by the intersection of an
    /// ellipse and two radials. The pie is outlined by using the pen and
    /// filled by using the brush; these are defined in the playback device
    /// context.
    META_PIE = 0x081A,
    /// This record specifies the transfer of a block of pixels according to a
    /// raster operation, with possible expansion or contraction.
    META_STRETCHBLT = 0x0B23,
    /// This record makes it possible to access capabilities of a particular
    /// printing device that are not directly available through other WMF
    /// records.
    META_ESCAPE = 0x0626,
    /// This record inverts the colors in a specified region.
    META_INVERTREGION = 0x012A,
    /// This record paints a specified region by using the brush that is
    /// defined in the playback device context.
    META_PAINTREGION = 0x012B,
    /// This record specifies the clipping region in the playback device
    /// context.
    META_SELECTCLIPREGION = 0x012C,
    /// This record specifies a graphics object in the playback device context.
    /// The new object replaces the previous object of the same type, if one is
    /// defined.
    META_SELECTOBJECT = 0x012D,
    /// This record defines the text-alignment values in the playback device
    /// context.
    META_SETTEXTALIGN = 0x012E,
    /// This record draws an elliptical arc.
    META_ARC = 0x0817,
    /// This record draws a chord, which is a region bounded by the
    /// intersection of an ellipse and a line segment. The chord is outlined by
    /// using the pen and filled by using the brush; these are defined in the
    /// playback device context.
    META_CHORD = 0x0830,
    /// This record specifies the transfer of a block of pixels according to a
    /// raster operation.
    META_BITBLT = 0x0922,
    /// This record outputs a character string by using the font, background
    /// color, and text color; these are defined in the playback device
    /// context. Optionally, dimensions can be provided for clipping, opaquing,
    /// or both.
    META_EXTTEXTOUT = 0x0a32,
    /// This record sets a block of pixels using device-independent color data.
    META_SETDIBTODEV = 0x0d33,
    /// This record specifies the transfer of a block of pixels in
    /// device-independent format according to a raster operation.
    META_DIBBITBLT = 0x0940,
    /// This record specifies the transfer of a block of pixels in
    /// device-independent format according to a raster operation, with
    /// possible expansion or contraction.
    META_DIBSTRETCHBLT = 0x0b41,
    /// This record specifies the transfer of color data from a block of pixels
    /// in device-independent format according to a raster operation, with
    /// possible expansion or contraction.
    META_STRETCHDIB = 0x0f43,
    /// This record deletes a graphics object, which can be a pen, brush, font,
    /// region, or palette.
    META_DELETEOBJECT = 0x01f0,
    /// This record defines a logical palette.
    META_CREATEPALETTE = 0x00f7,
    /// This record defines a brush with a pattern specified by a DIB.
    META_CREATEPATTERNBRUSH = 0x01F9,
    /// This record defines a pen with specified style, width, and color.
    META_CREATEPENINDIRECT = 0x02FA,
    /// This record defines a font with specified characteristics.
    META_CREATEFONTINDIRECT = 0x02FB,
    /// This record defines a brush with specified style, color, and pattern.
    META_CREATEBRUSHINDIRECT = 0x02FC,
    /// This record defines a region.
    ///
    /// The high-order byte of the WMF record type values SHOULD be ignored for
    /// all record types except the following: (For most WMF record types,
    /// the high-order byte of the RecordFunction field signifies the minimum
    /// number of 16-bit parameters, ideally specified in the WMF record;
    /// however, the value is not reliable for that purpose.)
    ///
    /// - META_BITBLT
    /// - META_DIBBITBLT
    /// - META_DIBSTRETCHBLT
    /// - META_POLYGON
    /// - META_POLYLINE
    /// - META_SETPALENTRIES
    /// - META_STRETCHBLT
    ///
    /// The meanings of the high-order bytes of these record type fields are
    /// specified in the respective sections that define them.
    ///
    /// A record type is not defined for the WMF header record, because only
    /// one can be present as the first record in the metafile.
    META_CREATEREGION = 0x06FF,
}

crate::wmf::parser::constants::impl_parser!(RecordType, u16);
