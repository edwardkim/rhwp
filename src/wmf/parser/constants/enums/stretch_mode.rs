/// The StretchMode Enumeration specifies the bitmap stretching mode, which
/// defines how the system combines rows or columns of a bitmap with existing
/// pixels.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, strum::FromRepr, strum::EnumIter)]
#[repr(u16)]
pub enum StretchMode {
    /// Performs a Boolean AND operation by using the color values for the
    /// eliminated and existing pixels. If the bitmap is a monochrome bitmap,
    /// this mode preserves black pixels at the expense of white pixels.
    /// (Windows 95 and Windows 98: The symbolic name "STRETCH_ANDSCANS" is
    /// synonymous with this value.)
    BLACKONWHITE = 0x0001,
    /// Performs a Boolean OR operation by using the color values for the
    /// eliminated and existing pixels. If the bitmap is a monochrome bitmap,
    /// this mode preserves white pixels at the expense of black pixels.
    /// (Windows 95 and Windows 98: The symbolic name "STRETCH_ORSCANS" is
    /// synonymous with this value.)
    WHITEONBLACK = 0x0002,
    /// Deletes the pixels. This mode deletes all eliminated lines of pixels
    /// without trying to preserve their information. (Windows 95 and Windows
    /// 98: The symbolic name "STRETCH_DELETESCANS" is synonymous with this
    /// value.)
    COLORONCOLOR = 0x0003,
    /// Maps pixels from the source rectangle into blocks of pixels in the
    /// destination rectangle. The average color over the destination block of
    /// pixels approximates the color of the source pixels. (Windows 95 and
    /// Windows 98: The symbolic name "STRETCH_HALFTONE" is synonymous with
    /// this value.)
    ///
    /// After setting the HALFTONE stretching mode, the brush origin MUST be
    /// set to avoid misalignment artifacts.
    HALFTONE = 0x0004,
}

crate::wmf::parser::constants::impl_parser!(StretchMode, u16);
