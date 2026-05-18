/// The GamutMappingIntent Enumeration specifies the relationship between
/// logical and physical colors. (Windows NT 3.1, Windows NT 3.5, and Windows NT
/// 3.51: This functionality is not supported.)
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, strum::FromRepr, strum::EnumIter)]
#[repr(u32)]
pub enum GamutMappingIntent {
    /// Specifies that saturation SHOULD be maintained. Typically used for
    /// business charts and other situations in which dithering is not
    /// required.
    ///
    /// Intent: Graphic
    ///
    /// ICC name: Saturation
    LCS_GM_BUSINESS = 0x00000001,
    /// Specifies that a colorimetric match SHOULD be maintained. Typically
    /// used for graphic designs and named colors.
    ///
    /// Intent: Proof
    ///
    /// ICC name: Relative Colorimetric
    LCS_GM_GRAPHICS = 0x00000002,
    /// Specifies that contrast SHOULD be maintained. Typically used for
    /// photographs and natural images.
    ///
    /// Intent: Picture
    ///
    /// ICC name: Perceptual
    LCS_GM_IMAGES = 0x00000004,
    /// Specifies that the white point SHOULD be maintained. Typically used
    /// when logical colors MUST be matched to their nearest physical color in
    /// the destination color gamut.
    ///
    /// Intent: Match
    ///
    /// ICC name: Absolute Colorimetric
    LCS_GM_ABS_COLORIMETRIC = 0x00000008,
}

crate::wmf::parser::constants::impl_parser!(GamutMappingIntent, u32);
