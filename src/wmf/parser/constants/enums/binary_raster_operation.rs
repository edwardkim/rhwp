/// The BinaryRasterOperation Enumeration section lists the binary
/// raster-operation codes. Raster-operation codes define how metafile
/// processing combines the bits from the selected pen with the bits in the
/// destination bitmap.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, strum::FromRepr, strum::EnumIter)]
#[repr(u16)]
pub enum BinaryRasterOperation {
    /// 0, Pixel is always 0.
    R2_BLACK = 0x0001,
    /// DPon, Pixel is the inverse of the R2_MERGEPEN color.
    R2_NOTMERGEPEN = 0x0002,
    /// DPna, Pixel is a combination of the screen color and the inverse of the
    /// pen color.
    R2_MASKNOTPEN = 0x0003,
    /// Pn, Pixel is the inverse of the pen color.
    R2_NOTCOPYPEN = 0x0004,
    /// PDna, Pixel is a combination of the colors common to both the pen and
    /// the inverse of the screen.
    R2_MASKPENNOT = 0x0005,
    /// Dn, Pixel is the inverse of the screen color.
    R2_NOT = 0x0006,
    /// DPx, Pixel is a combination of the colors in the pen or in the screen,
    /// but not in both.
    R2_XORPEN = 0x0007,
    /// DPan, Pixel is the inverse of the R2_MASKPEN color.
    R2_NOTMASKPEN = 0x0008,
    /// DPa, Pixel is a combination of the colors common to both the pen and
    /// the screen.
    R2_MASKPEN = 0x0009,
    /// DPxn, Pixel is the inverse of the R2_XORPEN color.
    R2_NOTXORPEN = 0x000A,
    /// D, Pixel remains unchanged.
    R2_NOP = 0x000B,
    /// DPno, Pixel is a combination of the colors common to both the screen
    /// and the inverse of the pen.
    R2_MERGENOTPEN = 0x000C,
    /// P, Pixel is the pen color.
    R2_COPYPEN = 0x000D,
    /// PDno, Pixel is a combination of the pen color and the inverse of the
    /// screen color.
    R2_MERGEPENNOT = 0x000E,
    /// DPo, Pixel is a combination of the pen color and the screen color.
    R2_MERGEPEN = 0x000F,
    /// 1, Pixel is always 1
    R2_WHITE = 0x0010,
}

crate::wmf::parser::constants::impl_parser!(BinaryRasterOperation, u16);
