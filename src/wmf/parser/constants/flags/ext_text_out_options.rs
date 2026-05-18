/// ExtTextOutOptions Flags specify various characteristics of the output of
/// text. These flags can be combined to specify multiple options.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, strum::FromRepr, strum::EnumIter)]
#[repr(u16)]
pub enum ExtTextOutOptions {
    /// Indicates that the background color that is defined in the playback
    /// device context SHOULD be used to fill the rectangle.
    ETO_OPAQUE = 0x0002,
    /// Indicates that the text SHOULD be clipped to the rectangle.
    ETO_CLIPPED = 0x0004,
    /// Indicates that the string to be output SHOULD NOT require further
    /// processing with respect to the placement of the characters, and an
    /// array of character placement values SHOULD be provided. This character
    /// placement process is useful for fonts in which diacritical characters
    /// affect character spacing. (Windows NT 3.1, Windows NT 3.5, and Windows
    /// NT 3.51: This function is not supported.)
    ETO_GLYPH_INDEX = 0x0010,
    /// Indicates that the text MUST be laid out in right-to-left reading
    /// order, instead of the default left-to-right order. This SHOULD be
    /// applied only when the font that is defined in the playback device
    /// context is either Hebrew or Arabic. (Windows NT 3.1, Windows NT 3.5,
    /// and Windows NT 3.51: This function is not supported.)
    ETO_RTLREADING = 0x0080,
    /// Indicates that to display numbers, digits appropriate to the locale
    /// SHOULD be used. (Windows NT 3.1, Windows NT 3.5, and Windows NT 3.51:
    /// This function is not supported.)
    ETO_NUMERICSLOCAL = 0x0400,
    /// Indicates that to display numbers, European digits SHOULD be used.
    /// (Windows NT 3.1, Windows NT 3.5, and Windows NT 3.51: This function is
    /// not supported.)
    ETO_NUMERICSLATIN = 0x0800,
    /// Indicates that both horizontal and vertical character displacement
    /// values SHOULD be provided. (Windows NT 3.1, Windows NT 3.5, Windows NT
    /// 3.51, Windows 95, Windows NT 4.0, Windows 98, and Windows Millennium
    /// Edition: This function is not supported.)
    ETO_PDY = 0x2000,
}

crate::wmf::parser::constants::impl_parser!(ExtTextOutOptions, u16);
