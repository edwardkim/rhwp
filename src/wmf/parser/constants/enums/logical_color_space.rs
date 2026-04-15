/// The LogicalColorSpace Enumeration specifies the type of color space.
/// (Windows NT 3.1, Windows NT 3.5, and Windows NT 3.51: This functionality is
/// not supported.)
///
/// The LogicalColorSpaceV5 Enumeration is used to specify where to find color
/// profile information for a DeviceIndependentBitmap (DIB) Object that has a
/// header of type BitmapV5Header Object.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, strum::FromRepr, strum::EnumIter)]
#[repr(u32)]
pub enum LogicalColorSpace {
    /// Color values are calibrated red green blue (RGB) values.
    LCS_CALIBRATED_RGB = 0x00000000,
    /// The value is an encoding of the ASCII characters "sRGB", and it
    /// indicates that the color values are sRGB values.
    LCS_sRGB = 0x73524742,
    /// The value is an encoding of the ASCII characters "Win ", including the
    /// trailing space, and it indicates that the color values are Windows
    /// default color space values.
    LCS_WINDOWS_COLOR_SPACE = 0x57696E20,
    /// The value consists of the string "LINK" from the Windows character set
    /// (code page 1252). It indicates that the color profile MUST be linked
    /// with the DIB Object.
    LCS_PROFILE_LINKED = 0x4C494E4B,
    /// The value consists of the string "MBED" from the Windows character set
    /// (code page 1252). It indicates that the color profile MUST be embedded
    /// in the DIB Object.
    LCS_PROFILE_EMBEDDED = 0x4D424544,
}

crate::wmf::parser::constants::impl_parser!(LogicalColorSpace, u32);
