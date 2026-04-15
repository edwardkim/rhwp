/// TextAlignmentMode Flags specify the relationship between a reference point
/// and a bounding rectangle, for text alignment. These flags can be combined to
/// specify multiple options, with the restriction that only one flag can be
/// chosen that alters the drawing position in the playback device context.
///
/// Horizontal text alignment is performed when the font has a horizontal
/// default baseline.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, strum::FromRepr, strum::EnumIter)]
#[repr(u16)]
pub enum TextAlignmentMode {
    /// The drawing position in the playback device context MUST NOT be updated
    /// after each text output call. The reference point MUST be passed to the
    /// text output function.
    // TA_NOUPDATECP = 0x0000,
    /// The reference point MUST be on the left edge of the bounding rectangle.
    TA_LEFT = 0x0000,
    /// The reference point MUST be on the top edge of the bounding rectangle.
    // TA_TOP = 0x0000,
    /// The drawing position in the playback device context MUST be updated
    /// after each text output call. It MUST be used as the reference point.
    TA_UPDATECP = 0x0001,
    /// The reference point MUST be on the right edge of the bounding
    /// rectangle.
    TA_RIGHT = 0x0002,
    /// The reference point MUST be aligned horizontally with the center of the
    /// bounding rectangle.
    TA_CENTER = 0x0006,
    /// The reference point MUST be on the bottom edge of the bounding
    /// rectangle.
    TA_BOTTOM = 0x0008,
    /// The reference point MUST be on the baseline of the text.
    TA_BASELINE = 0x0018,
}

crate::wmf::parser::constants::impl_parser!(TextAlignmentMode, u16);
