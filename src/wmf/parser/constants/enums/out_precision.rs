/// The OutPrecision enumeration defines values for output precision, which is
/// the requirement for the font mapper to match specific font parameters,
/// including height, width, character orientation, escapement, pitch, and font
/// type.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, strum::FromRepr, strum::EnumIter)]
#[repr(u8)]
pub enum OutPrecision {
    /// A value that specifies default behavior.
    OUT_DEFAULT_PRECIS = 0x00000000,
    /// A value that is returned when rasterized fonts are enumerated.
    OUT_STRING_PRECIS = 0x00000001,
    /// A value that is returned when TrueType and other outline fonts, and
    /// vector fonts are enumerated.
    OUT_STROKE_PRECIS = 0x00000003,
    /// A value that specifies the choice of a TrueType font when the system
    /// contains multiple fonts with the same name.
    OUT_TT_PRECIS = 0x00000004,
    /// A value that specifies the choice of a device font when the system
    /// contains multiple fonts with the same name.
    OUT_DEVICE_PRECIS = 0x00000005,
    /// A value that specifies the choice of a rasterized font when the system
    /// contains multiple fonts with the same name.
    OUT_RASTER_PRECIS = 0x00000006,
    /// A value that specifies the requirement for only TrueType fonts. If
    /// there are no TrueType fonts installed in the system, default behavior
    /// is specified.
    OUT_TT_ONLY_PRECIS = 0x00000007,
    /// A value that specifies the requirement for TrueType and other outline
    /// fonts.
    OUT_OUTLINE_PRECIS = 0x00000008,
    /// A value that specifies a preference for TrueType and other outline
    /// fonts.
    OUT_SCREEN_OUTLINE_PRECIS = 0x00000009,
    /// A value that specifies a requirement for only PostScript fonts. If
    /// there are no PostScript fonts installed in the system, default behavior
    /// is specified.
    OUT_PS_ONLY_PRECIS = 0x0000000A,
}

crate::wmf::parser::constants::impl_parser!(OutPrecision, u8);
