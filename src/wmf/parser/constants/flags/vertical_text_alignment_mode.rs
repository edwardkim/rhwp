/// VerticalTextAlignmentMode Flags specify the relationship between a reference
/// point and a bounding rectangle, for text alignment. These flags can be
/// combined to specify multiple options, with the restriction that only one
/// flag can be chosen that alters the drawing position in the playback device
/// context.
///
/// Vertical text alignment is performed when the font has a vertical default
/// baseline, such as Kanji.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, strum::FromRepr, strum::EnumIter)]
#[repr(u16)]
pub enum VerticalTextAlignmentMode {
    /// The reference point MUST be on the top edge of the bounding rectangle.
    VTA_TOP = 0x0000,
    /// The reference point MUST be on the right edge of the bounding
    /// rectangle.
    // VTA_RIGHT = 0x0000,
    /// The reference point MUST be on the bottom edge of the bounding
    /// rectangle.
    VTA_BOTTOM = 0x0002,
    /// The reference point MUST be aligned vertically with the center of the
    /// bounding rectangle.
    VTA_CENTER = 0x0006,
    /// The reference point MUST be on the left edge of the bounding rectangle.
    VTA_LEFT = 0x0008,
    /// The reference point MUST be on the baseline of the text.
    VTA_BASELINE = 0x0018,
}

crate::wmf::parser::constants::impl_parser!(VerticalTextAlignmentMode, u16);
