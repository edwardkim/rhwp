/// The BrushStyle Enumeration specifies the different possible brush types that
/// can be used in graphics operations. For more information, see the
/// specification of the Brush Object.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, strum::FromRepr, strum::EnumIter)]
#[repr(u16)]
pub enum BrushStyle {
    /// A brush that paints a single, constant color, either solid or dithered.
    BS_SOLID = 0x0000,
    /// A brush that does nothing. Using a BS_NULL brush in a graphics
    /// operation MUST have the same effect as using no brush at all. (In
    /// Windows implementations, BS_HOLLOW was added as a duplicate symbolic
    /// name for BS_NULL, because BS_NULL was too easily mistaken for a NULL
    /// pointer. BS_HOLLOW is used by an application when GDI requires a
    /// non-NULL brush parameter, but the application requires that no brush be
    /// used.)
    BS_NULL = 0x0001,
    /// A brush that paints a predefined simple pattern, or "hatch", onto a
    /// solid background.
    BS_HATCHED = 0x0002,
    /// A brush that paints a pattern defined by a bitmap, which can be a
    /// Bitmap16 Object or a DeviceIndependentBitmap Object.
    BS_PATTERN = 0x0003,
    /// Not supported.
    BS_INDEXED = 0x0004,
    /// A pattern brush specified by a DIB.
    BS_DIBPATTERN = 0x0005,
    /// A pattern brush specified by a DIB.
    BS_DIBPATTERNPT = 0x0006,
    /// Not supported.
    BS_PATTERN8X8 = 0x0007,
    /// Not supported.
    BS_DIBPATTERN8X8 = 0x0008,
    /// Not supported.
    BS_MONOPATTERN = 0x0009,
}

crate::wmf::parser::constants::impl_parser!(BrushStyle, u16);
