/// The PitchFont Enumeration defines values that are used for specifying
/// characteristics of a font. The values are used to indicate whether the
/// characters in a font have a fixed or variable width, or pitch.
///
/// In a Font Object, when a FamilyFont Enumeration value is packed into a byte
/// with a PitchFont value, the result is a PitchAndFamily Object.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, strum::FromRepr, strum::EnumIter)]
#[repr(u8)]
pub enum PitchFont {
    /// The default pitch, which is implementation-dependent.
    DEFAULT_PITCH = 0,
    /// A fixed pitch, which means that all the characters in the font occupy
    /// the same width when output in a string.
    FIXED_PITCH = 1,
    /// A variable pitch, which means that the characters in the font occupy
    /// widths that are proportional to the actual widths of the glyphs when
    /// output in a string. For example, the "i" and space characters usually
    /// have much smaller widths than a "W" or "O" character.
    VARIABLE_PITCH = 2,
}
