/// The FamilyFont Enumeration specifies the font family. Font families describe
/// the look of a font in a general way. They are intended for specifying fonts
/// when the exact typeface desired is not available.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, strum::FromRepr, strum::EnumIter)]
#[repr(u8)]
pub enum FamilyFont {
    /// The default font is specified, which is implementation-dependent.
    FF_DONTCARE = 0x00,
    /// Fonts with variable stroke widths, which are proportional to the actual
    /// widths of the glyphs, and which have serifs. "MS Serif" is an example.
    FF_ROMAN = 0x01,
    /// Fonts with variable stroke widths, which are proportional to the actual
    /// widths of the glyphs, and which do not have serifs. "MS Sans Serif" is
    /// an example.
    FF_SWISS = 0x02,
    /// Fonts with constant stroke width, with or without serifs. Fixed-width
    /// fonts are usually modern. "Pica", "Elite", and "Courier New" are
    /// examples.
    FF_MODERN = 0x03,
    /// Fonts designed to look like handwriting. "Script" and "Cursive" are
    /// examples.
    FF_SCRIPT = 0x04,
    /// Novelty fonts. "Old English" is an example. In a Font Object, when a
    /// FamilyFont value is packed into a byte with a PitchFont Enumeration
    /// value, the result is a PitchAndFamily Object.
    FF_DECORATIVE = 0x05,
}
