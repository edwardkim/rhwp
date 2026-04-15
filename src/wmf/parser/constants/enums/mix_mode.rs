/// The MixMode Enumeration specifies the background mix mode for text, hatched
/// brushes, and other nonsolid pen styles.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, strum::FromRepr, strum::EnumIter)]
#[repr(u16)]
pub enum MixMode {
    /// The background remains untouched.
    TRANSPARENT = 0x0001,
    /// The background is filled with the background color that is currently
    /// defined in the playback device context before the text, hatched brush,
    /// or pen is drawn.
    OPAQUE = 0x0002,
}

crate::wmf::parser::constants::impl_parser!(MixMode, u16);
